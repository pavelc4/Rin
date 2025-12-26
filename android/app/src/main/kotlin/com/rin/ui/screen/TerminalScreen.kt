package com.rin.ui.screen

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import com.rin.RinLib
import com.rin.ui.components.ExtraKeysBar
import com.rin.ui.components.TerminalSurface

@Composable
fun TerminalScreen(
    engineHandle: Long,
    modifier: Modifier = Modifier
) {
    var ctrlPressed by remember { mutableStateOf(false) }

    Column(
        modifier = modifier
            .fillMaxSize()
            .systemBarsPadding()
            .imePadding()
    ) {
        TerminalSurface(
            engineHandle = engineHandle,
            ctrlPressed = ctrlPressed,
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f),
            onInput = { data ->
                if (engineHandle != 0L) {
                    RinLib.write(engineHandle, data)
                }
            }
        )

        ExtraKeysBar(
            onKeyPress = { code ->
                if (engineHandle != 0L) {
                    RinLib.write(engineHandle, code.toByteArray())
                }
            },
            onCtrlToggle = { active ->
                ctrlPressed = active
            },
            modifier = Modifier.fillMaxWidth()
        )
    }
}
