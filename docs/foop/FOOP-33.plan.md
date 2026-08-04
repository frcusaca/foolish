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
| 1 — `Identifier` | ✅ **fully done** (2026-08-04) — `BraneFir.characterizations` migrated to `Characterizations`; `NormalBraneFir.characterizations` correctly out of scope (separate wire type, see checkbox) |
| 2 — Creation `⬤`/`{*}` | ✅ **fully done** — was showing all-unchecked, now corrected; every item verified present in code |
| 3 — `default_equal` | ✅ **fully done** (2026-08-04) — 9 direct truth-table unit tests added; creation-vs-integer confirmed `NotEqual` per human ruling 2026-08-03 (every integer is itself a creation). |
| 4 — Null-characterized constants | ✅ **complete** (2026-08-04) — ancestral/same-brane conflict detection via `StatementFir` self-check (new `self_weak` field), NF via `settled_result()` override, concatenation collision handling with transitive poisoning. 14 new unit tests. See the phase's design notes for two architectural findings made during implementation (self-reference plumbing, the `NkFir::nk` pre-constanic bug). |
| 5 — `system.foo` composition | ⚠️ **implemented and verified (2026-08-04), ONE open finding escalated to Open Questions** — `system_foo.rs` composes `system.foo` as root with `program` as its last member, wired into the one production entry point (`UbcaEvaluator::evaluate`). Found and fixed a real bug where Phase 4's NF refusal was enforced but never rendered (4th bypass site, in `evaluator.rs`). 280/280 unit tests pass; einmo suite green except ONE frozen `verified/` FOOP-62 baseline whose exact iteration-budget snapshot composition unavoidably shifts — see the phase's own checkbox and FOOP-33.md's Open Questions for the full writeup; needs a human decision, not guessed around. |
| 5.5 — Sequencer renders named creations | ⛔ **CROSSED OUT (2026-08-04) — superseded by Phase 9, not implemented under this heading** — the literal design (a `foolish-core::Fir::Creation`-side search) was investigated and found not implementable as written (identity/parent-chain only exists on the `foolish-ubca` side by conversion time). The capability itself ships as Phase 9's `get_display_name()` bridge, resolved at the `foolish-ubca`→`foolish-core` conversion boundary — inside FOOP-33, not the separate future FOOP originally proposed. All four of this phase's checkboxes are struck through with a pointer to their Phase 9 equivalent. |
| 6 — Comparison operators | ✅ **complete (2026-08-04)** — `ComparisonFir` + `ComparisonOp` enum in `system_foo.rs` (ONE kind, not five; the Rust comparison is the only per-operator difference). `system.foo` declares `'lt`/`'gt`/`'le`/`'ge`/`'eq` as ordinary creations; their real bodies are installed via a new `compiler.rs` `BodyOverride` hook, and every use resolves by ordinary ancestral search + detachment/recoordination — no name-based special-casing anywhere. Key traced finding: an ECONSTANIC operand must settle the comparison ECONSTANIC, **not** NK (NK is terminal and would poison the definition so no search could ever hand it out). 9 new unit tests incl. `comparison_nyes_transitions` (all three terminal states); 2 new einmo baselines. 295 unit tests pass; einmo suite in its starting state. |
| 7 — Docs/Tests | 🟡 partial, done piecemeal, not formally tracked |
| 7R — Phase 3 value-search regression repair | ✅ done (earlier session) |
| 9 — Sequencer renders creation names | 🟡 **queued (2026-08-04), design written, not started** — the `get_display_name()` bridge. Depends on the FVM-side `CreationFir::get_display_name()` (in progress separately); largely independent of Phase 6 (different code path), so order doesn't matter. **May be punted past merge** — nothing depends on it and today's glyph fallback is correct, just less informative. Needs a human decision on the `foolish-core::Fir::Creation` shape (it must stop being a unit variant) and its JSON/hssnap stability. Closes the long-open "Creation value render form in hssnap" question. See Phase 9's section. |
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
- [x] Migrate `BraneFir.characterizations` (`foolish-ubca/src/fir_kinds.rs`, `BraneFir` struct)
      to `Characterizations`. **Scope correction**: the core-fir `NormalBraneFir.characterizations`
      (`foolish-core/src/fir.rs`) is a separate, JSON-serializable wire type in a crate
      (`foolish-core`) that does not depend on `foolish-ubca` and cannot reference
      `foolish-ubca::identifier::Characterizations` — it correctly keeps its `Vec<String>` shape
      for hssnap round-tripping; only `foolish-ubca::BraneFir` (the FVM-internal FIR) was in
      scope. Extended `Characterizations` with `from_brane_parts(Vec<String>)` (a brane has no
      name, so `is_nully_characterizing_coordinate_name()` is always `false` for it) and
      `components() -> &[String]` (the raw ordered list, needed by
      `as_brane_characterizations()` for the sequencer's `a'b'` trailing rendering, which is
      unchanged). Updated all 6 `BraneFir` construction sites (`compiler.rs` ×2 incl. the
      detach/recoordinate clone path, `fir_kinds.rs` ×3, `fir_trait.rs` ×1 test scaffold).
      New tests: `identifier::tests::brane_characterizations_retain_raw_components`,
      `brane_characterizations_empty_when_none`, `default_characterizations_are_empty_and_not_nully`
      (pure `Characterizations` unit tests), plus FVM-level
      `fir_kinds::tests::brane_fir_reports_its_own_characterizations` (compiles `{x = a'b'{y=1;};}`,
      asserts `as_brane_characterizations() == ["a", "b"]`) and
      `brane_fir_with_no_characterizations_reports_empty`.
      (2026-08-04 — verified: `cargo test -p foolish-ubca --lib` 251 passed incl. 4 new
      identifier tests + 2 new fir_kinds tests; `cargo test --workspace` all green;
      `run_einmo_tests` unaffected — sequencer rendering path untouched behaviorally.)
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

- [x] **GAP (verified 2026-08-03) — no dedicated `default_equal` truth-table unit tests
      exist.** `default_equal` (`fir_kinds.rs:445`) is only exercised indirectly through
      `matcher_value_reject_non_integer_candidate` and the two `value_search_pattern_
      referencing_a_creation_*` tests added 2026-08-03 (creation-vs-creation only). No test
      directly calls `default_equal` with an integer/integer pair, an NK operand, or a
      brane/brane pair. Still open — write these before considering Phase 3 done.
      **CLOSED 2026-08-04**: added 9 direct `default_equal` unit tests (`fir_kinds.rs`, `mod
      tests`, grouped under "default_equal truth table (FOOP-33 §2, Phase 3 gap)" right before
      `matcher_value_reject_non_integer_candidate`): same-integer ⇒ `Equal`, different integers
      ⇒ `NotEqual`, same creation `Rc` ⇒ `Equal`, distinct creations ⇒ `NotEqual`, either operand
      `NK` ⇒ `Unknowable` (both argument orders), same `NK` `Rc` compared to itself ⇒ still
      `Unknowable` (NKs are never equal to each other, even themselves), creation-vs-integer ⇒
      `NotEqual`, brane-vs-integer ⇒ `NotEqual`, two branes ⇒ `Unknowable`. All 9 pass; no code
      change to `default_equal` itself was needed — this was purely the missing test coverage.
      (2026-08-04 — verified: `cargo test -p foolish-ubca --lib -- default_equal_` 9/9 pass;
      `cargo test --workspace` 260 foolish-ubca tests (was 251) all green; `run_einmo_tests`
      unaffected.)
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

## Phase 4 — Null-characterized name constants — COMPLETE (2026-08-04)

- [x] Unit tests: ancestral null-constant conflict — ancestor `'k=1`, descendant `'k=2` ⇒
      descendant `get_value()` returns `NF("'k not-foolish")` (NF, not plain NK — see Phase 1
      NF task); `Equal` redefinition (same creation) ⇒ permitted; **poison scope** — a sibling
      brane resolving `k` elsewhere (or not at all) is unaffected; descendant "is this a
      null-characterized coordinate name?" query.
      (2026-08-04 — `fir_kinds.rs` `mod tests`: `null_const_first_definition_is_permitted`,
      `null_const_same_brane_conflicting_redefinition_settles_nf`,
      `null_const_get_value_via_value_returns_the_nf_nk`,
      `null_const_same_creation_redefinition_is_permitted`,
      `null_const_ancestral_conflict_via_ab_search`,
      `null_const_poison_scope_sibling_brane_unaffected`,
      `null_const_descendant_query_true_for_ancestor_null_const_false_otherwise`,
      `null_const_rule_does_not_fire_on_plain_names` — 8 tests, all pass.)
- [x] `StatementFir`'s own step (Braning, self-check — see design note below) — for each
      statement with `is_nully_characterizing_coordinate_name()`, walk IB (same brane, earlier
      statements) then AB (ancestor branes) for a same-named prior null-const; on a
      **non-`Equal`** value (by `default_equal`) set `nf_reason` **once** (terminal, no
      re-alarm), read via a `settled_result()` override that returns a fresh, already-`Nk`
      `NkFir` with reason `"'<name> not-foolish"`. No new FIR kind/NYES state — reuses `NkFir`.
      (2026-08-04 — `foolish-ubca/src/fir_kinds.rs`: `StatementFir::check_null_const_conflict`,
      `StatementFir::settled_result` override, `NF_PREFIX`/`is_nf_reason` from Phase 1 wired up
      for the first time.)

      **Design note — deviates from the plan's original "BraneFir step" framing, for a
      concrete reason found during implementation.** `fir_op_step(&self, scope: &Scope)` has
      no `self_ref: &FirRef` parameter anywhere in the `Fir` trait, and tracing `step_inner`
      confirmed `scope.current_statement` is NOT reliably a statement's own `FirRef` at the
      point its OWN `fir_op_step` runs (it's set for the scope used to step a statement's
      *body*, one level down — not the scope the statement's own step receives). `BraneFir`
      does hold each child's `FirRef` directly (via `foolish_children()`), but `BraneFir`
      itself faces the identical self-reference problem one level up. The fix: `StatementFir`
      gained a `self_weak: Weak<RefCell<dyn Fir>>` field, established via `Rc::new_cyclic` at
      construction (the same established pattern `ProtoBrane.parent` already uses, one level
      up) — every `StatementFir` construction site (6 total, across `compiler.rs`,
      `fir_kinds.rs`, `fir_trait.rs`) was updated. With `self_weak` upgraded to a real `FirRef`,
      the statement calls the ordinary `_ib_search`/`_ab_search` default trait methods ON
      ITSELF — verified against a live trace (see the two precondition regression tests below)
      that this correctly finds an earlier same-brane null-const via IB, or an ancestor-brane
      one via AB, using EXISTING search machinery, no new engine code.
      (2026-08-04 — precondition tests pinning the search behavior this depends on:
      `stmt_ib_search_finds_earlier_null_characterized_sibling_by_searchable_name`,
      `stmt_ab_search_finds_ancestral_null_characterized_definition`.)

      **A second, related architectural finding**: readers that resolve "a found statement's
      value" (`clone_stmt_result` in `SearchFir`, and `IndexFir`'s two contexted/anchored
      paths) all read `foolish_children().first()` directly — the raw parse-time body —
      bypassing any statement-level indirection entirely. Since `foolish_children` is
      documented as immutable, fixed-shape topology (no public mutator to swap an element),
      and the crate's `set_nyes` ownership contract (FOOP-62 #10, documented in
      `proto_brane.rs`) forbids one FIR mutating another's `nyes` from outside, the NK
      substitution could not be done by reaching into the body FIR. Instead, `StatementFir`
      gained a `settled_result()` override (previously always `None` — a plain statement pushes
      no `ubc_children`) that returns `Some(nk)` ONLY when `nf_reason` is set, `None`
      otherwise (fully backward compatible). All three direct-body-read call sites were updated
      to prefer `settled_result()` via a shared helper, `statement_value_for_comparison`, before
      falling back to the raw body — unifying what was three near-duplicate inline patterns
      into one.
- [x] Concatenation collision handling: replaced the blind clone loop in
      `ConcatenationFir::populate_concat_helpers` with a collision-aware merge
      (`ConcatenationFir::apply_null_const_rule_to_merged_stmt`) applying the same rule (same
      `NF("'<name> not-foolish")`) against already-merged statements, searched nearest-first so
      poisoning is transitive through a chain of 3+ same-name clones.
      (2026-08-04 — `foolish-ubca/src/fir_kinds.rs`.)

      **Design note — a clone built via `constanic_clone_at` from an already-constanic source
      is constructed DIRECTLY at its terminal `Nyes`** (`Nyes::transform_for_clone`), skipping
      `Prembrionic`/`Embryonic`/`Braning` entirely — so `StatementFir::check_null_const_conflict`
      (which lives in the `Braning` arm) never runs for a concatenation-merged clone. This is
      exactly why concatenation needs its OWN, separate application of the rule, confirmed by
      first implementing WITHOUT it and observing (via a live trace, not assumed) that merged
      duplicates were never refused. A new default trait method, `Fir::set_nf_reason(&self,
      reason: String)` (no-op default; `StatementFir` overrides it), lets the concatenation
      merge — which only has a `FirRef`, not a concrete `&StatementFir` — set the refusal on a
      clone it just built, without downcasting.

      **A subtle bug found and fixed via live tracing**: the first `settled_result()` override
      built its `NkFir` via `NkFir::nk(reason, parent)`, which constructs at `Nyes::Prembrionic`
      (needs a step to reach `Nk`). The concatenation merge's own null-const comparison reads a
      PRIOR statement's `settled_result()` directly, without ever stepping it — so a `Prembrionic`
      NK failed the `is_constanic()` gate and was silently skipped, meaning a THIRD conflicting
      redefinition in a 3-way merge was wrongly left unrefused (confirmed via a
      `temporary_reproduce_to_debug_*` test, then fixed and deleted per the debugging skill's
      discipline). Fix: `settled_result()` now constructs its `NkFir` directly at `Nyes::Nk`
      (bypassing `NkFir::nk`'s pre-constanic default) — `settled_result`'s own contract is "the
      constanic gate is already applied," so what it returns must already BE constanic.
- [x] Unit test the concatenation case `{A={'a=1}, B = A A A}` (later `'a`'s → `NF`, first
      intact) and `{A={'a=⬤}, B=A A}` (same creation ⇒ both permitted — value-sensitive).
      (2026-08-04 — `null_const_concatenation_collision_later_duplicates_settle_nf` (same
      integer value across clones → all permitted, proving `default_equal` not "duplicate
      name"), `null_const_concatenation_collision_with_conflicting_values_settles_nf` (3-way
      merge with 3 DIFFERENT values → first permitted, 2nd and 3rd both NF — transitivity),
      `null_const_concatenation_same_creation_is_permitted_value_sensitive` (same creation Rc
      across 2 merged clones → both permitted), `null_const_concatenation_empty_and_single_
      operand_merge_without_spurious_nf` (regression guard against false positives) — 4 tests,
      all pass.)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-08-04 — verified: `cargo test -p foolish-ubca --lib` 274 tests (was 260) all pass,
      including 8 null-const + 4 concatenation + 2 search-precondition new tests (14 total new
      for Phase 4); `cargo test --workspace` all green; `run_einmo_tests` 169/169 unaffected.
      `cargo fmt`/`cargo clippy -D warnings` clean on every line touched — one pre-existing
      clippy warning remains in an unrelated function (`FirRefNavExt::deepest_econstanic_in_
      chain`, `fir_kinds.rs`) that predates this phase and was left untouched per AGENTS.md's
      "don't fix unrelated pre-existing debt" guidance.)

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

- [x] In the evaluator, embed at compile time and keep the source string:

      ```rust
      const SYSTEM_FOO_SRC: &str = include_str!(concat!(env!("OUT_DIR"), "/system.foo"));
      ```
      (2026-08-04 — new module `foolish-ubca/src/system_foo.rs`, `pub const SYSTEM_FOO_SRC`,
      exactly as specified. Pinned by `system_foo::tests::system_foo_src_embeds_true_and_false`.)

- [x] Implicitly compose `SYSTEM_FOO_SRC` with the user program as a member named `program`
      (not wrapped in a `'program` null-characterized statement — see the banner above), before
      `step_to_settled`, on the crate's one production entry path
      (`foolish-ubca/src/evaluator.rs::UbcaEvaluator::evaluate` — the only entry path that
      exists; CLI/REPL both go through it, there is no separate path to also update). Not
      opt-in. `system.foo`'s own AST gains one more statement, `program = {user source}`,
      appended last; compiles the combined AST as one unit → one self-rooting `BraneFir`. The
      evaluator extracts the `program` member's result via `stmt_at(stmt_count() - 1)` in Rust
      (`system_foo::program_result`), not the system brane's own result and not via a Foolish
      search.
      (2026-08-04 — `system_foo::compose_program_with_system` + `system_foo::program_result`,
      wired into `evaluator.rs`'s `evaluate`. Pinned by 6 unit tests in `system_foo.rs`:
      `compose_appends_program_as_last_statement`, `composed_root_is_self_rooting`,
      `program_line_numbers_are_preserved`, `program_resolves_true_and_false_ancestrally`,
      `ab_search_terminates_at_system_root_no_infinite_walk`,
      `program_redefining_true_to_a_conflicting_value_is_refused`.

      **Two findings from live-tracing this against the FVM (not assumed)**:
      1. A bare `True` (no leading `'`) can NEVER resolve to `system.foo`'s `'True` — confirmed
         by first writing a test with bare `True` and watching it settle `ECONSTANIC` (search
         miss), then re-reading FOOP-33.md §3 line ~324 ("`'True` does not match a plainly-named
         `True`") and fixing the test to use `'True`. This is the spec's own documented rule,
         not a bug — but it means the plan's own prior wording "True/False resolve as ordinary
         sibling lookups" (no quote shown) is easy to misread; the user-facing syntax IS `'True`.
      2. **A real bug found and fixed**: the null-const rule's refusal (Phase 4) was enforced
         internally (`StatementFir::settled_result()` correctly returned the NF `NkFir`) but was
         NEVER RENDERED — `evaluator.rs`'s `proto_to_core_fir_inner` (`FirKind::Statement` and
         `FirKind::Brane` arms) read `foolish_children().first()` directly, the same
         bypass-the-refusal pattern Phase 4 had already fixed at 3 OTHER call sites in
         `fir_kinds.rs`, but this 4th site — the actual sequencer/einmo rendering path — was
         missed because Phase 4's own unit tests never rendered through it. Caught by writing
         the Phase-5 approval test `foop/33/boolean/null_char_constant.foo` and observing
         `'True=3` in the OUTPUT (should have been the NF NK) before promoting — never assumed
         correct from the evaluator/unit-test level alone. Fixed by widening
         `fir_kinds::statement_value_for_comparison` to `pub(crate)` and routing both
         `evaluator.rs` sites through it. Re-ran the FULL workspace suite after the fix: no
         other baseline shifted, confirming this bug was previously entirely unexercised
         end-to-end (Phase 4 shipped genuinely inert at the rendering layer until now).)
- [x] ~~Preserve the user program's line numbers~~ — **DROP, moot under composition** (see
      banner above: statement indices are per-brane, 0-based, unaffected by siblings in a
      different brane). Confirmed moot, not merely assumed:
      `system_foo::tests::program_line_numbers_are_preserved` pins that a one-line user program
      still reports line 0.
- [x] Approval `.foo` tests: creation + identity; quote-bearing search; `True`/`False`
      resolve as ordinary sibling lookups within `system.foo` (not `_ab_search` ancestry);
      `'True=3` ⇒ NF (`"'True not-foolish"`) while `'True='True` permitted.
      (2026-08-04 — `foop/33/boolean/constants.foo` (`'True`/`'False` resolve to distinct
      creations, referencing `'True` twice gives the same creation), `foop/33/boolean/
      null_char_constant.foo` (`'True='True` permitted, `'True=3` refused with the NF NK
      rendered in OUTPUT — the exact case that caught the rendering bug above). Both promoted
      to `checked/` (no `verified/` twin; reviewed and justified line-by-line before
      promoting, not auto-accepted).

      **⛔ ONE UNRESOLVED FINDING, escalated to Open Questions, not silently fixed**:
      composing `system.foo` shifts exactly WHEN the shared `MAX_STEPS = 10_000` iteration
      budget is exhausted for programs that deliberately never settle, because per-statement
      step work now interleaves between `system.foo`'s own settling and the user program's.
      This broke ONE einmo baseline — `foop/62/infinite_loop.foo.einmo` — which has a
      `verified/` twin (frozen; AGENTS.md forbids promoting over it without a human reviewer's
      key) and pins an EXACT NYES snapshot at the moment the budget is exhausted (root
      `NK(ITERATION-EXCEEDED,...)`); with composition the same input now shows root `BRANING`
      (budget not yet exhausted at that point in the composite tree). Every OTHER test in the
      suite — 279 (later 280 including the 2 new ones above) — matches byte-for-byte, unchanged.
      Traced (not guessed) that a `MAX_STEPS` bump is not a principled one-time fix: step
      consumption depends on task-queue interleaving order between `system.foo`'s and the
      user's statements, not a separable flat "prefix cost," and Phase 6 will change
      `system.foo`'s size again (2 statements → 7), re-shifting this same boundary. See
      FOOP-33.md's Open Questions (dated 2026-08-04) for the full writeup and the two design
      options that would actually close this (not just patch this one test). Left for human
      decision. `output/` for this one file reverted to match `checked/`, not promoted, not
      worked around.
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-08-04 — `cargo test -p foolish-ubca --lib`: 280 tests pass (was 274), 0 fail.
      `cargo test --workspace`: everything green EXCEPT `run_einmo_tests`, which fails on
      exactly the one escalated, documented FOOP-62 finding above — no other divergence.
      `cargo fmt`/`cargo clippy -D warnings` clean on every line touched in this phase
      (`system_foo.rs`, `evaluator.rs`, `fir_kinds.rs`'s `statement_value_for_comparison`
      visibility widening, `lib.rs`, `compiler.rs`'s `AstnCompilerExt` visibility widening);
      same pre-existing unrelated clippy warning as prior phases, left untouched. Phase 5 is
      NOT declared fully green — the one FOOP-62 divergence is a real, open, human-facing
      decision, not swept under a passing test run.)

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

> ## 🚫 PUNTED to a future FOOP 2026-08-04 — not implemented as part of FOOP-33
>
> - [ ] TODO: this phase is punted OUT of FOOP-33 entirely, into a NEW, separate, future FOOP
>       (not yet created or numbered). FOOP-33 will not implement it.
>
> **Human-proposed resolution (one-line summary):** `CreationFir::get_recent_name()`, added in
> `foolish-ubca/src/fir_kinds.rs`, performs a `?=$CREATION`-style value search from its own
> statement to find the most recently used name referencing it — entirely within `foolish-ubca`,
> where real `Rc` identity and the real parent chain exist, sidestepping the blocker below. HOW
> the sequencer (which only sees the lossy `foolish-core::Fir` tree) reaches this
> `foolish-ubca`-only method is still unresolved and is explicitly left as a subtlety to flush
> out later — not to be guessed at when this new FOOP is written.
>
> **Full detail:** see `docs/foop/FOOP-33.md`'s Open Questions, "Creation *value* render form in
> `hssnap`" entry (2026-08-04, later update) — includes the original blocker writeup below and
> the human's proposed two-step resolution in full.
>
> ---
>
> ### Original blocker (2026-08-04, investigated, not implemented) — retained for context
>
> Investigated (not guessed) before writing any code. The design as written cannot be
> implemented literally: (1) the identity-and-parent-chain search this needs can only run
> against `foolish-ubca`'s live `FirRef` tree (confirmed via a live trace,
> `system_foo::tests::referenced_creations_own_parent_chain_reaches_its_defining_brane` — a
> referenced creation's own parent chain correctly reaches back to `system.foo`), NOT against
> `foolish-core::Fir`, which is what `FirSequencer` actually renders and which has already lost
> both identity and brane context by conversion time (`Fir::Creation` is a bare unit variant).
> So the search belongs in `evaluator.rs`'s conversion step, not literally in
> `foolish-core/src/sequencer.rs` as the checkbox below states (stale — written before the
> composition/conversion split existed). (2) Even once found, there is no existing
> `foolish-core::Fir` variant that renders as bare undecorated text to carry a found name
> forward into `FirSequencer`'s output, and the design explicitly insists `Fir::Creation`
> "remains a unit variant" — these two constraints together leave no implementation path
> without a genuine new decision (new field/variant + its JSON/hssnap shape, or a different
> mechanism). Full writeup: FOOP-33.md's Open Questions, "Creation *value* render form in
> `hssnap`" entry (updated 2026-08-04). **Not blocking Phase 6** — Phase 6's own stated
> dependency is Phase 5 (composition), not Phase 5.5 (rendering) — so work continued past this
> point rather than stalling on it.

The sequencer currently renders all creations as `⬤`. When a creation originates from a
null-characterized statement (like `'True = ⬤` in `system.foo`), the sequencer should render
the characterized name instead (e.g. `'True`). If no name is known, fall back to `⬤`.

**Design**: the name is NOT stored on `Fir::Creation`. `Fir::Creation` remains a unit variant.
When the sequencer encounters a creation, it searches the containing brane for a
null-characterized statement whose value is that creation, using the pattern
`?'[a-zA-Z_0-9]+=CREATION_REF` (same identifier pattern as the parser). If found, render the
characterized name (e.g. `'True`). If not found, render `⬤`. This is consistent with how
Foolish resolves names — through search, not stored metadata.

- [~] ~~Sequencer (`foolish-core/src/sequencer.rs:614`): when rendering a creation, search the
      containing brane using `?'[a-zA-Z_0-9]+=CREATION_REF`. The search looks for a
      null-characterized statement whose value (`Rc::ptr_eq`) matches the creation. If found,
      render the characterized name; otherwise render `⬤`.~~ SUPERSEDED 2026-08-04 — this
      literal design (a `foolish-core::Fir::Creation`-side search) was found not implementable
      as written; see "Original blocker" above. The capability is instead delivered as
      Phase 9's `get_display_name()` bridge, resolved at the `foolish-ubca`→`foolish-core`
      conversion boundary where real identity still exists.
- [~] ~~The sequencer needs access to the containing brane to perform the search. This may
      require passing the brane context through the rendering pipeline, or having the
      sequencer walk up the parent chain from the creation FIR.~~ SUPERSEDED 2026-08-04 — moot
      under Phase 9's conversion-time resolution; the sequencer itself never needs brane access.
- [~] ~~Unit tests: creation from null-characterized statement renders as `'True`; anonymous
      creation renders as `⬤`.~~ SUPERSEDED 2026-08-04 — tracked as Phase 9's own test-plan
      items instead.
- [~] ~~Update einmo baselines for any snapshots that now show `'True`/`'False` instead of
      `⬤`.~~ SUPERSEDED 2026-08-04 — tracked as Phase 9's own einmo-repromotion item instead.

**Phase 5.5 status: entirely superseded, not implemented under this heading.** The rendering
capability it wanted ships as Phase 9 within FOOP-33 (not the separate future FOOP the 🚫
banner above originally proposed) once Phase 9's own gate is satisfied. This phase is closed;
no further work is tracked here.

## Phase 6 — Comparison operators via brane search (revised)

> ## ✅ IMPLEMENTED (2026-08-04) — this gate is discharged
>
> Phase 6 is **complete**. The design summarised below is what was built; see the checked
> items after this banner for what each piece became in code, and the STATUS SUMMARY row for
> the short version. The prose *below the `---` divider* remains superseded historical record
> — it describes an evaluator-special-casing approach that was **not** used (`'lt` is never
> recognised by name at the use site).
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
> **Mechanism VERIFIED against a live FVM trace (2026-08-04); implementation now WRITTEN
> (2026-08-04).** Phase 5's `system.foo` composition is now in place, and the load-bearing
> assumption above — that `'lt`'s `#-2`/`#-1` lookups sit ECONSTANIC rather than settling
> terminal NK — is confirmed: `{only = <<#-1>>;}` settles the index to ECONSTANIC, pinned by
> `fir_kinds::tests::sff_marked_unanchored_index_out_of_bounds_settles_econstanic`. (An earlier
> reading that Phase 6 was blocked tested a **bare** `#-1`, which does settle NK — a different
> construct; the SFF marker is what makes the difference, since `build_fir`'s `under_sff` rule
> builds descendant search kinds ECONSTANIC so they never run. See FOOP-33.md's 2026-08-04
> correction entry.) No `IndexFir` change is needed. The implementation — `system_foo.rs`'s
> comparison FIR and the extension of `system/system.foo` — has since been written; the
> mechanism is additionally pinned end-to-end in pure Foolish by
> `fir_kinds::tests::sff_index_operand_recoordinates_to_the_referencing_branes_neighbors`
> (`{defn = <<#-2>>; use = {5, 9, defn};}` → `defn` reads 5 inside `use`).
>
> **Ordering (human-directed):** (1) all pre-existing tests pass — **done**, suite green as
> of 2026-08-03. (2) `'True`/`'False` introduced via the `system.foo` composition (Phase 5).
> (3) *only then* comparisons, per §5.0.
>
> The prose below this point is retained as a historical record of the superseded
> `19fe78ef` design, not as an instruction — it is close to, but not identical with, the
> current design; confirm against FOOP-33.md §5.0's evening revision before using it.

- [x] **GATE: implement Phase 5 (`system.foo` composition) first; then implement Phase 6 per
      FOOP-33.md §5.0's evening revision** — postfix `'lt`/`'gt`/`'le`/`'ge`/`'eq` via
      `<<#-2>>`/`<<#-1>>`, ordinary ancestral search into `system.foo` (no concatenation),
      detachment/recoordination to resolve the operand lookups, `$`-extraction of the result,
      new `system_foo.rs` Rust module. Confirm the detachment/recoordination behavior against
      a live trace once `system.foo` exists to test against, before trusting the design further.
      **⛔ BLOCKED 2026-08-04 — traced, found a genuine, verified break, not implemented
      further.** Phase 5 is done (gate satisfied). Confirmed the `$`-vs-concatenation-precedence
      research task resolves cleanly — `{1, 2, 'lt}$` parses exactly as needed, no parser
      change (`brane_literal_dollar_reads_the_whole_literals_tail`,
      `foolish-parser/src/parser.rs`). But tracing the detachment/recoordination mechanism the
      design depends on, as explicitly instructed, found it does NOT work as assumed: an
      **unanchored** `IndexFir` (`#-1`/`#-2`, no dot — exactly the shape `'lt`'s operand lookups
      need) settles `Nyes::Nk` on an out-of-bounds target, not `ECONSTANIC` as the design states.
      `Nk` is TERMINAL — it can never later gain a value via recoordination the way `ECONSTANIC`
      can. Pinned by `fir_kinds::tests::unanchored_index_out_of_bounds_settles_nk_not_econstanic`.
      Reverted the `system/system.foo` extension (5 comparison creations) added during
      investigation — see the Phase 6 STOP note below and FOOP-33.md's Open Questions (dated
      2026-08-04) for the full writeup and the 3 candidate resolutions, none chosen unilaterally.
      Per the task's own explicit instruction ("STOP and report back — do not silently invent a
      workaround"), Phase 6 stops here pending a human decision.

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

- [x] **Research: `$` vs concatenation precedence — RESOLVED 2026-08-04, no parser change
      needed.** The checkbox's own key question used STALE syntax from the superseded
      postfix-concatenation prose (`{1,3}'lt$`); confirmed live that this form does NOT even
      parse as a comparison at all — `'lt` (leading `'`) does not trigger
      `is_concatenation_continuation` (which only recognizes `Token::Ident`/`LBrace`/`LParen`/
      `Up`/`LtLt`/`Lt`, not `Token::Apostrophe`), so `{1,3}'lt$` parses as TWO separate
      statements (`{1,3}` then an anonymous `'lt$`), not one expression. This is moot: the
      CURRENT, settled syntax (FOOP-33.md §5.0's evening revision) is `{1, 2, 'lt}$` — `'lt` as
      an ordinary comma-separated MEMBER inside a brane LITERAL, not postfix-concatenated after
      a closing `}`. Verified this parses exactly as intended, no parser change needed:
      `foolish-parser/src/parser.rs`'s `brane_literal_dollar_reads_the_whole_literals_tail` test
      confirms `{1, 2, 'lt}$` parses as `HeadTail{is_head:false, anchor:Brane{statements:[1, 2,
      'lt]}}` — `$` (tail) applied to the WHOLE brane literal, exactly as needed for `'lt`'s
      computed result to be read out.
- [x] **Delete** the token-level infix comparison operators — **already done** by the earlier
      revert; verified this session that `LTOp`/`GTOp`/`Le`/`Ge`/`EqOp` no longer appear in
      `foolish-parser/src/token.rs`, nor their parser matchers, lexer `\o` recognition,
      `OperatorFir::combine` arms, or sequencer `op_display()` rendering. Nothing left to
      delete. (2026-08-04)
- [x] **Update `system.foo`**: added `'lt`/`'gt`/`'le`/`'ge`/`'eq` as ordinary
      null-characterized creations alongside `'True`/`'False`. Confirmed the einmo suite is
      unchanged by this — system.foo gaining members cannot renumber statements inside
      `program`'s brane, since line numbers are per-brane indices. (2026-08-04)
- [x] **Comparison FIRs in `system_foo.rs`** — implemented, but NOT by evaluator
      special-casing as the superseded prose below describes. `'lt` is never recognised by
      name at the use site: `system.foo`'s `'lt = ⬤` placeholder bodies are replaced with a
      `ComparisonFir` as the composed root is compiled (a new `BodyOverride` hook in
      `compiler.rs`, so brane/statement construction stays in the compiler), and every use
      resolves by ordinary ancestral search + detachment/recoordination.

      **ONE FIR kind, not five.** `ComparisonFir` + a `ComparisonOp` enum: all five share the
      entire structure and differ only in which Rust comparison runs. Mirrors `OperatorFir`'s
      single-type-plus-op-tag shape, but with a real enum instead of its `op: String`, per
      `rust_instructions.md`'s "finite word-domains → enum".

      The operands are compiled from Foolish source (`<<#-2>>`/`<<#-1>>`) through `build_fir`
      rather than hand-assembled, so the `under_sff` rule applies exactly as it does to any
      other Foolish and cannot drift from it; they are pushed with
      `push_foolish_child_sff_marked`, whose panic-on-violation is the intended guard. A new
      `FirKind::Comparison` arm in `constanic_clone_at` is what makes recoordination work.
      (2026-08-04)
- [x] **Design finding, traced not assumed**: an ECONSTANIC operand must settle the comparison
      **ECONSTANIC, not NK**. NK is terminal and would poison the `'lt` *definition*, so a
      search for `'lt` would hit `check_body_nyes`'s `NkStop` and never hand the definition out
      to be recoordinated — the operator could never be used anywhere. Found by stepping the
      FVM (a plain `q = 'lt` settled `Nk` while `w = 'True` settled `Constant`). (2026-08-04)
- [x] Unit tests — 8 new in `system_foo.rs`, plus one in `fir_kinds.rs` pinning the underlying
      mechanism in pure Foolish (`sff_index_operand_recoordinates_to_the_referencing_branes_neighbors`).
      Cover: all five operators × both outcomes with `Rc::ptr_eq` against `system.foo`'s OWN
      `'True`/`'False` (referential identity, FOOP-33 §5); operand ORDER (`{1,2,'lt}` vs
      `{2,1,'lt}` must differ, so a swap cannot pass); two uses in one program each reading
      their own brane's neighbours; the non-integer NK case; and that a bare `{1,2,'lt}`
      without `$` is the brane, not the boolean. (2026-08-04)
- [x] `comparison_nyes_transitions` — required by AGENTS.md for a new FIR kind. Pins all THREE
      terminal states: ECONSTANIC inside `system.foo`, CONSTANT when recoordinated onto integer
      operands, NK on a non-integer operand. (2026-08-04)
- [x] Einmo tests: added `foop/33/boolean/comparison_operators.foo` (all five operators, both
      outcomes) and `foop/33/boolean/comparison_non_integer.foo` (the NK case). Every OUTPUT
      line justified against the arithmetic before promoting, per AGENTS.md; promoted with an
      explicit filter, exactly 2 files, no pre-existing baseline touched. The stale
      `int_comparators.foo`/`comprehensive.foo` items from the superseded infix design do not
      apply — those inputs were removed with the revert. (2026-08-04)
- [x] Run all tests — old and new — and make sure they all pass correctly. 295 unit tests pass
      (was 287). The einmo suite is in exactly its starting state: the single
      `foop/62/infinite_loop` divergence is the known, `verified/`-frozen one awaiting a human
      decision, deliberately untouched. (2026-08-04)

## Phase 7 — Documentation and Tests

- [ ] Document the null-characterized name-constant rule and universal characterizations
      (update `docs/vintage_legacy/CREATION.md` cross-refs and add engineering notes under
      `docs/ubc1/how`); update AGENTS.md §Foolish Terminology / §Searches as needed
      (with the "## Last Updated

**Date**: 2026-08-04
**Updated By**: Claude Code / claude-opus-5
**Changes**: **Phase 6 UNBLOCKED** — the earlier "blocked" finding tested a bare `#-1` rather
than the SFF-marked `<<#-1>>` the design specifies; the SFF form is built ECONSTANIC and never
runs, which is exactly what detachment/recoordination requires. Verified live and pinned by
`sff_marked_unanchored_index_out_of_bounds_settles_econstanic`. Status table's Phase 6 row and
the Phase 6 STOP gate updated accordingly; implementation still to be written (`system_foo.rs`'s
five comparison FIRs, extending `system/system.foo`). Also this session: Phase 5.5 punted to a
future FOOP; `foolish_children` encapsulation (`push_foolish_child`/`get_foolish_child`) with an
SFF-mark sanity guard; `sift_*` naming convention added to AGENTS.md terminology. Full history in
`git log` on this file.

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

## Phase 9 — Sequencer renders creation names (the `get_display_name` bridge)

**Depends on** the FVM-side `CreationFir::get_display_name()` (in progress separately). Largely
independent of Phase 6 (comparison operators) — different code path — so it can land in either
order. **May be punted past FOOP-33's merge** if it is not ready; nothing else depends on it,
and the fallback (rendering the bare creation glyph) is what ships today and is not wrong, just
less informative.

### The problem, precisely

The name lives in `foolish-ubca` (real `Rc` identity, real parent chain). The renderer lives in
`foolish-core` (`FirSequencer`), which only ever sees `foolish-core::Fir` — a **lossy**
conversion that discards identity and parent context. The whole bridge is one line today:

- `foolish-ubca/src/evaluator.rs` (`proto_to_core_fir_inner`): `FirKind::Creation =>
  core_fir::Fir::Creation`
- `foolish-core/src/sequencer.rs` (§"11. Creation"): returns the glyph unconditionally, with
  no name available
- `foolish-core/src/fir.rs`: `Fir::Creation` is a **unit variant**, serialized to/from JSON as
  `{"type": "Creation", "state": …}` (`to_json_val`, and `"Creation" => Ok(Fir::Creation)` on
  the way back)

### The design

Resolve the name at **conversion time** (where identity still exists), carry it as data, and
let the sequencer render what it is handed. `get_display_name()` already returns
`Option<String>` — `Some(name)` only when the creation is the entire RHS of a named statement,
`None` when it is a sub-expression — so the option maps directly onto "render the name" vs
"render the glyph". Worked cases:

- `{'a={*}; b='a}` → `Some("'a")` → sequencer renders `b='a`
- `{'a=1+{*}; b='a}` → `None` → sequencer renders the glyph (the creation is an operand of
  `+`, so the statement's name belongs to the whole expression, not to the creation)

- [ ] **Decide the `foolish-core::Fir::Creation` shape.** It must stop being a bare unit
      variant in order to carry a name. Options, to be chosen with the human before coding:
      (a) `Creation(Option<String>)`; (b) `Creation { name: Option<String> }`; (c) a separate
      `Fir::NamedCreation(String)` alongside the existing unit `Fir::Creation`. Weigh against:
      how many `match` arms break (there are ~8 `Fir::Creation` sites in
      `foolish-core/src/fir.rs` alone), and the JSON/`hssnap` stability consequence below.
- [ ] **Pin the JSON shape before touching any snapshot.** `Fir::Creation` currently
      round-trips as `{"type":"Creation","state":…}`. Adding a field changes serialized output
      for EVERY creation, named or not, unless the field is omitted when `None` (serde
      `skip_serializing_if`). Decide, and write the decision into FOOP-33.md — this is the
      "Creation *value* render form in `hssnap`" Open Question, which has been open since
      Phase 2 and should be **closed** by this phase.
- [ ] Thread the name at the conversion boundary: `proto_to_core_fir_inner`'s
      `FirKind::Creation` arm calls `get_display_name()` and stores the result on the built
      `core_fir::Fir`. Confirm that arm has the creation's own `FirRef` available
      (`get_display_name` needs it to walk `.parent`); if not, thread it through rather than
      reaching for a global.
- [ ] Sequencer: render `Some(name)` as the name, `None` as the glyph. Keep the fallback
      total — an unnamed creation must still render exactly as it does today.
- [ ] Unit tests (`foolish-core`): a named creation renders its name; an unnamed one renders
      the glyph; JSON round-trips both without loss.
- [ ] **Einmo baselines.** This is the first change in FOOP-33 that alters *rendered output*
      for existing tests. `foop/33/boolean/constants.foo` and
      `foop/33/boolean/null_char_constant.foo` both carry a comment predicting exactly this
      (`'True`/`'False` instead of the glyph) and saying they will need re-promoting when it
      lands — do that, and remove the now-stale comments. Justify every changed OUTPUT line
      per AGENTS.md's promote discipline. Check the whole suite for other creation-rendering
      baselines that shift.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

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

**Date**: 2026-08-04
**Updated By**: Claude Code / claude-opus-5
**Changes**: **Phase 5.5 CROSSED OUT** — its four checkboxes struck through with pointers to
Phase 9 (which now delivers the same capability, inside FOOP-33, at the conversion boundary
rather than sequencer-side); STATUS SUMMARY row rewritten from "punted to a future FOOP" to
"superseded by Phase 9, not implemented under this heading" — the separate future FOOP once
proposed for it is no longer the plan. **Fixed a real rendering bug** found by the human right
after Phase 6 merged: `ComparisonFir::resolve_boolean` (`system_foo.rs`) called `.value()`
directly on the STATEMENT `_ab_search` returns (`'True = ⬤`) instead of unwrapping to its body,
so `{1,2,'lt}$` settled to the whole `{'True=⬤}` statement wrapper instead of the bare creation
— visibly wrong (`'True={*}`-style output). Fixed by routing through
`statement_value_for_comparison`, the one documented "what does this statement resolve to"
accessor, exactly as `IndexFir`'s `$` search already does for its own result; verified via CLI
(`{r={1,2,'lt}$;}` now renders `r=⬤`, the correct pre-Phase-9 form) and re-promoted the
`comparison_operators.foo.einmo` baseline (all 10 lines, each individually justified — see
commit `8ac047e2`). Completed the Phase 6 branch merge into `foop-33-creation-postulate`
(resolved a Last-Updated-log conflict in this file and in `FOOP-33.md`, both touched
concurrently by the Phase 6 and `get_display_name` subagent branches; underlying Rust files
merged with no conflicts). Cleaned up both now-finished worktrees. Dispatched Phase 9
(`get_display_name()` sequencer bridge) to a subagent; two automatic `isolation: "worktree"`
spawn attempts landed on a stale pre-Rust commit (`origin/HEAD`/`origin/main` still point at an
abandoned Java/Maven-era commit, `4e0401ce`, left over from before this repo's trunk moved to
`jia` — a spawn-mechanism bug, not a task issue) and correctly self-aborted without guessing;
worked around by manually creating the Phase 9 worktree off the correct local branch tip and
directing the agent there directly. This log keeps only the single newest entry per the
Markdown File Update Protocol; full history in `git log` on this file.
