package com.rin

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.rin.ui.screen.TerminalScreen
import com.rin.ui.theme.RinTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        setContent {
            var engineHandle by remember { mutableLongStateOf(0L) }

            DisposableEffect(Unit) {
                engineHandle = RinLib.createEngine(80, 24, 14.0f)
                onDispose {
                    if (engineHandle != 0L) {
                        RinLib.destroyEngine(engineHandle)
                    }
                }
            }

            RinTheme {
                TerminalScreen(engineHandle = engineHandle)
            }
        }
    }
}
