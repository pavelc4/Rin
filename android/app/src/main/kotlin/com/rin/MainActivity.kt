package com.rin

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.Build
import android.os.Bundle
import android.os.IBinder
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.toArgb
import com.rin.permission.StoragePermissionHelper
import com.rin.service.TerminalSessionService
import com.rin.terminal.SessionManager
import com.rin.ui.screen.SetupScreen
import com.rin.ui.screen.TerminalScreen
import com.rin.ui.screen.getStoredUsername
import com.rin.ui.theme.RinTheme
import java.io.File

class MainActivity : ComponentActivity() {

    private var terminalService: TerminalSessionService? = null
    private var serviceBound = false

    private val serviceConnection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            val binder = service as TerminalSessionService.LocalBinder
            terminalService = binder.service
            serviceBound = true
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            terminalService = null
            serviceBound = false
        }
    }

    private fun installPrebuiltBinaries() {
        val binDir = File(filesDir, "usr/bin").also { it.mkdirs() }
        val rpkgBin = File(binDir, "rpkg")
        val nativeDir = applicationInfo.nativeLibraryDir
        val nativeLib = File(nativeDir, "librpkg_cli.so")

        if (nativeLib.exists()) {
            rpkgBin.delete()
            try {
                android.system.Os.symlink(nativeLib.absolutePath, rpkgBin.absolutePath)
                Log.i("Rin", "Symlinked librpkg_cli.so to rpkg")
            } catch (e: Exception) {
                Log.e("Rin", "Failed to symlink rpkg: ${e.message}")
            }
        } else {
            Log.w("Rin", "Native library librpkg_cli.so not found in $nativeDir")
        }

        // Install rin-perm-storage
        val permStorageScript = File(binDir, "rin-perm-storage")
        val prefix = filesDir.absolutePath
        permStorageScript.writeText("""
            #!/system/bin/sh
            PERM_FILE="$prefix/.storage_permission"
            REQUEST_FILE="$prefix/.rin_request_perm"

            if [ -f "${'$'}PERM_FILE" ]; then
                exit 0
            fi

            echo ""
            echo "\033[36m  ____  _       \033[0m"
            echo "\033[36m |  _ \(_)_ __  \033[0m"
            echo "\033[36m | |_) | | '_ \ \033[0m"
            echo "\033[36m |  _ <| | | | |\033[0m"
            echo "\033[36m |_| \_\_|_| |_|\033[0m"
            echo ""
            echo "\033[33mRequesting storage permission...\033[0m"
            echo "\033[90mPlease grant access in the system dialog.\033[0m"
            echo ""

            # Signal Kotlin layer to show permission dialog
            echo "request" > "${'$'}REQUEST_FILE"

            # Poll for result (max 60 seconds)
            COUNTER=0
            while [ ${'$'}COUNTER -lt 120 ]; do
                if [ -f "${'$'}PERM_FILE" ]; then
                    echo "\033[32m✓ Storage permission granted!\033[0m"
                    echo "\033[33mYou can now use rpkg commands.\033[0m"
                    echo ""
                    rm -f "${'$'}REQUEST_FILE"
                    exit 0
                fi
                sleep 0.5
                COUNTER=${'$'}((COUNTER + 1))
            done

            echo "\033[31m✗ Permission request timed out.\033[0m"
            echo "\033[33mPlease try again: rin-perm-storage\033[0m"
            echo ""
            rm -f "${'$'}REQUEST_FILE"
            exit 1
        """.trimIndent() + "\n")
        permStorageScript.setExecutable(true)
        Log.i("Rin", "Installed rin-perm-storage script")
    }

    private fun startTerminalService() {
        val intent = Intent(this, TerminalSessionService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }
        bindService(intent, serviceConnection, Context.BIND_AUTO_CREATE)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        installPrebuiltBinaries()
        startTerminalService()

        setContent {
            var username by remember { mutableStateOf(getStoredUsername(this)) }

            RinTheme {
                val surfaceColor = MaterialTheme.colorScheme.surface.toArgb()
                SideEffect {
                    window.statusBarColor = surfaceColor
                    window.navigationBarColor = surfaceColor
                }
                if (username == null) {
                    SetupScreen(
                        onComplete = { name ->
                            username = name
                        },
                        onRequestStorage = {
                            RinPermStorage.requestStoragePermission(this@MainActivity) { granted ->
                                if (granted) {
                                    val permFile = File(filesDir, ".storage_permission")
                                    permFile.writeText("granted")
                                }
                            }
                        }
                    )
                } else {
                    val homeDir = filesDir.resolve("home").also { it.mkdirs() }
                    val currentUser = username ?: "user"

                    val prefix = filesDir.absolutePath
                    val mkshrc = homeDir.resolve(".mkshrc")
                    mkshrc.writeText("""
                        USER=${"$"}{USER:-$currentUser}
                        export PATH=/system/xbin:/system/bin:/system/sbin:$prefix/usr/bin:${"$"}{PATH}
                        export LD_LIBRARY_PATH=$prefix/usr/lib:${"$"}{LD_LIBRARY_PATH}
                        
                        _rin_prompt() {
                            local pwd_display="${"$"}PWD"
                            if [ "${"$"}PWD" = "${"$"}HOME" ]; then
                                pwd_display="~"
                            elif [ "${"$"}{PWD#${"$"}HOME/}" != "${"$"}PWD" ]; then
                                pwd_display="~/${"$"}{PWD#${"$"}HOME/}"
                            fi
                            
                            if [ "${"$"}(id -u)" = "0" ]; then
                                printf '\033[31mroot\033[0m@rin-%s:\033[34m%s\033[0m# ' "${"$"}USER" "${"$"}pwd_display"
                            else
                                printf 'rin@%s:\033[34m%s\033[0m${"$"} ' "${"$"}USER" "${"$"}pwd_display"
                            fi
                        }
                        
                        PS1='${"$"}(_rin_prompt)'
                        
                        # Auto-request storage permission if not yet granted
                        if [ ! -f "$prefix/.storage_permission" ]; then
                            rin-perm-storage
                        fi
                    """.trimIndent() + "\n")
                    
                    // Create rpkg wrapper
                    val binDir = File(filesDir, "usr/bin")
                    binDir.mkdirs()
                    val rpkgWrapper = File(binDir, "rpkg-real")
                    val rpkgScript = File(binDir, "rpkg")
                    
                    if (rpkgScript.exists() && !rpkgWrapper.exists() && !rpkgScript.isDirectory) {
                        try {
                            // Check symlink
                            val isSymlink = try {
                                android.system.Os.readlink(rpkgScript.absolutePath)
                                true
                            } catch (e: Exception) {
                                false
                            }
                            
                            if (isSymlink) {
                                // Rename symlink
                                rpkgScript.renameTo(rpkgWrapper)
                                
                                // Create wrapper
                                rpkgScript.writeText("""
                                    #!/system/bin/sh
                                    PERM_FILE="$prefix/.storage_permission"
                                    if [ ! -f "${"$"}PERM_FILE" ]; then
                                        echo ""
                                        echo "\033[31m\033[1mError: Storage permission required!\033[0m"
                                        echo "\033[33mRun 'rin-perm-storage' to grant access before using rpkg\033[0m"
                                        echo ""
                                        exit 1
                                    fi
                                    exec $prefix/usr/bin/rpkg-real "${"$"}@"
                                """.trimIndent())
                                rpkgScript.setExecutable(true)
                            }
                        } catch (e: Exception) {
                            Log.e("Rin", "Failed to create rpkg wrapper: ${e.message}")
                        }
                    }

                    val sessionManager = remember {
                        SessionManager(
                            context = this@MainActivity,
                            homeDir = homeDir.absolutePath,
                            username = currentUser
                        )
                    }

                    // Create the first session
                    DisposableEffect(sessionManager) {
                        if (sessionManager.sessions.isEmpty()) {
                            sessionManager.createSession()
                        }
                        terminalService?.sessionManager = sessionManager

                        onDispose {
                            sessionManager.destroyAll()
                        }
                    }
                    val sessionCount = sessionManager.sessionCount
                    SideEffect {
                        terminalService?.sessionManager = sessionManager
                        terminalService?.updateNotification(sessionCount)
                    }

                    TerminalScreen(sessionManager = sessionManager)
                }
            }
        }
    }

    override fun onResume() {
        super.onResume()
        val triggerFile = File(filesDir, ".rin_request_perm")
        if (triggerFile.exists()) {
            if (StoragePermissionHelper.hasSystemStoragePermission(this)) {
                StoragePermissionHelper.setStoragePermissionGranted(this, true)
                val permFile = File(filesDir, ".storage_permission")
                permFile.writeText("granted")
                triggerFile.delete()
                Log.i("Rin", "Storage permission detected on resume, marker written")
            }
        }
    }

    override fun onDestroy() {
        if (serviceBound) {
            unbindService(serviceConnection)
            serviceBound = false
        }
        super.onDestroy()
    }
    
    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == RinPermStorage.REQUEST_CODE_STORAGE) {
            val granted = grantResults.isNotEmpty() && 
                         grantResults.all { it == android.content.pm.PackageManager.PERMISSION_GRANTED }
            if (granted) {
                StoragePermissionHelper.setStoragePermissionGranted(this, true)
                val permFile = File(filesDir, ".storage_permission")
                permFile.writeText("granted")
            }
        }
    }
}
