# AI Agent Development Guide

This document provides instructions for AI agents (including Claude Code, GitHub Copilot, Cursor, and other AI coding assistants) working on the Foolish project.

## Use Common Sense
Apply industry standard best practices liberally. Use colloquial java and scala language patterns based on the installed versions.(25 and 3.8.1 presently).
Documentation is organized under docs/ in subdirectories: howto/ (tutorials), why/ (philosophy), how/ (engineering), todo/ (project tracking), and vintage_legacy/ (legacy documents being reorganized).


## Development process
Due to the nature of human-driven development, AI should always write the tests first. Approval tests and unit tests, write the tests with most important features, and unclear corner cases written as tests to not only check behavior, but also to document what it looks like.
Ask permission before coding new features or reparing bugs in languages other than ANTLR4, Java or Foolish. Ignore build errors in other language directories if you must and `test -amd` in the java directories.

## Overview

Foolish is a revolutionary programming language with parallel Java and Scala implementations. This guide helps AI agents navigate the unique build requirements and environment-specific setup needed for development.

**Multiple AI agents collaborate on this project.** This document serves as the shared knowledge base for all AI coding assistants (Claude Code, GitHub Copilot, Cursor, and others) to ensure consistent understanding of the project structure, build processes, testing workflows, and coding conventions.

## Build Requirements

- **Java Version**: Java 25 (Temurin recommended)
- **Scala Version**: 3.8.1
- **Build Tool**: Maven (multi-module project)
- **ANTLR**: 4.13.2 (for grammar generation)

## Project Segmentation
Software projects May be large or small. Their complexity and diffiulty may also vary. Generally speaking we use these terms for disjoint components of softare:
  - Major
    - This is a noun, That "specification file is for a major", or an adjective "that is a major specification"
    - This is a very large feature, that may break many existing functionalities while implementing
    - Some extensive exchange with human may be required.
    - Some multi-modal analysis, including web-searches, prototyping, analysis, etc.
    - aka Major Feature, Major release, Major upgrade, etc.
    - Example: "Centralize and fully sepcification of CLI interface by gathering features from all the existing implementations. Resolve any conflicts or redundancies. Then update all implementation to follow new specification."
    - Example: "DHT for discovering peers for different purposes: mutual attestation, calendar replication, capability-matching, etc."
  - Phase
    - a Major feature may be implemented in many phases
    - Example: Research, Discuss and Q&A with Human, Design and implement tests, Implementation feature, Code Review, Security Review, Fresh-eye review, merge to alpha, etc.
  - Stage
    - each phase may contain many stages
    - Example for Research: Analyze code, web search, pose research questions, combination and synthesis, etc.
  - Step
    - Each stage may be several steps.
    - Example: Search Arxiv, Search Google Schollar, Search wikipedia, Search reddit, Search Google Groups,
    - Example: Change the entire project name from "Fortias" to "Foretias".
  - Task
    - Each step may be several tasks.
    - Tasks are smaller very well defined jobs, typically using tool or simple updates.
    - Example: Alter spelling of "Fortias" to "Foretias" in all file names
    - Example: Alter spelling of "Fortias" to "Foretias" in C11 code.
    - Example: Alter spelling of "Fortias" to "Foretias" in rs code.
It is very important, given a request from user that correspond to a feature request or software change, to set a scope size. After scoping, perhaps the new request may be placed into an existing larger sized poject, or cause a split of existing project to form similar sized projects. Ultimately correctness and implementation efficiency is the goal achieved through organization, consideration and communication.

When request is small, you may combine Major/Phase/Stage into a single unit.

## Development Organization

### FOOP (Foolish Optimization Process)

FOOP documents are the Foolish equivalent of Python's PEP or Scala's SIP. They propose, discuss, and track changes to the Foolish language and its reference implementations.

- **Location**: `docs/foop/FOOP-###.md`
- **Index**: `docs/foop/INDEX.md` (canonical list, sorted by number)
- **Template**: `docs/foop/FOOP-template.md`
- **Meta-FOOP**: [FOOP-1](docs/foop/FOOP-1.md) defines the process itself

A FOOP progresses through statuses: `Draft` → `Brewing` (ready for BDFL review) → `Final` (accepted) → `Implementing` (active coding) → complete. Each FOOP is assigned to a `phase` (phase-1 through phase-7, or `meta` for process documents).
### FOOP Numbering is Little Endian
FOOP-1 is before FOOP-2, FOOP-9 is the one before FOOP-01, and so on and so forth. To list the directory in order of oldest to newest, use this command:
```bash
ls docs/foop|rev|sort -V|rev
```

### FOOP Naming Convention (Critical)
The identifier `FOOP-01` uniquely identifies an optimization step. In free text, use "FOOP 01" (no dash, space instead). This convention reduces the risk of digit reversal: writing "FOOP 01" in prose makes it harder to accidentally type "FOOP 10". In sentences, use the space form: "FOOP's 01, 11, 21 are the only pre-teen foops we will implement." Reserve the dash form `FOOP-01` for filenames, code references, and formal citations only.
*always* use this command to list the FOOPs to establish ordering.

The **filename digits ARE the identifier**. The `foop:` frontmatter
field is a separate numeric sort key, equal to the digits reversed.
Do NOT use the sort-key value as the identifier in prose. Examples:

| Filename     | Identifier (use this) | Sort key (frontmatter only) |
|--------------|-----------------------|-----------------------------|
| `FOOP-9.md`  | FOOP-9                | 9                           |
| `FOOP-01.md` | FOOP-01               | 10                          |
| `FOOP-21.md` | FOOP-21               | 12                          |
| `FOOP-51.md` | FOOP-51               | 15                          |

### FOOP Numbering Helper Script

Use `docs/foop/scripts/foop_check.py` to manage FOOP numbering. Run it
before creating a new FOOP and periodically to catch drift:

```bash
python3 docs/foop/scripts/foop_check.py check     # verify consecutive numbering
python3 docs/foop/scripts/foop_check.py get_last  # most recent FOOP
python3 docs/foop/scripts/foop_check.py gen_next  # filename for next FOOP
python3 docs/foop/scripts/foop_check.py list      # all FOOPs in chronological order
```

When creating a new FOOP, **always** run `gen_next` first to get the
correct filename and identifier. The script handles the little-endian
encoding for you.

### Plan Files for FOOP Implementation

When implementing a FOOP, write a detailed plan to `docs/foop/FOOP-###.plan.md` (lowercase extension). The plan breaks the FOOP into concrete, trackable tasks using checkboxes.

#### Checkbox Format

Checkboxes in a plan file track progress. When an item is checked off, **always place a timestamp (to the minute) on the next line with indent into the bulleted list**:

```markdown
- [ ] Task not yet done
- [x] Task completed                    ← bad (no timestamp)
- [x] Task completed                    ← good it is
      (2026-05-06 13:11)                ← timestamped properly
```

This gives both agents and humans a clear view of how work is progressing over time.

When a specification is considered VERY important but interfering with current highest priorities, it is marked with `[x] backburnered`. To be revived by removing the `[x] backburnered` marker. These plans are to be excluded when agent or human asks for plans that are: ready, pending, iterating, in progress, developing, active, etc. backburnered plans can only be found and addressed directly by using the words "backburnered plan(s)".

```markdown
- [x] backburnered
      (2026-05-06 14:00)
- [ ] Do this or system will break
- [ ] And fix that bug
- [ ] ...
```

Canceled features shall be marked as "not to be done" using the marker "[-] don't do this". An entirely deprecated plans hall have a "[x] canceled" box at the top. The agent should first add the canceled check item, then mark all todo's with per-item cancelation "[-] each one". Here is the example of properly canceled spec
```markdown
- [x] canceled. Optionally explain there's a new spec see FOOP-####
      (2026-05-06 14:00)
- [-] Do this or system will break
- [-] And fix that bug
- [-] ...
```

#### Worktree Branch Tracking

If a worktree branch is used for implementation, the plan **must** document the lifecycle of that worktree as explicit, separate checkbox tasks placed at appropriate points in the plan. The workpath shall always be:

```
WORKTREE_BRANCH_NAME=short_description-foop-<NUMBER>
WORKTREE_FULL_FS_PATH=${HOME}/tmp/foolish-worktrees/short_description-foop-<NUMBER>

## The branch is created this way from the starting branch and path
# cd $STARTING_PATH ## User normally starts in this directory
# git checkout $STARTING_BRANCH ## Again, user normally already has this branch checked out.
git worktree add -b "$WORKTREE_BRANCH_NAME" "$WORKTREE_FULL_FS_PATH"
```

The short_description in the path should be generated as part of the .plan.md generation. It is possible because the specification is already made and a short description should be possible. the "foop-<NUBER>" suffix should match the name of the foop file as well as the plan file. Once set, this path name

Agent with permission to work on the main foolish directory also has permission to work on a worktree added from the foretias directory. If asking for permission, ask once for the entire worktree branch: "${WORKTREE_FULL_FS_PATH}" not a subdirectory.

```markdown
- [ ] Create worktree at ${HOME}/tmp/foolish-worktrees/constanic-clone-foop-7 with branch `foop/foop-7-constanic-clone`
...
  (implementation tasks here)
...
- [ ] Verify all work is complete in ${HOME}/tmp/foolish-worktrees/3841-foop-7 and committed to `foop/foop-7-constanic-clone`
- [ ] Merge `foop/foop-7-constanic-clone` to alpha
```

#### Sub-Tasks

If a task proves larger than expected and splits into multiple sub-tasks, indent them under the parent. Use completed sub-tasks to justify why the split occurred:

```markdown
- [ ] Merge ${BRANCH_NAME} to ${STARTING_BRANCH}
  - [x] Detected complex merge situation requiring additional work
        (2026-05-06 14:00)
  - [ ] Update ${BRANCH_NAME} to follow new coding style
  - [ ] Update ${BRANCH_NAME} to use new API call convention
  - [x] Merged breaking changes from alpha
        (2026-05-06 14:31)
  - [ ] Repair ALL tests in ${STARTING_BRANCH} in ${STARTING_PATH}
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES will Agent continue past this point automatically!!
  - [ ] Cleanup ${WORKTREE_FULL_FS_PATH}
    - [ ] Check that _PLAN.md has all but Cleanup checkboxes completed
    - [ ] Remove "${WORKTREE_FULL_FS_PATH}"
    - [ ] This is the last checkbox to be checked in my _PLAN.md
```

This pattern is common because Foolish uses `git merge` (not rebase), so merge conflicts on `alpha` may trigger follow-up repair work.

## Development tools
Please use plugins and mcp's for performing disk operations, file searches and file edits. Use fully specified regular expressions (covering various cases), through mcp or using `sed` directly. These means of editing are much faster than regenerating the entire document. Each time regexp is used to for updates, please reread updated document before replacing original document.  Use Github mcp to perform git related actions.

When commiting to Git, always state project segment and software version and model version:
```git
Major: Refactor CLI, Phase: Discussion with Human--complete
opencode 1.14.39, Qwen3.6-27B-AWQ-BF16-INT4
```

## Development Rules
**NEVER** start file changes for project Phase or larger WHEN any tests are broken.
**NEVER** start large project segment work WHEN ANY tests are broken even if there're notes indicating those breakage are known. The test has to be manually disabled by human OR repaired and committed.

## How To Write Rust Code

This chapter applies to Rust code in both projects:

Optimize in this order:

1. **Correctness**
2. **Readability and maintainability**
3. **Testability**
4. **Efficiency**
5. **Style principles**

Do not sacrifice correctness for cleverness, abstraction, minimalism, or performance. Do not sacrifice readability unless there is a measured, justified efficiency need.

### General Rust Style

Write Rust that a careful human maintainer can understand quickly.

Prefer:

- Prefer: Explicit data flow.
- Prefer: Small functions with clear names.
- Prefer: Local reasoning over global cleverness.
- Prefer: Strong types over comments explaining weak types.
- Prefer: Exhaustive matching over implicit behavior.
- Prefer: Simple ownership over shared mutable state.
- Prefer: Boring, obvious code over clever code.

Avoid:

- Avoid: Magic behavior hidden behind traits, macros, or global state.
- Avoid: Type gymnastics that obscure intent.
- Avoid: Excessive generic abstraction.
- Avoid: Large functions that mix validation, transformation, I/O, and mutation.
- Avoid: Panics in library or protocol logic.
- Avoid: Silent error recovery in security-sensitive code.

Use comments to explain **why**, not what. If the code needs a comment to explain what it does, first try to make the code clearer.

### Project-Aware Priorities

Code is **ALWAYD** security-critical.

Rust code must make invalid protocol states difficult or impossible to represent. Prefer explicit state machines, newtypes, checked constructors, and narrow APIs. Be strict with parsing, validation, serialization, signatures, timestamps, peer identity, replay protection, and boundary checks.

Parser, compiler, and interpreter code should make phases obvious. Keep syntax trees, typed representations, lowered forms, bytecode/intermediate forms, environments, and runtime values distinct unless there is a strong reason to merge them.

### API Design

Design APIs around invariants.
Document behaviors and invariances by writing tests before coding.
Code deliberately to satisfy features.
Pass tests before comit.
Prefer constructors that validate:

```rust
impl Timestamp {
    pub fn new(value: u64) -> Result<Self, TimestampError> {
        if value == 0 {
            return Err(TimestampError::Zero);
        }

        Ok(Self(value))
    }
}

Do not expose fields that allow invalid states unless the type is intentionally plain data.

Prefer narrow public APIs. Keep modules private by default. Expose only what other modules actually need.

Use newtypes for semantically distinct values:

```rust
pub struct PeerIdBytes(Vec<u8>);
pub struct SignatureBytes(Vec<u8>);
pub struct AttestationId([u8; 32]);
```

Do not pass unrelated byte arrays, strings, or integers through the same generic type if the values mean different things.

### Error Handling

Use `Result<T, E>` for recoverable failures.

Do not use `unwrap`, `expect`, or `panic!` in production logic except when proving an internal invariant that truly cannot fail. In security, protocol, parser, compiler, interpreter, FFI, and network code, avoid them almost entirely.

Good:

```rust
let message = Message::decode(bytes)
    .map_err(ProtocolError::InvalidMessage)?;
```

Bad:

```rust
let message = Message::decode(bytes).unwrap();
```

Errors should be specific enough for callers to act on them.

Prefer domain errors:

```rust
pub enum AttestationError {
    InvalidTimestamp,
    InvalidSignature,
    UnknownPeer,
    ReplayDetected,
    StorageFailure(StorageError),
}
```

Avoid stringly-typed errors for core logic.

Error messages may be human-readable, but program logic should not depend on parsing error strings.

For Foolish diagnostics, distinguish internal errors from user-facing language errors. A syntax error in user code is not a Rust panic.

### Enum Dispatch

Matching on enums is acceptable and often preferred.

It is fine to dispatch by matching an enum and then calling a concrete method, including a fully qualified method path when that is clearer or more efficient.

Example:

```rust
match node {
    Expr::Call(call) => CallExpr::type_check(call, ctx),
    Expr::Lambda(lambda) => LambdaExpr::type_check(lambda, ctx),
    Expr::Literal(literal) => LiteralExpr::type_check(literal, ctx),
}
```

This is acceptable even if the method belongs to a trait implemented by the struct holding the data, especially when it improves readability, avoids unnecessary dynamic dispatch, or makes optimization easier.

Do not replace clear enum dispatch with trait objects solely because “polymorphism is cleaner.” Use trait objects when runtime extensibility or object-safe abstraction is genuinely useful.

Prefer enums when:

* The set of variants is known and finite.
* Exhaustiveness matters.
* State transitions must be explicit.
* Serialization/deserialization depends on variant identity.
* Compiler optimization benefits from static dispatch.

Prefer traits when:

* Multiple independent types share behavior.
* The set of implementors may grow externally.
* The API needs behavior abstraction more than variant inspection.

### Traits and Generics

Use traits to express meaningful behavior, not to hide simple function calls.

Good traits are small, named after capabilities, and have stable semantics:

```rust
pub trait Clock {
    fn now(&self) -> Result<Timestamp, ClockError>;
}
```

Avoid broad traits with many unrelated methods.

Avoid generic parameters unless they provide real value. A concrete type is often easier to read, test, and optimize.

Good:

```rust
pub fn verify_attestation(
    attestation: &Attestation,
    keyring: &Keyring,
) -> Result<(), VerificationError> {
    // ...
}
```

Do not write generic abstraction just in case future code might need it.

When using generics, keep bounds close to the function that needs them. Avoid spreading complex bounds across the codebase.

### Ownership and Borrowing

Prefer clear ownership boundaries.

Use borrowed data when the caller retains ownership:

```rust
pub fn parse_module(source: &str) -> Result<ModuleAst, ParseError>
```

Use owned data when the value must outlive the caller or cross threads/tasks:

```rust
pub struct NetworkCommand {
    pub payload: Vec<u8>,
}
```

Avoid unnecessary cloning. But do not contort code into unreadable shapes to avoid a cheap clone outside hot paths.

If cloning is meaningful or expensive, make it visible and intentional.

Use `Arc` for shared ownership across threads/tasks. Use `Rc` only in single-threaded code. Use interior mutability only when it simplifies a real ownership problem, not as a shortcut around design.

Avoid shared mutable state. If needed, isolate it behind a small API.

### Concurrency and Async

Concurrency must be explicit and testable.

For Foretias network and P2P code, separate:

* Protocol state.
* Network I/O.
* Storage.
* Cryptographic verification.
* Time sources.
* Peer management.
* Retry/backoff logic.

Do not bury protocol decisions inside async tasks where they are hard to test.

Prefer message-passing or narrow synchronization APIs over wide shared locks.

Avoid holding locks across `.await`.

Bad:

```rust
let mut state = self.state.lock().await;
self.network.send(message).await?;
state.mark_sent(id);
```

Better:

```rust
{
    let mut state = self.state.lock().await;
    state.mark_pending(id);
}

self.network.send(message).await?;

{
    let mut state = self.state.lock().await;
    state.mark_sent(id);
}
```

Keep task lifetimes clear. Every spawned task should have:

* A clear owner.
* A shutdown path.
* Error handling.
* Tests where practical.

Do not ignore `JoinHandle`s unless the task is intentionally detached and documented.

### Cryptographic and Security-Sensitive Code

For Foretias, cryptographic code must be conservative.

Never invent cryptographic protocols or alter protocol details casually.

Do not use non-constant-time comparisons for secrets, signatures, MACs, or authentication tags when constant-time comparison is required.

Do not log secrets, private keys, raw credentials, sensitive peer material, or unreduced protocol internals.

Do not continue after cryptographic verification failure unless the protocol explicitly requires it.

Validate before trust:

```rust
let signed = SignedMessage::decode(bytes)?;
signed.verify(&trusted_keys)?;
let message = signed.into_verified_message();
```

Prefer types that distinguish unverified from verified data:

```rust
pub struct UnverifiedAttestation {
    bytes: Vec<u8>,
}

pub struct VerifiedAttestation {
    inner: Attestation,
}
```

Only trusted constructors should create verified types.

Do not expose test-only shortcuts in production APIs.

### Time Handling

Do not call system time deep inside protocol logic. Inject a clock.

Good:

```rust
pub trait Clock {
    fn now(&self) -> Result<Timestamp, ClockError>;
}
```

This makes tests deterministic and prevents hidden dependencies.

Distinguish:

* Local observation time.
* Claimed timestamp.
* Verified timestamp.
* Network receive time.
* Consensus or attestation time, if applicable.

Never compare timestamps without knowing which kind they are.

### FFI and C11 Core Boundaries

Rust code that crosses into or out of the C11 core must be defensive.

FFI boundaries must:

* Validate pointers.
* Validate lengths.
* Define ownership clearly.
* Avoid panics crossing the boundary.
* Return explicit status/error codes.
* Document allocation and deallocation responsibility.
* Treat foreign data as untrusted.

Do not expose Rust references, Rust-owned layout assumptions, or panic behavior over FFI.

Use `#[repr(C)]` for FFI structs. Keep FFI types simple.

Wrap unsafe code in small safe abstractions:

```rust
pub fn verify_with_c_core(input: &[u8]) -> Result<VerificationResult, CoreError> {
    // Small, audited unsafe section.
    unsafe {
        // ...
    }
}
```

Every `unsafe` block must have a nearby safety comment explaining the invariant being upheld.

Unsafe code should be rare, isolated, and easy to audit.

### Serialization and Parsing

Parsing must be strict.

Reject malformed, ambiguous, non-canonical, or trailing data unless the format explicitly allows it.

Do not accept multiple encodings for the same logical value in security-sensitive formats unless required by protocol.

Keep parsing and validation separate when useful:

```rust
let raw = RawMessage::decode(bytes)?;
let message = raw.validate()?;
```

For Foolish, parser code should preserve source spans. Diagnostics should point to source locations wherever possible.

For Foretias, decoded wire messages must not become trusted domain objects until validation succeeds.

### Foolish Compiler and Interpreter Code

Keep language phases distinct.

Prefer separate types for:

* Tokens.
* Parsed AST.
* Desugared AST.
* Typed AST.
* Intermediate representation.
* Runtime values.
* Bytecode or lowered forms, if applicable.
* Diagnostics.

Avoid using one loose enum for every phase unless the project has deliberately chosen that architecture.

Compiler transformations should be explicit:

```rust
let tokens = lexer.lex(source)?;
let ast = parser.parse(tokens)?;
let typed = type_checker.check(ast)?;
let lowered = lowerer.lower(typed)?;
```

Each phase should be independently testable.

Interpreter behavior should be deterministic unless nondeterminism is a deliberate language feature.

Avoid mixing user-language errors with Rust implementation errors. User programs should not crash the interpreter through ordinary invalid input.

### Modules and File Organization

Organize code by responsibility, not by vague utility.

Good module names:

* `parser`
* `lexer`
* `diagnostics`
* `attestation`
* `verification`
* `wire`
* `peer`
* `storage`
* `clock`
* `ffi`

Avoid dumping unrelated helpers into large `utils` modules. A small helper module is acceptable only when the functions genuinely belong together.

Keep public module surfaces small. Re-export intentionally.

### Testing Requirements

Write tests for behavior, invariants, and edge cases.

Prefer deterministic tests. Inject clocks, RNGs, network handles, and storage backends where needed.

For Foretias, include tests for:

* Valid attestation verification.
* Invalid signatures.
* Timestamp boundary cases.
* Replay attempts.
* Malformed wire messages.
* Peer identity errors.
* Serialization round trips.
* FFI boundary failures.
* Shutdown and cancellation paths where applicable.

For Foolish, include tests for:

* Lexing.
* Parsing precedence and associativity.
* Syntax errors with spans.
* Type checking success and failure.
* Interpreter semantics.
* Compiler lowering.
* Regression cases.
* Invalid programs that should produce diagnostics, not panics.

Use property tests or fuzz tests where useful, especially for parsers, decoders, serialization, and protocol messages.

A bug fix should usually begin with writing of a a regression test that reporduces the error condition, repair, and commit of code passing new regression test.

### Performance

Write efficient Rust, but measure before making code obscure.

Prefer straightforward code unless profiling or clear algorithmic reasoning shows a problem.

Optimize algorithms before micro-optimizing syntax.

Accept enum matching, static dispatch, slices, iterators, and clear loops. Use whichever is more readable in context.

Avoid unnecessary allocations in hot paths. Prefer borrowing, slices, and preallocation where clear.

Do not introduce unsafe code for performance without strong justification and tests.

Document performance-sensitive decisions:

```rust
// This avoids allocating during peer message validation, which is on the inbound hot path.
```

### Logging and Observability

Logs should help diagnose behavior without leaking secrets.

Use structured logging where the project already does so.

Log:

* State transitions.
* Protocol failures.
* Peer connection changes.
* Retry exhaustion.
* Storage failures.
* Compiler phase failures when debugging Foolish.

Do not log:

* Private keys.
* Secret material.
* Raw credentials.
* Full untrusted payloads unless sanitized.
* User source code in contexts where that may be sensitive.

Errors should carry enough context for debugging, but not sensitive data.

### Panics and Assertions

Use `debug_assert!` for internal invariants that help catch bugs during development.

Use normal error handling for invalid external input.

External input includes:

* Network messages.
* Files.
* User source code.
* FFI input.
* Client-language bindings.
* Serialized data.
* Peer-provided data.
* Clock or storage failures.

A malformed packet, invalid program, bad timestamp, or null FFI pointer is not a reason to panic.

### Macros

Use macros sparingly.

A macro is acceptable when it removes unavoidable repetition while preserving clarity.

Avoid macros that hide control flow, error behavior, security checks, or generated public APIs.

Prefer functions, traits, or ordinary modules unless a macro is clearly better.

### Dependencies

Do not add dependencies casually.

Before adding a crate, consider:

* Security posture.
* Maintenance status.
* API stability.
* Transitive dependency weight.
* `no_std` or FFI implications, if relevant.
* Whether the project already has an equivalent dependency.
* Whether the crate affects cryptography, parsing, networking, or serialization.

For security-sensitive dependencies, prefer mature, audited, widely used crates.

Do not change cryptographic dependencies, serialization formats, protocol behavior, or public APIs without understanding compatibility and security impact.

### Client Bindings

Rust APIs exposed to Python, Java, C, or other clients must be stable, narrow, and explicit.

Do not leak internal Rust types into public binding contracts.

Separate internal errors from binding-layer errors.

Validate all foreign inputs. Convert foreign data into internal Rust domain types only after checks pass.

Binding APIs should be boring and hard to misuse.

### Code Review Checklist for AI Agents

Before finishing Rust changes, check:

* Does this preserve correctness?
* Are invalid states prevented or checked?
* Are all external inputs validated?
* Are errors explicit and useful?
* Are panics avoided in production paths?
* Is unsafe code isolated and justified?
* Are secrets protected from logs and errors?
* Is concurrency shutdown/error behavior clear?
* Are locks not held across `.await`?
* Are tests added or updated?
* Is the code readable by a human maintainer?
* Is performance acceptable without obscuring intent?
* Did public APIs, wire formats, FFI contracts, or serialized formats change?

If a change affects security, protocol compatibility, storage compatibility, language semantics, or public bindings, treat it as high-risk and document the reasoning in the code, tests, or commit notes.

### Final Rule

When uncertain, choose the design that is easiest to prove correct, easiest to test, and easiest for the next human to understand.

Correctness first. Then readability and maintainability. Then efficiency. Then principles and asethetics.


## Environment Detection

## Important Safety Guide Rails
Agents shall **NEVER** take restricted actions. For example 'chmod a+rw file' is not permitted. The most an agent can do in those respects is to suggest user to perform the action and give the
command sequence with the first word in all caps: 'CHMOD a+rw file'. This ensures that even the user cannot copy and paste it blindly. Every line of a multi-line suggestion shall have first word
case inverted. So, for example if agent suggests running program "Agent --reset-context", it shall recommend to the user to type "aGENT --reset-context".

Restricted actions are:

 * Changing permissions on any file. For example: 'chmod a+rw filename'
 * Altering maven, git and other softare configuration files, these include, not exclusively, ".gitigore", ".git", '.claude', ...
 * Never alter any approved approval files matching pattern "*.approved.foo"
 * Never alter any approved approval Foolish files matching pattern "*.approved.foo" Even if it is to change the number of steps taken.

For requesting restricted file changes, agents may suggest diff patch or full text of replacement content.


## Task Management
This project uses the todo skill for all task tracking. All todo files live
in docs/.../todo/ and are exclusively maintained by the skill — do not edit
them directly.

### Default session file
Each AI session writes to its own todo file by default:
docs/todo/AIAGENT-<session-id>.todo.md To switch to a project-specific todo
file, say "use the sprint-3 todo" or invoke /todo-use sprint-3 at any point
in the session.

### When starting any multi-step task
Before executing, read the active todo file and either map the work to
existing open items or add new ones. Write a session started Log entry
summarizing the plan and which IDs will be worked.

### While executing
Log progress on each item before starting it (in progress) and close it
with a meaningful summary when done (/todo-done, /todo-abandon, or
/todo-cancel). If new work is discovered mid-task, add it immediately.

### When finishing or pausing
Write a session ended Log entry listing what was completed, what remains,
and any context the next session needs.  General rule Keep the todo file
synchronized with actual work at all times using the commands of the skill.
It is the record of what happened, not just what is planned.


## Build Commands

All commands below run from the workspace root `/home/hcbusy/foolish-rust/foolish`.

### Rust Implementation

```bash
cd /home/hcbusy/foolish-rust/foolish

cargo check --workspace                          # Quick check (fastest validation)
cargo build --workspace                          # Build everything
cargo build --workspace --release               # Release build (LTO, stripped)
```

Binary after release: `target/release/foolish`

### Unit Tests

```bash
cargo test --workspace                           # All unit tests
cargo test -p foolish-core                       # One crate
cargo test -p foolish-core -- brane_search       # Specific test (substring match)
```

### Approval Tests (insta snapshots)

**foolish-core** approval tests use `insta` YAML snapshots stored in `foolish-core/src/snapshots/`.

**foolish-ubcb-cli** snapshot tests use `SnapshotSuite` (parallel eval via Rayon, sequential insta assertion). Snapshots live in `foolish-ubcb-cli/src/snapshots/`. When output differs from an approved `.snap`, insta writes a `.snap.new`. Review and approve with `cargo insta review` or accept all with `cargo insta accept`. To auto-accept in CI, set `INSTA_UPDATE=always`.

```bash
cargo test -p foolish-core -- approval                     # all core approval tests
cargo test -p foolish-ubcb-cli --lib                       # all UBCb snapshot suites
cargo test -p foolish-ubcb-cli --lib -- approval_all       # one suite (all files)
cargo test -p foolish-ubcb-cli --lib -- ubcb_test_literals # one file in a suite
INSTA_UPDATE=always cargo test -p foolish-ubcb-cli --lib   # auto-accept all new
cargo insta review                                         # interactive review/approve
cargo insta accept                                         # accept all .snap.new
```

### CLI Usage

```bash
cargo run -p foolish-cli -- run path/to/program.foo    # Evaluate a .foo file
cargo run -p foolish-cli -- step path/to/program.foo   # Step-by-step (debug)
cargo run -p foolish-cli -- repl                       # Interactive REPL
```

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

Separate languages read from the same test input resources directory to produce their own approval output.
A crossvalidation process checks that implementations in different languages are behaving identically.

**Snapshot workflow**: Run a test → if output differs, insta writes `.snap.new` →
`cargo insta review` to interactively approve/reject → `cargo insta accept` to bulk accept →
`INSTA_UPDATE=always` to auto-accept (useful in CI).

## Clarifications
* When user mentions "path/" first interpret it as relative path from the directory where claude code was invoked. This is normal behavior for most unix apps, for example if I "cat path/file" that path is resolved from the current path.
* Never directly edit `.approved.foo` files

### The Unicellular Brane Computer (UBC)

The **UBC is the reference implementation of Foolish**. It implements a unique evaluation model based on branes (containment structures).

#### FIR (Foolish Internal Representation)

FIR objects represent expressions during evaluation and progress through a multi-stage state machine:

```
## TBD: put NYSE state here:
- `CONSTANT`: ...
- `CONSTANIC` (say "CON-STAN-NICK"): CONSTANt IN Context - evaluation paused due to missing information (unbound identifiers)
...

#### Brane Reference Semantics: AB and IB

**Ancestral Brane (AB)** and **Immediate Brane (IB)** are critical context for name resolution:
- **IB**: Current context accumulated so far (lines before current expression)
- **AB**: Parent brane context containing the defining expression and its AB/IB

**Detachment and Coordination**: When a brane is referenced by name:
1. The brane was already partially resolved in its original AB/IB context
2. A clone is **detached** from its original AB/IB
3. The clone is **recoordinated** with new AB (the containing brane) and new IB (preceding lines)
4. Previously failed name searches can now resolve in the new context

In UBC implementation, this means creating a modified clone with new context. See `docs/vintage_legacy/ECOSYSTEM.md` for detailed semantics.

### Test Infrastructure

**Three-Tier Testing:**

1. **Unit Tests** (`*UnitTest.java`) - focused component tests in Java and Scala modules
2. **Approval Tests** (`*ApprovalTest.{java,scala}`) - snapshot-based integration tests in Java and Scala modules
3. **Cross-Validation** (separate `foolish-crossvalidation` module) - Cross validation will test different parser/compiler/vm implementations to check whether they behave the same way.

#### Approval Test Workflow
TBD
### Foolish Terminology (from STYLES.md)

- **Foolisher** - developer/user of Foolish
- **Nye** (say "nigh") - Not Yet Evaluated
- **NYES** (say "nice") - Not Yet Evaluated State
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

### Computational Tools Preference
When it is available, prefer to use python repl to perform math calculations, complex string manipulations, or even to perform regular expression substitutions.

## Documentation

### Directory Structure

- **`docs/ubc1/how`** - UBC1 engineering documentation - operational semantics, implementation details, reference
- **`docs/ubc1/todo`** - UBC1 project tracking - active development roadmap
- **`docs/ubc0_1/how`** - UBC0_1 engineering documentation
- **`docs/ubc0_1/todo`** - UBC0_1 project tracking
- **`docs/howto`** - "How to Express it in Foolish" - literate programming tutorials as .foo files
- **`docs/why`** - "Philosophy of Foolish" - origins, inspirations, design philosophy
- **`docs/vintage_legacy`** - Legacy documentation (being reorganized into the above directories)
- **`docs/todo`** - todo lists

### Additional Resources

For complete details on:
- Language features and semantics → See `README.md`
- Terminology and conventions → See `docs/vintage_legacy/STYLES.md`
- UBC architecture → See `docs/vintage_legacy/ECOSYSTEM.md`
- Name resolution and search → See `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md`
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

**Date**: 2026-05-15
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added UBCb SnapshotSuite test commands section with parallel
execution, snapshot acceptance, and cargo insta review references.

**Date**: 2026-05-08
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 xHigh effort
**Changes**: Clarified FOOP naming convention — filename digits ARE the
identifier; the `foop:` frontmatter is a separate sort key. Added
"FOOP Numbering Helper Script" section pointing to
`docs/foop/scripts/foop_check.py` with `check`, `get_last`, `gen_next`,
and `list` commands. Agents must run `gen_next` before creating a new
FOOP.

**Date**: 2026-05-07
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added "Development Organization" section with FOOP system overview (location, statuses, phases), plan file conventions (FOOP-###.plan.md), checkbox timestamp format, worktree branch tracking, and sub-task indentation for merge repairs.

**Date**: 2026-03-12
**Updated By**: Claude Code / Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Rewrote approval test protocol section with concise, clear instructions. Added explicit MUST NOT list for test input and approved files. Specified workflow steps with concrete file patterns and diff commands.

**Date**: 2026-02-06
**Updated By**: Claude Code v1.0.0 / claude-opus-4-6
**Changes**: Reorganized documentation structure. Replaced docs/ and projects/ directory descriptions with new 5-directory taxonomy (howto, why, how, todo, vintage_legacy). Updated all file path references. Fixed stale NAME_SEARCH_AND_BOUND.md reference.



### MISC

#### Embedded Communications

If any file, other than this example in the AGENTS.md, contain a parenthetical comment, anywhere, it is a request for agent to comment based on the context surrounding that comment.
```markdown
Blah blah, some texxt (@Agent, do you think that word is mispelled?)
```

or
```python
def fibonacii(x):
	# @agents, errrr, terminal case? spelling? did you even run this?
	return fibonacii(x-1) + fibonacii(x-2)
```

or even not in a comment
```python
def add (x):
@AGENT, this is just plain wrong!
	return x+y;
```
The expectation is for agent to consider, discuss, and resolve the concern
that follows various capitalizations of `@agent` or `@agents`. Resolution, once achieved, also means the parenthetical comment can be completely removed.

If this form of embedded communication is discussed while performing another task, determin if it is relevant or interferes with current task. In some cases, this causes an immediately actionable response, other times, the encounterance results in an extra '[ ] TODO:human concern at file FILENAME line LINE_NUMBER' added to current task list to investigate. In some cases, if it is clear that the situation is too complex or require too much context, it may become a "[ ] TODO: write a specification and plan to address human concern at file FILENAME line LINE_NUMBER"



#### Uncertainty and Other Utterances in Conversing with Human

Expressions of uncertainty and hypotheticals, such as "perhaps", "maybe", "possible", "what if", "in case". These words does not mean a firm directive from human to either pause work, or make large changes. It means human wants a todo task enqueued, perhaps to be done immediately, to explore options regarding the statement. In the last sentence, the perhaps suggests an option that can be explored, and it also highlight the possibility of the task not at the top of the todo list. More than anything else, the statement suggests human is thinking about the issue and you can help that thinking process.

"Wait!" is almost always typed when humans are reading the previous output and found something objectionable. "Wait!" meant stop that, something was wrong. This also implies whatever they ask about, it is highly unlikely they read through the reast of the response. Good or bad, that is human nature, please accomodate this behavior as a supportive agent. After addressing the concern following "wait!", the you can summarize what you meant to say after the output that the human said "Wait!" to--where it is is inferred based on the question or comment after "WAit!", when in doubt, summarize the whole response in the context of having addressed the human's concern.

"Continue." is uttered when the humans sees output on the screen that they think is incomplete. The best course of action, irrespective of actual status, is to summarize the progress made in the most recent few turns of conversation. If indeed the progress was ended or blocked by nonresponsive sub-agents, then take approrpiate action. If the short term task is truely complete, still output the summary, but also present outstanding todo items as well as other possible next steps for human to decide. Human may decide previous task is not complete and needs more work, or they may agree previous task was complete and move on to one of the options for next steps.


#### When in Doubt

When uncertain, choose the design that is easiest to prove correct, easiest to test, and easiest for the next human to understand.

Correctness first. Then readability and maintainability. Then efficiency. Then principles and asethetics.

