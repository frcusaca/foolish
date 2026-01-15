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

**IMPORTANT:** Before running any Maven commands in CCW, run:

```bash
/skill ccw-maven-setup
```

This skill automatically:
1. Detects if running in CCW (checks `CLAUDECODE` environment variable)
2. If in CCW:
   - Installs SDKMAN (if not present)
   - Installs latest stable Java 25 (Temurin) via SDKMAN
   - Starts a local Maven authentication proxy at `127.0.0.1:3128`
   - Configures `~/.m2/settings.xml` with proxy settings
3. If not in CCW (local environment):
   - Does nothing - assumes Java 25 is already installed

**The skill is idempotent** - safe to run multiple times.

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

## Maintenance Instructions

**Weekly Check**: After one week past the day of last update to this file (either by git timestamp or the Last Updated section below) please review this CLAUDE.md file for accuracy:

1. Verify that Claude Code-specific content (skills, CCW setup) is still accurate
2. Check if new Claude Code features need documentation
3. Ensure `AGENTS.md` is properly referenced and contains all project-specific details
4. Confirm this file focuses ONLY on Claude Code-specific features
5. Propose updates to the user if discrepancies are found
6. Update the updated section--even if user makes no update

When proposing updates, explain what has changed and why the documentation needs adjustment. After user review, update the "Last Updated" date below whether changes are accepted or the user confirms current state is acceptable.

## Last Updated

**Date**: 2026-01-15
**Status**: Restructured to separate Claude Code-specific content from general project information:
- Moved all project-specific details (architecture, UBC, FIR, testing, conventions) to `AGENTS.md`
- Kept only Claude Code-specific features: skills, CCW setup, commit format, branch naming
- Added prominent references to `AGENTS.md` for project information
- CLAUDE.md now focuses exclusively on Claude Code features while AGENTS.md serves all AI agents

**Key Change**: Claude Code users should read `AGENTS.md` for project details and this file for Claude-specific features.
**Reviewed by**: User requested separation of Claude-specific vs general AI agent content
