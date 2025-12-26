package com.rin.ui.theme

import android.os.Build
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext

private val FallbackDarkScheme = darkColorScheme()

@Composable
fun RinTheme(content: @Composable () -> Unit) {
    val colorScheme = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        dynamicDarkColorScheme(LocalContext.current)
    } else {
        FallbackDarkScheme
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}
