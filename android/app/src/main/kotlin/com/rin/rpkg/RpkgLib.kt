package com.rin.rpkg

object RpkgLib {
    init {
        try {
            System.loadLibrary("rpkg")
        } catch (e: UnsatisfiedLinkError) {
            e.printStackTrace()
        }
    }

    external fun execute(prefix: String, op: String, args: String): String
}
