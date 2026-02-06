# CLAUDE.md

## Use Common Sense
Consult `AGENTS.md` at the root directory.
Apply industry standard best practices liberally. Use colloquial java and scala language patterns based on the installed versions.(25 and 3.8.1 presently).
Documentation is organized under docs/ in subdirectories: howto/ (tutorials), why/ (philosophy), how/ (engineering), todo/ (project tracking), and old/ (legacy documents being reorganized).

## Session Hook Setup

The project includes a SessionStart hook in `.claude/settings.json` that configures the development environment. The hook may output environment variable settings that need to be run manually in the shell before builds work.

**After starting a new session:**
1. Check the hook output for any environment variable exports (e.g., `export JAVA_HOME=...`, `export PATH=...`)
2. If the session start hook provides additional instructions in the output, run them.
3. Then proceed with Maven builds


## Build and Test Skills

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

## Quick Build Reference

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

When fixing compilation errors, always skip tests first:
```bash
mvn clean compile -DskipTests
# Or even turn on debugging 
mvn clean compile -X -DskipTests
```

For detailed build strategies and debugging patterns, use the `maven-builder-for-foolish-language` skill.

### Grammar File Changes

Whenever the ANTLR grammar file is modified (`foolish-parser-java/src/main/antlr4/Foolish.g4`), you must regenerate
sources before that takes effect. Maven sometimes fails to see the file has changed.  This regenerates the parser/lexer from the grammar.

```bash
mvn clean generate-sources
```

### Approval Test Protocol

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

## Clarifications
* When user mentions "path/" first interpret it as relative path from the directory where claude code was invoked. This is normal behavior for most unix apps, for example if I "cat path/file" that path is resolved from the current path.
* Never directly edit `.approved.foo` files

## Git

### Git Commit Message Format

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


### Branch Naming
Branches must follow: `claude/<descriptive-name>-<session-id>`

Example: `claude/run-tests-8vk4v`

**CRITICAL**: Branch must start with `claude/` and end with matching session ID, otherwise push will fail with 403 HTTP error.

### Push Guidelines
- Always use: `git push -u origin <branch-name>`
- If push fails with network errors, retry up to 4 times with exponential backoff (2s, 4s, 8s, 16s)



## Documentation Organization

### Directory Structure

- **`docs/howto`** - "How to Express it in Foolish" - literate programming tutorials as .foo files
- **`docs/why`** - "Philosophy of Foolish" - origins, inspirations, design philosophy
- **`docs/how`** - "Engineering documentation" - operational semantics, implementation details, reference
- **`docs/todo`** - "Project documentation" - active project tracking and growth plans
- **`docs/old`** - Legacy documentation (being reorganized into the above directories)

(The `projects/` directory has been retired; its contents are in `docs/old/`.)

### Markdown File Update Protocol

**IMPORTANT**: When you modify any `*.md` file in this repository, you MUST update the "## Last Updated" section at the end of that file. See `AGENTS.md` for the complete protocol. Always include:

1. **Current timestamp** (YYYY-MM-DD format)
2. **Your agent identifier** (Claude Code v1.0.0 / claude-sonnet-4-5-20250929)
3. **Brief summary** of what was changed

This ensures all AI agents collaborating on this project can track documentation changes.

## Maintenance Instructions

**Weekly Check**: After one week past the day of last update to this file (either by git timestamp or the Last Updated section below) please review this CLAUDE.md file for accuracy:

1. Verify that Claude Code-specific content (skills, session hooks) is still accurate
2. Check if new Claude Code features need documentation
3. Ensure `AGENTS.md` is properly referenced and contains all project-specific details
4. Confirm this file focuses ONLY on Claude Code-specific features
5. Propose updates to the user if discrepancies are found
6. Update the Last Updated section below--even if user makes no changes

When proposing updates, explain what has changed and why the documentation needs adjustment. After user review, update the "Last Updated" date below whether changes are accepted or the user confirms current state is acceptable.

---

## Historical: Claude Code Web (CCW) Notes

**Note**: The following section documents historical attempts to use Claude Code Web with this project. As of January 2026, CCW is not recommended for this Maven-based Java project due to network limitations.

### CCW Limitations (January 2026)

**Date**: 2026-01-19
**Model**: claude-sonnet-4-5-20250929

This project requires Java 25 and Maven for builds. Claude Code Web is not currently configured to support Maven-based Java development.

**What Works in CCW:**
- Java 25 installation (via SessionStart hook using SDKMAN)
- Basic Java compilation
- Code editing and file management

**What Does Not Work:**
- Maven dependency downloads from Maven Central
- Full Maven builds requiring external artifacts
- Running tests that depend on Maven dependencies

**Why Maven Doesn't Work:**

As of January 2026, Claude Code Web's network security model prevents standard Maven dependency resolution. The environment's egress filtering interferes with Maven's HTTPS connections to artifact repositories. Various workarounds were attempted (proxy configuration, certificate manipulation, SSL bypasses) but each approach was progressively blocked or rendered ineffective by the security infrastructure.

This appears to be an inherent limitation of CCW's current architecture rather than a simple configuration issue. The same challenges affected other language ecosystems (like Rust) initially, so support may improve over time.

**Recommendation:**

**Use local development for this project.** All Maven functionality works normally in a local environment with Java 25 installed. CCW is approximately 3 months old (as of January 2026) and still maturing its support for different development stacks.

---

## Last Updated

**Date**: 2026-02-06
**Updated By**: Claude Code v1.0.0 / claude-opus-4-6
**Changes**: Reorganized documentation structure. Replaced docs/ and projects/ directory descriptions with new 5-directory taxonomy (howto, why, how, todo, old). The projects/ directory has been retired.
