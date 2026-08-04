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

### ⛔ STATUS SUMMARY (updated 2026-08-03 — read this before picking up the next phase)

Checkboxes had drifted badly out of sync with actual code state; each phase below was
individually re-verified against the codebase this session (see the per-phase notes for
exact evidence). Current real status, top to bottom:

| Phase | Status | Notes |
|---|---|---|
| 0 — Start | ✅ done | |
| 1 — `Identifier` | ⚠️ **one item open**: `BraneFir.characterizations`/`NormalBraneFir.characterizations` migration to `Characterizations` — genuinely not done, correctly unchecked |
| 2 — Creation `⬤`/`{*}` | ✅ **fully done** — was showing all-unchecked, now corrected; every item verified present in code |
| 3 — `default_equal` | ⚠️ **one open item**: no dedicated truth-table unit tests exist, only indirect coverage — write these before considering Phase 3 done. (The creation-vs-integer question is **resolved**: `NotEqual` is correct per human ruling 2026-08-03 — every integer is itself a creation, so a new creation can never equal one, decidably.) |
| 4 — Null-characterized constants | ❌ **verified not started** — no NF/ancestral-conflict/poison-scope/concatenation-collision code exists anywhere |
| 5 — `system.foo` composition | ❌ not started. Design **superseded** from the original "ancestral prelude" (system.foo as parent brane) to a **composition** model per 2026-08-03 human direction: `system.foo` is the root brane, the user's program is a **member** named `program`, the FVM returns it via `stmt_at(idx)` in Rust — not a Foolish search. See FOOP-33.md §4 for the corrected design; this phase's task list below still describes the superseded ancestral-prelude approach and needs rewriting to match before implementation starts. |
| 5.5 — Sequencer renders named creations | ❌ not started, depends on 5 |
| 6 — Comparison operators | ⛔ **BLOCKED, design settled tonight, no code yet** — reverted (was returning placeholder `1`/`0`); design converged through several exchanges to: postfix `<<#-2>> < <<#-1>>` in an ordinary brane literal (`{1, 2, 'lt}`, **no concatenation**), extracted via `$` (`comparison_result =$ {1, 2}'lt` or `{1, 2, 'lt}$` — the brane literal alone is not the full expression, it's the vessel the tail-read pulls the boolean out of). `'lt` resolved via ordinary ancestral search into `system.foo` (same as `'True`), then the existing **detachment/recoordination** mechanism (`AGENTS.md` "Detachment and Coordination") lets its previously-unresolved `#-2`/`#-1` lookups find real neighbors once recoordinated into the user's brane. A new `system_foo.rs` Rust module holds shared FIR logic across `'lt`/`'gt`/`'le`/`'ge`/`'eq`, differing only in the op step. See FOOP-33.md §5.0's evening revision banner for the full transcript. Not yet re-verified against a live trace — needs Phase 5 (`system.foo` composition) implemented first to test against. See the STOP gate at the head of Phase 6. |
| 7 — Docs/Tests | 🟡 partial, done piecemeal, not formally tracked |
| 7R — Phase 3 value-search regression repair | ✅ done (earlier session) |
| 8 — Merge | ❌ not started |

**Also fixed this session, not originally a plan item**: a bare unanchored `?pattern`/`?=pattern`
search (nothing preceding the `?`) was broken two ways — (1) the parser hardcoded its anchor to
an empty brane literal instead of "no anchor" (fixed: `Astn::RegexpSearch.anchor` is now
`Option<Box<Astn>>`); (2) a value search whose *pattern* references a creation (`?=a` where `a`
is `⬤`) was rejected outright by `check_value_pattern_ready`, and even when let through,
`default_equal` compared raw unresolved search nodes instead of their resolved values. Both
fixed; see `foolish-ubca/src/fir_kinds.rs` `check_value_pattern_ready`/`default_equal` and the
new regression tests `value_search_pattern_referencing_a_creation_*`. A **new open question**
about anchored value-search-miss semantics (NK vs ECONSTANIC) surfaced from this work and is
recorded in FOOP-33.md's Open Questions — it blocks `foop/33/creation/referential_equality.foo`
and needs a human decision before that baseline can be promoted.

**Recommended next step**: Phase 5 (`system.foo` composition) is the next real, unstarted,
unblocked work — but its task list (below) needs to be rewritten to match the composition
design in FOOP-33.md §4 before starting, since it still describes the superseded ancestral-
prelude approach.

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

## Phase 2 — The creation dot `⬤` (and `{*}` alias) — VERIFIED COMPLETE (2026-08-03)

Checkboxes below were left unchecked despite being implemented; verified against the code
directly (2026-08-03) rather than assumed. See the reconciliation note at the end of this file.

- [x] Lexer: emit **one** new token for `⬤` (U+2B24) only (`foolish-parser/src/lexer.rs`).
      Do **not** add `{*}` handling — `{`/`*`/`}` keep their `LBrace`/`Mul`/`RBrace` tokens.
      (verified 2026-08-03: `lexer.rs:278`, "Creation dot ⬤ (U+2B24)")
- [x] AST + parser: add `Astn::Creation`; parse it as a primary from *both* the `⬤` token and
      the `LBrace Mul RBrace` sequence (`foolish-parser/src/{ast.rs,parser.rs}`). Recognize
      `{*}` at brane-open by peeking `LBrace Mul RBrace`. This is collision-free: `*` is not a
      valid identifier/characterization name (`is_assignment_start` accepts only `Token::Ident`,
      `parser.rs:249`), so `{*}` can never be a real brane statement.
      (verified 2026-08-03: `ast.rs:150`, `parser.rs:982`)
- [x] **Parser unit test `parses_star_brane_as_creation`**: assert `{*}` and `⬤` both parse to
      `Astn::Creation`; assert the negatives keep their existing parse — `{ * }` (spaced),
      `{}` (empty brane), `{ *}` / `{* }`, and a brane that legitimately contains `*` in
      expression position (e.g. `{y = 2 * x}`) are **not** creations.
      (verified 2026-08-03: `parser.rs:1445`; did not re-verify every negative case listed)
- [x] FIR: add `CreationFir { core }` — **no id** — born `Independent`
      (`foolish-ubca/src/fir_kinds.rs`). **No counter, no registry.** Identity is the rust
      object (`Rc::ptr_eq`).
      (verified 2026-08-03: `fir_kinds.rs:2696`)
- [x] Clone discipline: constanic clone of a `CreationFir` returns the **same `Rc`**
      (identity-preserving). NOTE: `ProtoBrane::constanic_clone_at` (`fir_kinds.rs:180-185`)
      **already** returns `Rc::clone(fir_ref)` for `Independent` non-brane FIRs, so a born-
      `Independent` `CreationFir` gets this for free — do **not** add a `FirKind::Creation` arm
      that constructs a new `CreationFir` (that would break identity). Also do not derive/
      implement a deep `Clone` on `CreationFir` reachable by any other path; audit
      detachment/recoordination.
      (verified 2026-08-03: no `FirKind::Creation` arm exists in `constanic_clone_at`; not
      independently re-audited for every detachment/recoordination path)
- [x] **Unit test `creation_constanic_clone_preserves_identity`**: construct a `CreationFir`,
      run it through `ProtoBrane::constanic_clone_at(&creation, &parent, 0, false)`, and assert
      `Rc::ptr_eq(&creation, &clone)`. This pins the `fir_kinds.rs:180` behavior that the whole
      equality story rests on — a regression here silently breaks `x=⬤; y=x` equality. Add a
      companion assertion that two independently-built `CreationFir`s are **not** `ptr_eq`.
      (verified 2026-08-03: `fir_kinds.rs:7008`, both assertions present)
- [x] Compiler: build `CreationFir` from `Astn::Creation`.
      (verified 2026-08-03: `compiler.rs:226`)
- [x] Core-fir representation + sequencer rendering for a creation
      (`foolish-core/src/{fir.rs,sequencer.rs}`); sequencer always outputs `⬤` (never `{*}`);
      decide the stable `hssnap` value form (resolves an Open Question).
      (verified 2026-08-03: `Fir::Creation` variant present throughout `foolish-core/src/fir.rs`)
- [x] `creation_nyes_transitions` unit test (single-state `Independent` progression).
      (verified 2026-08-03: `fir_kinds.rs:6999`)

## Phase 3 — Default equality primitive (three-valued), used by search

- [ ] **GAP (verified 2026-08-03) — no dedicated `default_equal` truth-table unit tests
      exist.** `default_equal` (`fir_kinds.rs:445`) is only exercised indirectly through
      `matcher_value_reject_non_integer_candidate` and the two `value_search_pattern_
      referencing_a_creation_*` tests added 2026-08-03 (creation-vs-creation only). No test
      directly calls `default_equal` with an integer/integer pair, an NK operand, or a
      brane/brane pair. Still open — write these before considering Phase 3 done.
- [x] **Creation-vs-integer is `NotEqual` — RESOLVED 2026-08-03, plan text was wrong, code is
      correct.** This checkbox's original text said "creation-vs-integer ⇒ `Unknowable`" and
      "everything else is `Unknowable` (not `NotEqual`)", contradicting the shipped
      `default_equal` (`fir_kinds.rs:445-477`), which returns `NotEqual` for creation-vs-integer,
      brane-vs-integer, etc. Human's ruling: **every integer is itself a creation** — so a
      *new*, distinct creation can never equal any integer, by the same uniqueness rule that
      makes two distinct `⬤`s unequal. This is decidably `NotEqual`, not `Unknowable` — there is
      nothing unknown about it. Only brane-vs-brane stays `Unknowable` (brane-vs-brane
      equivalence is genuinely unspecified per FOOP-23). No code change; this checkbox's
      original wording is superseded by this note. `enum Equality { Equal, NotEqual, Unknowable
      }` and `default_equal(&FirRef, &FirRef) -> Equality` are implemented as described.
- [x] Refactor `SearchPredicate::Value` and `NameValue`
      (`foolish-ubca/src/fir_kinds.rs:1723`+) into a **greedy known-to-be-equal matcher**: call
      `default_equal` and map its three outcomes onto `MatchOutcome` (Approve/Reject/NkStop).
      Keep the "body must be constanic before comparison" contract (Gotcha #4).

## Phase 4 — Null-characterized name constants — VERIFIED NOT STARTED (2026-08-03)

Searched for `not-foolish`, `NF(`, an AB-chain-walking `BraneFir` step, and collision-aware
concatenation merge logic in `foolish-ubca/src/*.rs`: none found. Only the Phase-1
`is_nully_characterizing_coordinate_name()` accessor exists (well-tested on `Identifier`
itself) — the Phase-4-specific enforcement (ancestral conflict detection, NF, poison scope,
concatenation collision handling) has not been implemented. All four checkboxes below
correctly remain unchecked; no action needed here beyond confirming genuinely not done.

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

## Phase 5 — `system.foo` composition

> ## ⛔ REWRITTEN 2026-08-03 — supersedes the "ancestral prelude" design below the line
>
> The task list below this banner describes the **superseded** design: `system.foo` as a
> parent brane with the user program wrapped in a `'program` statement, name-resolution
> reaching `True`/`False` via `_ab_search` walking up a parent chain. Per human direction
> earlier this session, the design is now **composition**, not ancestry:
>
> ```
> system.foo = { 'True = ⬤, 'False = ⬤, [comparison members per §5.0], program = PROGRAM_BRANE }
> ```
>
> `system.foo` is the root brane. The user's program is an ordinary **member** of it, bound to
> the plain name `program` (not null-characterized — an ordinary statement). The FVM steps the
> whole composite brane to settlement, then **extracts the `program` member in Rust** via the
> `stmt_at(idx)` capability accessor (FOOP-13 A2) — **not** by evaluating a Foolish `#-1`/`$`
> search. The return path must not depend on the search engine this FOOP modifies. `program` is
> retrieved **positionally**, as the last statement of `system.foo` (`stmt_at(stmt_count() -
> 1)`); switching to name-based lookup is a documented, non-blocking suggestion for if/when
> `system.foo` grows complex enough that "last statement" becomes fragile (see FOOP-33.md §4).
>
> **The only real behavioral delta from the ancestral design**: the user's root brane is no
> longer its own parent — its parent is `system.foo` (`program`'s home brane), which is its own
> parent (self-rooting, terminating the walk). This was reviewed and judged not a significant
> problem: the two self-parent checks in the codebase (`fir_kinds.rs`, `_ab_search`-family
> loop-termination guards) are loop guards, not semantic assertions — they terminate wherever
> the fixed point actually is. `is_root()` (`proto_brane.rs`) will start answering `false` for
> the (former) user root; find its callers before implementing.
>
> **Line numbers need no preservation task.** Statement indices are 0-based and assigned
> per-brane via `.enumerate()` at compile time (`compiler.rs`). Sibling statements in
> `system.foo` cannot renumber statements *inside* `program`'s own brane — they belong to a
> different brane's numbering. The "preserve the user program's line numbers" checkbox below is
> **moot** under this design and should be dropped, not carried forward as a task.
>
> **Tasks below still need updating to match** — the `OUT_DIR`/`build.rs`/`include_str!`
> mechanism is unaffected and correct as written (verified: `build.rs` already exists and
> performs the copy; only the compile-time `include_str!` consumption side is missing). What
> needs rewriting: the "Construction" and "`'program` statement" bullets (composition, not a
> wrapper statement + parent-chain reach), and the line-number-preservation task (drop it).
> Comparison-related `system.foo` members (`'lt` etc.) are Phase 6's concern, gated separately —
> do not add them here.

**`OUT_DIR` mechanism (verified; no research needed — implement exactly this).** `OUT_DIR` is
the standard Cargo build-script variable (Cargo 1.93 in this repo), **not** `RESOURCE_PATH`.
Cargo sets `OUT_DIR` only while running `build.rs`. `env!("OUT_DIR")` and `include_str!` are
**compile-time** macros: they read the file and bake its **contents** into the binary during
compilation. At **runtime** `OUT_DIR` is not set and is not needed — the string is already
embedded. Do **not** call `std::env::var("OUT_DIR")` at runtime (it would return `Err`).
`foolish-ubca` is a **library** crate (`lib.rs`); `build.rs` sits at `foolish-ubca/build.rs`
(sibling of `Cargo.toml`) and the embed lives in the `evaluator` module (or a small `system`
module).

- [x] Create the repo-root **`system/`** folder and `system/system.foo` defining `'True=⬤`,
      `'False=⬤`. (verified 2026-08-03: `system/system.foo` exists with exactly this content)
- [x] Add `foolish-ubca/build.rs` with exactly this behavior (copy the root file into
      `OUT_DIR`, and re-run if it changes). (verified 2026-08-03: `foolish-ubca/build.rs`
      exists and performs exactly this copy — but nothing currently reads `OUT_DIR`'s copy;
      the `include_str!` consumption side below is the missing half)

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

- [ ] **NEEDS REWRITE (composition, not ancestry).** Implicitly compose `SYSTEM_FOO_SRC` with
      the user program as a member named `program` (not wrapped in a `'program`
      null-characterized statement — see the banner above), before `step_to_settled`, on every
      entry path (`foolish-ubca/src/evaluator.rs::evaluate`, REPL, CLI `run`/`step`). Not
      opt-in. `system.foo`'s own AST gains one more statement, `program = {user source}`,
      appended last; compile the combined AST as one unit → one self-rooting BraneFir. The
      evaluator extracts and returns the `program` member's result via `stmt_at(stmt_count() -
      1)` in Rust, not the system brane's own result and not via a Foolish search.
- [ ] ~~Preserve the user program's line numbers~~ — **DROP, moot under composition** (see
      banner above: statement indices are per-brane, 0-based, unaffected by siblings in a
      different brane).
- [ ] Approval `.foo` tests: creation + identity; quote-bearing search; `True`/`False`
      resolve as ordinary sibling lookups within `system.foo` (not `_ab_search` ancestry);
      `'True=3` ⇒ `NK("'True redefined")` while `'True='True` permitted.
      Generate `.snap.new`; **present to human** — never auto-accept.

---

*(Below this line: the original "ancestral prelude" task-list prose, retained as historical
context for the rewrite above — do NOT implement from it directly. Superseded 2026-08-03.)*

- [ ] **Implicitly** compile `SYSTEM_FOO_SRC` once and make it **THE root brane** (its own
      parent, self-rooting via `new_cyclic` — the same pattern used one level down today), with
      the user program as its child, before `step_to_settled`, on every entry path
      (`foolish-ubca/src/evaluator.rs::evaluate`, REPL, CLI `run`/`step`). `_ab_search`
      terminates at system.foo (parent == self). Not opt-in.

      **Construction**: parse system.foo to get its AST (a Brane with `'True` and `'False`
      statements). Add `'program = {user source}` as a statement to that AST. Compile the
      combined AST as one unit → one BraneFir (self-rooting). The FIR tree is built correctly
      from the start — no re-parenting, no changes to `parent` or `foolish_children` after
      construction. The evaluator returns the `'program` statement's result, not the system
      brane's result.

      **`'program` statement**: the user program brane is wrapped in a null-characterized
      statement named `'program`. This ensures the AB search parent chain works correctly:
      user_brane → StatementFir('program) → system.foo → self (terminates).

- [ ] **Preserve the user program's line numbers**: making system.foo the ancestor must not
      shift program line numbering (system.foo is a distinct brane above, with its own lines).
      Unit test via `as_stmt_line_number` / `step_until_line_number` on a one-line program.

## Phase 5.5 — Sequencer renders named creations

The sequencer currently renders all creations as `⬤`. When a creation originates from a
null-characterized statement (like `'True = ⬤` in `system.foo`), the sequencer should render
the characterized name instead (e.g. `'True`). If no name is known, fall back to `⬤`.

**Design**: the name is NOT stored on `Fir::Creation`. `Fir::Creation` remains a unit variant.
When the sequencer encounters a creation, it searches the containing brane for a
null-characterized statement whose value is that creation, using the pattern
`?'[a-zA-Z_0-9]+=CREATION_REF` (same identifier pattern as the parser). If found, render the
characterized name (e.g. `'True`). If not found, render `⬤`. This is consistent with how
Foolish resolves names — through search, not stored metadata.

- [ ] Sequencer (`foolish-core/src/sequencer.rs:614`): when rendering a creation, search the
      containing brane using `?'[a-zA-Z_0-9]+=CREATION_REF`. The search looks for a
      null-characterized statement whose value (`Rc::ptr_eq`) matches the creation. If found,
      render the characterized name; otherwise render `⬤`.
- [ ] The sequencer needs access to the containing brane to perform the search. This may require
      passing the brane context through the rendering pipeline, or having the sequencer walk up
      the parent chain from the creation FIR.
- [ ] Unit tests: creation from null-characterized statement renders as `'True`; anonymous
      creation renders as `⬤`.
- [ ] Update einmo baselines for any snapshots that now show `'True`/`'False` instead of `⬤`.

## Phase 6 — Comparison operators via brane search (revised)

> ## ⛔ STOP — INSPECT THE NEW SPECIFICATION BEFORE IMPLEMENTING ⛔
>
> **Do NOT implement any part of Phase 6 from the prose below.** The design is settled — see
> **FOOP-33.md §5.0's evening revision banner** ("REVISED AGAIN, SAME DAY") for the full,
> human-confirmed design and its reasoning. Summary:
>
> - **Placement is postfix**: `<<#-2>> < <<#-1>>`, both operands before `'lt`, same shape as
>   the plan's prior `19fe78ef` revision (the infix decision from earlier the same day was
>   itself reverted).
> - **No brane concatenation.** `{1, 2, 'lt}` is one ordinary brane literal, written directly.
> - **`'lt` resolves via ordinary ancestral search into `system.foo`**, same as `'True`. The
>   FVM does not special-case the name `'lt` at parse time.
> - **Detachment and recoordination — existing machinery, not new.** `'lt`'s `#-2`/`#-1`
>   lookups settle ECONSTANIC inside `system.foo` alone (no valid neighbors there); once the
>   reference is detached/recoordinated into the user's brane (the same mechanism documented
>   under "Detachment and Coordination" in `AGENTS.md`), those lookups find real neighbors —
>   `1` and `2` — and the comparison computes.
> - **The result is read out with `$`**: `comparison_result =$ {1, 2}'lt` or `{1, 2, 'lt}$` —
>   the brane literal by itself is not the full expression; `'lt`'s computed boolean becomes
>   the brane's tail, which `$` extracts.
> - **New Rust module**: `system_foo.rs` in `foolish-ubca`, one shared shape (operand lookup +
>   settling logic, possibly reusing `OperatorFir`'s existing structure) across `'lt`/`'gt`/
>   `'le`/`'ge`/`'eq`, differing only in the Rust comparison run in the op step.
>
> **Not yet implemented or independently re-verified against a live FVM trace** (design
> reached through discussion, no code written) — `'lt` genuinely needs Phase 5's `system.foo`
> composition to exist before it can be tested against. Implement Phase 5 first; return here
> once it's in place and confirm the detachment/recoordination read against a real trace before
> trusting it further.
>
> **Ordering (human-directed):** (1) all pre-existing tests pass — **done**, suite green as
> of 2026-08-03. (2) `'True`/`'False` introduced via the `system.foo` composition (Phase 5).
> (3) *only then* comparisons, per §5.0.
>
> The prose below this point is retained as a historical record of the superseded
> `19fe78ef` design, not as an instruction — it is close to, but not identical with, the
> current design; confirm against FOOP-33.md §5.0's evening revision before using it.

- [ ] **GATE: implement Phase 5 (`system.foo` composition) first; then implement Phase 6 per
      FOOP-33.md §5.0's evening revision** — postfix `'lt`/`'gt`/`'le`/`'ge`/`'eq` via
      `<<#-2>>`/`<<#-1>>`, ordinary ancestral search into `system.foo` (no concatenation),
      detachment/recoordination to resolve the operand lookups, `$`-extraction of the result,
      new `system_foo.rs` Rust module. Confirm the detachment/recoordination behavior against
      a live trace once `system.foo` exists to test against, before trusting the design further.

**Design change.** Comparison operators are no longer infix `\o<`/`\o>`/`\o<=`/`\o>=`/`\o==`
parsed at the token level. Instead, `system.foo` defines null-characterized creations `'lt`,
`'gt`, `'le`, `'ge`, `'eq`. The FVM, when it observes one of these names in a brane context,
interprets the brane's preceding elements as operands and performs the comparison in Rust,
producing `'True` or `'False` from `system.foo`.

**How `'lt` works (same mechanism as `+`, but brane-scoped).** The `'lt` operation is
implemented like the existing `OperatorFir` for `+`, except it provides a brane rather than
inline operands. When the FVM instantiates the `'lt` operation, it automatically creates
`foolish_children` containing two SFF-marked (StayFoolish) index searches:

- `<<#-1>>` — SFF index search for the last element (second operand)
- `<<#-2>>` — SFF index search for the second-to-last element (first operand)

These are **unanchored** SFF children — they resolve against the containing brane at step time.
The operator's stepping logic evaluates both SFF searches to get integer values, performs the
Rust comparison (`<`, `>`, `<=`, `>=`, `==`), and enqueues the result (`'True` or `'False`)
into `ubc_children`. All three elements — the two operands and the result — become members of
the containing brane if accessed. The `$` (tail) search retrieves the result.

**Syntax.** `{1, 3,}'lt$` — a brane with two values, followed by a value search for `'lt`
anchored to the tail (`$`). The search finds the `'lt` system definition. The `'lt` operation
instantiates `<<#-2>>` and `<<#-1>>` as SFF children, resolves them to `1` and `3`, compares
`1 < 3` in Rust, enqueues `'True` into `ubc_children`. The brane now has three accessible
elements: `1`, `3`, `'True`. The `$` search finds `'True`.

**Kept from old Phase 6:** the `OperatorFir` infrastructure stays. The five operator tokens
(`LTOp`, `GTOp`, `Le`, `Ge`, `EqOp`) and their parser matchers are **deleted** — comparison
is no longer syntactic sugar; it is brane search into system definitions.

- [ ] **Research: `$` vs concatenation precedence.** Before any comparison-brane work, resolve
      the associativity of `$` (tail search) versus concatenation. Key question: does `{1,3}'lt$`
      parse as `({1,3}'lt)$` (brane-then-search-then-tail) or `{1,3}('lt$)` (brane-then-search-
      with-tail)? The desired reading is the former — `'lt` is applied to the brane, then `$`
      retrieves the result. If the parser binds `$` tighter than the search-on-brane, explicit
      parens `({1,3}'lt)$` may be needed. Verify by testing current parser behavior with
      `{a}'x$` and `({a}'x)$`. Document the resolution here before proceeding.
- [ ] **Delete** from `foolish-parser/src/token.rs`: `LTOp`, `GTOp`, `Le`, `Ge`, `EqOp` tokens.
- [ ] **Delete** from `foolish-parser/src/parser.rs`: the `\o<`/`\o>`/`\o<=`/`\o>=`/`\o==`
      infix operator matchers and their precedence handling.
- [ ] **Delete** from `foolish-parser/src/lexer.rs`: the `\o` prefix and Unicode U+0332
      recognition for these five operators.
- [ ] **Delete** from `foolish-ubca/src/fir_kinds.rs` (`OperatorFir::combine`): the five
      comparison arms (`<=`, `>=`, `\\<`, `\\>`, `\\==`) and the `op if matches!` block.
      Keep the `+`, `-`, `*`, `/`, `%`, unary `-`, `$` arms.
- [ ] **Delete** comparison-related parser unit tests and sequencer `op_display()` rendering
      for these operators.
- [ ] **Update `system.foo`**: add five null-characterized system operations:
      ```foolish
      {!!system.foo
          'True  = ⬤
          'False = ⬤
          'lt    = ⬤    !! less-than system operation
          'gt    = ⬤    !! greater-than system operation
          'le    = ⬤    !! less-or-equal system operation
          'ge    = ⬤    !! greater-or-equal system operation
          'eq    = ⬤    !! equality system operation
      }
      ```
- [ ] **FVM evaluator special-casing**: when the evaluator encounters a search result that
      resolves to one of `'lt`, `'gt`, `'le`, `'ge`, `'eq` (identified by the creation's
      `Rc::ptr_eq` against the system root brane's definitions), create an `OperatorFir`-like
      structure that:
      1. Instantiates two SFF (StayFoolish) unanchored index searches as `foolish_children`:
         `<<#-2>>` (first operand) and `<<#-1>>` (second operand).
      2. Steps the SFF searches to resolve against the containing brane.
      3. Checks both resolved values are integers (else NK).
      4. Performs the Rust comparison (`<`, `>`, `<=`, `>=`, `==`).
      5. Resolves `'True` or `'False` from the system root brane.
      6. Enqueues the result into `ubc_children`.
      7. The result (plus the two operands) becomes accessible as brane members.
- [ ] Unit tests: `{1, 3,}'lt$` → `'True`; `{3, 1,}'lt$` → `'False`; `{⬤, 1,}'lt$` → NK.
      Verify all three elements are accessible as brane members.
- [ ] Einmo tests: update `int_comparators.foo`, `boolean/comparison_operators.foo`,
      `comprehensive.foo` to use the new brane-search syntax.

## Phase 7 — Documentation and Tests

- [ ] Document the null-characterized name-constant rule and universal characterizations
      (update `docs/vintage_legacy/CREATION.md` cross-refs and add engineering notes under
      `docs/ubc1/how`); update AGENTS.md §Foolish Terminology / §Searches as needed
      (with the "## Last Updated" protocol).
- [ ] Update AGENTS.md Code Style section: agents must use Unicode operator forms when writing
      Foolish code (`⬤` not `{*}`, `<̲=̲` not `\o<=`, etc.). The `\o` prefix is for keyboard
      input only.
- [ ] Create einmo tests under `foop/33/` with subdirectories:
      - `creation/` — basics, nilpotent, referential_equality
      - `creation_concat.foo` — null-constant rule in concatenation
      - `boolean/` — comparison_operators, constants, if_then_else
      - `characterizations/` — null_char_constant, nf_error, quote_bearing_search, proximity_rule
      - `int_comparators.foo` — Unicode + ASCII `\o` forms side by side
      - `comprehensive.foo` — all features interacting
- [ ] Promote **only** `foop/33/*` einmo baselines to `checked/` — and only after the
      suite is otherwise green (no foreign divergence). If any baseline to be promoted has a
      `verified/` twin, STOP and ask the human. (This replaces the earlier bare "promote all"
      box that was misused to overwrite 11 FOOP-23 `checked/` baselines — see
      "Problems Discovered During Implementation" in `FOOP-33.md`.)

## Phase 7R — Repair: Phase 3 value-search regression (spec §2 + `default_equal`)

> Discovered after Phases 1–7 committed. The suite is RED on exactly two FOOP-23 tests
> (`foop/23/comprehensive`, `foop/23/value_search_pattern_error`) — a regression introduced by
> Phase 3's `default_equal`. The defect is in **spec §2 rule 4** (it conflated "provably
> different kinds" with "genuinely unknowable"); the implementation followed the spec. Read
> §"Problems Discovered During Implementation" in `FOOP-33.md` before touching code.

- [x] (read §"Problems Discovered During Implementation" and revised §2 rule 4 in `FOOP-33.md`)
      (2026-08-02 21:15)
- [x] Revise `default_equal` fallthrough (`foolish-ubca/src/fir_kinds.rs:457`): different
      non-NK kinds where both are constanic (brane-vs-integer, integer-vs-creation,
      brane-vs-creation) → `Equality::NotEqual` (not `Unknowable`). Reserve `Unknowable` for:
      either operand `NK` (already line 448), or two branes (brane-vs-brane equivalence
      unspecified). Keep the integer-equality and creation-ptr_eq arms unchanged.
      (2026-08-02 21:15)
- [x] Fix the regression-locking unit test `matcher_value_reject_non_integer_candidate`
      (`fir_kinds.rs:4954`): the test **name** already says "reject"; change the assertion from
      `MatchOutcome::NkStop` ("brane-vs-integer is Unknowable → NkStop") to
      `MatchOutcome::Reject` ("brane-vs-integer is NotEqual → skip"). Add the companion case:
      two branes vs an integer pattern ⇒ `Unknowable` ⇒ `NkStop` (genuinely unknowable,
      unchanged).
      (2026-08-02 21:15)
- [x] Update the `default_equal` truth-table unit tests (§Test Plan line ~651) to the revised
      outcomes: creation-vs-integer ⇒ `NotEqual`; brane-vs-integer ⇒ `NotEqual`; two branes ⇒
      `Unknowable`; NK operand ⇒ `Unknowable`.
      (2026-08-02 21:15) — Note: no dedicated default_equal truth-table tests existed; the
      behavior is verified through the matcher tests and einmo suite.
- [x] Confirm the null-constant rule (§4) is **unaffected**: `'True=3` (creation vs integer)
      now resolves via `NotEqual` (was `Unknowable`), but §4 treats both as refusal — the
      observable NK result is unchanged. Re-run the `'True=3` ⇒ NK unit/approval test to prove
      no behavior change.
      (2026-08-02 21:15) — einmo suite passes; null-constant tests in FOOP-33 baselines unaffected.
- [x] Confirm comparison operators (§5) are **unaffected**: they use the evaluator's own
      integer-check, not `default_equal`. Re-run `comparison_nk` tests.
      (2026-08-02 21:15) — comparison operator einmo tests pass.
- [x] Run all tests — old and new — and make sure they all pass correctly.
      Must include: `cargo test -p foolish-ubca --lib -- run_einmo_tests` exits 0 (the two
      FOOP-23 divergences resolved: `output` matches `checked` again, no `promote` used), and
      `cargo test --workspace` exits 0.
      (2026-08-02 21:15) — 502+ tests pass, 0 fail.
- [x] If the suite is green and no foreign baseline diverges, promote **only** the new
      `foop/33/*` baselines (the 8 `only-in-output` tests) to `checked/`. Do not touch any
      `foop/23/*` or other foreign baseline.
      (2026-08-02 21:15) — Promoted 8 FOOP-33 baselines. Note: einmo promote re-signed all
      169 checked/ files (cosmetic timestamp changes); restored 161 foreign files via git checkout.
- [x] Sanity: re-read revised §2 rule 4 and the "Problems Discovered" section; confirm the
      code, the unit tests, the spec, and the einmo suite all agree on brane-vs-integer ⇒
      `NotEqual` ⇒ skip.
      (2026-08-02 21:15)

## Phase 8 — Merge

- [ ] Merge `foop-33-creation-postulate` to `jia`
  - [ ] Write and verify `foop_33_comprehensive.foo` (reserved name): creation, characterized
        names, quote-bearing search, referential equality, `system.foo` parent brane,
        null-constant refusal (incl. `A A A` concatenation), comparison via brane search
        (`{1, 3,}'lt$` → `'True`, `{3, 1,}'lt$` → `'False`, `{⬤, 1,}'lt$` → NK),
        interacting with prior features (nested branes, contexted `&` searches). Generate +
        verify `.snap.new`; final approval is human-signed.
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

**Date**: 2026-08-03 (evening)
**Updated By**: Claude Code / claude-opus-5
**Changes**: Phase 6 gate rewritten — the comparison-operator design **settled** through
discussion the same evening: postfix again (`<<#-2>> < <<#-1>>`), **no brane concatenation**
(`{1, 2, 'lt}` is one ordinary brane literal), `'lt` resolved via ordinary ancestral search
into `system.foo` (same as `'True`), the existing detachment/recoordination mechanism
resolves its operand lookups once recoordinated into the user's brane, result extracted with
`$`. New shared `system_foo.rs` Rust module for `'lt`/`'gt`/`'le`/`'ge`/`'eq`. Not yet
implemented — needs Phase 5's `system.foo` composition first to test against. Status summary
table's Phase 6 row updated to match. Earlier the same day: human resolved both open decisions
from the checkbox reconciliation — (1) anchored value-search-miss on creation inequality is
correctly `NK`, `referential_equality.foo` promoted, einmo suite 169/169 green; (2)
creation-vs-integer equality is correctly `NotEqual` (plan text was wrong, code was right;
corrected). Phase 3 now has exactly one open item: missing `default_equal` truth-table unit
tests. Full history in `git log` on this file.
