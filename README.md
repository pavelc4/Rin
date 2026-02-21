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
  A lightweight, pacman-style (`-S`, `-R`, dll) package manager written in Rust, utilizing the vast Termux repository ecosystem directly within the app.

- **W^X Execution Bypass**  
  Natively bypasses Android 10+ execute security restrictions (W^X) using intelligent JNI proxy wrappers, dynamic symbolic links, and ELF `.interp` patching, enabling standard Linux binaries and shell scripts to run smoothly without root.

- **Multicall Binary Support**  
  Intelligent proxy injection supports complex multicall binaries (like `coreutils`, `busybox`, `toybox`) and dynamic `.so` libraries right out of the box.

- **Material 3 UI**  
  Clean, modern, and highly responsive interface built natively with Jetpack Compose.

- **Fast & Minimal**  
  Lightweight architecture natively binding Rust binaries via JNI, optimized for speed.

---

## Development Setup

### Prerequisites

- Android Studio (latest version)
- Android SDK (API 31+)
- Rust toolchain (with `aarch64-linux-android` target)
- NDK 26.1.10909125

### Local Development

1. Clone the repository:

   ```bash
   git clone https://github.com/YourName/Rin.git
   cd Rin
   ```

2. Setup configuration files:

   ```bash
   cp local.properties.template local.properties
   cp gradle.properties.template gradle.properties
   ```

3. Edit `local.properties` with your Android SDK and NDK paths:

   ```properties
   sdk.dir=/path/to/your/android/sdk
   ndk.dir=/path/to/your/android/ndk/26.1.10909125
   ```

4. Build the Rust JNI binary via cargo and compile the APK:
   ```bash
   ./build_android.sh
   cd android
   ./gradlew assembleDebug
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

## License

Rin is open-sourced software licensed under the **MIT License**.  
See the [LICENSE](LICENSE) file for more information.
