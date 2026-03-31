# IOTA PTB Viewer - Android Demo

A pure-Rust Android app that fetches and displays recent Programmable Transaction Blocks (PTBs)
from the IOTA mainnet. Built with [Freya](https://github.com/marc2332/freya) GUI framework and
[iota-sdk](https://github.com/iotaledger/iota-rust-sdk).

> **Note:** Freya's Android support is experimental. This demo uses git dependencies pointing
> to [PR #1731](https://github.com/marc2332/freya/pull/1731) which adds the `freya-android`
> crate with soft keyboard and status bar support. Once a future Freya release includes
> Android support, the dependencies can be switched to crates.io versions.

## Features

- Live-updating list of recent PTBs from IOTA mainnet
- Shows transaction digest, sender address, Move function calls, and command summary
- Tap any transaction to see full details
- Dark theme optimized for mobile
- Also runs as a desktop app for development/testing

## Prerequisites

### 1. Android SDK (API level 36)

Install [Android Studio](https://developer.android.com/studio) or the
[command-line tools](https://developer.android.com/studio#command-line-tools-only).

```sh
export ANDROID_HOME="$HOME/Android/Sdk"  # typical Linux path
```

### 2. Android NDK (r26d)

Download from https://developer.android.com/ndk/downloads and extract:

```sh
# Linux
wget https://dl.google.com/android/repository/android-ndk-r26d-linux.zip
unzip android-ndk-r26d-linux.zip

export ANDROID_NDK_HOME="/path/to/android-ndk-r26d"
export ANDROID_NDK="$ANDROID_NDK_HOME"
```

### 3. Rust Android targets

```sh
rustup target add aarch64-linux-android
```

### 4. cargo-ndk

```sh
cargo install cargo-ndk
```

### 5. Gradle wrapper

The `AndroidApp/` directory needs the Gradle wrapper. If `gradlew` is not present, generate it:

```sh
cd AndroidApp/
gradle wrapper --gradle-version 9.2.1
```

Or download `gradlew` + `gradle-wrapper.jar` from the
[Gradle distributions](https://services.gradle.org/distributions/).

## Running on Desktop (for development)

No Android toolchain needed:

```sh
# From the iota-rust-sdk root:
cargo run -p android-demo --bin iota-ptb-viewer-desktop
```

This opens a 420x800 window simulating a phone screen.

## Building the Android APK

### Option A: Android Studio

1. Open `AndroidApp/` in Android Studio
2. Connect your phone via USB with USB debugging enabled
3. Click **Run**

### Option B: Command line

```sh
cd AndroidApp/

# Build debug APK (this also builds the Rust library via the buildRustLibrary task)
./gradlew assembleDebug
```

The APK will be at: `AndroidApp/app/build/outputs/apk/debug/app-debug.apk`

## Installing on a phone via USB

1. Enable **Developer Options** on your phone:
   - Go to Settings > About phone > tap "Build number" 7 times
2. Enable **USB debugging**:
   - Settings > Developer options > USB debugging
3. Connect the phone via USB and authorize the connection
4. Install the APK:

```sh
adb install AndroidApp/app/build/outputs/apk/debug/app-debug.apk
```

5. Launch "IOTA PTB Viewer" from the app drawer

## Troubleshooting

### `cargo ndk` fails with linker errors

Make sure `ANDROID_NDK_HOME` and `ANDROID_NDK` point to the NDK r26d directory, and that you have
the `aarch64-linux-android` target installed (`rustup target list --installed`).

### App crashes on launch with `RegisterNatives failed`

The Java `games-activity` version in `AndroidApp/gradle/libs.versions.toml` must match the
Rust `android-activity` crate version. Mismatched versions cause a JNI `RegisterNatives` failure
and an immediate crash. The mapping is:

| Rust `android-activity` | Java `games-activity` |
|-------------------------|-----------------------|
| 0.6.x                   | 3.0.5                |
| 0.5.x                   | 2.0.2                |

After updating either side, rebuild both the Rust library and the APK.

### App crashes on launch (other)

Check logcat for errors:

```sh
adb logcat -s RustStdoutStderr:D android_example:D
```

### Gradle can't find SDK

Ensure `ANDROID_HOME` is set, or create `AndroidApp/local.properties`:

```properties
sdk.dir=/path/to/Android/Sdk
```

## Architecture

```
src/
  lib.rs    - Android entry point (android_main via GameActivity/JNI)
  main.rs   - Desktop entry point (standard main function)
  app.rs    - Shared app logic: UI components + transaction fetching
```

The app uses Freya's component model (based on Dioxus signals) with:
- `iota_sdk::graphql_client::Client` for GraphQL queries to IOTA mainnet
- Async polling every 5 seconds for new transactions
- `VirtualScrollView`-style list for efficient rendering
