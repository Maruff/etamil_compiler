# eTamil for Android

Write an eTamil program on a phone, run it, read what it printed.

The app carries the real compiler — the same Rust crate the `etamil` command
line is built from, cross-compiled for Android and loaded through JNI. A
diagnostic here is the diagnostic `etamil --check` prints, bilingual text
included, because it comes from the same `Display` implementation. Nothing is
reimplemented in Kotlin.

Unlike the browser build, which gets the front end only, a phone gets the whole
language: `vm`, `module`, the bundled SQLite driver and the HTTP client are all
present. A program here can `இறக்கு` a module beside it, open a table and call
an API.

## You do not need Android Studio

Android Studio is an IDE. The things that build an APK are the SDK's
command-line tools, the NDK and Gradle — and none of them needs it. There are
two ways to get an APK, and the first installs nothing at all.

### 1. Build it in CI (nothing on your machine)

Push, then **Actions → Android app → Run workflow**. The APK arrives as a run
artifact named `etamil-android`.

```
Actions -> Android app -> Run workflow -> Run workflow
```

Download `etamil-android`, unzip it, and you have `app-debug.apk`. Install it by
copying it to a phone and opening it — Android will ask once for permission to
install from this source — or with `adb install app-debug.apk` if you happen to
have platform-tools.

The workflow is [`.github/workflows/android.yml`](../.github/workflows/android.yml).
It runs two jobs:

| Job | Cost | When | What it proves |
| --- | --- | --- | --- |
| `bridge` | ~1 min | every PR touching the app or the compiler | the JNI bridge still compiles against the compiler's API |
| `apk` | minutes | on demand, and on a `v*` tag | the whole thing cross-compiles and packages |

`bridge` type-checks `android/rust` against the **host** toolchain. The `jni`
crate builds anywhere, so a renamed function in `etamil_compiler` or a wrong
field name fails there in a minute rather than twenty minutes into an NDK build.

### 2. Build it locally, still without the IDE

Worth it only if you are iterating on the app itself; a round trip through CI is
slow when you are changing a button.

You need three things, none of which is an installer:

- **A JDK, 17 or newer.** `winget install EclipseAdoptium.Temurin.17.JDK`, or
  whatever your platform's package manager calls it.
- **The Android SDK command-line tools.** The "Command line tools only" zip from
  <https://developer.android.com/studio#command-line-tools-only>. Unzip it, then
  let `sdkmanager` fetch the two pieces the build actually needs:

  ```bash
  sdkmanager "platforms;android-35" "build-tools;35.0.0" "platform-tools"
  ```

- **The NDK**, which is what supplies the clang that links for each ABI:

  ```bash
  sdkmanager "ndk;27.2.12479018"
  ```

Point Gradle at the SDK with `android/local.properties` (git-ignored):

```properties
sdk.dir=C:\\Android\\sdk
```

Then, once:

```bash
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk --locked
```

And to build:

```bash
cd android/rust && cargo ndk -t arm64-v8a -o ../app/src/main/jniLibs build --release
```

```bash
cd android && gradle assembleDebug
```

The whole footprint is two unzipped folders and a Rust target. Deleting them
removes it; nothing registered anything.

## How it fits together

```
android/rust/src/lib.rs        JNI entry points -> JSON        (Rust)
        |                       links etamil_compiler by path
        v
libetamil_android.so           one per ABI, built by cargo-ndk
        |                       into app/src/main/jniLibs/<abi>/
        v
app/.../Etamil.kt              external fun declarations, JSON -> data classes
        |
        v
app/.../MainActivity.kt        editor, Run, Check, output pane
```

Two build tools, and neither drives the other: **cargo-ndk runs first** and drops
its `.so` files into `app/src/main/jniLibs`, where Gradle finds them as an
ordinary prebuilt library. Gradle never invokes cargo. This is a deliberate
choice over the usual Gradle-calls-cargo plugin — two tools each believing they
own the other's output is a harder problem to debug than one extra CI step.

### The bridge is a separate crate

`android/rust` is its own crate rather than a module of `etamil_compiler`,
because `jni` would otherwise join the compiler's dependency graph on every
platform and be compiled, for nothing, by everyone running `cargo build` on a
laptop. The wasm bindings can live inside the crate because `wasm-bindgen` is
declared per-target and vanishes from a native build; there is no equivalent
trick for a dependency that has to exist on the host too.

### Capture mode

The one change this app needed inside the compiler is in
[`etamil_compiler/src/vm/host.rs`](../etamil_compiler/src/vm/host.rs).

Android is a native build, so it takes that module's native half — where `அச்சு`
is a `println!` to a stdout nobody reads, `உள்ளிடு` blocks on a stdin that never
produces a byte, and `வெளியேறு` calls `std::process::exit`, which in an app ends
the app. The browser half had already solved all three, but a phone wants the
rest of the native build (real files, real sockets), so it cannot simply borrow
it.

So the native half gained a capture mode: output accumulates in a thread-local
buffer, input is supplied up front, and a non-zero exit is reported rather than
performed. It is off unless `begin_capture` is called, and nothing in the
compiler calls it — the command line, the REPL and the servers behave exactly as
they did.

It is covered by [`tests/host_capture.rs`](../etamil_compiler/tests/host_capture.rs),
which runs in the ordinary suite on any platform. That is the point: the app
cannot be built on a machine without an NDK, but everything the app *depends on*
can be tested on one.

## Pinned versions

Version drift between these four is the most likely cause of a build that worked
last week and does not today, so each is exact rather than a range.

| What | Version | Where | Has to agree with |
| --- | --- | --- | --- |
| Android Gradle Plugin | 8.7.3 | `build.gradle.kts` | Gradle, and the JDK |
| Gradle | 8.10.2 | `.github/workflows/android.yml` | AGP |
| Kotlin | 2.0.21 | `build.gradle.kts` | AGP |
| NDK | r27c | `.github/workflows/android.yml` | the Rust targets |

`compileSdk` and `targetSdk` are 35; `minSdk` is 24, which is Android 7.0.

There is no committed Gradle wrapper. A wrapper means committing
`gradle-wrapper.jar`, a binary nobody can review in a diff, and the version it
would pin is named in the workflow instead. If you want one, run `gradle wrapper
--gradle-version 8.10.2` in this directory once and commit what it writes.

## ABIs

`arm64-v8a` is every Android phone sold since roughly 2017. `x86_64` is what an
emulator runs, so a reviewer can install the artifact without owning a device.
Both are built every time.

`armeabi-v7a` — 32-bit ARM, still a great many cheap and older handsets — is
added only for a `v*` tag, because each ABI is a full build of the compiler and
its dependency tree, and a third one costs a third more wall clock for hardware
nobody is testing on day to day.

## Signing

The debug APK is signed with Gradle's automatic debug key: installable for
testing, not publishable. A release APK is built on a `v*` tag only if a signing
key is configured, because an unsigned release APK cannot be installed at all —
building one anyway would produce an artifact whose only use is to disappoint
whoever downloads it.

To enable it, add four repository secrets:

| Secret | What |
| --- | --- |
| `ETAMIL_KEYSTORE_BASE64` | the keystore file, base64-encoded |
| `ETAMIL_KEYSTORE_PASSWORD` | its password |
| `ETAMIL_KEY_ALIAS` | the key's alias inside it |
| `ETAMIL_KEY_PASSWORD` | that key's password |

## What the app does not do yet

- **No syntax highlighting, and no underlining of errors.** `Check` reports every
  diagnostic with its line and column, and the data to underline them is already
  there — `nativeDiagnostics` returns a length per diagnostic — but the editor is
  a plain `EditText` and does not use it.
- **No `--server`.** `வழி` routes parse and type-check, but nothing here starts
  a server, and an app holding a listening socket is a different kind of thing to
  reason about.
- **Only the bundled examples.** There is no way to open a file from elsewhere on
  the phone, and `nUlakam` is not shipped inside the APK, so `இறக்கு` can only
  reach files the app has unpacked into its own directory.
- **A placeholder icon.** `res/drawable/ic_launcher.xml` is geometry, not a
  letterform. See the comment in it for why.
