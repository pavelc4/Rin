package com.rin

import android.os.Bundle
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
import com.rin.ui.screen.SetupScreen
import com.rin.ui.screen.TerminalScreen
import com.rin.ui.screen.getStoredUsername
import com.rin.ui.theme.RinTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        setContent {
            var username by remember { mutableStateOf(getStoredUsername(this)) }
            var engineHandle by remember { mutableLongStateOf(0L) }

            RinTheme {
                // Set status bar color to match Monet surface
                val surfaceColor = MaterialTheme.colorScheme.surfaceContainerLowest.toArgb()
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
                        
                        // Create .mkshrc with dynamic PS1 prompt
                        val mkshrc = homeDir.resolve(".mkshrc")
                        // Use double quotes so $USER expands
                        mkshrc.writeText("""
                        USER=${"$"}{USER:-$currentUser}
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
