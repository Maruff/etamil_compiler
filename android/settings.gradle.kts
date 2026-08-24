// The Android app is its own Gradle build, rooted here rather than at the
// repository root: the repository is a Rust project with a Gradle app in it,
// not the other way round, and a settings file at the top would have Gradle
// scanning `etamil_compiler/` and `nUlakam/` for projects it will never find.

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    // Refuse a dependency declared anywhere but here. The default — letting a
    // module add its own repository — is how an APK ends up carrying an
    // artifact from somewhere nobody chose.
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "eTamil"
include(":app")
