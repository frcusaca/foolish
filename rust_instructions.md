# rust_instructions.md

Instructions for AI coding agents writing Rust in this repository. These are
**directives, not suggestions.** Every rule traces either to this project's own
conventions or to a real-world agent file, a curated rule set, or the Rust
project's own documentation. Where an external source backs a rule, a numbered
citation `(c#)` appears inline; the **Citations** section at the bottom resolves
them.

> **Citations are for maintaining this document, not for everyday coding.** You
> do not need to read or follow the sources while writing code — they exist so a
> future maintainer can verify, update, or challenge a rule. Follow the rule
> text itself.

The document is organized by the *strength and shape* of each instruction:

- **Priorities** — the prioritizing statements: what to optimize for and which
  construct wins when two compete.
- **Language patterns** — prescribed instructions stated *with the conditions*
  under which they apply.
- **Do's** — affirmative mandates, including the hard tooling gates.
- **Preferences** — weaker, softer defaults you should lean toward.
- **Don'ts** — prohibitions and anti-patterns.
- **Project-specific rules** — Foolish, Foretias, FFI, and binding rules that are
  particular to this repository and override any general guidance on conflict.

Some rules appear in more than one section by design (e.g. "use `?`" is a Do,
and "never `.unwrap()` in production" is a Don't).

---

## Project baseline

- **Edition:** `2024` (Rust 1.85+) — unlocks let-chains, async closures, and the
  2024 drop/temporary-scope semantics. All crates in the `foolish/` workspace are
  already on edition 2024. *(c1)*
- **MSRV:** state it in `Cargo.toml`; do not use features newer than it. Where a
  rule below is version-gated and the MSRV predates it, that rule does not apply.
- **Toolchain:** stable only, no nightly features. *(c2)*
- **Error crates already in the workspace:** `thiserror` 2 (matchable enums) and
  `anyhow` 1 (opaque application errors). Use these; do not add competing error
  crates.

---

## 1. Priorities

Two prioritizing axes apply. The **optimization order** governs what to optimize
*for*; the **construct-preference order** governs which *construct* wins when two
compete. They do not conflict — the first is about goals, the second about means.

### 1a. Optimization order (this project's law)

When goals compete, the earlier wins. Do not sacrifice an earlier goal for a
later one.

1. **Correctness** (and soundness — unsound code is never acceptable;
   `clippy::correctness` lints are bugs, not style). *(c3)*
2. **Readability and maintainability.**
3. **Testability.**
4. **Efficiency.**
5. **Style principles** — and delegate as much style as possible to the tools
   (rustfmt, clippy, RFC 430), so you spend no effort on whitespace, import
   order, or casing they already enforce.

Do not sacrifice correctness for cleverness, abstraction, minimalism, or
performance. Do not sacrifice readability unless there is a measured, justified
efficiency need.

### 1b. Construct-preference order

When two constructs compete, the earlier-named or left-hand side wins.

1. **Borrowing over cloning.** A clone is a deliberate runtime cost to be
   justified, never a borrow-checker workaround. Prefer, in order: borrow →
   restructure lifetimes → split the borrow → move → (last) clone.
2. **Immutability over mutability.** `mut` and `&mut` are the exception; reach
   for them only when you actually mutate.
3. **Encapsulation over exposure.** Private fields and behavior-based APIs over
   public fields and raw state.
4. **Make illegal states unrepresentable** over validating at call sites — encode
   invariants in the type system. *(c4, c5)*
5. **Types over generics, generics over `dyn`.** Reach for dynamic dispatch only
   when you need it. *(c6)*
6. **Standard traits over ad-hoc methods.** `From`/`TryFrom`/`Display`/`FromStr`
   over bespoke `to_x`/`from_x`. *(c7, c8)*
7. **Iterators over manual index loops; pattern matching over `unwrap` chains.**
8. **`std` over third-party crates** where `std` now suffices (`LazyLock`/
   `OnceLock` over `lazy_static`/`once_cell`).
9. **Error propagation over panicking.** `?` and typed errors over
   `.unwrap()`/`.expect()`.
10. **Compile time matters.** Prefer fewer, well-scoped crates and avoid
    gratuitous proc-macro dependencies. *(c9, c10)*

---

## 2. Language patterns

Prescribed instructions, each stated with the condition that triggers it.

### Ownership & borrowing
- **When writing a function signature**, take `&str` not `&String`, `&[T]` not
  `&Vec<T>`, `&T` not `T` — unless the function stores, consumes, or returns the
  value. *(c11)*
- **When the caller retains ownership**, take borrowed data
  (`fn parse_module(source: &str) -> Result<ModuleAst, ParseError>`); take owned
  data only when the value must outlive the caller or cross threads/tasks.
- **When a borrow removes an allocation at the call site**, accept
  `impl AsRef<str>` / `impl AsRef<Path>` / `impl IntoIterator<Item = T>`.
- **When a value is usually borrowed but occasionally owned**, return/store
  `Cow<'_, T>` instead of cloning unconditionally.
- **When you need shared ownership**, use `Arc` across threads/tasks and `Rc`
  only in single-threaded code. Use interior mutability only when it simplifies a
  real ownership problem, not as a shortcut around design.
- **When you reach for `Arc<Mutex<T>>` reflexively**, stop — it usually signals
  the data model is wrong; use it only for genuine shared ownership, isolated
  behind a small API. *(c11)*
- **When a lock guard would cross an `.await`**, restructure so it doesn't; use
  `tokio::sync` primitives for state held across await points and
  `spawn_blocking` for CPU-bound work. *(c11)*
- **After changing a function body**, re-audit every variable and parameter
  declared `mut` (and every `&mut`): if it is no longer mutated, downgrade it to
  immutable / `&T`, and remove all now-unnecessary cloning. This can cascade —
  removing one `&mut` may make a caller's binding, and that caller's parameter,
  no longer need `mut` either; follow the chain and downgrade each link to
  read-only as far as it propagates.

### Encapsulation & types
- **When constructing a type with invariants**, route through
  `new`/`try_new`/builders that validate, not struct literals from outside the
  module. A type's invariants are enforced by its own constructor/methods, making
  it impossible to construct an invalid instance from outside.
- **When a value has a distinct meaning** (an ID, an email, signature bytes), use
  a newtype (`struct UserId(u64)`) not a type alias, and do not route unrelated
  values through one generic byte/string/integer type.
- **When behavior reasons about a type's data**, put it in an `impl` block on the
  type that owns the data — do not write free functions that reach into the data
  structure. State and methods travel together; fields stay private; callers go
  through methods.
- **When a value must change *type* (not just data)**, use the typestate pattern:
  consume `self` and return the new type for the caller to swap in (e.g. a
  `Search` resolving to an `Int`). When a value changes data but not type,
  self-mutate via `&mut self`.
- **When a type answers a question about itself** (a predicate or projection like
  `state()`, `is_search()`, `as_int()`), expose it as a method that hides the
  `match` inside — do not force callers to match on external tags or variants.
  Reports return owned values or short-lived borrows, never a long-lived handle
  that lets a caller mutate shared state behind the owner's back.
- **When a public enum or struct may gain variants/fields later**, mark it
  `#[non_exhaustive]`.
- **When a function returns a `Result` or a value pointless to discard**, mark it
  `#[must_use]`.
- **When matching on your own enum**, enumerate variants — avoid a catch-all `_`
  so new variants force a compile error.

### Loops & matching
- **When transforming or filtering a collection**, use an iterator chain
  (`map`/`filter`/`filter_map`/`fold`/`zip`/`enumerate`/`windows`/`chunks`) and
  `collect`, not an indexed `for i in 0..n` loop.
- **When a `collect` can fail**, collect into `Result<Vec<_>, _>` with `?` to
  short-circuit.
- **When inserting-or-updating a map**, use the `entry()` API, not a double
  lookup.
- **When the final size is known**, pre-allocate with `with_capacity`.
- **When binding refutably on the happy path with an early return otherwise**,
  use `let … else` (stable 1.65):
  ```rust
  let Some(user) = lookup(id) else { return Err(Error::NotFound); };
  ```
- **When MSRV ≥ 1.88 and edition 2024**, flatten nested conditionals with
  let-chains: `if let Some(x) = a && x.is_valid() && let Ok(y) = f(x) { … }`.
  *(c12)* Otherwise use nested `if let` / `match`.
- **When checking a discriminant for a boolean**, use `matches!` instead of a
  full `match`.
- **When matching on a `&T`**, bind by reference; use `@` bindings, `|`
  alternatives, range patterns, and guards instead of nested `if`s.

### Traits & generics
- **When converting between types**, implement `From`/`TryFrom` (and let `?` use
  the `From` impl for error conversion) rather than ad-hoc methods.
- **When a function returns an iterator**, return `impl Trait` rather than boxing.
- **When defining a trait**, keep it small and named after a capability with
  stable semantics (`trait Clock { fn now(&self) -> Result<Timestamp, ClockError>; }`).
  Avoid broad traits with many unrelated methods.
- **When using generics**, keep bounds close to the function that needs them and
  do not spread complex bounds across the codebase. Use a concrete type until a
  real abstraction or a second caller justifies the generic.
- **When dispatching over a known, finite set of variants**, prefer matching the
  enum and calling a concrete method over a trait object — see *Enum dispatch*
  under Project-specific rules.
- **When writing new trait code with async methods (MSRV ≥ 1.75)**, use native
  `async fn` in traits / RPITIT; fall back to the `async-trait` crate only when
  you need `dyn` dispatch. *(c13)*
- **When a public type is defined**, derive `Debug` (and `Clone`/`PartialEq`/
  `Eq`/`Hash`/`Default`/`Ord` where sensible). *(c8, c14)*

### Globals & formatting
- **When you need a lazily-initialized static (MSRV ≥ 1.80)**, use
  `std::sync::LazyLock` (or `OnceLock`, 1.70); use `once_cell` only below that.
  Prefer avoiding global mutable state entirely. *(c15, c16)*
- **When interpolating a variable into a format string**, inline it:
  `format!("{x}")`. (Field access like `{self.x}` still needs the positional
  form.)

### Errors
- **When callers must branch on the failure mode**, define a matchable error enum
  with `thiserror`, with domain-specific variants. *(c17)*

  ```rust
  pub enum AttestationError {
      InvalidTimestamp,
      InvalidSignature,
      UnknownPeer,
      ReplayDetected,
      StorageFailure(StorageError),
  }
  ```
- **When callers only report or propagate**, use an opaque error — `anyhow`/
  `eyre` with `.context(...)` in applications; don't mix application error types.
  *(c18)*
- **When wrapping an underlying error**, preserve the chain with `#[source]` /
  `#[from]` and use `#[error(transparent)]`.
- **When recovering from a failure**, return `Result<T, E>` — error messages may
  be human-readable, but program logic must never depend on parsing error
  strings.

### `unsafe`
- **When `unsafe` is unavoidable**, keep the block minimal, wrap it in a small
  safe abstraction, document invariants in a `// SAFETY:` comment with plain-text
  reasoning, and ensure it passes Miri. Unsafe code should be rare, isolated, and
  easy to audit. *(c19)*

---

## 3. Do's

Affirmative mandates. A change is not complete until these hold.

### Tooling gates (hard — run before "done")
```bash
cargo fmt --all                                            # rustfmt owns formatting
cargo clippy --all-targets --all-features -- -D warnings   # warnings are errors
cargo test          # or: cargo nextest run
```
*(c4, c20)*

- **Do** configure lints centrally in `[workspace.lints]` / `[lints]` (Rust
  1.74+): deny `clippy::correctness`, warn and cherry-pick from
  `clippy::pedantic`.
- **Do** override a lint with `#[expect(...)]` plus a one-line reason, so stale
  overrides surface when fixed. *(c4, c21)*
- **Do** use `?` to propagate errors. (Reserve `expect` for true invariants, with
  a message saying *why* it cannot fail.)
- **Do** keep fields private and expose behavior; scope visibility with
  `pub(crate)` / `pub(super)`, and keep public module surfaces small with
  intentional re-exports.
- **Do** re-audit mutability after changing a function: if a `mut` variable or
  `&mut` parameter is no longer mutated, downgrade it to read-only, and follow
  the resulting cascade outward to callers.
- **Do** prefer iterator adaptors, `let … else`, `match`, and `matches!` over
  `.unwrap()` chains.
- **Do** derive standard traits and implement `From`/`Display`/`FromStr`.
- **Do** write tests *first* — write the most important behaviors, invariants,
  and unclear corner cases as tests before coding, so the tests document the
  feature. Pass the tests before committing. (See *Testing* in Project-specific
  rules for what to cover.)
- **Do** start a bug fix with a regression test that reproduces the failure, then
  repair, then commit the code that passes the new test.
- **Do** write `///` docs on public items: first sentence one line (≤ ~15 words),
  with `# Examples` / `# Errors` / `# Panics` / `# Safety` where they apply, and
  use `?` (not `.unwrap()`) in doc examples. *(c11, c22)*
- **Do** comment to explain *why*, not *what*. If a comment is needed to explain
  what the code does, first make the code clearer.
- **Do** follow RFC 430 casing: acronyms as words (`Uuid`, `parse_xml`).

### Self-check before submitting
1. Did I add any `.clone()`? Can each be a borrow or move?
2. Is every `mut` actually mutated? Is every `&mut` written through? After
   editing a function, did I re-check and downgrade any now-unused `mut`/`&mut`,
   following the cascade to callers?
3. Are new struct fields private with intentional visibility?
4. Did I use an index loop where an iterator reads clearer?
5. Any `.unwrap()`/`.expect()` that should be `?` or `let else`?
6. Did I match exhaustively and destructure rather than poke at fields?
7. Did I derive standard traits and use `From`/`Display`/`FromStr`?
8. Are lazy statics on `LazyLock`/`OnceLock`?
9. Are format args inlined (`{x}`)?
10. Did I write/run tests first, and a regression test for any bug fix?
11. Did I touch generated/vendored/do-not-edit code? (Revert if so.)
12. Does it pass `cargo fmt --check`, `cargo clippy -D warnings`, and tests?

---

## 4. Preferences

Weaker directives — defaults to lean toward, not hard gates.

- **Prefer** explicit data flow, small functions with clear names, local
  reasoning over global cleverness, and boring, obvious code over clever code.
- **Prefer** descriptive local names; the lowercased type name is a fine default.
  *(c9)*
- **Prefer** grouping imports std → external → `crate::` → `super`, with a blank
  line between groups.
- **Prefer** a plain `for` loop for purely side-effecting work over forcing
  `for_each` just to look functional.
- **Prefer** concrete types until a second caller or real abstraction justifies a
  generic.
- **Prefer** error combinators (`map`, `and_then`, `ok_or_else`,
  `unwrap_or_else`, `map_err`) where they read more clearly than a `match`.
- **Prefer** fewer crates; be wary of small helper crates and proc-macro deps
  that add compile time (`itertools`/`either` are reasonable exceptions).
  *(c9, c10)*
- **Prefer** avoiding global mutable state even where a `static` would compile.
  *(c16)*
- **Prefer** keeping a type and its `impl` blocks together in one module.
- **Prefer** organizing code by responsibility (`parser`, `lexer`,
  `diagnostics`, `wire`, `storage`, `clock`, `ffi`) over dumping helpers into a
  large `utils` module.
- **Prefer** macros sparingly: a macro is acceptable only when it removes
  unavoidable repetition while preserving clarity. Reach for functions, traits,
  or ordinary modules first.
- **Prefer** measuring before obscuring code for performance: optimize algorithms
  before micro-optimizing syntax, and document any performance-driven decision
  with a `//`-comment saying why.

---

## 5. Don'ts

Prohibitions and anti-patterns. The right-hand side is the replacement.

- **Don't `.clone()` to satisfy the borrow checker.** → borrow `&T`, restructure,
  or move.
- **Don't add `mut` you don't use** (binding or `&mut` parameter). → immutable
  binding / `&T`.
- **Don't `.unwrap()` / `.expect()` / `panic!` in library, protocol, parser,
  interpreter, FFI, or production paths.** → `?`, `let … else`, real error types.
  *(c2)*
- **Don't expose `pub` struct fields** to skip an accessor (unless the type is
  intentionally plain data). → private fields + `pub(crate)` + newtypes.
- **Don't use stringly-typed errors or `Box<dyn Error>` everywhere.** →
  `thiserror` enums (matchable) / `anyhow` (opaque), chosen by caller intent.
- **Don't hand-write `Display`/`From`/`Default`** that derive or `thiserror`
  gives you. → derive macros.
- **Don't write `&Vec<T>` / `&String` parameters.** → `&[T]` / `&str` (or
  `impl AsRef<…>`).
- **Don't loop by index to build a `Vec`.** → iterator chain + `collect`.
- **Don't double-lookup a map to insert/update.** → `entry()` API.
- **Don't use `lazy_static!` / bare `once_cell`** on a modern MSRV. →
  `LazyLock` / `OnceLock`.
- **Don't keep `extern crate`** (pre-2018). → plain `use`.
- **Don't use the `try!(...)` macro.** → `?`.
- **Don't add `mod.rs` files.** → path-based modules (`foo.rs` + `foo/`).
- **Don't write `format!("{}", x)`.** → `format!("{x}")`.
- **Don't reach for `async-trait` in new code** unless you need `dyn`. → native
  async fn in traits (1.75).
- **Don't use `#[allow(lint)]`.** → `#[expect(lint)]` with a reason.
- **Don't use `UUID` / `parse_XML` casing.** → `Uuid` / `parse_xml` (RFC 430).
- **Don't hold a `Mutex`/`RwLock` guard across `.await`.** → `tokio::sync`
  primitives / restructure.
- **Don't catch-all `_` on your own enums.** → enumerate variants.
- **Don't replace clear enum dispatch with trait objects** just because
  "polymorphism is cleaner." → match the enum; use `dyn` only for genuine runtime
  extensibility.
- **Don't write large functions** that mix validation, transformation, I/O, and
  mutation. → split by responsibility.
- **Don't bury protocol/decision logic inside async tasks** where it can't be
  tested. → separate protocol state from I/O.
- **Don't edit generated, vendored, or do-not-edit code.** *(c7)*
- **Don't write unsound code, ever.** *(c3)*

---

## 6. Project-specific rules

These are particular to this repository. **On any conflict with the general
guidance above, these win.**

### Project-aware priority: code is always security-critical
Treat all Rust here as security-critical. Make invalid protocol states difficult
or impossible to represent: prefer explicit state machines, newtypes, checked
constructors, and narrow APIs. Be strict with parsing, validation, serialization,
signatures, timestamps, peer identity, replay protection, and boundary checks.

### Foolish semantic immutability vs FIR evaluation state
Foolish statements are immutable and invariant once written, **except searches.**
FIR in the FVM may change as evaluation progresses, but at every step it must
still faithfully denote the same Foolish it came from, read through its current
Nyes (Not Yet Evaluated State). The *meaning* of a Foolish expression is fixed by
its text; only its searches are indeterminate until resolved. FIR is the evolving
record of evaluating that fixed meaning — its state transitions track progress,
not a change in meaning.

### Enum dispatch
Matching on an enum and calling a concrete method is acceptable and often
preferred, including a fully qualified method path when clearer or more efficient:

```rust
match node {
    Expr::Call(call) => CallExpr::type_check(call, ctx),
    Expr::Lambda(lambda) => LambdaExpr::type_check(lambda, ctx),
    Expr::Literal(literal) => LiteralExpr::type_check(literal, ctx),
}
```

This is fine even when the method belongs to a trait implemented by the struct
holding the data. Prefer **enums** when the variant set is known and finite,
exhaustiveness matters, state transitions must be explicit, serialization depends
on variant identity, or static dispatch helps optimization. Prefer **traits**
when multiple independent types share behavior, the implementor set may grow
externally, or you need behavior abstraction more than variant inspection.

### Compiler / interpreter phase separation
Keep language phases distinct — prefer separate types for tokens, parsed AST,
desugared AST, typed AST, intermediate representation, runtime values, lowered
forms, and diagnostics. Avoid one loose enum for every phase unless the project
deliberately chose that. Make transformations explicit and independently testable:

```rust
let tokens = lexer.lex(source)?;
let ast = parser.parse(tokens)?;
let typed = type_checker.check(ast)?;
let lowered = lowerer.lower(typed)?;
```

Interpreter behavior is deterministic unless nondeterminism is a deliberate
language feature. A syntax error in user code is a diagnostic, not a Rust panic;
never let ordinary invalid user input crash the interpreter, and never mix
user-language errors with Rust implementation errors.

### Serialization & parsing
Parsing must be strict: reject malformed, ambiguous, non-canonical, or trailing
data unless the format explicitly allows it, and don't accept multiple encodings
for one logical value in security-sensitive formats. Keep parsing and validation
separate where useful (`let raw = RawMessage::decode(bytes)?; let msg = raw.validate()?;`).
Foolish parser code preserves source spans; diagnostics point to source
locations. Foretias decoded wire messages do not become trusted domain objects
until validation succeeds.

### Cryptographic and security-sensitive code (Foretias)
- Never invent cryptographic protocols or alter protocol details casually.
- Use constant-time comparison for secrets, signatures, MACs, and auth tags where
  required.
- Validate before trust, and prefer types that distinguish unverified from
  verified data:
  ```rust
  let signed = SignedMessage::decode(bytes)?;
  signed.verify(&trusted_keys)?;
  let message = signed.into_verified_message();
  ```
  Only trusted constructors create verified types. Do not continue after a
  verification failure unless the protocol explicitly requires it.
- Do not log secrets, private keys, raw credentials, sensitive peer material, or
  unreduced protocol internals. Do not expose test-only shortcuts in production
  APIs.

### Time handling
Do not call system time deep inside protocol logic — inject a `Clock` trait so
tests are deterministic. Distinguish local observation time, claimed timestamp,
verified timestamp, network receive time, and consensus/attestation time. Never
compare timestamps without knowing which kind they are.

### Concurrency & async
Make concurrency explicit and testable. For Foretias network/P2P code, separate
protocol state, network I/O, storage, cryptographic verification, time sources,
peer management, and retry/backoff. Prefer message-passing or narrow
synchronization over wide shared locks; never hold a lock across `.await`. Every
spawned task has a clear owner, a shutdown path, error handling, and tests where
practical — don't ignore a `JoinHandle` unless the task is intentionally detached
and documented.

### FFI and C11 core boundaries
Rust crossing into or out of the C11 core must be defensive: validate pointers
and lengths, define ownership clearly, avoid panics crossing the boundary, return
explicit status/error codes, document allocation/deallocation responsibility, and
treat foreign data as untrusted. Use `#[repr(C)]` for FFI structs and keep them
simple. Do not expose Rust references, Rust layout assumptions, or panic behavior
over FFI. Wrap unsafe in a small, audited safe function with a `// SAFETY:`
comment.

### Client bindings
Rust APIs exposed to Python, Java, C, or other clients must be stable, narrow,
and explicit. Do not leak internal Rust types into binding contracts. Separate
internal errors from binding-layer errors. Validate all foreign inputs and
convert them into internal domain types only after checks pass. Binding APIs
should be boring and hard to misuse.

### Logging & observability
Log state transitions, protocol failures, peer connection changes, retry
exhaustion, storage failures, and compiler-phase failures when debugging Foolish.
Never log private keys, secret material, raw credentials, unsanitized untrusted
payloads, or user source code where that may be sensitive. Errors carry enough
context to debug, but no sensitive data.

### Dependencies
Do not add dependencies casually. Before adding a crate, weigh security posture,
maintenance status, API stability, transitive weight, `no_std`/FFI implications,
whether the project already has an equivalent, and whether it touches
cryptography, parsing, networking, or serialization. Prefer mature, audited,
widely used crates for security-sensitive needs. Do not change cryptographic
dependencies, serialization formats, protocol behavior, or public APIs without
understanding compatibility and security impact.

### Panics & assertions
Use `debug_assert!` for internal invariants during development; use normal error
handling for invalid external input — network messages, files, user source code,
FFI input, client-binding input, serialized data, peer-provided data, clock or
storage failures. A malformed packet, invalid program, bad timestamp, or null
FFI pointer is not a reason to panic.

### Testing
Write tests for behavior, invariants, and edge cases, and prefer deterministic
tests (inject clocks, RNGs, network handles, storage backends). For **Foretias**:
valid attestation verification, invalid signatures, timestamp boundaries, replay
attempts, malformed wire messages, peer identity errors, serialization round
trips, FFI boundary failures, shutdown/cancellation. For **Foolish**: lexing,
parsing precedence/associativity, syntax errors with spans, type-check success
and failure, interpreter semantics, compiler lowering, regression cases, and
invalid programs that must produce diagnostics rather than panics. Use property
or fuzz tests for parsers, decoders, serialization, and protocol messages.

### Debugging via unit tests
The easiest way to diagnose parser or FVM logic errors is to write the
offending code into a temporary unit test named
`temporary_reproduce_to_debug_description`. Inside this test you can inspect
the parse tree, take a controlled number of FVM steps while monitoring NYES
state and FIR tree structure, and inspect computed values. This is the
preferred method for debugging logic that is not obvious from snapshot diffs.
The intent is to repair the bug and then remove the temporary test. If a
legitimate regression test can be made to detect the same problem, rename it
appropriately and check it in with documentation.

> **Snapshot/approval tests are never auto-accepted.** AI agents must never run
> `cargo insta accept` or `INSTA_UPDATE=always`. Generate `.snap.new`, present to
> the human, and wait for explicit approval. See `AGENTS.md` for the full
> approval workflow and signature verification.

### Final rule
When uncertain, choose the design that is easiest to prove correct, easiest to
test, and easiest for the next human to understand. Correctness first, then
readability and maintainability, then efficiency, then principles and aesthetics.

---

## Disclosure

Disclose AI involvement in commit messages and PR descriptions, and ensure all
AI-generated code is human-verified before submission. *(c23, c24)*

---

## Citations

> **Maintenance reference only.** These sources back the rules above for the
> benefit of whoever updates or audits this document. You do not consult them to
> write code — follow the rule text.

- **c1** — Rust Blog, "Announcing Rust 1.85.0 and Rust 2024," 2025-02-20.
- **c2** — nushell/AGENTS.md (stable-only toolchain; "Never use `.unwrap()`").
- **c3** — Microsoft Rust guidelines, M-UNSOUND.
- **c4** — astral-sh/ruff project conventions (illegal-states-unrepresentable;
  central lint config; `#[expect]`; CI gates).
- **c5** — Microsoft Rust guidelines, M-STRONG-TYPES.
- **c6** — Microsoft Rust guidelines, M-DI-HIERARCHY.
- **c7** — Azure/azure-sdk-for-rust conventions (standard-trait conversions;
  do-not-edit generated code).
- **c8** — Rust API Guidelines, C-CONV and C-COMMON-TRAITS.
- **c9** — rust-analyzer style guide (naming; fewer crates; compile time).
- **c10** — Microsoft Rust guidelines, M-SMALLER-CRATES.
- **c11** — leonardomso/rust-skills (signatures; `Arc<Mutex>` smell; locks across
  await; doc examples).
- **c12** — Rust Blog, "Announcing Rust 1.88.0," 2025-06-26 (let-chains).
- **c13** — Rust Blog, async fn in traits, 2023-12-21.
- **c14** — Microsoft Rust guidelines, M-PUBLIC-DEBUG.
- **c15** — Rust Blog, `LazyLock` stabilization, 2024-07-25.
- **c16** — Microsoft Rust guidelines, M-AVOID-STATICS.
- **c17** — Microsoft Rust guidelines, M-ERRORS-CANONICAL-STRUCTS.
- **c18** — Microsoft Rust guidelines, M-APP-ERROR.
- **c19** — Microsoft Rust guidelines, M-UNSAFE.
- **c20** — astral-sh/ruff and uv CI (fmt + clippy `-D warnings` + test gates).
- **c21** — Microsoft Rust guidelines, M-LINT-OVERRIDE-EXPECT.
- **c22** — Microsoft Rust documentation guidelines.
- **c23** — rust-analyzer CLAUDE.md (AI disclosure / human verification).
- **c24** — pola-rs/polars AI_POLICY.md.

---

## Last Updated

**Date**: 2026-06-09
**Updated By**: Claude Code (Claude Code); Opus 4.8
**Changes**: Created `rust_instructions.md` by merging all Rust guidance from
`AGENTS.md` "How To Write Rust Code" into the cited general-Rust draft. Kept both
priority axes (project optimization order + construct-preference order).
Renumbered all inline citations to `(c#)` with a maintenance-only Citations
section. Added a Project-specific rules section (Foolish semantics, enum dispatch,
phase separation, crypto, time, concurrency, FFI, bindings, logging, deps, panics,
testing) that overrides general guidance on conflict. Confirmed all workspace
crates are on edition 2024.
