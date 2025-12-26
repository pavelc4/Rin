package com.rin

object RinLib {
    init {
        System.loadLibrary("rin")
    }

    external fun createEngine(width: Int, height: Int, fontSize: Float, homeDir: String, username: String): Long
    external fun destroyEngine(handle: Long)
    external fun write(handle: Long, data: ByteArray): Int
    external fun render(handle: Long): Int
    external fun resize(handle: Long, width: Int, height: Int): Int
    external fun getLine(handle: Long, y: Int): String
    external fun getCursorX(handle: Long): Int
    external fun getCursorY(handle: Long): Int
}
