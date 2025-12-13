plugins {
    id("java")
    id("org.jetbrains.intellij.platform") version "2.2.1"
}

group = "org.foolish"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    intellijPlatform {
        intellijIdeaCommunity("2024.3")

        // Use LSP4IntelliJ for LSP support
        plugin("com.redhat.devtools.lsp4ij:0.4.0")
    }
}

intellijPlatform {
    pluginConfiguration {
        id = "org.foolish.intellij"
        name = "Foolish Language"
        vendor {
            name = "Foolish Org"
        }
        description = """
            Support for the Foolish language.
            Includes syntax highlighting and LSP support.
        """

        ideaVersion {
            sinceBuild = "243"
            untilBuild = "251.*"
        }
    }
}

tasks {
    buildSearchableOptions {
        enabled = false
    }

    // Task to copy the LSP jar from the Maven build
    val copyLspServer = register<Copy>("copyLspServer") {
        from("../foolish-lsp-java/target/foolish-lsp-java-1.0-SNAPSHOT.jar")
        into("$buildDir/lsp-server") // Temporary location
        rename { "foolish-lsp.jar" }
        onlyIf { file("../foolish-lsp-java/target/foolish-lsp-java-1.0-SNAPSHOT.jar").exists() }
    }

    prepareSandbox {
        dependsOn(copyLspServer)
        from("$buildDir/lsp-server") {
            into("foolish-intellij-plugin/server") // Place in the plugin root
        }
    }
}
