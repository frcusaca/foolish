# AI Agent Development Guide

This document provides instructions for AI agents (including Claude Code, GitHub Copilot, Cursor, and other AI coding assistants) working on the Foolish project.

## Use Common Sense
Apply industry standard best practices liberally. Use colloquial java and scala language patterns based on the installed versions.(25 and 3.8.1 presently).
Documentation is organized under docs/ in subdirectories: howto/ (tutorials), why/ (philosophy), how/ (engineering), todo/ (project tracking), and old/ (legacy documents being reorganized).


## Development process
Due to the nature of human-driven development, AI should always write the tests first. Approval tests and unit tests, write the tests with most important features, and unclear corner cases written as tests to not only check behavior, but also to document what it looks like.

Always write feature in Java first. Work until all java tests are passing before starting to code the same feature in any other languages.

Ask permission before coding new features or reparing bugs in languages other than ANTLR4, Java or Foolish. Ignore build errors in other language directories if you must and `test -amd` in the java directories.

## Overview

Foolish is a revolutionary programming language with parallel Java and Scala implementations. This guide helps AI agents navigate the unique build requirements and environment-specific setup needed for development.

**Multiple AI agents collaborate on this project.** This document serves as the shared knowledge base for all AI coding assistants (Claude Code, GitHub Copilot, Cursor, and others) to ensure consistent understanding of the project structure, build processes, testing workflows, and coding conventions.

## Build Requirements

- **Java Version**: Java 25 (Temurin recommended)
- **Scala Version**: 3.8.1
- **Build Tool**: Maven (multi-module project)
- **ANTLR**: 4.13.2 (for grammar generation)

## Environment Detection

The project supports two primary development environments:

### Local Development
- Standard development environment on developer's machine
- Assumes Java 25 is already installed
- No proxy configuration needed
- Maven commands work directly: `mvn clean test`

## Build Commands
### Make project specific repository
To be completely safe, let's make our own copy of the repo by running the script located at the root of the project directory:
```bash
./session_starter.make.m2.4.me.sh
```
It outputs an export command for future maven 
"export MAVEN_OPTS="-Dmaven.repo.local=/home/hcubsy/AI_NAME/BRANCH/.m2.4.me"
with the AI_NAME and BRANCH name replaced with current path. Do that and we're set for this session in terms of dependencies.

### Cleaning
```bash
rm -rf ~/.m2/repository/org/foolish
mvn clean
```
The second command to clean out foolish repo is important. If maven repository is elsewhere, please remove the corresponding foolish code as well. This has proven to be
a problem for several systems where 'mvn install' was invoked at some point. It installed stale antlr source that prevented updates to g4 files from taking effect.

** IMPORTANT ** NEVER USE "mvn install". THAT COMMAND IS INCOMPATIBLE WITH THIS PROJECT'S BUILD PROCESS. ALWAYS DO A FULL CLEANING WHEN IN DOUBT.
Part of the Foolish project is to never have to write in all caps like that about a project written in Foolish.

### Basic Build and Test

```bash
# Full clean build. Do this at beginning of a session and after every merge or rebase operation. Also, anytime when debugging took more than 13 minutes, do a full rebuild.
# Everytime the 'foolish-parser-java/src/main/antlr4/Foolish.g4' file is updated, this command must be run at the root of the project to regenerate the parser.
rm -rf ~/.m2/repository/org/foolish ## Remove same everywhere else where m2 may store repository.
mvn clean generate-sources verify

# Parallel build (recommended)
mvn clean verify -am -fae -T $(($(nproc) * 2)) -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))

# Rebuild Antlr4 parser/lexer and parser from g4 file
# This needs to happen every time 'foolish-parser-java/src/main/antlr4/Foolish.g4' changes
mvn clean generate-sources -T $(($(nproc) * 2))

# The approval tests can be selected this way specifying module, class and then the test file filter
mvn verify -am -ff -pl foolish-core-java -Dtest=UbcApprovalTest -Dfoolish.test.filter=Shadow

# Just build (skip tests) when fixing compilation errors.
mvn verify -am -ff -DskipTests -T $(($(nproc) * 2))

# However, you may choose single threaded compilation to improve readability of compilation errors
mvn verify -am -ff -DskipTests

## Select a module to reduce build time and effort
mvn comile -ff -pl foolish-core-java -DskipTests -T $(($(nproc) * 2))

## Turn on debugging and stack trace for debugging build problems
mvn clean verify -am -ff -X -e -DskipTests
```

## Tests
### Overview

- Unit tests verify correctness of each component
- Approval tests illustrate behavior to users - focus on IMPORTANT and EASILY CONFUSED aspects
- Use sensible variable names from all available alphabets to improve expressivity
- New tests should use power-law distributed variable name lengths

### Running tests

```bash
# Run all tests with parallel execution
mvn verify -am -fae -T $(($(nproc) * 2)) -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4))

# Run specific test
mvn verify -am -ff -Dtest=ClassName#methodName
mvn verify -am -ff -pl foolish-core-java -Dtest=UbcApprovalTest -Dfoolish.test.filter=Shadow

## Approval Test Protocol

Approval tests can only be updated in one of three ways. Each change requires all subsequent stages to be performed.

1. Input File Changed: The `.foo` input file (for example `src/test/resources/org/foolish/fvm/inputs/`) is modified. Sometimes this adds tests for new functionalities, other times this may corrects tests.
2. Source code is changed so the system under test could produce different results.
3. Run the test: this produces a `.received.*` file, it should be examined ( generated in `src/test/resources/org/foolish/fvm/ubc/` (Java) or `src/test/resources/org/foolish/fvm/scubc/` (Scala)
4. Present the difference on screen using a side-by-side diff
   ```bash
   diff -y --color src/test/resources/org/foolish/fvm/ubc/testName.received.foo \
                   src/test/resources/org/foolish/fvm/ubc/testName.approved.foo
   ```
4. User Approves: After user approval, move `.received.foo` to `.approved.foo`
5. Subsequent commits and commit comment must mention the approval test was updated.


### Running approval tests
The command for running approval test inside the module foolish-core-java, filtering for input file that has
Shadow in the name, while in the top foolish directory it would be invoked this way:

```bash
mvn test -pl foolish-core-java -Dtest=UbcApprovalTest -Dfoolish.test.filter=Shadow
```
This runs just the selected approval class and filters input file names.

### Unit Test Redability
Unit tests are required to test correctness of internal state of the FVM. There are some infrastructure built
to help this. Unit test can generate a scafolding of Foolish brane using Foolish language. The unit test can 
then alter the initialized Foolish FIR, adding/subtracting or otherwise mutating it into the desired testing
situation. It is free to use the parser, the FoolishIndex and the root Brane's '.search(...)' method to make
the test itself easier to read to human reviewers of the test.

### Approval Test
Approval tests demonstrate the behavior of the Foolish VM by writing inputs in '.foo' files, running a special
VM to produce a final result. Sometimes the results could be Constanic other times they could be NK. As long
as it matches the expected output byte for byte, it is correct. The approval test program outputs more than
just the final brane, it outputs alarms generated along the way as well as number of steps it took to execute
the FVM before the input Foolish file became isConstanic.

Separate langauges read from the same test input resources directory to produce their own approval output.
A crossvalidation process checks that implementations in different languages are behaving identically.

## Clarifications
* When user mentions "path/" first interpret it as relative path from the directory where claude code was invoked. This is normal behavior for most unix apps, for example if I "cat path/file" that path is resolved from the current path.
* Never directly edit `.approved.foo` files

``

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
UNINITIALIZED → INITIALIZED → CHECKED → EVALUATING → CONSTANIC → CONSTANT
```

- `CONSTANT`: Fully evaluated (not "nye" = Not Yet Evaluated)
- `CONSTANIC` (say "CON-STAN-NICK"): CONSTANt IN Context - evaluation paused due to missing information (unbound identifiers)

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

In UBC implementation, this means creating a modified clone with new context. See `docs/old/ECOSYSTEM.md` for detailed semantics.

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
- See [Test section](## Test)

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

## Important Files

- **Grammar**: `foolish-parser-java/src/main/antlr4/Foolish.g4`
- **AST**: `foolish-parser-java/src/main/java/org/foolish/ast/AST.java`
- **Java UBC**: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/`
- **Scala UBC**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/`
- **Documentation**: `docs/` (legacy docs in `docs/old/`; new docs in `docs/howto/`, `docs/why/`, `docs/how/`, `docs/todo/`)
- **AI Instructions**: `.claude/CLAUDE.md` (Claude-specific guidance)

## Documentation

### Directory Structure

- **`docs/howto`** - "How to Express it in Foolish" - literate programming tutorials as .foo files
- **`docs/why`** - "Philosophy of Foolish" - origins, inspirations, design philosophy
- **`docs/how`** - "Engineering documentation" - operational semantics, implementation details, reference
- **`docs/todo`** - "Project documentation" - active project tracking and growth plans
- **`docs/old`** - Legacy documentation (being reorganized into the above directories)

(The `projects/` directory has been retired; its contents are in `docs/old/`.)


### Additional Resources

For complete details on:
- Language features and semantics → See `README.md`
- Terminology and conventions → See `docs/old/STYLES.md`
- UBC architecture → See `docs/old/ECOSYSTEM.md`
- Name resolution and search → See `docs/old/NAMES_SEARCHES_N_BOUNDS.md`
- Claude-specific guidance → See `.claude/CLAUDE.md`

## Quick Reference

## Markdown File Update Protocol

**IMPORTANT**: Whenever ANY AI agent modifies a `*.md` file in this repository, the agent MUST update the "## Last Updated" section at the end of that file with:

1. **Current timestamp** (YYYY-MM-DD format)
2. **Agent identifier** (as specific as possible, including model name/version)
3. **Brief summary** of what was changed

Example format:
```markdown
## Last Updated

**Date**: 2026-01-15
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-5-20250929
**Changes**: Added detailed UBC architecture documentation and test infrastructure workflows
```

This ensures all AI agents can track who modified documentation and when, maintaining clear collaboration history.

## Maintenance Instructions

**Weekly Check**: After one week past the day of last update to AGENTS.md (either by git timestamp or the Last Updated section below), please review this file for accuracy:

1. Verify that project structure, build commands, and setup instructions are still accurate
2. Check if new project conventions or workflows need documentation
3. Ensure UBC architecture details match current implementation
4. Confirm test infrastructure documentation reflects actual test structure
5. Verify that all AI agents have access to necessary information
6. Check that environment detection and CCW setup instructions are current
7. Propose updates to the user if discrepancies are found
8. Update the Last Updated section below--even if user makes no changes

When proposing updates, explain what has changed and why the documentation needs adjustment. After user review, update the "Last Updated" date below whether changes are accepted or the user confirms current state is acceptable.

## Last Updated

**Date**: 2026-02-06
**Updated By**: Claude Code v1.0.0 / claude-opus-4-6
**Changes**: Reorganized documentation structure. Replaced docs/ and projects/ directory descriptions with new 5-directory taxonomy (howto, why, how, todo, old). Updated all file path references. Fixed stale NAME_SEARCH_AND_BOUND.md reference.
