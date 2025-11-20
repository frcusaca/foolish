# Foolish LSP Work Document

This document captures the working plan for building a Java-based Language Server Protocol (LSP) implementation for Foolish, along with IDE integration notes.

## Current implementation status

- ✅ **Step 1 – Bootstrap module:** A dedicated `foolish-lsp-java` Maven module now lives alongside the existing Foolish build. It reuses the runtime/AST classes published by the root project and ships a shaded `foolish-lsp-java.jar` hosting a stub `FoolishLanguageServer` so editors can already spawn the binary for smoke testing.
- 🔜 **Next steps:** Gradually replace the placeholder document/workspace services with real parsing, diagnostics, and formatting logic from the core module per the roadmap below.

## Architecture overview

### Existing building blocks
- Foolish programs live in `.foo` files composed of nested *branes*. Computation derives from proximity (adjacent branes) while containment organizes values. Editors must surface this mental model.
- Concatenation is adjacency in reverse-Polish order (RPN), which is how functions, derivations, and OO-style extension are expressed. See `samples/hello.foo` for a canonical example that concatenates `{1,2}` with `myProduct` and dereferences via `^`, `$`, and `#`.
- Brane search operators (`.`, `$`, `#`, `/`, `//`, `??`) are fundamental language constructs, so any tooling roadmap must include understanding of these postfix search paths.
- The repo ships an ANTLR4 grammar, AST model, AST builder, and formatter. These are the ready-made parsing and pretty-printing components to embed inside the LSP server.
- Runtime semantics are modeled through the FVM environment (`Env`, `CharacterizedIdentifier`) and the Unicellular Brane Computer (UBC), so we can reuse the same scope-resolution logic for tooling features like go-to-definition, diagnostics, or live evaluation.

### Proposed Java LSP architecture
1. **Language server process** – Create a new Maven submodule (e.g., `foolish-lsp-java`) that depends on the existing `antlr4-runtime`, AST, and FVM code. Use `org.eclipse.lsp4j` to implement `FoolishLanguageServer` and package it into an executable JAR (`java -jar foolish-lsp-java.jar`).
2. **Document & workspace services** – Maintain an in-memory workspace of `.foo` documents with incremental text updates, file-system watchers for on-disk changes, and dependency tracking for multi-file brane compositions.
3. **Parsing pipeline** – On each change, run the ANTLR lexer/parser (`Foolish.g4`) and build an AST via `ASTBuilder`, caching results and emitting syntax diagnostics through LSP `PublishDiagnostics`. Future enhancements can expose the parse tree for folding or outline views.
4. **Semantic services** – Reuse `Env`/`CharacterizedIdentifier` to build semantic tables per brane, tracking SSA-style bindings so definitions, references, and search operators resolve consistently with runtime semantics.
5. **Evaluation hook (optional)** – Wrap the UBC (`UnicellularBraneComputer`) so the LSP can simulate execution or step through NYE/FK states, enabling code lenses like “Evaluate brane.”
6. **Formatting & code actions** – Surface the existing `ASTFormatter` via the `textDocument/formatting` handler and build code actions (wrap statements into a brane, normalize characterizations) by manipulating AST nodes.
7. **Search-aware navigation UI** – Model brane search operators as first-class commands so editor clients can expose palette entries or keybindings that match the language’s navigation semantics.

### Build & publishing workflow
1. **Server build** – Extend Maven to produce an LSP-specific shaded JAR (using `maven-shade-plugin`) bundling lsp4j plus the grammar/runtime.
2. **Versioning** – Tag releases in GitHub and publish the JAR as a release asset; optionally push to Maven Central so JetBrains plugins can depend on it.
3. **VS Code client** – Create a TypeScript extension using `vscode-languageclient` that launches the bundled JAR. Publish via `vsce`/`npm`.
4. **IntelliJ client** – Use JetBrains’ LSP API (`com.intellij.platform.lsp`) to launch the same JAR, declare `.foo` as a language, and publish via JetBrains Marketplace. Alternatively, wrap the Java logic directly into a plugin module later for tighter integration.

### Feature roadmap
| Phase | Scope | Key deliverables |
| --- | --- | --- |
| **0 – Infrastructure** | Wiring, diagnostics | Document sync, incremental parsing, syntax errors from ANTLR, AST caching, formatting via `ASTFormatter`. |
| **1 – MVP** | Navigation & authoring | Hover showing canonical characterizations, go-to-definition, find references by replaying `Env` scopes, outline view of branes/assignments, RPN snippets. |
| **2 – Search & evaluation** | Foolish-specific semantics | Search-aware completions (`.`, `$`, `#`, `/`, `//`, `??`), brane coordinate previews, optional inline evaluation via the UBC. |
| **3 – Extended ecosystem** | Advanced tooling | Refactoring/traceability tooling, AI-assisted generation, mutable brane editing commands, or visualization of relational coordinates. |

## Development notes

> **Build alignment:** Keep every toolchain Maven-first (server, CLI launcher, VS Code assets, IntelliJ plugin). Run the normal `mvn install` at repo root to publish the latest `org.foolish:foolish` snapshot locally. The LSP module is now part of the root Maven reactor, so a standard `mvn verify` or `mvn install` from the repo root will build and test both the core and `foolish-lsp-java`. Avoid introducing Gradle just for plugin packaging.

### Building the LSP server
1. Add a Maven module `foolish-lsp-java` with dependencies on the core runtime, AST, and ANTLR artifacts already defined in the root `pom.xml`. **Status:** ✅ done – see `foolish-lsp-java/pom.xml`.
2. Implement `FoolishLanguageServer` using LSP4J and expose a `main` entry point.
3. Configure the module’s POM with:
   - `maven-compiler-plugin` targeting Java 21 (aligned with the repo).
   - `maven-shade-plugin` to produce a standalone `foolish-lsp-java.jar`.
4. Build flow:
   - Run `mvn verify` (or `mvn install`) at the repo root to build/tests both modules and shade the server. Locate the artifact under `foolish-lsp-java/target/`.

### Loading into VS Code for testing
1. Scaffold a VS Code extension (TypeScript) using `yo code` or manual setup.
2. Add `vscode-languageclient` and configure the client to spawn `java -jar server/foolish-lsp-java.jar` (bundle the jar or download on activation).
3. Use the `Run Extension` task in VS Code to open a new Extension Development Host window and verify features across `.foo` files.
4. Package for testers via `vsce package` and share the `.vsix`.

### IntelliJ IDEA development workflow
1. **Plugin setup**
   - Keep the toolchain Maven-first: scaffold an IntelliJ Platform plugin project that uses the [`intellij-platform-maven-plugin`](https://plugins.jetbrains.com/docs/intellij/tools-maven.html) so it aligns with the rest of the Foolish build. Add the `com.intellij.platform.lsp` module as a dependency.
   - Declare `.foo` as a language/file type, optionally referencing Foolish icons and colors.
   - Add an LSP server definition that launches the shaded `foolish-lsp-java.jar` (point to the local build path or remote download).
2. **Building & running**
   - Invoke `mvn verify` (or `mvn package`) inside the plugin project to assemble the ZIP under `target/`. Keep the Foolish repo checked out adjacent to the plugin project so the JAR path is stable.
   - During development, run `mvn -P runIde verify` (the profile provided by the IntelliJ Maven plugin) to start a sandboxed IDE with the plugin pre-installed. Point the LSP configuration to the freshly built `foolish-lsp-java.jar`.
3. **Reloading the plugin**
   - Stop the sandbox IDE, rebuild via Maven, and relaunch. IntelliJ automatically reloads the plugin when the sandbox restarts.
   - For on-the-fly reloading without restarting, use the “Load/Unload Custom Plugin” action (Ctrl+Shift+A) in the sandbox to unload the current build and load the new ZIP from `target/`.
4. **Distributing to testers**
   - Share the Maven-built ZIP from `target/*.zip`. Testers can install it via *Settings → Plugins → Gear → Install Plugin from Disk…*, then configure the LSP server path under *Settings → Languages & Frameworks → LSP*.

### Unloading / updating the IntelliJ plugin during development
1. Open the sandbox IDE spawned by `mvn -P runIde verify`.
2. Use *Settings → Plugins* to disable or uninstall the Foolish plugin when testing new builds.
3. Alternatively, invoke “Load/Unload Custom Plugin” to hot-unload without touching settings.
4. After rebuilding the plugin (pointing to the updated JAR), re-run `mvn -P runIde verify` or load the new ZIP to validate changes.

These notes should keep VS Code and IntelliJ testers in sync while iterating on the Foolish LSP MVP.
