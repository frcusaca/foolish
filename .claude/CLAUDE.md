# CLAUDE.md

This file provides Claude Code-specific guidance when working on the Foolish project.

## For All AI Agents - Read AGENTS.md First

**IMPORTANT**: This repository includes comprehensive project guidance in `AGENTS.md` at the root directory. **ALL AI agents** (including Claude Code, GitHub Copilot, Cursor, and other AI coding assistants) should consult **`AGENTS.md`** for:

- Environment detection and setup (Claude Code Web vs local development)
- Build requirements and commands (Java 25, Scala 3.3.7, ANTLR 4.13.2, Maven)
- Project structure and multi-module Maven architecture
- The Unicellular Brane Computer (UBC) implementation details
- FIR (Foolish Internal Representation) and state machine
- BraneMemory, scope resolution, AB/IB semantics
- Test infrastructure (unit tests, approval tests, cross-validation)
- Foolish language terminology and coding conventions
- Git workflow and branch naming conventions
- Common development tasks with complete examples

**The sections below provide Claude Code-specific instructions only.** For general project information, always consult `AGENTS.md` first.

---

## Claude Code-Specific Features

### Build and Test Skills

Claude Code provides specialized skills for build management. For comprehensive Maven build strategies with parallel execution, intelligent test running, and targeted debugging workflows, use:

```bash
/skill maven-builder-for-foolish-language
```

This skill provides:
- Parallel execution strategies with dynamic resource detection
- Test debugging workflows
- Approval test management and parallelization
- Compilation error handling
- Decision trees for build selection
- Output analysis strategies

See `.claude/skills/maven-builder-for-foolish-language/` for full documentation.

### Claude Code Web (CCW) Setup

**This project requires Java 25.** Local development assumes Java 25 is installed. Claude Code Web provides Java 21 by default, so setup is required.

**Automatic Setup via SessionStart Hook:**

This project is configured with a SessionStart hook that **automatically runs CCW setup synchronously** when your session starts. This ensures Java 25 and Maven proxy are fully ready before your session begins.

**What happens automatically:**
1. Detects if running in CCW (checks `CLAUDECODE` environment variable)
2. If in CCW (waits for completion before session starts):
   - Installs SDKMAN (if not present)
   - Installs latest stable Java 25 (Temurin) via SDKMAN
   - Exports `JAVA_HOME` and updates `PATH` for the session
   - Starts a local Maven authentication proxy at `127.0.0.1:3128`
   - Verifies proxy is listening and ready
   - Configures `~/.m2/settings.xml` with HTTP + HTTPS proxy settings
   - Extracts and imports CA certificates into Java truststore (fixes PKIX errors)
   - Sets `MAVEN_OPTS` to use custom truststore
3. If not in CCW (local environment):
   - Does nothing - assumes Java 25 is already installed

**The setup is synchronous and idempotent:**
- ⏱️ First run: ~60 seconds (downloads Java 25)
- ⚡ Subsequent runs: ~10 seconds (cached)
- ✅ Session is guaranteed ready for Maven builds when it starts
- 🔄 Safe to run multiple times (won't reinstall if already present)
- 🔐 Handles both proxy routing AND TLS certificate trust

**Technical Implementation:**

This setup implements a **two-part solution** from [GitHub Issue #13372](https://github.com/anthropics/claude-code/issues/13372) and the [LinkedIn article by Tarun Lalwani](https://www.linkedin.com/pulse/fixing-maven-build-issues-claude-code-web-ccw-tarun-lalwani-8n7oc):

1. **Proxy Routing**: Local Python proxy handles JWT authentication, Maven uses localhost:3128
2. **TLS Trust**: CA certificate extraction and Java truststore configuration to prevent PKIX errors

### Known CCW Limitations (Updated: 2026-01-19)

**What Works:**
- ✅ Java 25 installation and environment setup
- ✅ Maven proxy configuration (Python proxy + settings.xml)
- ✅ Parser module compilation (ANTLR grammar processing)
- ✅ Basic dependency downloads (100+ artifacts successfully downloaded)

**What's Blocked:**
- ❌ **PKIX Certificate Errors**: Some HTTPS Maven downloads fail with "unable to find valid certification path to requested target"
- ❌ **Root Cause**: CCW's TLS inspection proxy uses "Anthropic sandbox-egress-production TLS Inspection CA"
- ❌ **Certificate Catch-22**: Cannot extract the CA certificate programmatically because the SSL connection itself requires the trust we're trying to establish

**Attempted Solutions:**
- Tried automatic CA certificate extraction via `openssl s_client` - blocked by TLS inspection
- Tried Maven SSL flags (`-Dmaven.wagon.http.ssl.insecure=true`) - ineffective at Java HTTP client level
- Implemented truststore import logic - ready to use but needs the CA certificate file

**Workarounds:**

1. **Work Locally** (Recommended):
   - Full Maven functionality with Java 25
   - No proxy or certificate issues
   - All tests and builds work normally

2. **Manual CA Certificate** (If Available):
   - Place the Anthropic CA cert at: `support/shared/claude/skills/ccw-maven-setup/ccw-proxy-ca.pem`
   - SessionStart hook will automatically import it into Java truststore
   - Maven builds should work after restart

3. **Partial Functionality** (Current State):
   - Parser compiles successfully
   - Some dependencies download via HTTP
   - HTTPS-dependent artifacts may fail

**Request to Anthropic:**
- Add Maven Central (`repo.maven.apache.org`, `*.maven.apache.org`) to NO_PROXY environment variable
- Or provide the TLS inspection CA certificate for manual trust configuration
- Similar to the fix applied for Rust's crates.io (Issue #10307)

**Manual Setup (optional):**

If you need to run setup manually for any reason:
```bash
/skill ccw-maven-setup
```

For details on why CCW needs a proxy and how the setup works, see the "Claude Code Web Setup" section in `AGENTS.md`.

### Quick Build Reference

Basic build and test:
```bash
mvn clean generate-sources compile test
```

Optimized parallel builds:
```bash
# Clean parallel build (2 threads per core, dynamically calculated)
mvn clean compile -T $(($(nproc) * 2))

# Parallel tests (4 threads per core for test execution)
mvn test -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))

# Combined clean build with parallel tests
mvn clean test -T $(($(nproc) * 2)) -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))
```

**Important**: When fixing compilation errors, always skip tests first:
```bash
mvn clean compile -T $(($(nproc) * 2)) -DskipTests
```

For detailed build strategies and debugging patterns, use the `maven-builder-for-foolish-language` skill.

## Git Commit Message Format

Claude Code commit messages should include:
- Current version of Claude Code
- Model identifier

Example:
```
Add RegExp search to brane operations

Implemented pattern-based search using RegexpSearchFiroe.
Added tests and updated documentation.

Claude Code v1.0.0 / claude-sonnet-4-5-20250929
```

## Git Workflow

### Branch Naming
Branches must follow: `claude/<descriptive-name>-<session-id>`

Example: `claude/run-tests-8vk4v`

**CRITICAL**: Branch must start with `claude/` and end with matching session ID, otherwise push will fail with 403 HTTP error.

### Push Guidelines
- Always use: `git push -u origin <branch-name>`
- If push fails with network errors, retry up to 4 times with exponential backoff (2s, 4s, 8s, 16s)

## Project-Specific Information

For all project-specific information including:
- Multi-module Maven structure and dependencies
- UBC (Unicellular Brane Computer) architecture
- FIR state machine and evaluation strategy
- BraneMemory and scope resolution
- AB/IB (Ancestral Brane / Immediate Brane) semantics
- Test infrastructure and approval test workflows
- Foolish language terminology and coding conventions
- Important file locations
- Adding new approval tests
- Debugging workflows

**Consult `AGENTS.md` at the root directory.**

---

## Markdown File Update Protocol

**IMPORTANT**: When you modify any `*.md` file in this repository, you MUST update the "## Last Updated" section at the end of that file. See `AGENTS.md` for the complete protocol. Always include:

1. **Current timestamp** (YYYY-MM-DD format)
2. **Your agent identifier** (Claude Code v1.0.0 / claude-sonnet-4-5-20250929)
3. **Brief summary** of what was changed

This ensures all AI agents collaborating on this project can track documentation changes.

## Maintenance Instructions

**Weekly Check**: After one week past the day of last update to this file (either by git timestamp or the Last Updated section below) please review this CLAUDE.md file for accuracy:

1. Verify that Claude Code-specific content (skills, CCW setup) is still accurate
2. Check if new Claude Code features need documentation
3. Ensure `AGENTS.md` is properly referenced and contains all project-specific details
4. Confirm this file focuses ONLY on Claude Code-specific features
5. Propose updates to the user if discrepancies are found
6. Update the Last Updated section below--even if user makes no changes

When proposing updates, explain what has changed and why the documentation needs adjustment. After user review, update the "Last Updated" date below whether changes are accepted or the user confirms current state is acceptable.

## Last Updated

**Date**: 2026-01-19
**Updated By**: Claude Code v2.1.1 / claude-sonnet-4-5-20250929
**Changes**: Added "Known CCW Limitations" section documenting PKIX certificate errors that prevent full Maven functionality in CCW. Detailed what works (Java 25, proxy routing, parser compilation), what's blocked (HTTPS downloads due to TLS inspection CA), attempted solutions, and workarounds. Improved CCW Maven proxy setup based on GitHub Issue #13372 and LinkedIn article with synchronous mode, HTTP+HTTPS proxy configuration, readiness verification, and CA certificate import logic. All Maven commands use standard `mvn` as proxy configuration is handled automatically by SessionStart hook.
