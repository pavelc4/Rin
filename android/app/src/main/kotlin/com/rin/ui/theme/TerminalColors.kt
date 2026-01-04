package com.rin.ui.theme

import android.os.Build
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext

data class TerminalColorScheme(
    val background: Int,
    val foreground: Int,
    val cursor: Int,
    // ANSI 16 colors mapped to Monet
    val black: Int,
    val red: Int,
    val green: Int,
    val yellow: Int,
    val blue: Int,
    val magenta: Int,
    val cyan: Int,
    val white: Int,
    val brightBlack: Int,
    val brightRed: Int,
    val brightGreen: Int,
    val brightYellow: Int,
    val brightBlue: Int,
    val brightMagenta: Int,
    val brightCyan: Int,
    val brightWhite: Int,
)

val LocalTerminalColors = compositionLocalOf { createFallbackScheme() }

/**
 * Creates a terminal color scheme from Material You dynamic colors
 */
@Composable
fun rememberTerminalColorScheme(): TerminalColorScheme {
    val colorScheme = MaterialTheme.colorScheme

    return TerminalColorScheme(
        // Background: surfaceContainerLowest for deep dark
        background = colorScheme.surfaceContainerLowest.toArgb(),
        // Foreground: onSurface
        foreground = colorScheme.onSurface.toArgb(),
        // Cursor: primary
        cursor = colorScheme.primary.toArgb(),

        // ANSI colors mapped to Monet palette
        // Normal colors - black uses darker variant, not background (so it's visible)
        black = colorScheme.surfaceDim.toArgb(),
        red = colorScheme.error.toArgb(),
        green = colorScheme.tertiary.toArgb(),
        yellow = colorScheme.primaryContainer.toArgb(),
        blue = colorScheme.primary.toArgb(),
        magenta = colorScheme.secondary.toArgb(),
        cyan = colorScheme.tertiaryContainer.toArgb(),
        white = colorScheme.onSurface.toArgb(),  // Light color for text

        // Bright colors (using container/on variants for consistency)
        brightBlack = colorScheme.outline.toArgb(),
        brightRed = colorScheme.errorContainer.toArgb(),
        brightGreen = colorScheme.tertiary.lighten(0.3f).toArgb(),
        brightYellow = colorScheme.primary.lighten(0.3f).toArgb(),
        brightBlue = colorScheme.primary.lighten(0.2f).toArgb(),
        brightMagenta = colorScheme.secondary.lighten(0.2f).toArgb(),
        brightCyan = colorScheme.tertiary.lighten(0.3f).toArgb(),
        brightWhite = colorScheme.onSurface.toArgb(),  // Same as white, light text
    )
}

private fun Color.lighten(factor: Float): Color {
    return Color(
        red = (red + (1f - red) * factor).coerceIn(0f, 1f),
        green = (green + (1f - green) * factor).coerceIn(0f, 1f),
        blue = (blue + (1f - blue) * factor).coerceIn(0f, 1f),
        alpha = alpha
    )
}

private fun createFallbackScheme(): TerminalColorScheme {
    return TerminalColorScheme(
        background = 0xFF0D0D0D.toInt(),
        foreground = 0xFFE0E0E0.toInt(),
        cursor = 0xFFBB86FC.toInt(),
        black = 0xFF0D0D0D.toInt(),
        red = 0xFFCF6679.toInt(),
        green = 0xFF03DAC6.toInt(),
        yellow = 0xFFFFEB3B.toInt(),
        blue = 0xFFBB86FC.toInt(),
        magenta = 0xFFFF7597.toInt(),
        cyan = 0xFF03DAC6.toInt(),
        white = 0xFFE0E0E0.toInt(),
        brightBlack = 0xFF666666.toInt(),
        brightRed = 0xFFF48FB1.toInt(),
        brightGreen = 0xFF69F0AE.toInt(),
        brightYellow = 0xFFFFFF00.toInt(),
        brightBlue = 0xFFD1C4E9.toInt(),
        brightMagenta = 0xFFFF80AB.toInt(),
        brightCyan = 0xFF84FFFF.toInt(),
        brightWhite = 0xFFFFFFFF.toInt(),
    )
}
