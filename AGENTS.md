# AI Agent Development Guide

This document provides instructions for AI agents (including Claude Code, GitHub Copilot, Cursor, and other AI coding assistants) working on the Foolish project.

## Overview

Foolish is a revolutionary programming language with parallel Java and Scala implementations. This guide helps AI agents navigate the unique build requirements and environment-specific setup needed for development.

## Build Requirements

- **Java Version**: Java 25 (Temurin recommended)
- **Scala Version**: 3.3.7
- **Build Tool**: Maven (multi-module project)
- **ANTLR**: 4.13.2 (for grammar generation)

## Environment Detection

The project supports two primary development environments:

### Local Development
- Standard development environment on developer's machine
- Assumes Java 25 is already installed
- No proxy configuration needed
- Maven commands work directly: `mvn clean test`

### Claude Code Web (CCW)
- Cloud-based development environment provided by Claude Code
- Requires special setup for Java 25 and Maven proxy
- Environment variable: `CLAUDECODE=1` (always set in CCW)
- Additional variables:
  - `CLAUDE_CODE_REMOTE_ENVIRONMENT_TYPE=cloud_default`
  - `CLAUDE_CODE_PROXY_RESOLVES_HOSTS=true`
  - `CLAUDE_CODE_SESSION_ID` (unique session identifier)

## Claude Code Web Setup

**CRITICAL**: When working in Claude Code Web, you MUST run the CCW setup skill before any Maven operations.

### Running the Setup

```bash
/skill ccw-maven-setup
```

Or directly:
```bash
.claude/skills/ccw-maven-setup/prep_if_ccw.sh
```

### What the Setup Does

The skill (`ccw-maven-setup`) automatically detects the environment and:

**In Claude Code Web (CCW):**
1. Installs SDKMAN if not present
2. Installs latest stable Java 25 (Temurin) via SDKMAN
3. Starts a local Maven authentication proxy at `127.0.0.1:3128`
4. Configures `~/.m2/settings.xml` with proxy settings

**In Local Environments:**
- Does nothing (exits immediately)
- Assumes Java 25 is already available

### Why CCW Needs Special Setup

Maven doesn't honor standard `HTTP_PROXY`/`HTTPS_PROXY` environment variables for authentication. The CCW environment uses a proxy with JWT-based authentication that Maven cannot handle directly.

**The Solution:**
A local Python proxy runs on `localhost:3128` that:
- Accepts Maven connections without authentication
- Forwards requests to the CCW upstream proxy with proper authentication headers
- Automatically extracts and uses JWT tokens from environment variables

**Technical References:**
- [GitHub Issue #13372](https://github.com/anthropics/claude-code/issues/13372) - Maven/Gradle builds failing in CCW
- [LinkedIn Article](https://www.linkedin.com/pulse/fixing-maven-build-issues-claude-code-web-ccw-tarun-lalwani-8n7oc) - Detailed workaround explanation

## Build Commands

### Basic Build and Test

```bash
# Full clean build with tests
mvn clean generate-sources compile test

# Parallel build (recommended)
mvn clean test -T $(($(nproc) * 2)) -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))
```

### Compilation Only

```bash
# When fixing compilation errors, skip tests
mvn clean compile -T $(($(nproc) * 2)) -DskipTests
```

### Running Tests

```bash
# Run all tests with parallel execution
mvn test -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))

# Run specific test
mvn test -Dtest=ClassName#methodName
```

### Build Skills

For comprehensive build strategies, use:
```bash
/skill maven-builder-for-foolish-language
```

This skill provides:
- Parallel execution strategies
- Test debugging workflows
- Approval test management
- Compilation error handling

## Project Structure

Multi-module Maven project with parallel implementations:

```
foolish-parent (root POM)
├── foolish-parser-java       (ANTLR grammar, AST, shared)
├── foolish-core-java         (Java UBC implementation)
├── foolish-core-scala        (Scala UBC implementation)
├── foolish-lsp-java          (Language Server Protocol)
└── foolish-crossvalidation   (Cross-validation tests)
```

**Critical Constraint**: Java and Scala implementations must produce byte-identical approval test outputs.

## Key Concepts for AI Agents

### The Unicellular Brane Computer (UBC)

The reference implementation of Foolish uses a unique evaluation model:
- **Branes**: Containment structures similar to cellular organization
- **FIR**: Foolish Internal Representation with multi-stage state machine
- **BraneMemory**: Hierarchical scoping with retrospective search
- **AB/IB**: Ancestral Brane and Immediate Brane contexts for name resolution

### Test Structure

**Three-tier testing:**
1. **Unit Tests** - Component-level tests
2. **Approval Tests** - Snapshot-based integration tests
3. **Cross-Validation** - Ensures Java/Scala output identity

**Approval Test Workflow:**
- Input files: `test-resources/org/foolish/fvm/inputs/*.foo`
- Java outputs: `foolish-core-java/src/test/resources/org/foolish/fvm/ubc/*.approved.foo`
- Scala outputs: `foolish-core-scala/src/test/resources/org/foolish/fvm/scubc/*.approved.foo`

### Coding Conventions

- Tabs for depth markers (108 character width)
- `.foo` extension for Foolish programs
- Variable names: power-law distribution (mean 3.5 chars)
- Use diverse Unicode: Latin, Greek, Cyrillic, Hebrew, Arabic, Chinese, Sanskrit
- Full-width space (＿) in approval tests shows indentation precisely

## Git Workflow for AI Agents

### Branch Naming
Branches must follow the pattern: `claude/<descriptive-name>-<session-id>`

Example: `claude/run-tests-8vk4v`

### Commit Message Format
Include model information:
```
Summary of changes

Detailed description...

Claude Code v1.0.0 / claude-sonnet-4.5
```

### Push Guidelines
- Always use: `git push -u origin <branch-name>`
- Branch must start with `claude/` and end with session ID
- If push fails with network errors, retry up to 4 times with exponential backoff (2s, 4s, 8s, 16s)

## Common Tasks

### First Time Setup (CCW Only)
```bash
# 1. Run CCW setup skill
/skill ccw-maven-setup

# 2. Verify Java version
java -version  # Should show Java 25

# 3. Run build
mvn clean test -T $(($(nproc) * 2)) -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))
```

### Adding a New Approval Test
1. Create input file: `test-resources/org/foolish/fvm/inputs/yourTestName.foo`
2. Add test method to appropriate test class
3. Run tests to generate `.received.foo` files
4. Review diffs and approve:
   ```bash
   # Show diffs
   find . -name "*.received.foo" -exec sh -c 'diff -u "${0%.received.foo}.approved.foo" "$0" || true' {} \;

   # Approve
   find . -name "*.received.foo" -exec sh -c 'mv "$0" "${0%.received.foo}.approved.foo"' {} \;
   ```

### Debugging Compilation Errors
```bash
# Always skip tests when fixing compilation
mvn clean compile -T $(($(nproc) * 2)) -DskipTests

# After compilation succeeds, run tests
mvn test -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))
```

## Important Files

- **Grammar**: `foolish-parser-java/src/main/antlr4/Foolish.g4`
- **AST**: `foolish-parser-java/src/main/java/org/foolish/ast/AST.java`
- **Java UBC**: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/`
- **Scala UBC**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/`
- **Documentation**: `docs/` (README.md, STYLES.md, ECOSYSTEM.md, etc.)
- **AI Instructions**: `.claude/CLAUDE.md` (Claude-specific guidance)

## Additional Resources

For complete details on:
- Language features and semantics → See `README.md`
- Terminology and conventions → See `docs/STYLES.md`
- UBC architecture → See `docs/ECOSYSTEM.md`
- Name resolution and search → See `docs/NAME_SEARCH_AND_BOUND.md`
- Claude-specific guidance → See `.claude/CLAUDE.md`

## Quick Reference

**Environment Check:**
```bash
if [ -n "$CLAUDECODE" ]; then
    echo "Running in Claude Code Web"
else
    echo "Running in local environment"
fi
```

**Full Workflow (CCW):**
```bash
# Setup (once per session)
/skill ccw-maven-setup

# Development cycle
mvn clean test -T $(($(nproc) * 2)) -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))

# Commit and push
git add .
git commit -m "Your changes

Claude Code v1.0.0 / claude-sonnet-4.5"
git push -u origin claude/your-branch-name
```

**Full Workflow (Local):**
```bash
# No setup needed - just develop
mvn clean test

# Commit and push
git add .
git commit -m "Your changes"
git push
```

---

**Last Updated**: 2026-01-15
**Maintained By**: Project contributors
**Purpose**: Enable AI agents to effectively contribute to the Foolish project
