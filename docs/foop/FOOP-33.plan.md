# FOOP-33 Plan — Creation Postulate → Booleans

> Execute **only** after reading `FOOP-33.md` (the specification). This plan assumes its
> context. Tasks run top to bottom; a parent is checked only after its children. Nothing here
> is started yet — this FOOP is in the **design/specification phase**. The FOOP files are
> authored on the `jia` branch; a worktree is created only when work `begun`.

## Worktree parameters (expanded per foop.md)

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/yolo/src
WORKTREE_BRANCH_NAME=foop-33-creation-postulate
WORKTREE_FULL_FS_PATH=/yolo/foolish_worktrees/foop-33-creation-postulate
```

Created (at `begun`) with:

```bash
git worktree add -b "foop-33-creation-postulate" \
  "/yolo/foolish_worktrees/foop-33-creation-postulate"
cd "/yolo/foolish_worktrees/foop-33-creation-postulate"
```

From that point, **all** work — including edits to `FOOP-33.md` and this plan — happens only
in the worktree until merge.

### Why this phase order (logical dependencies)

The phases are ordered so each builds only on settled predecessors:

1. **`Identifier` first** — the `Identifier`/`Characterizations` types and quote-bearing search
   stand alone (the parser already emits characterizations); nothing else depends on creation
   yet. Doing this first also isolates the `StatementFir` name→`Identifier` refactor before
   creation churn.
2. **Creation** — introduces the `CreationFir` value that (3) and (4) compare.
3. **`default_equal`** — needs a creation to exist (2) so the creation-identity rule is
   testable; it is the equality (4) reuses.
4. **Null-constant rule** — needs `default_equal` (3) to decide equal-vs-conflicting, and
   needs creation (2) as the canonical null-const value. It uses **any ancestor**, so its
   brane-step and concatenation unit tests are hand-built and do **not** require `system.foo`.
5. **`system.foo`** — the ancestor that makes `True`/`False` real; its *approval* tests
   (`'True=3`→NK end-to-end) sit here because they need the prelude installed, even though the
   *rule* they exercise was already unit-tested in (4).
6. **Comparison operators** — need `system.foo` (5) because they return `'True`/`'False` from
   the system root brane. They also need the integer infrastructure that already exists.

Gotcha: do not move the `system.foo`-dependent approval tests earlier, and do not make the
Phase-4 rule reach into `system.foo` specifically — it must work for any ancestral brane.

## Phase 0 — Start

- [x] Update Foolish documentation (AGENTS.md, `foop.md`, or a new `docs/howto/worktrees.md`)
      to establish the worktree path convention: worktrees are placed in a directory **relative
      to the project root** at `../foolish_worktrees/<branch-name>`. For this project (root at
      `/yolo/src`), that resolves to `/yolo/foolish_worktrees/<branch-name>`. Document this so
      all future FOOPs and agents use the convention consistently. Include the rationale: keeps
      worktrees close to the project, avoids polluting `~/tmp/`, and is path-independent of the
      user's home directory.
      (2026-07-30 12:00)
- [x] Confirm all tests green on `jia` before starting (no Phase-or-larger work on broken
      tests; `.snap.new.check` files with `@agent` comments are the only permitted exception).
      (2026-07-30 12:00)
- [x] Check the `begun` box in `FOOP-33.md` frontmatter, commit on `jia` ("work commenced on
      FOOP-33").
      (2026-07-30 12:00)
- [x] Create worktree `/yolo/foolish_worktrees/foop-33-creation-postulate` with
      branch `foop-33-creation-postulate` from `jia`.
      (2026-07-30 12:00)

## Phase 1 — The `Identifier` (LHS) becomes first-class (tests first)

- [x] Write unit tests for the new `Identifier` / `Characterizations` types (pure, no FVM):
      canonicalization (`a' b'c'd'e''x` → `name()` `"x"`, `characterized_name()`
      `"a'b'c'd'e''x"` with the space removed, characterization string `"a'b'c'd'e''"`); plain
      name → `characterized_name()==name()`; if the span representation is used, accessors return
      `&str` into the source (no fresh per-statement alloc);
      `is_nully_characterizing_coordinate_name()` **true** for `a'b'c''name` and bare `'name`,
      **false** for plain `name`, `a'b'c'name`, and interior-null `a''b'name` (proximity rule).
      (2026-07-30 12:15)
- [x] Add the `Identifier` struct — store **either** byte-range spans into the original source
      (preferred, when available) **or** three canonical strings (fully-characterized name,
      name, characterization string). Accessors `name()`, `characterized_name()`,
      `is_nully_characterizing_coordinate_name()`. Add `Characterizations` **minimal for this
      FOOP** — only `is_nully_characterizing_coordinate_name()`; per-`'` component extraction is
      deferred. Place both in the shared location.
      (2026-07-30 12:15)
- [ ] Migrate `BraneFir.characterizations` (`foolish-ubca/src/fir_kinds.rs:717`) and the
      core-fir `NormalBraneFir.characterizations` to `Characterizations`; keep the sequencer's
      trailing-`'` rendering (`foolish-core/src/sequencer.rs:514`).
- [x] Replace `StatementFir.name: String` with `identifier: Identifier`
      (`foolish-ubca/src/fir_kinds.rs:632`); `name()` delegates to `identifier.name()`; update
      constructor/`statement()` helper and all `StatementFir` construction sites. (Migration is
      free — the ubca FVM does not read characterizations today; refactor away.)
      (2026-07-30 12:15)
- [x] Build the `Identifier` in the compiler from `Astn::Assignment`'s name + characterizations
      (canonicalized) in `foolish-ubca/src/compiler.rs` (stop discarding characterizations);
      update the compiler test that currently asserts discard.
      (2026-07-30 12:15)
- [x] **Fold `'` back into the search pattern** (Gotcha #3): a `'`-bearing *reference* (e.g.
      `?'True`) currently compiles from `id` only (`compiler.rs:119`) and **loses** the `'`
      (parser keeps `characterizations` and `id` separate, `parser.rs:183`). Reconstruct the
      characterized pattern (characterizations + id) when the reference carries characterizations.
      Compiler test: `?'True` → pattern `'True`, not `True`.
      (2026-07-30 12:20)
- [x] Extend name-search matching so the **matcher chooses the projection**: a pattern
      containing `'` matches on the candidate's `Identifier::characterized_name()`; a pattern
      without `'` on `Identifier::name()` (`SearchFir::matches_pattern` / `SearchPredicate::Name`).
      (2026-07-30 12:20)
- [x] Unit test the quote-bearing search rule (`a'b'x` found by `?a'b'x`, missed by `?x`).
      (2026-07-30 12:20)
- [x] Introduce **NF (Not Foolish)** — a special sub-condition of NK for violations of Foolish's
      own rules (as opposed to "not knowable" in the search-miss sense). The first (and for this
      FOOP, only) case: **overwriting a null-characterized name constant**. When `'T=1` is
      followed by `'T=2` (non-equal redefinition of a nil-characterized name), the result is NF
      rather than a plain NK. NF is terminal and behaves identically to NK in all downstream
      machinery (step, search, concatenation) — it is a *semantic label*, not a new control flow.
      Unit test: `{a='T=1; 'T=2}` — second `'T` settles NF, verify the NK reason string
      distinguishes NF from generic NK (e.g. `"'T not-foolish"` vs `"'T redefined"`).
      (2026-07-30 12:20)

## Phase 2 — The creation dot `⬤` (and `{*}` alias)

- [ ] Lexer: emit **one** new token for `⬤` (U+2B24) only (`foolish-parser/src/lexer.rs`).
      Do **not** add `{*}` handling — `{`/`*`/`}` keep their `LBrace`/`Mul`/`RBrace` tokens.
- [ ] AST + parser: add `Astn::Creation`; parse it as a primary from *both* the `⬤` token and
      the `LBrace Mul RBrace` sequence (`foolish-parser/src/{ast.rs,parser.rs}`). Recognize
      `{*}` at brane-open by peeking `LBrace Mul RBrace`. This is collision-free: `*` is not a
      valid identifier/characterization name (`is_assignment_start` accepts only `Token::Ident`,
      `parser.rs:249`), so `{*}` can never be a real brane statement.
- [ ] **Parser unit test `parses_star_brane_as_creation`**: assert `{*}` and `⬤` both parse to
      `Astn::Creation`; assert the negatives keep their existing parse — `{ * }` (spaced),
      `{}` (empty brane), `{ *}` / `{* }`, and a brane that legitimately contains `*` in
      expression position (e.g. `{y = 2 * x}`) are **not** creations.
- [ ] FIR: add `CreationFir { core }` — **no id** — born `Independent`
      (`foolish-ubca/src/fir_kinds.rs`). **No counter, no registry.** Identity is the rust
      object (`Rc::ptr_eq`).
- [ ] Clone discipline: constanic clone of a `CreationFir` returns the **same `Rc`**
      (identity-preserving). NOTE: `ProtoBrane::constanic_clone_at` (`fir_kinds.rs:180-185`)
      **already** returns `Rc::clone(fir_ref)` for `Independent` non-brane FIRs, so a born-
      `Independent` `CreationFir` gets this for free — do **not** add a `FirKind::Creation` arm
      that constructs a new `CreationFir` (that would break identity). Also do not derive/
      implement a deep `Clone` on `CreationFir` reachable by any other path; audit
      detachment/recoordination.
- [ ] **Unit test `creation_constanic_clone_preserves_identity`**: construct a `CreationFir`,
      run it through `ProtoBrane::constanic_clone_at(&creation, &parent, 0, false)`, and assert
      `Rc::ptr_eq(&creation, &clone)`. This pins the `fir_kinds.rs:180` behavior that the whole
      equality story rests on — a regression here silently breaks `x=⬤; y=x` equality. Add a
      companion assertion that two independently-built `CreationFir`s are **not** `ptr_eq`.
- [ ] Compiler: build `CreationFir` from `Astn::Creation`.
- [ ] Core-fir representation + sequencer rendering for a creation
      (`foolish-core/src/{fir.rs,sequencer.rs}`); sequencer always outputs `⬤` (never `{*}`);
      decide the stable `hssnap` value form (resolves an Open Question).
- [ ] `creation_nyes_transitions` unit test (single-state `Independent` progression).

## Phase 3 — Default equality primitive (three-valued), used by search

- [ ] Unit tests for `default_equal -> Equality`: same integer ⇒ `Equal`; different ⇒
      `NotEqual`; same creation `Rc` ⇒ `Equal`; distinct creations ⇒ `NotEqual`; either NK
      (even same `Rc`) ⇒ `Unknowable`; creation-vs-integer ⇒ `Unknowable`; two branes ⇒
      `Unknowable`. And the matcher mapping: `Equal→Approve`, `NotEqual→Reject`,
      `Unknowable→NkStop`.
- [ ] Add `enum Equality { Equal, NotEqual, Unknowable }` and
      `default_equal(&FirRef, &FirRef) -> Equality`. Only two integers or two creations are
      comparable; **everything else is `Unknowable`** (not `NotEqual`).
- [ ] Refactor `SearchPredicate::Value` and `NameValue`
      (`foolish-ubca/src/fir_kinds.rs:1723`+) into a **greedy known-to-be-equal matcher**: call
      `default_equal` and map its three outcomes onto `MatchOutcome` (Approve/Reject/NkStop).
      Keep the "body must be constanic before comparison" contract (Gotcha #4).

## Phase 4 — Null-characterized name constants

- [ ] Unit tests: ancestral null-constant conflict — ancestor `'k=1`, descendant `'k=2` ⇒
      descendant `get_value()` returns `NF("'k not-foolish")` (NF, not plain NK — see Phase 1
      NF task); `Equal` redefinition (same creation) ⇒ permitted; **poison scope** — a sibling
      brane resolving `k` elsewhere (or not at all) is unaffected; descendant "is this a
      null-characterized coordinate name?" query.
- [ ] `BraneFir` step (PREMBRYONIC/EMBRYONIC): for each statement with
      `is_nully_characterizing_coordinate_name()`, walk the AB chain for a same-named ancestral
      null-const; on a **non-`Equal`** value (by `default_equal`) set the statement body to
      `NF("'<name> not-foolish")` **once** (terminal, no re-alarm); register ownership; answer
      descendant queries. No new FIR kind/NYES state — reuse `NkFir` with NF reason string.
- [ ] Concatenation collision handling: replace the blind clone loop in
      `ConcatenationFir` (`foolish-ubca/src/fir_kinds.rs:2162`) with a collision-aware merge
      applying the same rule (same `NF("'<name> not-foolish")`) against already-merged statements.
- [ ] Unit test the concatenation case `{A={'a=1}, B = A A A}` (later `'a`'s → `NF`, first
      intact) and `{A={'a=⬤}, B=A A}` (same creation ⇒ both permitted — value-sensitive).

## Phase 5 — `system.foo` ancestral prelude

**`OUT_DIR` mechanism (verified; no research needed — implement exactly this).** `OUT_DIR` is
the standard Cargo build-script variable (Cargo 1.93 in this repo), **not** `RESOURCE_PATH`.
Cargo sets `OUT_DIR` only while running `build.rs`. `env!("OUT_DIR")` and `include_str!` are
**compile-time** macros: they read the file and bake its **contents** into the binary during
compilation. At **runtime** `OUT_DIR` is not set and is not needed — the string is already
embedded. Do **not** call `std::env::var("OUT_DIR")` at runtime (it would return `Err`).
`foolish-ubca` is a **library** crate (`lib.rs`); `build.rs` sits at `foolish-ubca/build.rs`
(sibling of `Cargo.toml`) and the embed lives in the `evaluator` module (or a small `system`
module).

- [ ] Create the repo-root **`system/`** folder and `system/system.foo` defining `'True=⬤`,
      `'False=⬤`.
- [ ] Add `foolish-ubca/build.rs` with exactly this behavior (copy the root file into
      `OUT_DIR`, and re-run if it changes):

      ```rust
      // foolish-ubca/build.rs
      use std::{env, fs, path::Path};

      fn main() {
          // Repo root is two levels up from this crate dir (workspace member).
          let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
          let src = Path::new(&manifest).join("../system/system.foo");
          let out = Path::new(&env::var("OUT_DIR").unwrap()).join("system.foo");
          fs::copy(&src, &out).expect("copy system/system.foo into OUT_DIR");
          // Rebuild when the source prelude changes.
          println!("cargo:rerun-if-changed=../system/system.foo");
      }
      ```

- [ ] In the evaluator, embed at compile time and keep the source string:

      ```rust
      const SYSTEM_FOO_SRC: &str = include_str!(concat!(env!("OUT_DIR"), "/system.foo"));
      ```

- [ ] **Implicitly** compile `SYSTEM_FOO_SRC` once and make it **THE root brane** (its own
      parent, self-rooting via `new_cyclic` — the same pattern used one level down today), with
      the user program as its child, before `step_to_settled`, on every entry path
      (`foolish-ubca/src/evaluator.rs::evaluate`, REPL, CLI `run`/`step`). `_ab_search`
      terminates at system.foo (parent == self). Not opt-in.
- [ ] **Preserve the user program's line numbers**: making system.foo the ancestor must not
      shift program line numbering (system.foo is a distinct brane above, with its own lines).
      Unit test via `as_stmt_line_number` / `step_until_line_number` on a one-line program.
- [ ] Approval `.foo` tests: creation + identity; quote-bearing search; `True`/`False`
      resolve ancestrally; `'True=3` ⇒ `NK("'True redefined")` while `'True='True` permitted.
      Generate `.snap.new`; **present to human** — never auto-accept.

## Phase 6 — Comparison operators (`<`, `>`, `<=`, `>=`)

**Read §5 of FOOP-33.md before implementing.** Comparison operators produce `'True`/`'False`
from `system.foo` (Phase 5 must be complete). They extend the existing binary-operator
infrastructure.

- [ ] Lexer: add four new tokens `Lt` (`<`), `Gt` (`>`), `Le` (`<=`), `Ge` (`>=`) to
      `foolish-parser/src/token.rs`. Two-character tokens (`<=`, `>=`) must be recognized
      before single-character fallbacks in the lexer dispatch.
- [ ] Parser: add `<`, `>`, `<=`, `>=` as infix operators at the same precedence level as
      `+`/`-` (additive) in `foolish-parser/src/parser.rs`. Left-associative.
- [ ] Parser unit tests: `a < b` parses to a binary operation; `a + b < c` parses as
      `(a + b) < c` (precedence); `a < b < c` parses as `(a < b) < c` (left-associativity).
- [ ] Evaluator: add four new arms to the binary-operator dispatch in
      `foolish-ubca/src/evaluator.rs` (or equivalent). Each arm:
      1. Checks both operands are integers (else NK with reason
         `"comparison: non-integer operand"`).
      2. Performs the comparison.
      3. Resolves `'True` or `'False` from the system root brane (the same ancestral lookup
         any program uses — `_ib_search` or a cached reference to the system root).
      No new FIR kind — returns `CreationFir` or `NkFir`.
- [ ] Unit tests: `1 < 2` ⇒ `'True`; `2 < 1` ⇒ `'False`; `1 <= 1` ⇒ `'True`; `3 > 5` ⇒
      `'False`; `5 >= 5` ⇒ `'True`. Non-integer operand (`⬤ < 3`) ⇒ NK. Verify the returned
      `'True` is `Rc::ptr_eq` with the `'True` in `system.foo` (referential identity).
- [ ] Approval tests: `comparison_basic.foo`, `comparison_equal.foo`, `comparison_nk.foo`,
      `comparison_if_then.foo` (the motivating example — comparison feeding into value search).
      Generate `.snap.new`; present to human.

## Phase 7 — Documentation

- [ ] Document the null-characterized name-constant rule and universal characterizations
      (update `docs/vintage_legacy/CREATION.md` cross-refs and add engineering notes under
      `docs/ubc1/how`); update AGENTS.md §Foolish Terminology / §Searches as needed
      (with the "## Last Updated" protocol).

## Phase 8 — Merge

- [ ] Merge `foop-33-creation-postulate` to `jia`
  - [ ] Write and verify `foop_33_comprehensive.foo` (reserved name): creation, characterized
        names, quote-bearing search, referential equality, `system.foo` parent brane,
        null-constant refusal (incl. `A A A` concatenation), comparison operators (`<`, `>`,
        `<=`, `>=` producing `'True`/`'False`/NK), interacting with prior features (nested
        branes, contexted `&` searches). Generate + verify `.snap.new`; final approval is
        human-signed.
  - [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace` all green.
  - [ ] Verify all work complete in the worktree and committed to
        `foop-33-creation-postulate`.
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. Do NOT continue
        past this point automatically.
    - [ ] Present the human with `cd /yolo/foolish_worktrees/foop-33-creation-postulate`
          and ask them to review snapshots BEFORE checking the parent box.
  - [ ] Merge to `jia`; repair any merge-conflict fallout and re-green all tests.
  - [ ] Cleanup
    - [ ] Confirm every box but Cleanup is checked.
    - [ ] Remove `/yolo/foolish_worktrees/foop-33-creation-postulate`.
    - [ ] Last box checked in this block.

## Last Updated

**Date**: 2026-07-30
**Updated By**: Sisyphus / xiaomi/mimo-v2.5-pro
**Changes (round 8, per Atlas)**: (1) Added Phase 6 — comparison operators (`<`, `>`, `<=`, `>=`):
lexer tokens, parser precedence (additive level), evaluator arms resolving `'True`/`'False`
from system root brane, unit tests, approval tests. (2) Renumbered Phase 6→7 (docs), Phase
7→8 (merge). (3) Updated worktree path convention to `../foolish_worktrees/` relative to
project root (`/yolo/foolish_worktrees/foop-33-creation-postulate`). (4) Added Phase 0 task to
document the new worktree path convention in Foolish docs. (5) Updated merge/cleanup paths and
comprehensive test description to include comparison operators.
**Date**: 2026-07-08
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (round 7)**: Phase 1 — `Identifier` stores spans-into-source or three canonical
strings; added explicit fold-`'`-into-pattern compiler task+test (Gotcha #3). Phase 3 — equality
is three-valued `Equality`; matcher is greedy known-to-be-equal (Equal/NotEqual/Unknowable →
Approve/Reject/NkStop). Phase 4 — null-const refusal is `get_value()→NK("'<name> redefined")`
(reuse `NkFir`, set once, terminal); poison-scope test (siblings unaffected). Phase 5 —
system.foo is implicit/built-in and IS the root (its own parent, program is child); added
line-number-preservation task+test.
**Changes (round 6)**: Phase 2 — `{*}` now recognized at the parser (emit `Astn::Creation` from
`LBrace Mul RBrace`); lexer adds only the `⬤` token. Collision-free (`*` isn't a valid name at
brane-statement position). Added explicit parser test `parses_star_brane_as_creation` with the
negative set (`{ * }`, `{}`, `{y = 2 * x}` keep existing parse).
**Changes (round 5)**: Phase 1 retitled "The `Identifier` (LHS) becomes first-class"; tasks
now build an `Identifier` (owns whitespace-stripped LHS `text` + `name` span + a minimal
`Characterizations` reporting only `is_nully_characterizing_coordinate_name`), replace
`StatementFir.name` with `identifier`, and make the matcher pick `characterized_name()` vs
`name()`. Dependency note updated to "Identifier first".
**Changes (round 4)**: Added a "Why this phase order (logical dependencies)" note (each phase
builds on settled predecessors; Phase-4 rule uses any ancestor and does NOT require
`system.foo`; system-dependent approval tests stay in Phase 5). Added explicit Phase-2 unit
test `creation_constanic_clone_preserves_identity` pinning the `fir_kinds.rs:180` same-`Rc`
behavior, and hardened the clone-discipline task (do not add a `FirKind::Creation` clone arm).
**Changes (round 3)**: Phase 2 — `CreationFir { core }` with **no id** (identity = `Rc::ptr_eq`,
no counter/registry) + explicit clone-discipline task (constanic clone returns same `Rc`; any
other clone forbidden). Phase 5 — rewritten with the **verified** repo-root `system/` +
`build.rs`→`OUT_DIR`→compile-time `include_str!` mechanism, including copy-paste `build.rs` and
embed code and a note that `OUT_DIR` is Cargo-standard and compile-time only (no runtime
access, not `RESOURCE_PATH`); no research needed at implementation time.
**Round 2**: Phase 1 `Characterizations` = one owned string + subspans +
`is_nully_characterizing_coordinate_name` (name-adjacent only); Phase 2 `{*}` alias; Phase 3
"default equality primitive (`default_equal`), used by search" (refactor, not add); Phase 4
uses `default_equal` + renamed null method.
**Initial plan**: ordered phases for characterizations, ⬤ creation, equality via search,
null-characterized name constants (brane + concatenation), `system.foo` ancestral prelude,
docs, worktree/merge lifecycle. Design phase; nothing begun.

---

## STASHING NOTES (temporary — remove once read and resumed)

**Date**: 2026-07-30
**Stashed by**: Sisyphus / xiaomi/mimo-v2.5-pro
**Reason**: Server reboot. Work in progress on Phase 2.

### Where we are

**Worktree**: `/yolo/foolish_worktrees/foop-33-creation-postulate`
**Branch**: `foop-33-creation-postulate`
**Last commit**: `7ac9638d` — Phase 2 WIP (lexer/parser/AST for creation dot)

### Phase 1 — COMPLETE ✅
All tasks done:
- `Identifier` / `Characterizations` types in `foolish-ubca/src/identifier.rs` (10 unit tests)
- `StatementFir.name` → `identifier: Identifier` migration
- Compiler folds `'` back into search pattern (Gotcha #3)
- `_search_brane` chooses projection: pattern with `'` → `characterized_name()`, else `name()`
- NF (Not Foolish) constant `NF_PREFIX` and `is_nf_reason()` in `fir_kinds.rs`
- All 241 foolish-ubca tests pass, all 68 foolish-core tests pass

### Phase 2 — IN PROGRESS (lexer/parser/AST done, FIR/compiler not yet)

**Done:**
- `Token::Creation` added to `foolish-parser/src/token.rs`
- Lexer: `⬤` (U+2B24) token + `{*}` detection at character level (no interior whitespace)
- `Astn::Creation` added to `foolish-parser/src/ast.rs` with Display (renders as `⬤`)
- Parser: `Token::Creation` handled in `parse_primary`
- Compiler: `validate_astn` accepts `Astn::Creation`

**NOT YET DONE (resume here):**
1. `CreationFir { core }` — new FIR kind in `foolish-ubca/src/fir_kinds.rs`, born `Independent`
2. `build_fir` arm for `Astn::Creation` in compiler
3. Core-fir representation + sequencer rendering (always renders `⬤`, never `{*}`)
4. `creation_nyes_transitions` unit test (single-state `Independent` progression)
5. `creation_constanic_clone_preserves_identity` unit test
6. Parser unit test `parses_star_brane_as_creation` with negative set
7. Mark Phase 2 tasks in plan

### Key design reminders
- `CreationFir` has NO id field. Identity = `Rc::ptr_eq`. No counter, no registry.
- Constanic clone of `CreationFir` returns SAME `Rc` (already works via `fir_kinds.rs:180` branch for `Independent` non-branes). Do NOT add a `FirKind::Creation` clone arm.
- Do NOT derive/implement deep `Clone` on `CreationFir`.
- `{*}` is lexer-level (character stream), `⬤` is token-level. Both become `Token::Creation`.
- `{ * }` (with spaces) keeps existing parse (brane containing `*` expression).
- Sequencer always renders `⬤`, never `{*}`.

### Files changed so far
- `foolish-parser/src/token.rs` — `Token::Creation`
- `foolish-parser/src/lexer.rs` — `⬤` + `{*}` handling
- `foolish-parser/src/ast.rs` — `Astn::Creation` + Display
- `foolish-parser/src/parser.rs` — `parse_primary` case
- `foolish-ubca/src/compiler.rs` — `validate_astn` accepts Creation
- `foolish-ubca/src/identifier.rs` — NEW: Identifier/Characterizations
- `foolish-ubca/src/lib.rs` — `pub(crate) mod identifier`
- `foolish-ubca/src/fir_kinds.rs` — StatementFir.identifier, NF_PREFIX, _search_brane projection
- `foolish-ubca/src/fir_trait.rs` — `as_stmt_identifier()` on Fir trait
