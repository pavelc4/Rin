package com.rin.ui.components

import android.content.Context
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.Typeface
import android.text.InputType
import android.util.TypedValue
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.ScaleGestureDetector
import android.view.View
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputMethodManager
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import com.rin.RinLib
import com.rin.ui.theme.TerminalColorScheme
import com.rin.ui.theme.rememberTerminalColorScheme

@Composable
fun TerminalSurface(
    engineHandle: Long,
    ctrlPressed: Boolean,
    modifier: Modifier = Modifier,
    onInput: (ByteArray) -> Unit = {}
) {
    var fontSize by remember { mutableFloatStateOf(18f) }
    var ctrlState by remember { mutableStateOf(ctrlPressed) }
    val colorScheme = rememberTerminalColorScheme()
    
    // Update ctrlState when prop changes
    DisposableEffect(ctrlPressed) {
        ctrlState = ctrlPressed
        onDispose { }
    }

    AndroidView(
        factory = { context ->
            TerminalCanvasView(context).apply {
                this.engineHandle = engineHandle
                this.fontSize = fontSize
                this.onInputCallback = onInput
                this.ctrlPressedProvider = { ctrlState }
                this.colorScheme = colorScheme
            }
        },
        modifier = modifier,
        update = { view ->
            view.engineHandle = engineHandle
            view.fontSize = fontSize
            view.onInputCallback = onInput
            view.ctrlPressedProvider = { ctrlState }
            view.colorScheme = colorScheme
            view.invalidate()
        }
    )
}

private class TerminalCanvasView(context: Context) : View(context) {
    var engineHandle: Long = 0L
    var fontSize: Float = 18f
        set(value) {
            field = value
            updatePaint()
        }
    var onInputCallback: (ByteArray) -> Unit = {}
    var ctrlPressedProvider: () -> Boolean = { false }
    var colorScheme: TerminalColorScheme? = null

    private var cols = 80
    private var rows = 24

    private val textPaint = Paint().apply {
        color = Color.WHITE
        typeface = Typeface.MONOSPACE
        isAntiAlias = true
    }

    private val cursorPaint = Paint().apply {
        color = Color.WHITE
        alpha = 150
    }

    private val charWidth: Float
        get() = textPaint.measureText("W")

    private val lineHeight: Float
        get() = textPaint.fontSpacing

    private val scaleDetector = ScaleGestureDetector(context, object : ScaleGestureDetector.SimpleOnScaleGestureListener() {
        override fun onScale(detector: ScaleGestureDetector): Boolean {
            val newSize = fontSize * detector.scaleFactor
            if (newSize in 10f..40f) {
                fontSize = newSize
                handleResize()
                invalidate()
            }
            return true
        }
    })

    init {
        isFocusable = true
        isFocusableInTouchMode = true
        updatePaint()
        
        // Auto-refresh
        postDelayed(object : Runnable {
            override fun run() {
                invalidate()
                postDelayed(this, 16)
            }
        }, 16)
    }

    private fun updatePaint() {
        textPaint.textSize = TypedValue.applyDimension(
            TypedValue.COMPLEX_UNIT_SP,
            fontSize,
            resources.displayMetrics
        )
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        handleResize()
    }

    private fun handleResize() {
        if (width > 0 && height > 0 && charWidth > 0 && lineHeight > 0) {
            cols = (width / charWidth).toInt().coerceAtLeast(1)
            rows = (height / lineHeight).toInt().coerceAtLeast(1)
            if (engineHandle != 0L) {
                RinLib.resize(engineHandle, cols, rows)
            }
        }
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        scaleDetector.onTouchEvent(event)
        if (scaleDetector.isInProgress) return true
        
        if (event.action == MotionEvent.ACTION_DOWN) {
            requestFocus()
            val imm = context.getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager
            imm.showSoftInput(this, InputMethodManager.SHOW_IMPLICIT)
        }
        return true
    }

    override fun onCheckIsTextEditor(): Boolean = true

    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
        outAttrs.inputType = InputType.TYPE_NULL // Raw key events preferred
        outAttrs.imeOptions = EditorInfo.IME_ACTION_NONE or EditorInfo.IME_FLAG_NO_FULLSCREEN

        return object : BaseInputConnection(this, true) {
            private var composingText = StringBuilder()

            override fun setComposingText(text: CharSequence, newCursorPosition: Int): Boolean {
                // For predictive keyboards - send each new character immediately
                val newText = text.toString()
                if (newText.length > composingText.length) {
                    val newChars = newText.substring(composingText.length)
                    sendToTerminal(newChars)
                }
                composingText.clear()
                composingText.append(newText)
                return true
            }

            override fun finishComposingText(): Boolean {
                composingText.clear()
                return true
            }

            override fun commitText(text: CharSequence, newCursorPosition: Int): Boolean {
                // Clear any composing text first
                val committed = text.toString()
                if (composingText.isNotEmpty()) {
                    // Already sent via setComposingText, just clear
                    composingText.clear()
                } else {
                    // Direct commit (no composing)
                    sendToTerminal(committed)
                }
                return true
            }

            private fun sendToTerminal(text: String) {
                if (text.isEmpty()) return
                val data = if (ctrlPressedProvider() && text.length == 1) {
                    val char = text[0].lowercaseChar()
                    if (char in 'a'..'z') {
                        byteArrayOf((char.code - 96).toByte())
                    } else {
                        text.toByteArray()
                    }
                } else {
                    text.toByteArray()
                }
                onInputCallback(data)
                invalidate()
            }

            override fun sendKeyEvent(event: KeyEvent): Boolean {
                if (event.action == KeyEvent.ACTION_DOWN) {
                    when (event.keyCode) {
                        KeyEvent.KEYCODE_ENTER -> {
                            onInputCallback("\r".toByteArray())
                            invalidate()
                            return true
                        }
                        KeyEvent.KEYCODE_DEL -> {
                            onInputCallback(byteArrayOf(0x7F))
                            invalidate()
                            return true
                        }
                    }
                    // Handle character keys directly
                    val unicodeChar = event.unicodeChar
                    if (unicodeChar != 0) {
                        sendToTerminal(unicodeChar.toChar().toString())
                        return true
                    }
                }
                return super.sendKeyEvent(event)
            }

            override fun deleteSurroundingText(beforeLength: Int, afterLength: Int): Boolean {
                repeat(beforeLength) {
                    onInputCallback(byteArrayOf(0x7F))
                }
                invalidate()
                return true
            }
        }
    }

    // Map RGB to ANSI color index (approximate match)
    // Returns null for colors we don't want to theme (keep original)
    private fun rgbToAnsiIndex(r: Int, g: Int, b: Int): Int? {
        // Only map specific ANSI colors to Monet
        // Keep white/gray colors as-is for visibility  
        return when {
            r == 0 && g == 0 && b == 0 -> 0                   // Black
            r == 205 && g == 49 && b == 49 -> 1               // Red
            r == 13 && g == 188 && b == 121 -> 2              // Green
            r == 229 && g == 229 && b == 16 -> 3              // Yellow
            r == 36 && g == 114 && b == 200 -> 4              // Blue
            r == 188 && g == 63 && b == 188 -> 5              // Magenta
            r == 17 && g == 168 && b == 205 -> 6              // Cyan
            // Skip white (7) - keep original for visibility
            r == 102 && g == 102 && b == 102 -> 8             // Bright Black (gray)
            r == 241 && g == 76 && b == 76 -> 9               // Bright Red
            r == 35 && g == 209 && b == 139 -> 10             // Bright Green
            r == 245 && g == 245 && b == 67 -> 11             // Bright Yellow
            r == 59 && g == 142 && b == 234 -> 12             // Bright Blue
            r == 214 && g == 112 && b == 214 -> 13            // Bright Magenta
            r == 41 && g == 184 && b == 219 -> 14             // Bright Cyan
            // Skip bright white (15) - keep original for visibility
            else -> null  // True color or unmatched, keep as-is
        }
    }

    // Get Monet color for ANSI index
    private fun getMonetColor(index: Int): Int {
        val scheme = colorScheme ?: return Color.WHITE
        return when (index) {
            0 -> scheme.black
            1 -> scheme.red
            2 -> scheme.green
            3 -> scheme.yellow
            4 -> scheme.blue
            5 -> scheme.magenta
            6 -> scheme.cyan
            7 -> scheme.white
            8 -> scheme.brightBlack
            9 -> scheme.brightRed
            10 -> scheme.brightGreen
            11 -> scheme.brightYellow
            12 -> scheme.brightBlue
            13 -> scheme.brightMagenta
            14 -> scheme.brightCyan
            15 -> scheme.brightWhite
            else -> scheme.foreground
        }
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        val scheme = colorScheme
        canvas.drawColor(scheme?.background ?: 0xFF0D0D0D.toInt())

        if (engineHandle == 0L) return

        val bgPaint = Paint()
        val fgPaint = Paint().apply {
            typeface = Typeface.MONOSPACE
            isAntiAlias = true
            textSize = textPaint.textSize
        }

        // Draw cells with colors
        for (y in 0 until rows) {
            val cellData = RinLib.getCellData(engineHandle, y)
            if (cellData.isEmpty()) continue

            val cells = cellData.split("\n").filter { it.isNotEmpty() }
            var xPos = 0

            for (cellStr in cells) {
                val parts = cellStr.split("\t")
                if (parts.size < 4) {
                    xPos++
                    continue
                }

                val char = parts[0]
                val fgParts = parts[1].split(",")
                val bgParts = parts[2].split(",")
                val flags = parts[3]

                // Parse RGB colors
                val fgR = fgParts.getOrNull(0)?.toIntOrNull() ?: 255
                val fgG = fgParts.getOrNull(1)?.toIntOrNull() ?: 255
                val fgB = fgParts.getOrNull(2)?.toIntOrNull() ?: 255
                val bgR = bgParts.getOrNull(0)?.toIntOrNull() ?: 0
                val bgG = bgParts.getOrNull(1)?.toIntOrNull() ?: 0
                val bgB = bgParts.getOrNull(2)?.toIntOrNull() ?: 0

                // Map to Monet colors if standard ANSI, otherwise use true color
                val fgColor = rgbToAnsiIndex(fgR, fgG, fgB)?.let { getMonetColor(it) }
                    ?: Color.rgb(fgR, fgG, fgB)
                val bgColor = rgbToAnsiIndex(bgR, bgG, bgB)?.let { getMonetColor(it) }
                    ?: Color.rgb(bgR, bgG, bgB)

                val schemeBg = scheme?.background ?: 0xFF0D0D0D.toInt()

                // Draw background if not default
                if (bgColor != schemeBg && bgColor != Color.BLACK) {
                    bgPaint.color = bgColor
                    canvas.drawRect(
                        xPos * charWidth,
                        y * lineHeight,
                        (xPos + 1) * charWidth,
                        (y + 1) * lineHeight,
                        bgPaint
                    )
                }

                // Apply text styles
                fgPaint.color = fgColor
                fgPaint.isFakeBoldText = flags.contains('b')
                fgPaint.textSkewX = if (flags.contains('i')) -0.25f else 0f
                if (flags.contains('d')) {
                    fgPaint.alpha = 150
                } else {
                    fgPaint.alpha = 255
                }

                // Draw character
                if (char.isNotEmpty() && char != " ") {
                    canvas.drawText(
                        char,
                        xPos * charWidth,
                        (y + 1) * lineHeight - fgPaint.descent(),
                        fgPaint
                    )
                }
                xPos += if (flags.contains('w')) 2 else 1
            }
        }

        // Draw cursor with Monet primary color
        cursorPaint.color = scheme?.cursor ?: Color.WHITE
        cursorPaint.alpha = 180
        
        val cx = RinLib.getCursorX(engineHandle)
        val cy = RinLib.getCursorY(engineHandle)
        if (cx < cols && cy < rows) {
            canvas.drawRect(
                cx * charWidth,
                cy * lineHeight,
                (cx + 1) * charWidth,
                (cy + 1) * lineHeight,
                cursorPaint
            )
        }
    }
}
