<h1 align="center">
    Rin Terminal
</h1>

<p align="center">
    <img src="https://img.shields.io/badge/Kotlin-7F52FF?style=for-the-badge&colorA=363A4F&logo=kotlin&logoColor=D9E0EE">
    <img src="https://img.shields.io/badge/Jetpack%20Compose-7F52FF?style=for-the-badge&colorA=363A4F&logo=jetpack-compose&logoColor=D9E0EE">
    <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&colorA=363A4F&logo=rust&logoColor=D9E0EE">
    <img src="https://img.shields.io/badge/Android-3DDC84?style=for-the-badge&colorA=363A4F&logo=android&logoColor=D9E0EE">
</p>

---

## About

**Rin Terminal** is a modern Android terminal emulator built with **Kotlin** and **Jetpack Compose**, integrated with a native pacman-style package manager written in **Rust**.

It provides **seamless terminal access and robust package management** on Android 10+ devices, utilizing innovative execution bypasses to resolve modern Android restrictions, while prioritizing **clarity, performance, and minimal system overhead**.

---

## Features

- **Built-in Package Manager (`rpkg`)**  
  A lightweight, pacman-style (`-S`, `-R`, etc.) package manager written in Rust, utilizing the vast Termux repository ecosystem directly within the app.

- **Multicall Binary & Library Support**  
  Intelligent proxy injection supports complex multicall binaries (like `coreutils`, `busybox`, `toybox`) and dynamic `.so` libraries right out of the box, seamlessly bypassing Android's W^X execution restrictions.

- **Material 3 UI**  
  Clean, modern, and highly responsive interface built natively with Jetpack Compose.

- **Fast & Minimal**  
  Lightweight architecture natively binding Rust binaries via JNI.

---

## Development Setup

### Prerequisites

- Android Studio (latest version)
- Android SDK (API 31+)
- Rust toolchain with `cargo-ndk` installed
- NDK 28.2.13676358 (or compatible version)

### Local Development

1. Clone the repository:

   ```bash
   git clone https://github.com/pavelc4/Rin
   cd Rin
   ```

2. Install `cargo-ndk` (required for building Rust libraries for Android):

   ```bash
   cargo install cargo-ndk
   ```

3. Set up Android NDK environment variable:

   **Linux/macOS:**
   ```bash
   export ANDROID_NDK_HOME=/path/to/your/android/sdk/ndk/28.2.13676358
   ```

   **Windows:**
   ```cmd
   set ANDROID_NDK_HOME=C:\Users\YourName\AppData\Local\Android\Sdk\ndk\28.2.13676358
   ```

4. Build the Rust JNI binary and compile the APK:

   **Linux/macOS:**
   ```bash
   ./build_android.sh
   cd android
   ./gradlew assembleDebug
   ```

   **Windows:**
   ```cmd
   build_android.bat
   cd android
   gradlew.bat assembleDebug
   ```

### Security Notice

- `local.properties` and `gradle.properties` are gitignored for security
- Keystore files are never committed to the repository
- All builds are reproducible and verifiable

---

## Requirements

- **Android Version** – Android 10 (API 29) or above
- **Architecture** – ARM64 (`aarch64`)

---

## Credits

- [Termux](https://termux.dev/) - The premier Android terminal emulator and Linux environment app that pioneered modern terminal capabilities on Android. `rpkg` leverages their incredible package repository ecosystem.

---

## License

Rin is open-sourced software licensed under the **MIT License**.  
See the [LICENSE](LICENSE) file for more information.
