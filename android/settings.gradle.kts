pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
    plugins {
        id("com.android.application") version "9.0.0"
        id("org.jetbrains.kotlin.android") version "2.3.10"
        id("org.jetbrains.kotlin.plugin.compose") version "2.3.10"
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "RinAndroid"
include(":app")
