# FOOP-33 Plan — Creation Postulate → Booleans

> Execute **only** after reading `FOOP-33.md` (the specification). This plan assumes its
> context. Tasks run top to bottom; a parent is checked only after its children. Nothing here
> is started yet — this FOOP is in the **design/specification phase**. The FOOP files are
> authored on the `jia` branch; a worktree is created only when work `begun`.

## Worktree parameters (expanded per foop.md)

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/home/hcbusy/foolish-rust
WORKTREE_BRANCH_NAME=foop-33-creation-postulate
WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-33-creation-postulate
```

Created (at `begun`) with:

```bash
git worktree add -b "foop-33-creation-postulate" \
  "/home/hcbusy/tmp/foolish-worktrees/foop-33-creation-postulate"
cd "/home/hcbusy/tmp/foolish-worktrees/foop-33-creation-postulate"
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

Gotcha: do not move the `system.foo`-dependent approval tests earlier, and do not make the
Phase-4 rule reach into `system.foo` specifically — it must work for any ancestral brane.

## Phase 0 — Start

- [ ] Confirm all tests green on `jia` before starting (no Phase-or-larger work on broken
      tests; `.snap.new.check` files with `@agent` comments are the only permitted exception).
- [ ] Check the `begun` box in `FOOP-33.md` frontmatter, commit on `jia` ("work commenced on
      FOOP-33").
- [ ] Create worktree `/home/hcbusy/tmp/foolish-worktrees/foop-33-creation-postulate` with
      branch `foop-33-creation-postulate` from `jia`.

## Phase 1 — The `Identifier` (LHS) becomes first-class (tests first)

- [ ] Write unit tests for the new `Identifier` / `Characterizations` types (pure, no FVM):
      whitespace stripping (`a' b'c'd'e''x` → `text` `"a'b'c'd'e''x"`, `name()` `"x"`,
      `characterized_name()` `"a'b'c'd'e''x"`); `name()`/`characterized_name()` are subspans of
      the one owned `text` (no fresh alloc; for a plain name `characterized_name()==name()`);
      `is_nully_characterizing_coordinate_name()` **true** for `a'b'c''name` and bare `'name`,
      **false** for plain `name`, `a'b'c'name`, and interior-null `a''b'name` (proximity rule).
- [ ] Add the `Identifier` struct (owns `text: String`; `name: Range<usize>`; contains a
      `Characterizations`) with `name()`, `characterized_name()`,
      `is_nully_characterizing_coordinate_name()`. Add `Characterizations` **minimal for this
      FOOP** — only `is_nully_characterizing_coordinate_name()`; per-`'` component extraction is
      deferred. Place both in the shared location.
- [ ] Migrate `BraneFir.characterizations` (`foolish-ubca/src/fir_kinds.rs:717`) and the
      core-fir `NormalBraneFir.characterizations` to `Characterizations`; keep the sequencer's
      trailing-`'` rendering (`foolish-core/src/sequencer.rs:514`).
- [ ] Replace `StatementFir.name: String` with `identifier: Identifier`
      (`foolish-ubca/src/fir_kinds.rs:632`); `name()` delegates to `identifier.name()`; update
      constructor/`statement()` helper and all `StatementFir` construction sites.
- [ ] Build the `Identifier` in the compiler from `Astn::Assignment`'s name +
      characterizations (whitespace-stripped) in `foolish-ubca/src/compiler.rs` (stop
      discarding characterizations); update the compiler test that currently asserts discard.
- [ ] Extend name-search matching so the **matcher chooses the projection**: a pattern
      containing `'` matches on the candidate's `Identifier::characterized_name()`; a pattern
      without `'` on `Identifier::name()` (`SearchFir::matches_pattern` / `SearchPredicate::Name`).
- [ ] Unit test the quote-bearing search rule (`a'b'x` found by `?a'b'x`, missed by `?x`).

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

## Phase 3 — Default equality primitive, used by search

- [ ] Unit tests for `default_equal`: same integer ⇒ true; same creation ⇒ true; distinct
      creations ⇒ false; NK vs NK ⇒ false; distinct branes ⇒ false. And the matcher delegates
      (same creation ⇒ Approve; distinct ⇒ Reject; integer paths unchanged).
- [ ] Add `default_equal(&FirRef, &FirRef) -> bool` (NK guard, integer, creation-identity).
- [ ] Refactor `SearchPredicate::Value` and `NameValue`
      (`foolish-ubca/src/fir_kinds.rs:1723`+) to *call* `default_equal` instead of comparing
      `as_i64()` inline.

## Phase 4 — Null-characterized name constants

- [ ] Unit tests: ancestral null-constant refusal (same name, unequal value ⇒ statement NK
      via `default_equal`; equal value ⇒ permitted); descendant "is this a null-characterized
      coordinate name?" query.
- [ ] `BraneFir` step (PREMBRYONIC/EMBRYONIC): for each statement with
      `is_nully_characterizing_coordinate_name()`, ancestral check via the AB chain; refuse→NK on
      unequal redefinition (by `default_equal`); ownership registration; answer descendant
      queries.
- [ ] Concatenation collision handling: replace the blind clone loop in
      `ConcatenationFir` (`foolish-ubca/src/fir_kinds.rs:2162`) with a collision-aware merge
      applying the null-constant rule against already-merged statements.
- [ ] Unit test the concatenation case `{A={'a=1}, B = A A A}` (later `'a`'s NK, first
      intact; equal-value duplicates permitted).

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

- [ ] Compile `SYSTEM_FOO_SRC` once to a brane and install it as the AB parent of each program
      root brane in `foolish-ubca/src/evaluator.rs::evaluate` (and REPL/CLI `run`/`step`
      paths). Note: today each root brane self-roots via `new_cyclic` — this changes the parent
      wiring so `_ab_search` falls through to the system brane.
- [ ] Approval `.foo` tests: creation + identity; quote-bearing search; `True`/`False`
      resolve ancestrally; `'True=3` ⇒ NK while `'True='True` permitted. Generate `.snap.new`;
      **present to human** — never auto-accept.

## Phase 6 — Documentation

- [ ] Document the null-characterized name-constant rule and universal characterizations
      (update `docs/vintage_legacy/CREATION.md` cross-refs and add engineering notes under
      `docs/ubc1/how`); update AGENTS.md §Foolish Terminology / §Searches as needed
      (with the "## Last Updated" protocol).

## Phase 7 — Merge

- [ ] Merge `foop-33-creation-postulate` to `jia`
  - [ ] Write and verify `foop_33_comprehensive.foo` (reserved name): creation, characterized
        names, quote-bearing search, referential equality, `system.foo` parent brane,
        null-constant refusal (incl. `A A A` concatenation), interacting with prior features
        (nested branes, contexted `&` searches). Generate + verify `.snap.new`; final
        approval is human-signed.
  - [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace` all green.
  - [ ] Verify all work complete in the worktree and committed to
        `foop-33-creation-postulate`.
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. Do NOT continue
        past this point automatically.
    - [ ] Present the human with `cd /home/hcbusy/tmp/foolish-worktrees/foop-33-creation-postulate`
          and ask them to review snapshots BEFORE checking the parent box.
  - [ ] Merge to `jia`; repair any merge-conflict fallout and re-green all tests.
  - [ ] Cleanup
    - [ ] Confirm every box but Cleanup is checked.
    - [ ] Remove `/home/hcbusy/tmp/foolish-worktrees/foop-33-creation-postulate`.
    - [ ] Last box checked in this block.

## Last Updated

**Date**: 2026-07-07
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
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
