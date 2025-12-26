
package com.rin

import android.content.Context
import android.graphics.Color
import android.graphics.Typeface
import android.os.Build
import android.os.Bundle
import android.view.Gravity
import android.view.ViewGroup
import android.view.WindowInsets
import android.view.WindowManager
import android.widget.Button
import android.widget.HorizontalScrollView
import android.widget.LinearLayout
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import android.util.TypedValue
import android.graphics.drawable.GradientDrawable

class MainActivity : AppCompatActivity() {
    private var engineHandle: Long = 0
    private lateinit var terminalView: TerminalView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Edge-to-edge
        WindowCompat.setDecorFitsSystemWindows(window, false)
        window.statusBarColor = Color.TRANSPARENT
        window.navigationBarColor = Color.TRANSPARENT
        
        // Root Layout
        val rootLayout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.MATCH_PARENT
            )
            setBackgroundColor(Color.BLACK)
        }

        ViewCompat.setOnApplyWindowInsetsListener(rootLayout) { view, windowInsets ->
            val systemBars = windowInsets.getInsets(WindowInsetsCompat.Type.systemBars())
            val ime = windowInsets.getInsets(WindowInsetsCompat.Type.ime())

            
            val bottomPadding = if (ime.bottom > 0) ime.bottom else systemBars.bottom
            
            view.setPadding(systemBars.left, systemBars.top, systemBars.right, bottomPadding)
            WindowInsetsCompat.CONSUMED
        }

        // Terminal View (Takes remaining space)
        terminalView = TerminalView(this).apply {
            layoutParams = LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                0,
                1f
            )
        }
        rootLayout.addView(terminalView)

        // Extra Keys Bar
        val keysBar = createExtraKeysBar()
        rootLayout.addView(keysBar)

        setContentView(rootLayout)

        // Initialize engine
        // Initial size will be updated in onSizeChanged of View
        engineHandle = RinLib.createEngine(80, 24, 14.0f)
        terminalView.attachEngine(engineHandle)
    }

    private fun createExtraKeysBar(): LinearLayout {
        val mainContainer = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT
            )
            setBackgroundColor(0xFF1E1E1E.toInt())
        }

        // Row 1 keys
        val row1Keys = listOf(
            "ESC" to "\u001b",
            "/" to "/",
            "-" to "-",
            "HOME" to "\u001b[H",
            "▲" to "\u001b[A",
            "END" to "\u001b[F",
            "PGUP" to "\u001b[5~"
        )

        // Row 2 keys
        val row2Keys = listOf(
            "TAB" to "\t",
            "CTRL" to "CTRL",
            "ALT" to "ALT",
            "◄" to "\u001b[D",
            "▼" to "\u001b[B",
            "►" to "\u001b[C",
            "PGDN" to "\u001b[6~"
        )

        mainContainer.addView(createKeyRow(row1Keys))
        mainContainer.addView(createKeyRow(row2Keys))

        return mainContainer
    }

    private fun createKeyRow(keys: List<Pair<String, String>>): HorizontalScrollView {
        val scrollView = HorizontalScrollView(this).apply {
            layoutParams = LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT
            )
            isHorizontalScrollBarEnabled = false
            isFillViewport = true // Important for layout distribution
        }

        val rowContainer = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            layoutParams = android.widget.FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT
            )
        }

        val displayMetrics = resources.displayMetrics
        val screenWidth = displayMetrics.widthPixels
        val marginPx = 4 // Small margin
        // We have 7 keys. Calculate width to fit them exactly.
        val buttonWidth = (screenWidth / 7) - (marginPx * 2)

        for ((label, code) in keys) {
            val button = Button(this).apply {
                text = label
                isAllCaps = true
                typeface = Typeface.MONOSPACE
                textSize = 12f 
                setTextColor(Color.WHITE)
                
                val bg = GradientDrawable().apply {
                    setColor(0xFF424242.toInt())
                    cornerRadius = 12f
                    setStroke(1, 0xFF616161.toInt())
                }
                background = bg
                
                minWidth = 0
                minimumWidth = 0
                includeFontPadding = false
                stateListAnimator = null
                
                layoutParams = LinearLayout.LayoutParams(
                    buttonWidth, // Fixed calculated width
                    TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_DIP, 42f, displayMetrics).toInt()
                ).apply {
                    setMargins(marginPx, 6, marginPx, 6)
                }
                
                setPadding(0, 0, 0, 0)

                setOnClickListener {
                    if (label == "CTRL") {
                        terminalView.ctrlPressed = !terminalView.ctrlPressed
                        val newBg = GradientDrawable().apply {
                             setColor(if (terminalView.ctrlPressed) 0xFF00ACC1.toInt() else 0xFF424242.toInt())
                             cornerRadius = 12f
                             setStroke(1, 0xFF616161.toInt())
                        }
                        background = newBg
                        setTextColor(Color.WHITE)
                    } else {
                        if (label != "ALT") {
                            sendInput(code)
                        }
                    }
                }
            }
            rowContainer.addView(button)
        }

        scrollView.addView(rowContainer)
        return scrollView
    }

    private fun sendInput(data: String) {
        if (engineHandle != 0L) {
            RinLib.write(engineHandle, data.toByteArray())
            terminalView.invalidate()
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        if (engineHandle != 0L) {
            RinLib.destroyEngine(engineHandle)
            engineHandle = 0
        }
    }
}
