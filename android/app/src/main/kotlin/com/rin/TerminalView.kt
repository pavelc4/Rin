package com.rin

import android.content.Context
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.Typeface
import android.util.AttributeSet
import android.view.View
import android.util.TypedValue

class TerminalView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = 0
) : View(context, attrs, defStyleAttr) {

    init {
        isFocusable = true
        isFocusableInTouchMode = true
    }

    private var engineHandle: Long = 0
    private val textPaint = Paint().apply {
        color = Color.WHITE
        // Convert 14sp to pixels
        textSize = TypedValue.applyDimension(
            TypedValue.COMPLEX_UNIT_SP,
            14f,
            context.resources.displayMetrics
        )
        typeface = Typeface.MONOSPACE
        isAntiAlias = true
    }
    private val charWidth: Float
        get() = textPaint.measureText("W")
    
    private val lineHeight: Float
        get() = textPaint.fontSpacing

    private var rows = 0

    private var cols = 0

    fun attachEngine(handle: Long) {
        this.engineHandle = handle
        // Force resize if we already have dimensions
        if (width > 0 && height > 0) {
            onSizeChanged(width, height, 0, 0)
        }
        invalidate()
    }

    private val scaleDetector = android.view.ScaleGestureDetector(context, object : android.view.ScaleGestureDetector.SimpleOnScaleGestureListener() {
        override fun onScale(detector: android.view.ScaleGestureDetector): Boolean {
            val scaleFactor = detector.scaleFactor
            val newSize = textPaint.textSize * scaleFactor
            // Clamp size between 8sp and 64sp in pixels
            val minSize = TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_SP, 8f, resources.displayMetrics)
            val maxSize = TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_SP, 64f, resources.displayMetrics)
            
            if (newSize in minSize..maxSize) {
                textPaint.textSize = newSize
                // Trigger resize
                if (width > 0 && height > 0) {
                     cols = (width / charWidth).toInt().coerceAtLeast(1)
                     rows = (height / lineHeight).toInt().coerceAtLeast(1)
                     if (engineHandle != 0L) {
                         RinLib.resize(engineHandle, cols, rows)
                         invalidate()
                     }
                }
            }
            return true
        }
    })

    override fun onTouchEvent(event: android.view.MotionEvent): Boolean {
        scaleDetector.onTouchEvent(event)
        if (scaleDetector.isInProgress) return true
        
        if (event.action == android.view.MotionEvent.ACTION_DOWN) {
            requestFocus()
            val imm = context.getSystemService(Context.INPUT_METHOD_SERVICE) as android.view.inputmethod.InputMethodManager
            imm.showSoftInput(this, android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT)
        }
        return true
    }

    var ctrlPressed: Boolean = false

    override fun onCreateInputConnection(outAttrs: android.view.inputmethod.EditorInfo): android.view.inputmethod.InputConnection {
        outAttrs.inputType = android.text.InputType.TYPE_CLASS_TEXT or android.text.InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
        outAttrs.imeOptions = android.view.inputmethod.EditorInfo.IME_ACTION_NONE

        return object : android.view.inputmethod.BaseInputConnection(this, false) {
            override fun commitText(text: CharSequence, newCursorPosition: Int): Boolean {
                if (engineHandle != 0L) {
                    val input = text.toString()
                    if (ctrlPressed && input.isNotEmpty() && input[0].lowercaseChar() in 'a'..'z') {
                         // Convert to control char: 'a' (97) -> 1, 'z' (122) -> 26
                         val charCode = input[0].lowercaseChar().code - 96
                         RinLib.write(engineHandle, byteArrayOf(charCode.toByte()))
                    } else {
                        RinLib.write(engineHandle, input.toByteArray())
                    }
                    invalidate()
                }
                return true
            }

            override fun sendKeyEvent(event: android.view.KeyEvent): Boolean {
                if (event.action == android.view.KeyEvent.ACTION_DOWN) {
                    if (event.keyCode == android.view.KeyEvent.KEYCODE_ENTER) {
                        RinLib.write(engineHandle, "\r".toByteArray())
                        return true
                    }
                    if (event.keyCode == android.view.KeyEvent.KEYCODE_DEL) {
                        RinLib.write(engineHandle, byteArrayOf(0x08))
                        return true
                    }
                }
                return super.sendKeyEvent(event)
            }
            
            override fun deleteSurroundingText(beforeLength: Int, afterLength: Int): Boolean {
                 if (beforeLength == 1 && afterLength == 0) {
                     RinLib.write(engineHandle, byteArrayOf(0x08))
                     return true
                 }
                 return super.deleteSurroundingText(beforeLength, afterLength)
            }
        }
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        if (engineHandle != 0L && w > 0 && h > 0) {
            cols = (w / charWidth).toInt().coerceAtLeast(1)
            rows = (h / lineHeight).toInt().coerceAtLeast(1)
            RinLib.resize(engineHandle, cols, rows)
        }
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        canvas.drawColor(Color.BLACK) // Background

        if (engineHandle == 0L) return

        for (y in 0 until rows) {
            val line = RinLib.getLine(engineHandle, y)
            if (line.isNotEmpty()) {
                canvas.drawText(line, 0f, (y + 1) * lineHeight - textPaint.descent(), textPaint)
            }
        }

        // Draw cursor (simple block)
        val cx = RinLib.getCursorX(engineHandle)
        val cy = RinLib.getCursorY(engineHandle)
        if (cx < cols && cy < rows) {
            val cursorX = cx * charWidth
            val cursorY = cy * lineHeight
            textPaint.alpha = 128
            canvas.drawRect(cursorX, cursorY, cursorX + charWidth, cursorY + lineHeight, textPaint)
            textPaint.alpha = 255
        }
        postInvalidateDelayed(16)
    }
}
