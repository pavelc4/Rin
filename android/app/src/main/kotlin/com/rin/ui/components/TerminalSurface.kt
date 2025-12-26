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

@Composable
fun TerminalSurface(
    engineHandle: Long,
    ctrlPressed: Boolean,
    modifier: Modifier = Modifier,
    onInput: (ByteArray) -> Unit = {}
) {
    var fontSize by remember { mutableFloatStateOf(18f) }
    var ctrlState by remember { mutableStateOf(ctrlPressed) }
    
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
            }
        },
        modifier = modifier,
        update = { view ->
            view.engineHandle = engineHandle
            view.fontSize = fontSize
            view.onInputCallback = onInput
            view.ctrlPressedProvider = { ctrlState }
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

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        canvas.drawColor(0xFF0D0D0D.toInt())

        if (engineHandle == 0L) return

        // Draw text
        for (y in 0 until rows) {
            val line = RinLib.getLine(engineHandle, y)
            if (line.isNotEmpty()) {
                canvas.drawText(
                    line,
                    0f,
                    (y + 1) * lineHeight - textPaint.descent(),
                    textPaint
                )
            }
        }

        // Draw cursor
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
