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

**CRITICAL**: When working in Claude Code Web (CCW), you MUST run the CCW setup before any Maven operations.

### Running the Setup

**Claude Code users:**
```bash
/skill ccw-maven-setup
```

**All AI agents (alternative method):**
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

### Build Skills (Claude Code)

**Claude Code users** can access comprehensive build strategies via:
```bash
/skill maven-builder-for-foolish-language
```

This skill provides:
- Parallel execution strategies
- Test debugging workflows
- Approval test management
- Compilation error handling

**Other AI agents**: Refer to the maven-builder-for-foolish-language skill documentation at `.claude/skills/maven-builder-for-foolish-language/` for detailed build patterns and strategies.

## Project Structure

### Multi-Module Maven Structure

The codebase uses a **multi-module Maven project** with parallel Java and Scala implementations:

```
foolish-parent (root POM)
├── foolish-parser-java       (ANTLR grammar, AST, shared by both implementations)
├── foolish-core-java         (Java UBC implementation)
├── foolish-core-scala        (Scala UBC implementation)
├── foolish-lsp-java          (Language Server Protocol)
└── foolish-crossvalidation   (Cross-validation tests verifying Java/Scala output identity)
```

**Key Dependencies:**
- Both Java and Scala core modules depend on the shared parser
- Scala core depends on Java core for shared test utilities
- LSP server uses the Java implementation
- Cross-validation module depends on both Java and Scala core modules
- Requirements: Java 25, Scala 3.3.7, ANTLR 4.13.2

### The Unicellular Brane Computer (UBC)

The **UBC is the reference implementation of Foolish**. It implements a unique evaluation model based on branes (containment structures).

#### FIR (Foolish Internal Representation)

FIR objects represent expressions during evaluation and progress through a multi-stage state machine:

```
UNINITIALIZED → INITIALIZED → REFERENCES_IDENTIFIED → ALLOCATED
  → RESOLVED → EVALUATING → CONSTANT
```

Only `CONSTANT` means fully evaluated (not "nye" = Not Yet Evaluated).

**Key FIR Types:**
- `ValueFiroe` - constants (integers, strings)
- `NKFiroe` - "Not Known" values (`???`), errors
- `BraneFiroe` - evaluates branes `{...}`
- `AssignmentFiroe` - variable bindings `x = expr`
- `IdentifierFiroe` - variable references with optional characterizations
- `BinaryFiroe` / `UnaryFiroe` - arithmetic/logical operators
- `IfFiroe` - conditional expressions
- `SearchUpFiroe` - `↑` operator for upward scope traversal
- `RegexpSearchFiroe` - pattern-based brane search

#### BraneMemory: Hierarchical Scoping

`BraneMemory` implements Foolish's unique scope resolution:
- **Retrospective search**: searches backwards in current brane, then upwards through parent branes
- Names resolve based on proximity: "containment creates organization, proximity creates combination"
- Supports both exact identifier matching and regular expression queries

#### Brane Reference Semantics: AB and IB

**Ancestral Brane (AB)** and **Immediate Brane (IB)** are critical context for name resolution:
- **IB**: Current context accumulated so far (lines before current expression)
- **AB**: Parent brane context containing the defining expression and its AB/IB

**Detachment and Coordination**: When a brane is referenced by name:
1. The brane was already partially resolved in its original AB/IB context
2. A clone is **detached** from its original AB/IB
3. The clone is **recoordinated** with new AB (the containing brane) and new IB (preceding lines)
4. Previously failed name searches can now resolve in the new context

In UBC implementation, this means creating a modified clone with new context. See `docs/ECOSYSTEM.md` for detailed semantics.

#### Evaluation Strategy

`FiroeWithBraneMind` implements **breadth-first evaluation** with state-aware stepping:
- Maintains `braneMind` (LinkedList<FIR>) - evaluation queue
- Maintains `braneMemory` (BraneMemory) - completed evaluations
- Different phases prevent evaluation order issues and enable partial/abstract evaluation

### Parallel Java/Scala Implementations

**Shared Components:**
- AST (Java records in parser module)
- ANTLR grammar (`Foolish.g4`)
- Test input files (`.foo` programs in `test-resources/`)

**Parallel Components:**
- FIR implementations (`org.foolish.fvm.ubc` vs `org.foolish.fvm.scubc`)
- Test classes (JUnit vs ScalaTest)
- Separate approval output directories

**Critical Constraint:** Both implementations must produce **byte-identical** approval outputs. Cross-validation tests in the `foolish-crossvalidation` module enforce this by comparing all approval files.

### Test Infrastructure

**Three-Tier Testing:**

1. **Unit Tests** (`*UnitTest.java`) - focused component tests in Java and Scala modules
2. **Approval Tests** (`*ApprovalTest.{java,scala}`) - snapshot-based integration tests in Java and Scala modules
3. **Cross-Validation** (separate `foolish-crossvalidation` module) - runs after Java and Scala tests to ensure output identity

#### Approval Test Workflow

**Directory Structure:**
- **Inputs (shared)**: `test-resources/org/foolish/fvm/inputs/*.foo`
- **Java outputs**: `foolish-core-java/src/test/resources/org/foolish/fvm/ubc/*.approved.foo`
- **Scala outputs**: `foolish-core-scala/src/test/resources/org/foolish/fvm/scubc/*.approved.foo`

**Test Classes:**
- `ParserApprovalTest.java` - AST parsing verification
- `UbcApprovalTest.java` - Java UBC evaluation
- `ScUbcApprovalTest.scala` - Scala UBC evaluation

**Output Format:**
- Approval files show: Input → Parsed AST → UBC Evaluation steps → Final Result
- Use full-width space (＿) to show indentation depth precisely
- Both Java and Scala implementations already (and must continue to) produce byte-identical outputs

## Language-Specific Conventions

### Foolish Terminology (from STYLES.md)

- **Foolisher** - developer/user of Foolish
- **Nye** (say "nigh") - Not Yet Evaluated state
- **No-no** - The `???` unknown value
- **Ordinate** - a name associated with a brane
- **Coordinate** - brane member names used for relational access
- **Lexed** - feature parses to AST
- **Interpreted** - feature fully implemented in VM

### Code Style

- Tabs for depth markers (reduces storage)
- 108 character width for documents
- `.foo` extension for Foolish programs
- Full-width space (＿) in approval tests shows indentation precisely
- Variable names follow power-law distribution (mean 3.5 chars short, 5 chars long)
- Use diverse Unicode: Latin, Greek, Cyrillic, Hebrew, Arabic, Chinese, Sanskrit

### Writing Tests

- Unit tests verify correctness of each component
- Approval tests illustrate behavior to users - focus on IMPORTANT and EASILY CONFUSED aspects
- Use sensible variable names from all available alphabets to improve expressivity
- New tests should use power-law distributed variable name lengths

### Debugging

- Reproduce errors in approval tests, breaking complex issues into smaller test cases
- Use configuration flags in test comments (`!! --verbose !!`) for verbose output
- Printf-style debugging only as last resort
- Verify fixes don't break other tests; keep useful intermediate tests as regression guards

## Git Workflow for AI Agents

### Branch Naming

Branches should follow the pattern: `<agent-prefix>/<descriptive-name>-<session-id>`

Examples:
- `claude/run-tests-8vk4v` (Claude Code)
- `copilot/fix-parser-abc123` (GitHub Copilot)
- `cursor/add-feature-xyz789` (Cursor)

**Note**: Some repositories may enforce specific branch naming patterns. Check repository settings or consult with maintainers.

### Commit Message Format

Include AI agent and model information in commit messages:
```
Summary of changes

Detailed description of what was changed and why...

[AI Agent Name] [Version] / [Model ID]
```

Examples:
```
Add RegExp search to brane operations

Implemented pattern-based search using RegexpSearchFiroe.
Added tests and updated documentation.

Claude Code v1.0.0 / claude-sonnet-4-5-20250929
```

```
Fix type inference bug in FIR resolution

GitHub Copilot / gpt-4
```

### Push Guidelines

- Always use: `git push -u origin <branch-name>` for first push
- If push fails with network errors, retry up to 4 times with exponential backoff (2s, 4s, 8s, 16s)
- **For Claude Code specifically**: Branch must start with `claude/` and end with session ID, otherwise push will fail with 403

## Common Tasks

### First Time Setup (CCW Only)

**Claude Code:**
```bash
# 1. Run CCW setup skill
/skill ccw-maven-setup

# 2. Verify Java version
java -version  # Should show Java 25

# 3. Run build
mvn clean test -T $(($(nproc) * 2)) -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))
```

**Other AI agents:**
```bash
# 1. Run setup script directly
.claude/skills/ccw-maven-setup/prep_if_ccw.sh

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

**Full Workflow (CCW - Claude Code):**
```bash
# Setup (once per session)
/skill ccw-maven-setup

# Development cycle
mvn clean test -T $(($(nproc) * 2)) -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))

# Commit and push
git add .
git commit -m "Your changes

Claude Code v1.0.0 / claude-sonnet-4-5-20250929"
git push -u origin claude/your-branch-name
```

**Full Workflow (CCW - Other AI agents):**
```bash
# Setup (once per session)
.claude/skills/ccw-maven-setup/prep_if_ccw.sh

# Development cycle
mvn clean test -T $(($(nproc) * 2)) -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))

# Commit and push
git add .
git commit -m "Your changes

[AI Agent Name] / [Model ID]"
git push -u origin <agent-prefix>/your-branch-name
```

**Full Workflow (Local - All agents):**
```bash
# No setup needed - just develop
mvn clean test

# Commit and push
git add .
git commit -m "Your changes

[AI Agent Name] / [Model ID]"
git push
```

---

**Last Updated**: 2026-01-15
**Maintained By**: Project contributors
**Purpose**: Enable AI agents to effectively contribute to the Foolish project
