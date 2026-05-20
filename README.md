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

**Rin Terminal** is a modern Android terminal emulator with a built-in package manager powered by the Termux ecosystem. Built with **Kotlin**, **Jetpack Compose**, and native **Rust** via JNI.

---

## Screenshots

<p align="center">
    <img src="doc/image/ss01.jpg" width="30%" />
    <img src="doc/image/ss02.jpg" width="30%" />
    <img src="doc/image/ss03.jpg" width="30%" />
</p>
<p align="center">
    <img src="doc/image/ss04.jpg" width="30%" />
    <img src="doc/image/ss05.jpg" width="30%" />
    <img src="doc/image/ss06.jpg" width="30%" />
</p>

---

## Features

- **Built-in Package Manager (`rpkg`)**  
  A lightweight, pacman-style package manager written in Rust that taps into the Termux repository ecosystem.  
  → [Full rpkg documentation](doc/package-manager.md)

- **Material 3 UI**  
  Clean, modern interface built with Jetpack Compose.

- **Fast & Minimal**  
  Native Rust binaries via JNI — lightweight architecture with minimal overhead.

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

- [Termux](https://termux.dev/) – The premier Android terminal emulator and Linux environment. `rpkg` leverages their incredible package repository ecosystem.

---

## License

Rin is open-sourced software licensed under the **MIT License**.  
See the [LICENSE](LICENSE) file for more information.
