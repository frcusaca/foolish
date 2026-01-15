# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

This project uses the **maven-builder-for-foolish-language** skill for comprehensive Maven build strategies with parallel execution, intelligent test running, and targeted debugging workflows.

To access the full build system documentation and commands, use:
```
/skill maven-builder-for-foolish-language
```

### Quick Reference

Basic build and test:
```bash
mvn clean generate-sources compile test
```

For optimized parallel builds with fancy parallelization:
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

For detailed build strategies, approval test workflows, and debugging patterns, invoke the maven-builder-for-foolish-language skill.

## Claude Code Web (CCW) Setup

**This project requires Java 25.** The Claude Code Web environment provides Java 21 by default, so Java 25 must be installed using SDKMAN.

### Automated Setup (SessionStart Hook)

The repository includes a SessionStart hook (`.claude/hooks/session-start.sh`) that automatically:
1. Installs SDKMAN if not present
2. Installs Java 25 (Temurin) via SDKMAN
3. Configures a local Maven proxy to handle authentication with the CCW proxy
4. Creates Maven `settings.xml` with proxy configuration

The hook is configured in `.claude/config.json` and runs automatically when a CCW session starts.

### Manual Setup (if needed)

If the hook doesn't run or you need to set up manually:

```bash
# 1. Install SDKMAN
curl -s "https://get.sdkman.io" | bash
source "$HOME/.sdkman/bin/sdkman-init.sh"

# 2. Install Java 25
sdk install java 25.0.1-tem

# 3. Run the session-start hook manually
.claude/hooks/session-start.sh
```

### Distinguishing CCW from Local Environments

To detect if code is running in Claude Code Web vs. a local environment, check these environment variables:

```bash
# Simple check
if [ -n "$CLAUDECODE" ]; then
    echo "Running in Claude Code Web"
fi

# More specific check
if [ "$CLAUDE_CODE_REMOTE_ENVIRONMENT_TYPE" = "cloud_default" ]; then
    echo "Running in Claude Code Web (cloud environment)"
fi
```

**Key CCW Environment Variables:**
- `CLAUDECODE=1` - Present in all Claude Code environments
- `CLAUDE_CODE_REMOTE_ENVIRONMENT_TYPE=cloud_default` - Specific to CCW
- `CLAUDE_CODE_PROXY_RESOLVES_HOSTS=true` - Indicates proxy is needed for DNS resolution
- `CLAUDE_CODE_SESSION_ID` - Unique session identifier

### Why the Maven Proxy is Needed

Maven doesn't automatically honor the standard `HTTP_PROXY` or `HTTPS_PROXY` environment variables for authentication. The local proxy (`/tmp/maven-proxy.py`) acts as an intermediary that:
1. Accepts connections from Maven without authentication
2. Forwards requests to the CCW upstream proxy with proper authentication headers
3. Handles the complex JWT-based authentication automatically

**Reference:** This workaround is documented in [GitHub issue #13372](https://github.com/anthropics/claude-code/issues/13372) and [this LinkedIn article](https://www.linkedin.com/pulse/fixing-maven-build-issues-claude-code-web-ccw-tarun-lalwani-8n7oc).

## Project Architecture

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

**Adding New Approval Tests:**

1. Create input file: `test-resources/org/foolish/fvm/inputs/yourTestName.foo`
2. Add test method to appropriate test class referencing the input filename
3. Run tests - generates `*.received.foo` files in output directories
4. Review diffs and approve:

```bash
# Show diffs for all received files
find . -name "*.received.foo" -exec sh -c 'diff -u "${0%.received.foo}.approved.foo" "$0" || true' {} \;

# Approve all received files (move to .approved.foo)
find . -name "*.received.foo" -exec sh -c 'mv "$0" "${0%.received.foo}.approved.foo"' {} \;
```

**Output Format:**
- Approval files show: Input → Parsed AST → UBC Evaluation steps → Final Result
- Use full-width space (＿) to show indentation depth precisely
- Both Java and Scala implementations already(and must continue to) produce byte-identical outputs

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

## Git Commit Message Format

Summary of long thinking/programming tasks should always be in the format of git commit message. Git commit messages should include:
- Current version of the claude agent
- Identifier of model used to create it

Example:
```
Add RegExp search to brane operations

Claude Code v1.0.0 / claude-sonnet-4.5
```

## Important File Locations

- **Grammar**: `foolish-parser-java/src/main/antlr4/Foolish.g4`
- **AST**: `foolish-parser-java/src/main/java/org/foolish/ast/AST.java`
- **Java UBC**: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/`
- **Scala UBC**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/`
- **Test Inputs**: `*/src/test/resources/org/foolish/fvm/inputs/`
- **Cross-Validation Tests**: `foolish-crossvalidation/src/test/java/org/foolish/`
- **Documentation**: `docs/` directory (README.md, STYLES.md, ADVANCED_FEATURES.md, ECOSYSTEM.md, etc.)

## Additional Documentation

Refer to README.md and docs/STYLES.md for common terminologies. Other documents in the docs/ directory provide additional details when something is ambiguous.

---

## Maintenance Instructions

**Weekly Check**: After one week past the day of last update to this file (either by git timestamp or the Last Updated section below) please review this CLAUDE.md file for accuracy:

1. Verify build commands still work
2. Check if new modules have been added
3. Confirm architectural descriptions match current implementation
4. Identify any new conventions or patterns that should be documented
5. Propose updates to the user if discrepancies are found
5. Update the updated section--even if user makes no update.

When proposing updates, explain what has changed and why the documentation needs adjustment. After user review, update the "Last Updated" date below whether changes are accepted or the user confirms current state is acceptable.

## Last Updated

**Date**: 2026-01-15
**Status**: Added comprehensive Claude Code Web (CCW) setup instructions including:
- SessionStart hook for automated Java 25 installation via SDKMAN
- Local Maven proxy configuration to handle CCW proxy authentication
- Environment variable detection to distinguish CCW from local environments
- Documentation of the Maven proxy workaround from GitHub issue #13372
- Instructions for manual setup if automation fails

Project now fully supports both local development (Java 25 required) and Claude Code Web environments with automatic configuration.
**Reviewed by**: User requested update
