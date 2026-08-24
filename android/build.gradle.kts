// Plugin versions are pinned here and resolved in `app/build.gradle.kts`.
//
// `apply false` means this file only states which versions exist; the app module
// decides which to apply. With one module that is a distinction without a
// difference, but it is the layout every Android project expects, and a second
// module later costs nothing.
//
// Bumping either of these is the single most likely cause of a build that
// worked yesterday and does not today, so they are exact versions rather than
// ranges. See android/README.md for what each one has to agree with.
plugins {
    id("com.android.application") version "8.7.3" apply false
    id("org.jetbrains.kotlin.android") version "2.0.21" apply false
}
