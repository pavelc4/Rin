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
fun rememberTerminalColorScheme(
    darkTheme: Boolean = androidx.compose.foundation.isSystemInDarkTheme()
): TerminalColorScheme {
    val colorScheme = MaterialTheme.colorScheme

    return TerminalColorScheme(
        // Background: surface (Monet tinted dark/light)
        background = colorScheme.surface.toArgb(),
        // Foreground: onSurface
        foreground = colorScheme.onSurface.toArgb(),
        // Cursor: primary
        cursor = colorScheme.primary.toArgb(),

        // ANSI colors - Adaptive Standard Palette
        // Dark Mode: Standard bright Xterm colors
        // Light Mode: Darker variants for readability on white background
        black = if (darkTheme) 0xFF000000.toInt() else 0xFF000000.toInt(),
        red = if (darkTheme) 0xFFF44336.toInt() else 0xFFD32F2F.toInt(),
        green = if (darkTheme) 0xFF4CAF50.toInt() else 0xFF388E3C.toInt(),
        yellow = if (darkTheme) 0xFFFFEB3B.toInt() else 0xFFFBC02D.toInt(),
        blue = if (darkTheme) 0xFF2196F3.toInt() else 0xFF1976D2.toInt(),
        magenta = if (darkTheme) 0xFF9C27B0.toInt() else 0xFF7B1FA2.toInt(),
        cyan = colorScheme.primary.toArgb(), // Monet-themed "Rin" banner
        white = if (darkTheme) 0xFFE0E0E0.toInt() else 0xFF424242.toInt(), // Light Grey vs Dark Grey

        // Bright colors
        brightBlack = colorScheme.onSurfaceVariant.toArgb(), // Monet Grey for Banner/Comments
        brightRed = if (darkTheme) 0xFFFF8A80.toInt() else 0xFFD32F2F.toInt(),
        brightGreen = if (darkTheme) 0xFFB9F6CA.toInt() else 0xFF388E3C.toInt(),
        brightYellow = if (darkTheme) 0xFFFFFF8D.toInt() else 0xFFFBC02D.toInt(),
        brightBlue = if (darkTheme) 0xFF82B1FF.toInt() else 0xFF1976D2.toInt(),
        brightMagenta = if (darkTheme) 0xFFEA80FC.toInt() else 0xFF7B1FA2.toInt(),
        brightCyan = colorScheme.tertiary.toArgb(), // Monet varaint for bright cyan
        brightWhite = if (darkTheme) 0xFFFFFFFF.toInt() else 0xFF212121.toInt(), // White vs Near Black
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
