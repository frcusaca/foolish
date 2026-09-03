# FOOP-36.plan — foolish-rendering-sequencer

**Read `docs/foop/FOOP-36.md` before executing this plan.** The plan is derived from the
specification and assumes its context; section pointers below (`§1`, `§2.1`, `§3`…) refer to
that file.

**Worktree variables, expanded:**

```
WORKTREE_ORIGIN_BRANCH  = jia
WORKTREE_ORIGIN_PATH    = /yolo/foolish
WORKTREE_BRANCH_NAME    = foop-36-foolish-rendering-sequencer
WORKTREE_FULL_FS_PATH   = /yolo/foolish/../foolish_worktrees/foop-36-foolish-rendering-sequencer
```

**Scope guard, standing for the whole plan.** This FOOP adds a sequencer to `foolish-ubca2`
and rewrites `foolish-ubca2`'s einmo baselines. It **does not modify**:

- `foolish-core/src/sequencer.rs` — not one line (§6);
- `foolish-ubca/**` — anything at all;
- any `foolish-ubca/einmo_suite/**` baseline.

If a task appears to require touching one of those, **stop and report** — it means the design
has been misread or §FIR Impact Q2 has bitten. The only sanctioned exception is an **additive
default method** on `FirQueryable` (§FIR Impact), which is a `foolish-core/src/fir.rs` change,
must be reported before it is made, and must break no implementor.

---

## Broad structure — three movements

Read this before the phases. The plan is deliberately shaped so the hardest question — *can a
person write the expected output from the spec alone?* — is answered FIRST, on one
hand-authored case, before any baseline is generated and before the renderer exists.

| | Movement | Phases | What it establishes |
|---|---|---|---|
| **0.5** | **Vocabulary fix (fail-fast, skippable)** | 0.5 | Qualify every "settled" in `foolish-ubca2` with its NYES group (§0.1). Mechanical and behaviour-free. **Skip it the moment it stops being mechanical** — nothing later depends on it. |
| **I** | **New test, written by hand** | 3 | A brand-new `einmo_suite2/` holding ONE case whose expected OUTPUT is **typed from the specification before the renderer is written**. The renderer is then built until it reproduces what was typed. |
| **II** | **Feature completion** | 4 | The renderer is finished against that fixed target: round-trip properties (§2/§2.1) proven, every §3 row covered, `Detailed` delegation pinned. `einmo_suite/` is untouched and still green on the OLD rendering. |
| **III** | **Replacement** | 5–7 | `einmo_suite2` **becomes the suite**: the 179 inputs are copied across, rendered under the new sequencer, reviewed case by case, and `cargo test` is pointed at it. `einmo_suite/` is left frozen and still green as the reference to diff against — everything is done EXCEPT removing it. |

Why this order, and not the obvious one:

- **The hand-written case is the FOOP's own acceptance test.** If Movement I proves
  impractical, the FOOP has failed at its stated purpose (making einmo expectations writable)
  and it is far cheaper to discover that on one case than after 179 have been promoted.
- **The renderer is developed against a human-authored target, not its own output.** Building
  it against `einmo_suite/` first would mean reviewing 179 outputs the renderer itself produced
  — precisely the "does this match what the evaluator printed?" reasoning the Promotion Review
  Gate forbids as justification.
- **Green at every step.** `einmo_suite2/` is green from its first commit; `einmo_suite/` stays
  green on the old rendering until Movement III deliberately moves it. There is no long window
  where the tree is broken.

Phases 0–2 are preliminaries (safety checks, the two blocking design questions, and the
skeleton); Phase 8 is the merge.

**Who executes which phase** is specified in `FOOP-36.md` §"Plan of Execution for Plan":
model selection is **per-phase, not per-FOOP**. Phases 1, 3b–3c, 6 and 7 carry the judgment
(a larger model — Opus/Sonnet on Claude); phases 2, 3d, 4, 5 and 8 are execution against a
fixed target (a smaller model — Sonnata, GPT-terra, or Qwen3.8-27B). That section also lists
the four things that **must not be delegated** regardless of model size.

### How to work this plan

1. **Read `docs/foop/FOOP-36.md` first.** The plan says what to do; the spec says why, and its
   §-numbers are cited throughout. You need §0 (terminology), §3 (the rendering rule), and §4
   before writing any code.
2. **Work top to bottom.** Phases are ordered by dependency. Do not start a phase whose
   predecessor's test-gate checkbox is unchecked.
3. **Check each box as you finish it, with a timestamp on the next indented line** —
   `(YYYY-MM-DD HH:MM)`. The boxes are the record of what is done.
4. **When a checkbox says STOP, stop.** Those mark decisions that are the human's, not yours.
   Report what you found and wait.
5. **Accumulate doubts; report them once, at the end of a phase**, rather than interrupting per
   item (AGENTS.md §"The agent is responsible for correctness").
6. **A failing test is broken code, not a stale baseline.** The one exception is Phase 5, where
   `einmo_gate_checked` failing is the intended effect and is stated as such.
7. **Never `einmo promote` outside the Phase 6 and 7 gates**, and never over another FOOP's
   diverged baseline.

### Orientation — the facts you need, so you need not go find them

*This plan is written to be executable by an agent with a modest context budget. The
code facts below were established while the FOOP was written; they are current as of
2026-09-02. **Verify before relying on any of them, but do not re-derive them from scratch.***

**Files you will touch** (all paths from the repo root):

| Path | Role | Size |
|---|---|---|
| `foolish-ubca2/src/sequencer.rs` | **you create this** — the new renderer | — |
| `foolish-ubca2/src/lib.rs` | register the module | 36 lines |
| `foolish-ubca2/src/ubca_snapshot_tester.rs` | the einmo adapter; ONE call site changes in Phase 5 | 236 lines |
| `foolish-ubca2/einmo_suite2/` | **you create this** — Movement I's hand-written suite | — |
| `foolish-core/src/sequencer.rs` | **READ ONLY** — `Detailed` delegates to it; never edit | 814 lines |
| `foolish-core/src/fir.rs` | **READ ONLY** — `FirQueryable` accessors, `Nyes` | 2663 lines |

**The trait you render from** — `FirQueryable` in `foolish-core/src/fir.rs` (~line 570).
Every accessor is `hs_*` and returns `Option`, so rendering is a dispatch chain:
`hs_constant_int`, `hs_nk`, `hs_operator`, `hs_search`, `hs_index`, `hs_stay_foolish`,
`hs_stay_fully_foolish`, `hs_concatenation`, `hs_brane`, `hs_creation` / `hs_creation_name`,
`hs_alarm`, `hs_is_tail_concatenation`, plus `hs_state() -> Nyes` and `hs_variant()`.
`foolish-core/src/sequencer.rs`'s `render_fir` is the worked example of that dispatch — read
it once; your `Foolish` mode is the same dispatch with different bodies.

**`hs_search` returns a 7-tuple** (`SearchQuery`, `fir.rs` ~749):
`(pattern, direction, anchored, anchor, result, is_value, value)`. For §3 you use
`pattern` + `direction` + `anchored` + `anchor` and **ignore `result`** — that is the
evaluator's conclusion. `hs_index` returns `(offset, anchored, anchor, result)`; same rule.

**`Nyes`** (`fir.rs` ~115) has 8 variants: `Prembrionic`, `Embryonic`, `Braning` (pre-constanic);
`Econstanic`, `Woconstanic`, `Constant`, `Independent`, `Nk` (constanic). Helpers:
`is_constanic()`, `is_nye()`, `should_show_nyes()`. Note the spelling **`Prembrionic`** in code
vs `PREMBRYONIC` in prose.

**Facts already verified — do not spend context re-checking:**

- `_` in an identifier is lexed to **U+02CD `ˍ`** (`foolish-parser/src/lexer.rs` `SEP`, line 11;
  pushed at line 454) and `ˍ` is accepted as an identifier char on input (`is_id_sep`, line 100).
  So `myˍvar` round-trips. This is why output shows `nonˍexistent`.
- `!!` line comments and `!!!` block comments are lexed and discarded
  (`lexer.rs` lines 121–124, 327, 361). This is what makes §4's annotations free.
- Search patterns are stored **regex-wrapped**: `?x` becomes `pattern='^x$'`. Rendering the
  written form means un-wrapping `^…$`. Confirm the un-wrapping is total (Phase 1, Q2).
- `hs_search` exposes `anchor` and `result` as **separate slots**, so a resolved search still
  carries its written form (Phase 1, Q5).
- `foolish-ubca2/src/fvm_storage.rs` (~3368) has `proto_to_core_fir`, which dispatches into a
  separate `proto_to_core_fir_sff_body` (~3378) for SFF interiors — that path rebuilds searches
  as `SearchFir`s carrying pattern + anchoring (Phase 1, Q5).
- `foolish-core/src/sequencer.rs` has **4 pre-existing clippy warnings** (lines 187, 537, 563,
  743). Not yours to fix; the scope guard forbids touching that file.
- **Terminology — the authority is AGENTS.md §Foolish Terminology**, restated in `FOOP-36.md`
  §0. Read one of them before writing rendering code.
  - **Constanic**: any terminal NYES — ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, NK.
    Pre-constanic (nigh): PREMBRYONIC, EMBRYONIC, BRANING.
  - **Constantew** (CONSTANT EveryWhere): CONSTANT, INDEPENDENT, NK — won't change no matter
    what. Constantew ⊂ constanic.
  - **Conclusive** (**Conc**): NYES is CONSTANT or INDEPENDENT — it reached a value.
    **Inconclusive** is everything else, INCLUDING pre-constanic states. The phrase
    **"inconclusive constanic"** narrows to the terminal ones: WOCONSTANIC, ECONSTANIC, NK —
    and that narrower phrase is what §3's rule is stated over.
  - Conclusive and constantew are different cuts, differing exactly on **NK**: constantew, yet
    inconclusive. **Rendering keys on conclusive.**
  - No predicate for conclusive exists in the code (`fir.rs` has `is_constanic()` and
    `is_nnk_constanic()` only). If you add one, keep it in `foolish-ubca2` — `foolish-core`
    is off-limits per the scope guard.
  - **"Settled" is prose, not a predicate.** `foolish-ubca2` uses the word heavily (138
    occurrences, plus a real `FirPointer::settled_result` accessor) but **`is_settled()` does
    not exist** — `lib.rs` line 24 documents it and FOOP-62 §Terminology specifies it; neither
    was implemented. Do not call it. Use `is_constanic()`, or the conclusive predicate you add.
  - [ ] (Phase 0.5 fixes `lib.rs` line 24's stale `is_settled()` claim; if that phase was
        cancelled, fix the comment here instead.)
- **The separator is `①` (U+2460) for ubca2, NOT `!!` (§4.2).** Verified from the artifacts:
  `foolish-ubca2/einmo_suite`'s files carry `#einmo 1 encoding=utf-8 separator=①\n`, while the
  older `foolish-ubca/einmo_suite` still carries `separator=!!\n`. **FOOP-92's spec text and
  `einmo_suite/einmo.toml`'s comment both say `!!` and are STALE** — the toml claims the
  separator is "set in code via `TestConfig::foolish_separator()`", but
  `ubca_snapshot_tester.rs` calls plain `TestConfig::new(...)`, so einmo's `①` default applies.
  `einmo/src/format.rs::serialize` substring-checks each section body and returns
  `EinmoError::SeparatorCollision` — a hard error at write time. For ubca2 suites: **no line
  may be or end with exactly `①`**; `!!` is unrestricted there and is just a Foolish comment.
- **Line width.** `foolish-core/src/sequencer.rs` line 14 declares `const LINE_BUDGET: usize =
  128`. Your `Foolish` mode uses **108** (§4.1). It is the single-vs-multi-line threshold,
  threaded down as the `line_hint` parameter and reduced by indent at each nesting level — the
  existing machinery is correct, only the constant differs. It is **soft**: measured across the
  current 5,435 output lines, 20 exceed 108 and 3 exceed even 128. Do NOT add a corpus-wide
  width assertion (§4.1 says why).

**Commands** (full forms; `README.md` §"Running specific tests" is the central reference):

```bash
cargo test -p foolish-ubca2 --lib                          # ubca2 unit tests (134 at Phase 0)
cargo test -p foolish-ubca2 --lib -- einmo_gate_checked    # ubca2 einmo gate
cargo test -p foolish-ubca  --lib -- einmo_gate_checked    # sibling — must never move
cargo test -p foolish-ubca2 --lib -- sequencer             # your new tests
cargo fmt --all
cargo clippy -p foolish-ubca2 --all-targets -- -D warnings # scope to ubca2, NOT the workspace
```

**The three einmo gates must not run concurrently** — they share `output/`. The existing tests
serialize on a `static GATE_LOCK: Mutex<()>`; see the module docs at the top of
`ubca_snapshot_tester.rs`, which explain the "catastrophe crumb" failure you get without it.

---

## Phase 0 — Begin

- [ ] Begin work: commit `FOOP-36.md` and `FOOP-36.plan.md` to origin, check `begun: [x]` in
      `FOOP-36.md` frontmatter
- [x] Sequencing against FOOP-26 (§Q4) — **DECIDED by the human 2026-09-02: FOOP-36 goes
      first.** No need to re-ask. If FOOP-26 has nonetheless begun in a worktree, say so and
      pause rather than racing it.
      (2026-09-02 10:15)
- [ ] Confirm the tree is green BEFORE any change (AGENTS.md: never start Phase+ work with
      broken tests). Record the result in this plan.
  - [ ] `cargo test -p foolish-ubca2 --lib` — record pass count (expected 134/134)
  - [ ] `cargo test -p foolish-ubca --lib -- einmo_gate_checked` — record result; this is the
        **T5 "before" reading** the final phase compares against
  - [ ] **`einmo_gate_verified` on `foolish-ubca2` PASSES today** — `verified/` holds all 179
        human-signed artifacts (measured 2026-09-02; whole crate 134/134). FOOP-16 and
        `ubca_snapshot_tester.rs`'s doc comment both claim `verified/` is empty and the gate is
        expected to fail: **that is STALE, do not trust it.** Re-measure and record the actual
        result, since everything downstream depends on it.
  - [x] §Q6 (`verified/` is populated — all 179 cases have a frozen twin) — **DECIDED by the
        human 2026-09-02: option (a).** The agent reviews and promotes `output` → `checked`
        case by case as normal; **the human then mass-verifies `checked` → `verified` in one
        pass afterwards.** So `einmo_gate_verified` IS EXPECTED TO BE RED from Movement III
        until that re-attestation — that is accepted, not a defect to chase. **Never
        `#[ignore]` it** (AGENTS.md). And note the human's mass-verify presumes a real per-case
        review has already happened; it does not replace one.
        (2026-09-02 10:15)
  - [ ] Also fix `ubca_snapshot_tester.rs`'s stale `einmo_gate_verified` doc comment (it says
        `verified/` "is still empty here" and the failure "is intentional"). Comment only.
- [ ] Create worktree at
      `/yolo/foolish/../foolish_worktrees/foop-36-foolish-rendering-sequencer` with branch
      `foop-36-foolish-rendering-sequencer`:
      `git worktree add -b foop-36-foolish-rendering-sequencer /yolo/foolish/../foolish_worktrees/foop-36-foolish-rendering-sequencer`
- [ ] **All work from here happens in the worktree** — including edits to `docs/foop/`
      (`foop.md` §worktree discipline).

---

## Phase 0.5 — Qualify every "settled" with its NYES group (FAIL-EARLY, may be cancelled)

*A small, self-contained, mechanical change placed first so that if it goes badly it is cheap
to abandon. It touches no behaviour: every edit is a rename or a comment. Its purpose is to
make the codebase say which NYES group it means, so that everything after it — and every agent
reading `foolish-ubca2` afterwards — is working in unambiguous vocabulary.*

> **FAIL FAST. SKIP IF CUMBERSOME.** This phase is a convenience, not a prerequisite: nothing
> later in this FOOP depends on it, because §3's rule is stated in §0's vocabulary regardless
> of what the code calls things. **Abandon it the moment it stops being mechanical** — you do
> not need permission and you do not need to finish what you started. Concretely, stop and skip
> if: a rename reaches outside `foolish-ubca2/src/`; the crate does not stay 134/134 after a
> pure rename; a name turns out to be ambiguous enough to need thought rather than lookup; or
> FOOP-26/FOOP-46 have already touched the same lines. To skip: `git checkout` the phase's
> changes, mark it `[x] Canceled` per `foop.md`, note why in one line, and go to Phase 1.
> **A skipped Phase 0.5 costs this FOOP nothing.** Grinding through a cumbersome one costs it
> a merge conflict.

**Why this is fail-early rather than deferred.** `foolish-ubca2` is being edited concurrently by
FOOP-26 and FOOP-46. A rename touching ~20 call sites either lands before they diverge or
becomes a merge problem. Doing it first, verifying, and then deciding is cheaper than either
carrying the ambiguity through this whole FOOP or discovering the conflict at merge time.

**The classification to apply is `FOOP-36.md` §0.1** — read it before starting. Every "settled"
in `foolish-ubca2` means exactly one of: constanic, conclusive, constantew, or a computed
classification that may still be pre-constanic. The qualifier states which.

- [ ] (read `FOOP-36.md` §0, §0.1 and §0.1.1)
- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run: `cargo test -p foolish-ubca2 --lib` (the whole crate — this is a rename, so
      EVERYTHING must stay green, all 134). Record the before-count.
- [ ] **Give all four §0 groups a predicate on `NyesExt`** (`foolish-ubca2/src/nyes_ext.rs`),
      beside the existing `is_constanic()` and `is_constantew()` — §0.1.2. Conclusive has none
      today, which is why five sites hand-roll the state list; pre-constanic has one only in
      `foolish-core`, under the older name, with zero callers.
  - [ ] ```rust
        /// Pre-constanic (nigh): PREMBRYONIC, EMBRYONIC, BRANING — still stepping.
        fn is_preconstanic(&self) -> bool {
            !self.is_constanic()
        }

        /// Not Yet Evaluated — the older name for the same group. An alias, kept
        /// so the traditional Foolish vocabulary still reads.
        fn is_nye(&self) -> bool {
            self.is_preconstanic()
        }

        /// Conclusive: the FIR reached a value — CONSTANT or INDEPENDENT (FOOP-36 §0).
        /// Distinct from `is_constantew()`, which also admits NK: NK is constant
        /// everywhere yet never produced a value.
        fn is_conclusive(&self) -> bool {
            matches!(self, Nyes::Constant | Nyes::Independent)
        }
        ```
        **`is_preconstanic` is primary and `is_nye` delegates to it**, so the four read
        uniformly while the traditional name keeps working.
  - [ ] This does not touch `foolish-core`. `foolish_core::Nyes::is_nye()` exists at
        `fir.rs:143` with **zero callers workspace-wide** (verify this still holds), so the
        `NyesExt` method shadows nothing in practice. **Do not edit `foolish-core`** — the
        scope guard forbids it.
  - [ ] Unit tests in that module's `tests`, in the same shape as the existing
        `constantew_states()` — assert over `ALL_NYES`:
    - [ ] `conclusive_states()` and `preconstanic_states()`
    - [ ] `is_nye_is_alias_for_preconstanic()` — the two agree on every state
    - [ ] `conclusive_is_subset_of_constantew()`, mirroring
          `constantew_is_subset_of_constanic()`
    - [ ] `conclusive_and_constantew_differ_exactly_on_nk()` — §0's load-bearing
          distinction, worth pinning explicitly
    - [ ] `preconstanic_is_complement_of_constanic()` — every state is exactly one
  - [ ] Update the module doc comment at the top of `nyes_ext.rs`: it lists "Three categories"
        and omits conclusive entirely.
- [ ] **Replace the five hand-rolled conclusive matches** with `.is_conclusive()`:
      `fvm_storage.rs` lines **818, 2007, 3739, 3810, 3950** (each is
      `matches!(…, Nyes::Constant | Nyes::Independent)`). Verify each really is a conclusive
      test and not an accident of the same two states appearing together — read it, do not
      sed it blindly. Line 1375 (`nyes_from_found`'s match arm) is a mapping, not a test;
      leave it.
- [ ] **Code renames** in `foolish-ubca2/src/fvm_storage.rs`, mechanical and behaviour-free:
  - [ ] `FirPointer::settled_result` → `settled_constanic_result` (line ~639). It gates on
        `is_constanic()`, and §0.1.1 confirms that is genuinely what the slot holds — do NOT
        use `constantew`, which the StayFoolish path (902–904) would falsify.
  - [ ] `FirCursor::settled_result` → `settled_constanic_result` (line ~1602)
  - [ ] `step_to_settled` → `step_to_constanic` (line ~3272, plus its re-export at ~4825 and
        its caller in `evaluator.rs`)
  - [ ] `all_settled` → `all_foolish_children_conclusive` (line ~816). **This is the one that carries
        real information**: it gates on `Constant | Independent`, so an ECONSTANIC operand is
        constanic but still queued as a task. It is a local `bool`, not a function.
  - [ ] `let settled = …decide_nyes_due_to_children(…)` (line ~1070) → `let decided_nyes = …`.
        It can hold **Braning**, so calling it "settled" is actively wrong.
  - [ ] `settled_nyes = nyes_from_found(…)` (line ~968) → `constanic_nyes` (its output is
        always constanic; its input need not be).
  - [ ] `anchor_settled` (line ~3178) → `anchor_constanic`
  - [ ] Test names: `indep_int_stepping_already_settled_is_noop` → `…_already_conclusive_…`;
        `operator_pushes_tasks_for_unsettled_operands` → `…_for_inconclusive_operands`;
        `revive_constanic_unwraps_stay_foolish_to_its_settled_result` → `…_settled_constanic_result`.
- [ ] **Comment fixes**, no code change:
  - [ ] `lib.rs` line 24 claims `NyesExt` adds `is_settled()`. **It does not exist** — the trait
        provides `is_constanic()` and `is_nnk_constanic()` only. Correct the line.
  - [ ] Doc comments in `fvm_storage.rs` using a bare "settled": qualify each with the group
        §0.1 assigns it. Where a comment is already unambiguous from context, leave it.
- [ ] **Verify.** Run `cargo test -p foolish-ubca2 --lib` — must be 134/134, the same as the
      before-count. A rename changes no behaviour; **any** test movement means something else
      was changed and must be undone.
- [ ] `cargo fmt --all`; `cargo clippy -p foolish-ubca2 --all-targets -- -D warnings` clean.
- [ ] Confirm `foolish-core` and `foolish-ubca` are untouched: `git diff --stat` shows changes
      only under `foolish-ubca2/src/`.
- [ ] **DECISION POINT — move on, or cancel.** (If you already skipped per the fail-fast rule
      above, this is where you record that.) Report to the human:
  - [ ] the diff size (files, lines, call sites touched);
  - [ ] whether the crate stayed 134/134;
  - [ ] any site where §0.1's classification turned out to be wrong when read closely —
        **that is a finding worth more than the rename**, because it means the survey
        mis-read the code;
  - [ ] whether FOOP-26 or FOOP-46 have begun touching the same lines.
  Then either **move on** (the renames stay) or **cancel** (`git checkout` the changes, mark
  this phase `[x] Canceled` per `foop.md`, proceed to Phase 1 unchanged). Cancelling is a
  legitimate outcome, not a failure to report apologetically.

---

## Phase 1 — Resolve Q2 before writing any rendering code

*This phase writes no rendering code. It answers the one remaining open question that could
change the FOOP's blast radius, BEFORE any code depends on the answer (§Open Questions Q2).
Q5 was dissolved by the human; the confirmation left below is cheap and worth doing.*

- [ ] (read §3 and §FIR Impact of `FOOP-36.md`)
- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run unit tests: `foolish-core::sequencer_tests`, `foolish-ubca2::identifier`. No einmo
      cases yet — this phase adds no rendering.
- [ ] For each §3 table row, inspect the `FirQueryable` accessors in
      `foolish-core/src/fir.rs` (`hs_search`, `hs_operator`, `hs_index`, `hs_concatenation`,
      `hs_stay_foolish`, `hs_stay_fully_foolish`, `hs_brane`, `hs_creation_name`) and record in
      this plan, one line per row: **can the written form be reconstructed from what is
      exposed?**
  - [ ] Operator written form — glyph + operands from `hs_operator`
  - [ ] Search written form — `?` / `~` / `.` / `&`-forms from `hs_search`'s pattern +
        direction + anchored triple. **Note the known hazard:** the pattern is stored
        regex-wrapped (`'^a$'`), so the written name must be recovered from it — confirm the
        unwrapping is total and unambiguous, or report it as needing an accessor.
  - [ ] Index written form — `#N` / `^` / `$` from `hs_index`'s offset + anchored
  - [ ] Concatenation written form — juxtaposition from `hs_concatenation`'s elements
  - [ ] SF / SFF written form — `<`/`>`, `<<`/`>>` + interior
- [x] **Q5 — DISSOLVED by the human 2026-09-02: "constants should always be rendered in
      Foolish."** A settled search IS its value, so `result = {y = 1;}?y` rendering `result = 1`
      is correct and complete — the search disappearing is the evaluator succeeding, not
      information lost. The FIR already separates the two cases structurally (resolved →
      CONSTANT, no `SearchFir` left; unresolved → the `SearchFir` survives), so **no provenance
      marking, no new accessor, nothing to decide.** §3 was rewritten accordingly.
      (2026-09-02 14:05)
- [ ] **Confirm §0.1.1 — which NYES states a `settled_result` slot actually holds.** The gate
      (`fvm_storage.rs:639`) tests `is_constanic()` on the owner, but two mechanisms narrow
      what lands in the slot: `Nyes::transform_for_clone` preserves only CONSTANT/INDEPENDENT/NK
      (= **constantew**) and turns everything else EMBRYONIC; and `push_ubc_child` (line 151)
      queues a non-constanic child as a task so it gets stepped. Instrument a run over the
      corpus and record the observed distribution of result NYES. **This tells you which arms
      of §3's predicate the corpus exercises** — if ECONSTANIC/WOCONSTANIC results turn out to
      be rare or absent, say so, because the `einmo_suite2` cases must then cover them
      deliberately rather than incidentally.
  - [ ] Confirm §3's dispatch on real FIRs — cheap, and the basis of everything downstream:
        evaluate `misc/search_with_multiple_matches` (`r = b?a.*`, anchored) and
        `misc/undeclared_identifier` (`x = non_existent`, unanchored) and record what reaches
        the sequencer. Under §3 the first renders `r = b?a.*` (anchor, then search) and the
        second `x = nonˍexistent` (search alone) — both are constanic searches and both revert.
- [ ] **Q7 — does a trailing use site render its value, or revert to a search?** Evaluate
      `misc/sff_resolves_on_each_use` (`{a=1; b=2; s=<<a+b>>; a=10; s;}`) and inspect the FIR
      at the trailing `s;`. If it is a `SearchFir` for `s`, §3 says it renders `s`; if it is
      the constant `12`, it renders `12`. The committed baseline shows `12` but was produced by
      the OLD renderer, which collapsed searches to values regardless — so it does not settle
      the question. **Record the answer in the plan**: it fixes how a whole family of
      trailing-use-site lines renders across the corpus, and it is the single largest
      determinant of what the 179 migrated baselines will look like. Neither answer is a
      problem; guessing is.
- [ ] **Decision point.** Q5 is already resolved (above). If every §3 written form
      reconstructs from existing accessors, record "Q2 resolved: no accessor needed" and
      proceed. If any does NOT: **stop and report to the human** the specific row, what is
      missing, and the proposed additive `FirQueryable` default method — per the scope guard
      that is the one sanctioned `foolish-core` change and not an agent's call to make. If any row does NOT: **stop and report to the
      human** the specific row, what is missing, and the proposed additive `FirQueryable`
      default method (returning `Option<…>`, defaulting to `None`) — per the scope guard, this
      is the one sanctioned `foolish-core` change and it is not an agent's call to make
      silently.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 2 — The sequencer skeleton and `Detailed` mode

*Smallest thing that compiles and proves the delegation contract. No `Foolish` rendering yet.*

- [ ] (read §1 and §6 of `FOOP-36.md`)
- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run unit tests: `foolish-ubca2::sequencer`. Run this subset frequently while
      implementing; add new tests to this list as they are written.
- [ ] Create `foolish-ubca2/src/sequencer.rs` with `SequenceMode` (`Foolish` default,
      `Detailed`) and `Ubca2Sequencer::format(&dyn FirQueryable, SequenceMode) -> String`,
      exactly as §1 gives the signature. The whole of Phase 2 is this much code:

      ```rust
      use foolish_core::fir::FirQueryable;

      /// Max line width (AGENTS.md §Code Style: 108-char documents).
      /// The single-vs-multi-line threshold, reduced by indent at each
      /// nesting level. A target, not a guarantee — see FOOP-36 §4.1.
      const LINE_BUDGET: usize = 108;

      #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
      pub enum SequenceMode {
          #[default]
          Foolish,
          Detailed,
      }

      pub struct Ubca2Sequencer;

      impl Ubca2Sequencer {
          pub fn format(fir: &dyn FirQueryable, mode: SequenceMode) -> String {
              match mode {
                  // Phase 2: Foolish temporarily delegates too, so the crate
                  // compiles and every baseline still passes. Phase 3 replaces
                  // this arm with the real renderer.
                  SequenceMode::Foolish => Self::format_detailed(fir),
                  SequenceMode::Detailed => Self::format_detailed(fir),
              }
          }

          fn format_detailed(fir: &dyn FirQueryable) -> String {
              foolish_core::FirSequencer::format(&foolish_core::clone_steppable(fir))
          }
      }
      ```

      Note `clone_steppable` — `FirSequencer::format` takes `&Fir`, not `&dyn FirQueryable`;
      `ubca_snapshot_tester.rs` line ~92 shows the same conversion in use.
- [ ] Register the module in `foolish-ubca2/src/lib.rs` (`pub mod sequencer;` plus the
      `pub use`), matching the existing module-doc style of that file.
- [ ] Implement `SequenceMode::Detailed` as **pure delegation** to
      `foolish_core::FirSequencer::format`. Not a reimplementation.
- [ ] Implement `SequenceMode::Foolish` as a temporary delegation to `Detailed`, so the crate
      compiles and every existing baseline still passes while Phase 3 fills it in.
- [ ] **T1 (delegation half)** — unit tests asserting `format(fir, Detailed)` is byte-equal to
      `foolish_core::FirSequencer::format(fir)` for at least: an int, a brane, an operator, a
      resolved search, an NK. This pins §1's contract so it cannot silently drift.
- [ ] Run all tests — old and new — and make sure they all pass correctly. **No einmo baseline
      may move in this phase** — if one does, `Detailed` is not pure delegation.

---

## Phase 3 — `einmo_suite2`: hand-write the expectations FIRST

*The bootstrap that makes this FOOP honest. A brand-new suite,
`foolish-ubca2/einmo_suite2/`, containing ONE einmo case for FOOP-36. Its expected OUTPUT is
**typed out by hand from the specification, before the renderer exists** — not generated and
reviewed. If the spec cannot be read and turned into expected output by a person, the FOOP has
failed at its stated purpose, and this phase is where that becomes visible instead of at
Phase 6 with 179 baselines already promoted.*

**Why a separate suite rather than a case in `einmo_suite/`.** The existing suite's 179
baselines are all in the OLD rendering until Phase 6. Mixing a new-rendering case into it means
the gate cannot pass until everything is converted — no incremental green. `einmo_suite2` is
green from its first commit and stays green, and it is where the renderer is developed against
a fixed, human-authored target.

### 3a — Create the suite

- [ ] (read §2, §2.1, §3, §3.1, §4, §5 of `FOOP-36.md` — ALL of them; you are about to write
      their expected output by hand)
- [ ] Create `foolish-ubca2/einmo_suite2/` with `input/`, `output/`, `checked/`, `verified/`,
      mirroring `einmo_suite/`'s layout.
- [ ] Copy `foolish-ubca2/einmo_suite/einmo.toml` and adapt the header comment. Keep
      `[signing.output] passphrase = ""`. Use a DISTINCT `[signing.checked]` passphrase
      (e.g. `foolish-ubca2-suite2`) so the two suites' stamps cannot be confused.
      Leave `verified` unconfigured, exactly as `einmo_suite/` does.
- [ ] **Separator: use `①` + LF** (U+2460), the same two-character sequence as
      `foolish-ubca2/einmo_suite`, so both ubca2 suites are configured alike and a case can move
      between them unchanged. That is einmo's default, so `TestConfig::new(...)` gives it —
      do **not** call `foolish_separator()`.
- [x] Fix the stale comment in `foolish-ubca2/einmo_suite/einmo.toml`: it claimed the suite
      used "the Foolish line-comment separator … set in code via
      `TestConfig::foolish_separator()`", which is not what happens — the tester calls plain
      `TestConfig::new(...)` and the artifacts carry `separator=①\n`. Comment corrected to
      describe the `①` default and to warn that `foolish-ubca` differs. Separator itself
      untouched; `einmo_gate_checked` re-run and passing.
      (2026-09-02 09:58)
- [ ] Add a `README.md` in `einmo_suite2/` stating what the suite is for: **the hand-authored
      rendering contract for FOOP-36**, one case, expectations written before the code.
- [x] **Einmo authoring instructions (§4.2)** — DONE ahead of the worktree, on `jia`, because
      these rules govern every Foolish einmo input and not just this FOOP's. Left here as the
      record of what was done and where.
      (2026-09-02 10:04)
  - [x] The **separator / per-suite `einmo.toml`** half is already done: AGENTS.md gained
        §"READ THE SUITE'S `einmo.toml` FIRST" under Approval Tests (einmo), and
        `foolish-ubca2/einmo_suite/einmo.toml`'s stale comment was corrected.
        (2026-09-02 09:58)
  - [x] The **comment-style** half is done too: AGENTS.md gained §"Comment style in Foolish
        einmo inputs" beside the toml rule (inline comments permitted; `!!!` fences
        blank-line-separated on BOTH sides; full-line `!!` comments blank-line-before and
        none-after, marking the code below them).
        Deliberately **not** in `einmo.README.md` — that documents einmo the language-agnostic
        tool, and Foolish `!!` conventions do not belong there.
        (2026-09-02 10:02)
  - [x] The **separator collision** rule is stated in that AGENTS.md section: content must
        never contain the configured separator (`einmo/src/format.rs::serialize` substring-
        matches and hard-errors); the separator differs per suite (`①`+LF for ubca2, `!!`+LF
        for `foolish-ubca`); a real artifact's header line beats toml comments and FOOP prose.
        (2026-09-02 09:58)
  - [x] Markdown File Update Protocol followed: AGENTS.md's "## Last Updated" entry replaced
        (not appended).
        (2026-09-02 10:02)
  - [x] Checked `foolish-ubca2/einmo_suite/MAPPING.md` and `foop.md` for competing input-
        authoring guidance: neither mentions comment style or the separator, so there is
        nothing to point at AGENTS.md and nothing to de-duplicate.
        (2026-09-02 10:04)

### 3b — Write the input: one giant brane, one sub-brane per case group

- [ ] Write `foolish-ubca2/einmo_suite2/input/foop/36/rendering_contract.foo` as ONE top-level
      brane whose members are named sub-branes, one per group below. Use Unicode operator forms
      (`⬤`, `<̲`, `=̲=̲`) per AGENTS.md, and `!!` comments to say what each group asserts.
- [ ] **Apply §4.2's comment style throughout this file** — it is the first input written under
      the new rules and becomes the worked example others copy:
  - [ ] Each sub-brane group gets a `!!!` fenced heading, blank line before AND after
  - [ ] Within a group, a full-line `!!` comment marks the cases below it: blank line before,
        NO blank line after
  - [ ] Per-case remarks are short inline comments trailing the statement
  - [ ] **No line contains `①`** (U+2460) — the suite's separator; einmo refuses to serialize
        a section containing it. `!!` is unrestricted here.
- [ ] Every group below gets a sub-brane. **Write the case, and write what you expect it to
      render as, in a `!!` comment beside it** — the comment is the prediction, the einmo
      OUTPUT is the check.

  - [ ] **`leaves`** — the trivially-stable renderings (§3):
        integer `7`; negative integer; a bare creation `⬤`; a named creation `'True`
        (FOOP-33 original name); an empty brane `{}`; a brane of constants.
  - [ ] **`names`** — statement and identifier rendering (§3):
        plain `a = 1`; an underscore name `my_var = 2` (must render `myˍvar`, U+02CD, and
        re-lex — the round-trip hazard); a Unicode name (Greek/Cyrillic/Chinese per AGENTS.md);
        a characterized brane `a'b'{…}`; a null-characterized name `'k = 1`.
  - [ ] **`operators_written`** — the same predicate on operators: **when the result is an
        inconclusive constanic, render the op operating on its parameters**; when it is
        conclusive, render the VALUE. Cover both sides:
        - `3 + 4` → `7` (result conclusive — the operator is spent)
        - `a + b` with ECONSTANIC operands → `a + b` (no value was reached)
        - `1/0` → `1/0` with `!! NK: …` (result NK — §5's operator instance)
        Plus: a nested `1 + 2 * 3` (precedence must survive the round trip); unary minus;
        a comparison using `<̲`.
  - [ ] **`searches_written`** — §3's predicate, which keys on the search's **`result()`**, not
        on the search's own NYES: **when the result is an inconclusive constanic (§0), render
        the original search**; when it is conclusive, render its value. Cover both sides:
        - result **conclusive** (CONSTANT/INDEPENDENT) → collapses to the value
        - **Add the test line 816's rule lacks**: an operator with an ECONSTANIC operand must
          still queue it as a task. The existing
          `operator_pushes_tasks_for_unsettled_operands` uses PREMBRYONIC operands only, so it
          does not distinguish conclusive from constanic — and `all_settled` at
          `fvm_storage.rs:816` gates on `Constant | Independent`, i.e. conclusive (§0.1
          Group 2). That distinction is currently untested.
        - result ECONSTANIC (unanchored miss) → renders the search
        - result WOCONSTANIC → renders the search
        - result NK (anchored miss) → renders the search, NOT `???`
        - result absent → renders the search
        And both anchoring shapes: unanchored → the search alone (`?x`, `nonexistent`);
        anchored → anchor then search (`b?a.*`, `a.field`), the anchor rendered by these same
        rules. Include the regex-unwrap case (stored `'^a$'` must render `a`).
  - [ ] **`indexes_written`** — `#-1`; `#0`; `^`; `$`; the attached form `A =$ B` (FOOP-75 §4).
  - [ ] **`sf_sff`** — §3 + §3.1 together:
        `<x>` and `<<x>>` as NAMED statements (render written forms, delimiters kept);
        an SFF with an operator interior `<<a + b>>`; a nested `<<a + <<b>>>>`.
  - [ ] **`substitution`** — **§3.1's load-bearing group; the reason this FOOP has a §3.1.**
        The exact case `{x = 1; sf = <x>; sff = <<x>>; x = 10; sf; sff;}` — the named
        statements must render `<x>` / `<<x>>`, and the two TRAILING use sites must render
        `1` and `10` respectively. Two identical-looking output lines here means §3.1 is
        implemented wrong.
        Add a second, simpler substitution case: a resolved search used at a later position.
  - [ ] **`concatenation`** — §3.2's two-way split: a **merged** concatenation
        (`{{a=1}{b=2}{c=3}}` → `{{a=1, b=2, c=3}}`); an **unmerged** one rendering the
        juxtaposition with each constituent recursively simplified — include the spec's worked
        case `{f=3; a={a=1,aa=f}{b=notfound}not_found_brane{d=f}}`, which exercises a resolving
        constituent, a non-resolving one, a bare unresolved name, and a second resolving one in
        a single statement; concatenation of empty branes; and `⨃` appearing nowhere in any of
        it.
  - [ ] **`nk`** — §5: NK **reverts to the written Foolish**, it is NOT rendered `???`.
        `1/0` renders `1/0` with `!! NK: DIV-BY-ZERO: …` beside it (flag on). An NK *search*
        result renders as the SEARCH, not as NK — e.g. an anchored miss renders
        `miss = b?nonexistent  !! NK: …`. The ONLY `???` in output is where the Foolisher
        wrote `???` in source (the no-no literal renders as itself). Also cover: `4 =$ x`
        (the "4 is not a brane" reason); an over-length reason (60-char cap and `…`); a
        multi-line reason (newline and `①` collapse to a space).
  - [ ] **`flags`** — `SequenceOptions` (§4.1, §5): `comment_nk` off renders `a = 1/0;` with
        no annotation, on renders `a = 1/0;  !! NK: …`; a non-default `width` changes where
        lines break. The expression is identical under both `comment_nk` settings — only the
        annotation moves.
  - [ ] **`econstanic`** — §4 + FOOP-23: an unanchored miss `x = nonexistent` renders the
        SEARCH with `!! ECONSTANIC`, **not** `???`. This is the pair to the `nk` group and the
        distinction the einmo reviewer most needs to see side by side.
  - [ ] **`woconstanic`** — a statement whose searches all found but whose dependencies are
        themselves constanic; renders written form + `!! WOCONSTANIC`.
  - [ ] **`preconstanic`** — §2.1: at least one PREMBRYONIC, one EMBRYONIC and one BRANING
        node, rendered written-form with the state in `!!`. If a bounded-step case cannot be
        expressed as suite INPUT (the suite steps to settlement), record that here and cover
        these three in the T2b **unit** tests instead — say which, do not silently drop them.
  - [ ] **`comments`** — §4's placement rule under pressure: several annotated statements
        adjacent; an annotated multi-line construct (comment on the OPENING line); a statement
        that is both annotated and trailing (no `;`); confirm exactly two spaces before `!!`.
  - [ ] **`no_state_tokens`** — a brane that formerly rendered `{WOCONSTANIC` and now must
        render a bare `{`. The negative assertion: **no NYES token appears as syntax anywhere.**
  - [ ] **`width`** — §4.1's 108-char budget: a brane whose single-line form would exceed 108
        (must break, body indented); the same construct nested one level deeper (budget reduced
        by the indent, so it breaks sooner); and one case of each stated exception rendering
        intact rather than mangled — an unsplittable long identifier, an annotated line pushed
        over by its `!!`, and an echoed over-width source statement.

### 3c — Hand-write the expected OUTPUT

- [ ] **Before running anything**, write the expected OUTPUT for the whole case by hand, from
      the spec alone. Put it in the plan (or a scratch file committed alongside) so the
      prediction is on record and cannot be quietly revised after seeing real output.
- [ ] Install it as the suite's `checked/` artifact, signed with `einmo_suite2`'s configured
      checked passphrase. **The OUTPUT section is subject to the same collision rule** — if the
      hand-written expectation contains `①` anywhere, einmo will refuse to serialize it and the
      failure will look like a tooling bug rather than a typo.
- [ ] **This is the FOOP's own acceptance test.** If writing this by hand proves impractical,
      STOP and report — that is the FOOP failing at its stated purpose, and it is far cheaper
      to learn it here than after Phase 6.

### 3d — Write the suite runner, then make it pass

- [ ] Write `foolish-ubca2/src/ubca_snapshot_tester2.rs` (or an added module in the existing
      tester) with einmo gates for `einmo_suite2`, modelled on `ubca_snapshot_tester.rs`:
      `einmo_suite2_gate_output` and `einmo_suite2_gate_checked`. Its adapter uses
      **`Ubca2Sequencer::format(…, SequenceMode::Foolish)`** — the new renderer, from the
      start.
  - [ ] Take the SAME `GATE_LOCK` discipline as the existing gates, and confirm whether the
        lock must also serialize against `einmo_suite`'s three gates — they write different
        `output/` directories, so it may not; **verify rather than assume**, and write down
        which it is and why.
  - [ ] Do **not** add an `einmo_suite2_gate_verified` yet: `verified/` is empty and AGENTS.md
        forbids an agent marking a Verified-tier test `#[ignore]`. Adding a gate that must fail
        is a decision for the human — raise it, do not make it.
- [ ] **Now implement the renderer** (§3, §3.1, §4, §5) until this one case passes. This is the
      whole development loop for the sequencer: one hand-authored target, iterate against it.
  - [ ] Establish relevant tests. Use [these instructions](../../README.md#running-specific-tests)
        to run unit tests: `foolish-ubca2::sequencer`; run einmo case:
        `foop/36/rendering_contract` in `einmo_suite2`.
  - [ ] Implement group by group, in the order listed in 3b. Each group's sub-brane going green
        is a checkpoint.
  - [ ] While implementing the `concatenation` group (§3.2): split on whether the merge
        SUCCEEDED — `hs_concatenation()` returns `(elements, merged)`, and `merged.is_some()`
        is exactly that question. **Merged** → render the merged brane
        (`{{a=1}{b=2}{c=3}}` → `{{a=1, b=2, c=3}}`). **Unmerged** → render the juxtaposition
        `A B` with **each constituent rendered recursively through this same sequencer** — the
        simplest rendering of `foolish_children` under §3, element by element. Never emit `⨃`
        in either case; it is not input syntax. The spec's worked case must come out exactly as
        §3.2 shows, including `aa=f` AND `d=f` both resolving to `3` while `notfound` and
        `not_found_brane` stay written.
  - [ ] While implementing `searches_written` AND `operators_written` (§3): it is ONE predicate
        on the **result**, applied to both kinds — render the original expression when the
        result is absent or an **inconclusive constanic**; render the value when the result is
        **conclusive**. `hs_search()` and `hs_operator()` each hand you what you need.
        **Key on the RESULT's NYES, never on the node's own** — that is the single easiest
        mistake to make here, and it renders the wrong thing for any node whose result is a
        plain value. Write the predicate ONCE and share it between the two arms.
  - [ ] While implementing the `width` group (§4.1): set the budget to **108** and thread it
        as `line_hint`, reduced by indent at each level — mirror
        `foolish-core/src/sequencer.rs`'s existing `line_hint` plumbing rather than inventing
        new logic. Never split an atom; Foolish has no line-continuation syntax and splitting
        one would break Property 1.
  - [ ] While implementing the `nk` group (§5): reason is ONE line — newlines AND the einmo
        separator `①` (U+2460) collapse to a space; truncate to 60 chars with a trailing `…`;
        prefix the `Alarm` code when present (`!! DIV-BY-ZERO: division by zero`).
  - [ ] While implementing the `econstanic` / `woconstanic` / `preconstanic` groups (§4):
        annotation only — written-form rendering is already done by §3. **No comment for
        CONSTANT / INDEPENDENT.** **No NYES token may appear as syntax**, only inside `!!`.
        Comment placement is §4's rule, exactly: one per rendered line, after the `;` (or after
        the last token of a trailing statement), separated by **exactly two spaces**;
        multi-line constructs annotate their OPENING line.
  - [ ] **Every difference between the hand-written expectation and real output must be
        accounted for**, one at a time: either the renderer is wrong (fix it) or the
        prediction was wrong (fix it AND record why the spec misled you — that is a spec
        defect worth reporting).
- [ ] T1 unit tests alongside, per §Test Plan T1: exact rendered strings per §3 row, and the
      `Detailed`-delegation byte-equality tests from Phase 2.
- [ ] Run all tests — old and new — and make sure they all pass correctly.
      `einmo_suite/`'s gates are still on the OLD rendering and must still pass — Phase 5 is
      where they move.

## Phase 4 — Movement II: feature completion against the hand-written target

*The renderer now reproduces one hand-authored case. This phase finishes it — proving the
properties the whole design rests on — while `einmo_suite/` remains untouched and green on the
OLD rendering. Nothing here generates a baseline.*

- [ ] (read §2 and §2.1 of `FOOP-36.md`)
- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run unit tests: `foolish-ubca2::sequencer`, `foolish-ubca2::round_trip`; run einmo
      case: `foop/36/rendering_contract` in `einmo_suite2` (must stay green throughout).
- [ ] **T2 — round-trip properties (§2).** Write the test as the six literal steps, since that
      is the shape the human specified and it is what makes the assertion meaningful:
      1. compile the program; 2. step to finish; 3. output → `R1`; 4. compile `R1` (that it
      compiles IS Property 1); 5. step to finish; 6. output → `R2`; **assert `R2 == R1`.**
      Run it over a curated set covering every §3 row. Property 2 (step 6) is asserted **only**
      where the FIR is constanic (§2.1's table) — a pre-constanic FIR legitimately steps
      further on the second pass, so only steps 1–4 apply there.
      This is a stronger check than reading one rendering: a construct that renders to
      something even slightly different drifts on the second pass and the test catches it,
      with nobody having to predict the right answer in advance. It also settles §Q7
      empirically — whichever way a trailing use site renders, `R2 == R1` says whether that
      rendering is stable.
  - [ ] **The T2 input must instrument a VARIETY of constanic states**, not just constants:
        CONSTANT and INDEPENDENT values; ECONSTANIC searches (unanchored misses); WOCONSTANIC
        statements; and NK expressions (`1/0`, an anchored miss). NK is **constantew** (§5.1) so
        it re-settles NK on the second pass and `R2 == R1` holds; ECONSTANIC is
        **non-constantew**, so it is the interesting one to watch — a rendered ECONSTANIC search
        re-read in a new context may resolve differently, and T2 is what proves the rendering
        is nonetheless stable.
- [ ] **T2b — pre-constanic rendering (§2.1).** Build FIRs stepped a bounded number of steps
      (not to settlement) and assert: renders, **parses**, contains no NYES token as syntax,
      state appears only inside `!!`. Cover at least one PREMBRYONIC, one EMBRYONIC, one
      BRANING, and one case halted by an `ALARM:` mid-step. **Do not assert idempotence** —
      §2.1 explicitly does not require it of pre-constanic FIR. If Phase 3b's `preconstanic`
      group could not be expressed as suite INPUT, this is where those three states get their
      coverage.
- [ ] **T8 — comment style and separator safety (§4.2).** Assert the renderer never emits `①`
      (U+2460) anywhere — chiefly via an NK reason containing one, which §5 collapses to a
      space. Also check the §4.2 layout rules hold across every `.foo` input this FOOP authors.
- [ ] **T9 — flags (§4.1, §5).** `comment_nk` off renders `a = 1/0;` with no annotation, on
      renders `a = 1/0;  !! NK: …` — the EXPRESSION identical under both, only the annotation
      moving. A non-default `width` changes where lines break. Confirm the einmo adapter uses
      the defaults, so the corpus is reproducible.
- [ ] **T7 — line width (§4.1).** Unit tests that a construct over 108 chars at its indent
      breaks with its body indented, that nesting reduces the budget by the indent, and that
      the three exceptions (unsplittable atom, annotated line, echoed over-width source) render
      intact. **Not** a corpus-wide width assertion.
- [ ] Confirm every §3 table row has at least one T1 unit test and one `einmo_suite2` group.
      Name any row that does not, and close the gap.
- [ ] Confirm the `Detailed`-delegation byte-equality tests from Phase 2 still pass — the
      renderer's growth must not have perturbed the delegating mode.
- [ ] Run all tests — old and new — and make sure they all pass correctly. **`einmo_suite/`'s
      three gates must STILL PASS on the old rendering** — if one has moved, the adapter was
      switched early and Movement III has begun by accident.

---

## Phase 5 — Movement III: `einmo_suite2` becomes the suite

*`einmo_suite2` is not a scratch pad — it is `einmo_suite`'s replacement. This phase moves the
inputs across, renders their outputs under the new sequencer, and points `cargo test` at the
new suite. Everything is done EXCEPT removing the old directory, which stays in place,
untouched and still passing, as the reference to diff against.*

**Why the old suite is kept rather than migrated in place.** Re-rendering `einmo_suite`'s
baselines would leave the tree with no green record of what the old rendering produced — and
that record is exactly what a reviewer needs while judging 179 changed outputs. Keeping
`einmo_suite` frozen and building `einmo_suite2` alongside means both renderings are on disk
at once, the old gates stay green the whole way, and the `verified/` tier is never disturbed.
Deleting `einmo_suite` is a separate, later act for the human to authorize.

### 5a — Move the inputs across

- [ ] (read §2 of `FOOP-36.md`; re-read Phase 0's recorded "before" test readings)
- [ ] Copy every input from `foolish-ubca2/einmo_suite/input/**/*.foo` (179 files) into
      `foolish-ubca2/einmo_suite2/input/`, preserving the directory structure
      (`foop/<N>/…`, `misc/…`, `regression/…`). **Copy, do not move** — `einmo_suite/` must
      remain intact and passing.
- [ ] Copy `MAPPING.md` too, and add a note at its top recording that this suite's outputs are
      rendered by `Ubca2Sequencer` in `Foolish` mode (FOOP-36), unlike `einmo_suite/`'s.
- [ ] **T10 — coverage parity.** Write a test asserting `einmo_suite2/input/` contains an
      input for EVERY input in `einmo_suite/input/` — same relative paths, same count. Then
      confirm the total: 179 copied plus `foop/36/rendering_contract.foo` from Phase 3 (and
      `foop/36/comprehensive.foo` arrives in Phase 7). **This is the failure mode that matters
      most and is invisible from a green run** — a new suite that quietly tests less than the
      one it replaces.
- [ ] **T3 — corpus-wide round-trip.** One unit test walking every
      `einmo_suite2/input/**/*.foo`: evaluate, render in `Foolish` mode, assert the result
      **parses**. Property 1 only — not idempotence — so it stays fast and stays correct for
      non-settling cases (§2.1). **Run this BEFORE generating any output**: it is the cheapest
      check that the renderer survives the whole corpus, and it fails loudly without writing a
      single baseline.
- [ ] Fix whatever T3 finds. A parse failure is a renderer bug, never a baseline problem.

### 5b — Generate the outputs and hook up `cargo test`

- [ ] Run `einmo_suite2`'s output gate to render all 180 cases under the new sequencer. The
      adapter already uses `Ubca2Sequencer::format(…, SequenceMode::Foolish)` (Phase 3d) — no
      code change is needed here, which is the point of having built the suite that way.
- [ ] **Point `cargo test` at `einmo_suite2`.** After this checkbox, the crate's default test
      run exercises the new suite:
  - [ ] `einmo_suite2_gate_output` and `einmo_suite2_gate_checked` are the gates that must pass
        for the crate to be considered green.
  - [ ] `einmo_suite/`'s three gates **remain in place and must still pass**, unchanged, on the
        OLD rendering. They are the frozen reference. **Do not re-render, re-promote, or
        `#[ignore]` them.**
  - [ ] Both suites' gates take the `GATE_LOCK` discipline — they now share a test binary and
        write to different `output/` directories, but confirm rather than assume (Phase 3d
        recorded the answer).
  - [ ] **T11 — suite integrity.** `einmo_suite2` passes einmo's own soundness checks at each
        level (`results.integrity.is_clean()`), and its `einmo.toml` is configured per §4.2:
        separator `①`+LF, a `checked` passphrase distinct from `einmo_suite`'s, `verified`
        left unconfigured so a human must type one.
- [ ] Run all tests and verify each expectation:
  - [ ] `einmo_suite2`'s output gate: **passes** — all 180 cases render and self-verify
  - [ ] `einmo_suite2`'s checked gate: **fails for the 179 newly-copied cases** (no `checked/`
        baseline exists for them yet) and **passes for `foop/36/rendering_contract`**, whose
        baseline was hand-written in Phase 3. That split is the expected state going into
        Phase 6.
  - [ ] `einmo_suite/`'s three gates: **all still pass**, matching Phase 0's readings exactly.
        If any has moved, something re-rendered the old suite — STOP.
  - [ ] `foolish-ubca`'s `einmo_gate_checked`: **still passes**. If not, the scope guard has
        been violated — STOP.
  - [ ] `foolish-ubca2`'s unit tests: still pass

**Note what this endgame avoids.** Because `einmo_suite/` is never re-rendered, its
`verified/` tier is never invalidated, and §Q6's "the gate goes red until the human
re-attests" does not arise for it. What needs human attestation instead is `einmo_suite2`'s
own `verified/` tier, which starts empty — a new-suite question, not a broken-tier one. Raise
it with the human at Phase 6 rather than assuming either answer.

---

## Phase 6 — Promotion Review Gate: `einmo_suite2`'s 179 copied baselines

*Every copied case renders for the first time under the new sequencer, so every case must be
justified before it becomes `einmo_suite2/checked/`. This is the bulk of the work. It is
deliberately split by suite subdirectory — `foop.md`: a gate whose boxes are checked faster
than the cases could be read is a false record.*

**All promotion in this phase targets `einmo_suite2`.** `einmo_suite/` is frozen reference and
is never promoted, re-rendered, or otherwise touched.

**Standing instruction for every sub-block below.** For each case: read the INPUT, read the new
OUTPUT, and state **in your own words why each rendered line is what §3/§4/§5 require** — not
that it matches what the renderer emitted. Two questions specific to this FOOP:

1. **Is the OUTPUT valid Foolish?** (Property 1 — T3 checks it mechanically, but read it.)
2. **Did any VALUE change?** This FOOP changes rendering only. A `12` that became a `13`, or a
   settled case that became NK, is **a bug in this FOOP**, not a new baseline. Report it, do
   not promote it.

- [ ] Confirm the rest of the tree is green — `foolish-ubca`'s gates and `einmo_suite/`'s
      three gates all still pass, unchanged (T5)
- [ ] **`einmo_suite2/verified/` is EMPTY** — it is a brand-new suite, so no case here has a
      frozen twin and nothing is at risk of being overwritten. (Contrast `einmo_suite/`, whose
      `verified/` holds all 179 human-signed artifacts and which this FOOP does not touch.)
      Promote `output` → `checked` in `einmo_suite2` as normal; **leave `checked` → `verified`
      entirely to the human** (§Q6).
- [ ] **T12 — value non-regression. Diff each case against its `einmo_suite/checked/`
      counterpart** — the old rendering is still on disk precisely so this is possible. The
      question for each is not "does this match" (it must not) but **"is this the same program,
      said in Foolish?"** The rendering changes; the program's meaning must not. A `12` that
      became a `13`, or a settled case that became NK, is **a bug in this FOOP, not a new
      baseline** — report it, do not promote it. Mechanise the comparison where the shapes
      allow and read it where they do not.
- [ ] Re-read the in-force specifications the cases exercise: `FOOP-36.md` §3/§4/§5, plus
      `README.md` §"The Unknown" and `FOOP-23.md` §Specification for every NK result.
- [ ] Review `regression/` — 4 cases, each named individually in the sub-boxes
- [ ] Review `foop/9/` (2), `foop/13/` (5), `foop/16/` (1) — 8 cases
- [ ] Review `foop/23/` (11) — search semantics; the NK/ECONSTANIC distinction matters most here
- [ ] Review `foop/33/` (13) — creation original names (`'True`) must survive §3 unchanged
- [ ] Review `foop/41/` (1), `foop/42/` (1), `foop/62/` (5), `foop/65/` (4) — 11 cases
- [ ] Review `misc/` (132) — split into named sub-blocks of at most 20 cases each; name every
      case. Group by feature (sf/sff, seek, search, operators, unicode, alarms) so each block
      is reviewed against one part of the spec.
- [ ] Write the justification summary into this plan or the commit message: for each
      subdirectory, what changed in the rendering and why it is spec-correct; call out by name
      any case whose output surprised you.
- [ ] **Report ALL accumulated doubts to the human in ONE statement** — or record "no doubts".
      Blocking doubts stop here; non-blocking ones are reported alongside (AGENTS.md
      §"Accumulate doubts; report them once, at the end").
- [ ] `einmo promote output to checked foolish-ubca2/einmo_suite2`
- [ ] Re-run `cargo test -p foolish-ubca2 --lib -- einmo_suite2_gate_checked` — must exit 0
- [ ] Re-run `einmo_suite/`'s three gates — all must STILL pass, untouched

---

## Phase 7 — Comprehensive case

- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run einmo cases: `foop/36/comprehensive` in `einmo_suite2`; run unit tests:
      `foolish-ubca2::sequencer`, `foolish-ubca2::round_trip`.
- [ ] Write `foolish-ubca2/einmo_suite2/input/foop/36/comprehensive.foo` — at least one path
      through **every** §3 row, plus §4's five states and §5's NK forms, plus the comment
      placement rule where several annotated statements sit adjacent. Follow §4.2's comment
      style (fenced headings blank-line-separated both sides; full-line comments tight above
      the code they mark; no `①` anywhere). Use Unicode operator
      forms (`⬤`, `<̲`, `=̲=̲`) per AGENTS.md.
- [ ] **Write the expected OUTPUT by hand FIRST, before running it.** This is the FOOP's own
      acceptance test: if the OUTPUT cannot be predicted from the spec, the FOOP has not
      achieved what it claims. Record the hand-written prediction in this plan, then run, then
      account for **every** difference — each one is either a bug or a gap in the spec.
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] Review and promote `output` → `checked` for `foop/36/comprehensive`
  - [ ] Confirm the rest of the suite is green — no foreign-FOOP baseline diverges
  - [ ] Confirm this case has no `verified/` twin
  - [ ] Re-read `FOOP-36.md` §3/§4/§5 and `README.md` §"The Unknown" for any NK result
  - [ ] Review `foop/36/comprehensive` — every OUTPUT statement justified, and reconciled
        against the hand-written prediction above
  - [ ] Write the justification summary into this plan or the commit message
  - [ ] Report ALL accumulated doubts to the human in ONE statement — or record "no doubts"
  - [ ] `einmo promote output to checked foolish-ubca2/einmo_suite2`
  - [ ] Re-run `cargo test -p foolish-ubca2 --lib -- einmo_suite2_gate_checked` — must exit 0

---

## Phase 8 — Merge

- [ ] Verify all work is complete in
      `/yolo/foolish/../foolish_worktrees/foop-36-foolish-rendering-sequencer` and committed to
      `foop-36-foolish-rendering-sequencer`
- [ ] Confirm the scope guard held: `git diff jia --stat` shows **no** changes under
      `foolish-ubca/`, **no** changes to `foolish-core/src/sequencer.rs`, and **no changes to
      `foolish-ubca2/einmo_suite/`** beyond the einmo.toml comment fix already on `jia` — the
      old suite is frozen reference, not a thing this FOOP edits. (A `foolish-core/src/fir.rs`
      change appears only if Phase 1 reported and the human approved an additive accessor.)
- [ ] **T5 non-regression** — `cargo test -p foolish-ubca --lib -- einmo_gate_checked` passes,
      matching the Phase 0 "before" reading exactly
- [ ] **`einmo_suite/`'s three gates still pass**, unchanged, on the OLD rendering — including
      `einmo_gate_verified` against its 179 human-signed artifacts. This FOOP leaves that tier
      untouched, which is the whole benefit of replacing rather than migrating in place.
- [ ] **`einmo_suite2` is the suite `cargo test` exercises**, and its `checked/` tier is
      complete (180 cases). Its `verified/` tier is empty and awaits the human — raise it,
      do not `#[ignore]` its gate.
- [ ] **`einmo_suite/` is NOT removed by this FOOP.** Its retirement is a separate act, for the
      human to authorize once `einmo_suite2` has been trusted for a while. Say so explicitly in
      the merge report.
- [ ] `cargo fmt --all` and `cargo clippy -p foolish-ubca2 -- -D warnings` clean.
      **Note:** `foolish-core/src/sequencer.rs` has 4 pre-existing clippy **warnings** (lines
      187, 537, 563, 743 — `iter_next_slice` and friends), which become errors under a
      workspace-wide `-D warnings` gate. They are pre-existing and NOT this FOOP's to fix —
      indeed the scope guard forbids touching that file. Scope the clippy run to
      `-p foolish-ubca2`, and if a workspace `-D warnings` gate is demanded, report the
      conflict rather than editing `foolish-core/src/sequencer.rs` to satisfy it.
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] Merge `foop-36-foolish-rendering-sequencer` to `jia`
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO
        CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present the human with
          `cd /yolo/foolish/../foolish_worktrees/foop-36-foolish-rendering-sequencer` and ask
          them to review `einmo_suite2` BEFORE checking the parent checkbox. Say plainly that
          this FOOP **replaces `einmo_suite` with `einmo_suite2`** — 179 inputs copied across
          and re-rendered, `cargo test` now pointed at the new suite, the old one left frozen
          and still green for diffing, and **not** removed. The review question is "is this
          valid, predictable Foolish?", not "does it match".
  - [ ] Repair ALL tests in `jia` at `/yolo/foolish` if the merge broke any
- [ ] Cleanup `/yolo/foolish/../foolish_worktrees/foop-36-foolish-rendering-sequencer`
  - [ ] Check that this `.plan.md` has all but Cleanup checkboxes completed
  - [ ] Remove `/yolo/foolish/../foolish_worktrees/foop-36-foolish-rendering-sequencer`
  - [ ] This is the last sub-task checkbox to be checked in this block

---

## Last Updated

**Date**: 2026-09-02
**Updated By**: Claude Code / claude-opus-5
**Changes**: Edited for execution by a smaller agent. Added §"How to work this plan" — read the
spec first, work top to bottom, timestamp each box, stop where told, accumulate doubts and
report once, treat a failing test as broken code (except Phase 5's intended failure), never
promote outside the gates.

Adds **Phase 0.5** — a fail-fast, skippable vocabulary fix placed FIRST. It gives all four §0
NYES groups a predicate on `NyesExt` (`is_preconstanic` primary with `is_nye` as its alias, plus
the new `is_conclusive`; `is_constanic` and `is_constantew` already exist), replaces the five
hand-rolled `Constant | Independent` matches in `fvm_storage.rs`, and qualifies every "settled"
with its NYES group per §0.1 (`settled_result` →
`settled_constanic_result`, `all_settled` → `all_foolish_children_conclusive`,
`step_to_settled` → `step_to_constanic`, and the rest), plus `lib.rs`'s stale `is_settled()`
claim. Mechanical and behaviour-free, with an explicit skip rule — abandon it the moment it
stops being mechanical, since nothing later depends on it.

The plan runs in **three movements**: (I) hand-write the expectations in a new `einmo_suite2/`
before the renderer exists — the FOOP's own acceptance test; (II) complete the renderer against
that fixed target; (III) **replace the suite** — copy the 179 inputs into `einmo_suite2`, render
them, review case by case, and point `cargo test` at the new suite. The **cut-over is at the end
of the project**, so the development procedure is unchanged until Movement III. `einmo_suite`
is left frozen and green as the reference to diff against, and is NOT removed.

Phase 3 enumerates all case groups; the Orientation block carries verified code facts, the
trait shape, exact commands and §0's terminology inline so a modest-context agent need not
re-derive them. Phase 5 gains **T10** (coverage parity — every old input has a new counterpart)
and **T11** (suite integrity); Phase 6 gains **T12** (value non-regression: the rendering
changes, the program's meaning must not). Phase 0's two blocking questions are already answered
(Q4: FOOP-36 first; Q6: defused for the old suite, which is never re-rendered — the human
mass-verifies `einmo_suite2`'s new `verified/` tier after the per-case review).
