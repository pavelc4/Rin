package com.rin

import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.toArgb
import com.rin.rpkg.RpkgLib
import com.rin.ui.screen.SetupScreen
import com.rin.ui.screen.TerminalScreen
import com.rin.ui.screen.getStoredUsername
import com.rin.ui.theme.RinTheme
import java.io.File

class MainActivity : ComponentActivity() {
    private fun installPrebuiltBinaries() {
        val binDir = File(filesDir, "usr/bin").also { it.mkdirs() }
        val rpkgBin = File(binDir, "rpkg")
        val nativeDir = applicationInfo.nativeLibraryDir
        val nativeLib = File(nativeDir, "librpkg_cli.so")

        if (nativeLib.exists()) {
            rpkgBin.delete() // Ensure we recreate the symlink on app updates
            try {
                android.system.Os.symlink(nativeLib.absolutePath, rpkgBin.absolutePath)
                Log.i("Rin", "Symlinked librpkg_cli.so to rpkg")
            } catch (e: Exception) {
                Log.e("Rin", "Failed to symlink rpkg: ${e.message}")
            }
        } else {
            Log.w("Rin", "Native library librpkg_cli.so not found in $nativeDir")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        installPrebuiltBinaries()
        setContent {
            var username by remember { mutableStateOf(getStoredUsername(this)) }
            var engineHandle by remember { mutableLongStateOf(0L) }

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
                        }
                    )
                } else {
                    DisposableEffect(username) {
                        val homeDir = filesDir.resolve("home").also { it.mkdirs() }
                        val currentUser = username ?: "user"
                        
                        val prefix = filesDir.absolutePath
                        val mkshrc = homeDir.resolve(".mkshrc")
                        mkshrc.writeText("""
                        USER=${"$"}{USER:-$currentUser}
                        export PATH=$prefix/usr/bin:${"$"}{PATH}
                        export LD_LIBRARY_PATH=$prefix/usr/lib:${"$"}{LD_LIBRARY_PATH}
                        PS1="rin@${"$"}USER:~${"$"} "
                        """.trimIndent() + "\n")
                        engineHandle = RinLib.createEngine(
                            80, 24, 14.0f,
                            homeDir.absolutePath,
                            currentUser
                        )
                        onDispose {
                            if (engineHandle != 0L) {
                                RinLib.destroyEngine(engineHandle)
                            }
                        }
                    }

                    TerminalScreen(engineHandle = engineHandle)
                }
            }
        }
    }
}
