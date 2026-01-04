package com.rin.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

data class ExtraKey(
    val label: String,
    val code: String,
    val isModifier: Boolean = false
)

private val row1Keys = listOf(
    ExtraKey("ESC", "\u001b"),
    ExtraKey("/", "/"),
    ExtraKey("-", "-"),
    ExtraKey("HOME", "\u001b[H"),
    ExtraKey("▲", "\u001b[A"),
    ExtraKey("END", "\u001b[F"),
    ExtraKey("PGUP", "\u001b[5~")
)

private val row2Keys = listOf(
    ExtraKey("TAB", "\t"),
    ExtraKey("CTRL", "CTRL", isModifier = true),
    ExtraKey("ALT", "ALT", isModifier = true),
    ExtraKey("◄", "\u001b[D"),
    ExtraKey("▼", "\u001b[B"),
    ExtraKey("►", "\u001b[C"),
    ExtraKey("PGDN", "\u001b[6~")
)

@Composable
fun ExtraKeysBar(
    onKeyPress: (String) -> Unit,
    onCtrlToggle: (Boolean) -> Unit,
    modifier: Modifier = Modifier
) {
    var ctrlActive by remember { mutableStateOf(false) }

    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 4.dp, vertical = 4.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp)
    ) {
        KeyRow(
            keys = row1Keys,
            ctrlActive = ctrlActive,
            onKeyPress = onKeyPress,
            onCtrlToggle = { }
        )
        KeyRow(
            keys = row2Keys,
            ctrlActive = ctrlActive,
            onKeyPress = onKeyPress,
            onCtrlToggle = { active ->
                ctrlActive = active
                onCtrlToggle(active)
            }
        )
    }
}

@Composable
private fun KeyRow(
    keys: List<ExtraKey>,
    ctrlActive: Boolean,
    onKeyPress: (String) -> Unit,
    onCtrlToggle: (Boolean) -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(3.dp)
    ) {
        keys.forEach { key ->
            val isActive = key.label == "CTRL" && ctrlActive

            FilledTonalButton(
                onClick = {
                    when {
                        key.label == "CTRL" -> onCtrlToggle(!ctrlActive)
                        key.label == "ALT" -> { /* TODO */ }
                        else -> onKeyPress(key.code)
                    }
                },
                modifier = Modifier
                    .weight(1f)
                    .height(36.dp),
                shape = RoundedCornerShape(6.dp),
                colors = ButtonDefaults.filledTonalButtonColors(
                    containerColor = if (isActive) 
                        MaterialTheme.colorScheme.primary 
                    else 
                        MaterialTheme.colorScheme.surfaceVariant,
                    contentColor = if (isActive)
                        MaterialTheme.colorScheme.onPrimary
                    else
                        MaterialTheme.colorScheme.primary
                ),
                contentPadding = androidx.compose.foundation.layout.PaddingValues(0.dp)
            ) {
                Text(
                    text = key.label,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Medium,
                    maxLines = 1
                )
            }
        }
    }
}
