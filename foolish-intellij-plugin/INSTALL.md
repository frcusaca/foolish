# Foolish IntelliJ Plugin

This plugin provides support for the Foolish language in IntelliJ IDEA, including syntax highlighting and Language Server Protocol (LSP) integration.

## Installation Instructions

1.  **Build the Plugin**:
    From the root of the repository, navigate to the `foolish-intellij-plugin` directory and run the Gradle build command:
    ```bash
    cd foolish-intellij-plugin
    ./gradlew buildPlugin
    ```
    (Note: You may need to grant execution permissions to `gradlew` with `chmod +x gradlew` if not already set).

    The build artifact will be created at:
    `foolish-intellij-plugin/build/distributions/foolish-intellij-plugin-1.0-SNAPSHOT.zip`

2.  **Install in IntelliJ IDEA**:
    *   Open IntelliJ IDEA.
    *   Go to **Settings** (or **Preferences** on macOS) -> **Plugins**.
    *   Click the **Gear icon** ⚙️ in the top right corner.
    *   Select **Install Plugin from Disk...**.
    *   Navigate to and select the `foolish-intellij-plugin-1.0-SNAPSHOT.zip` file generated in step 1.
    *   Click **OK**.
    *   **Restart** the IDE when prompted.

3.  **Verification**:
    *   Open a file with the `.foo` extension.
    *   You should see basic syntax highlighting (keywords colored).
    *   The LSP server should start automatically. You can check the **LSP Console** (View -> Tool Windows -> LSP Console) to see the server status and logs.

## Requirements

*   IntelliJ IDEA 2024.3 or later (Community or Ultimate).
*   Java 21 Runtime (included with IntelliJ usually, but the plugin requires Java 21+ features).
