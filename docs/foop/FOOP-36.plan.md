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
has been misread. Foolish mode renders directly from ubca2's arena; `foolish-core` is used only
for `Detailed` conversion and delegation (§1, §FIR Impact).

---

## Broad structure — three movements

Read this before the phases. The plan is deliberately shaped so the hardest question — *can a
person write the expected output from the spec alone?* — is answered FIRST, on one
hand-authored case, before any baseline is generated and before the renderer exists.

| | Movement | Phases | What it establishes |
|---|---|---|---|
| **0.5** | **(moved to FOOP-56)** | 0.5 | The NYES-group vocabulary work became its own FOOP, scheduled BEFORE this one. Phase 0.5 now just confirms it landed. |
| **I** | **New test, written by hand** | 3 | A brand-new `einmo_suite2/` holding ONE case whose expected OUTPUT is **typed from the specification before the renderer is written**. The renderer is then built until it reproduces what was typed. |
| **II** | **Feature completion** | 4 | The renderer is finished against that fixed target: round-trip properties (§2/§2.1) proven, every §3 row covered, `Detailed` delegation pinned. `einmo_suite/` is untouched and still green on the OLD rendering. |
| **III** | **Replacement** | 5–7 (incl. 6.5) | `einmo_suite2` **becomes the suite**: the 179 inputs are copied across, rendered under the new sequencer, reviewed case by case, and `cargo test` is pointed at it. `einmo_suite/` is left frozen and still green as the reference to diff against — everything is done EXCEPT removing it. |

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
| `foolish-ubca2/src/evaluator.rs` | retain the arena through Foolish sequencing | 59 lines |
| `foolish-ubca2/src/fvm_storage.rs` | **READ ONLY** — arena FIR queried by the renderer | ~8200 lines |
| `foolish-ubca2/src/lib.rs` | register the module | 36 lines |
| `foolish-ubca2/src/ubca_snapshot_tester.rs` | the einmo adapter; ONE call site changes in Phase 5 | 236 lines |
| `foolish-ubca2/einmo_suite2/` | **you create this** — Movement I's hand-written suite | — |
| `foolish-core/src/sequencer.rs` | **READ ONLY** — `Detailed` delegates to it; never edit | 814 lines |
| `foolish-core/src/fir.rs` | **READ ONLY** — `Nyes` and Detailed compatibility FIR | 2663 lines |

**The FIR you render from** — `FVMStorage` + `FirPointer`, read through `FirCursor` in
`foolish-ubca2/src/fvm_storage.rs`. Dispatch on `FirCursor::node() -> &FirSpec`; read written
children from `foolish_children()`, produced results from `ubc_children()`, and NYES from
`get_nyes()`. `FirSpec::Search` itself retains `pattern`, `anchored`, `forward`,
`is_value_search`, and `contexted`; its written anchor/value operands remain in
`foolish_children` even after an anchored miss. Do not route Foolish mode through
`proto_to_core_fir`, which omits some of that metadata.

**`Nyes`** (`fir.rs` ~115) has 8 variants: `Prembrionic`, `Embryonic`, `Braning` (pre-constanic);
`Econstanic`, `Woconstanic`, `Constant`, `Independent`, `Nk` (constanic). Helpers:
`is_constanic()`, `is_nye()`, `should_show_nyes()`. FOOP-56 additionally provides
`is_preconstanic()`, `is_constantew()`, and `is_conclusive()` on `foolish-ubca2`'s `NyesExt`.
Note the spelling **`Prembrionic`** in code
vs `PREMBRYONIC` in prose.

**Facts already verified — do not spend context re-checking:**

- `_` in an identifier is lexed to **U+02CD `ˍ`** (`foolish-parser/src/lexer.rs` `SEP`, line 11;
  pushed at line 454) and `ˍ` is accepted as an identifier char on input (`is_id_sep`, line 100).
  So `myˍvar` round-trips. This is why output shows `nonˍexistent`.
- `!!` line comments and `!!!` block comments are lexed and discarded
  (`lexer.rs` lines 121–124, 327, 361). This is what makes §4's annotations free.
- Search patterns are stored **regex-wrapped**: identifier `x` becomes `pattern='^x$'`, while
  explicit regex searches retain their regex. Equivalent spellings are standardized rather
  than recovered byte-for-byte (human decision 2026-09-03); e.g. postfix `A = B$` renders as
  attached `A =$ B`.
- Arena searches expose written operands and produced results as separate slots:
  `foolish_children` holds anchor/value operands; `ubc_children[0]`, when present, is the
  produced result (Phase 1, Q5).
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
  - **Predicates** (added by FOOP-56, which lands before this FOOP): `is_preconstanic()` with
    `is_nye()` as its alias, `is_constanic()`, `is_constantew()`, `is_conclusive()` — all on
    `foolish-ubca2`'s `NyesExt`. Use them; do not hand-roll
    `matches!(…, Nyes::Constant | Nyes::Independent)`.
  - **"Settled" is prose, not a predicate** — `is_settled()` does not exist despite `lib.rs`
    and FOOP-62 claiming it. After FOOP-56 the uses are qualified: `settled_constanic_result`,
    `step_to_constanic`, `all_foolish_children_conclusive`, and so on.
  - [x] FOOP-56 corrected `lib.rs`'s stale `is_settled()` claim before this FOOP began.
    (2026-09-02 20:15)
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

- [x] Begin work: commit `FOOP-36.md` and `FOOP-36.plan.md` to origin, check `begun: [x]` in
      `FOOP-36.md` frontmatter
      (2026-09-02 20:35)
- [x] Sequencing against FOOP-26 (§Q4) — **DECIDED by the human 2026-09-02: FOOP-36 goes
      first.** No need to re-ask. If FOOP-26 has nonetheless begun in a worktree, say so and
      pause rather than racing it.
      (2026-09-02 10:15)
- [x] Confirm the tree is green BEFORE any change (AGENTS.md: never start Phase+ work with
      broken tests). Record the result in this plan.
      (2026-09-02 20:36)
  - [x] `cargo test -p foolish-ubca2 --lib` — 141 passed; 0 failed.
        (The planned count 134 predates FOOP-56's seven added tests.)
        (2026-09-02 20:36)
  - [x] `cargo test -p foolish-ubca --lib -- einmo_gate_checked` — 1 passed; 0 failed; this is the
        **T5 "before" reading** the final phase compares against
        (2026-09-02 20:36)
  - [x] **`einmo_gate_verified` on `foolish-ubca2` PASSES today** — 1 passed; 0 failed; `verified/` holds all 179
        human-signed artifacts (measured 2026-09-02; whole crate 134/134). FOOP-16 and
        `ubca_snapshot_tester.rs`'s doc comment both claim `verified/` is empty and the gate is
        expected to fail: **that is STALE, do not trust it.** Re-measure and record the actual
        result, since everything downstream depends on it.
        (2026-09-02 20:36)
  - [x] §Q6 (`verified/` is populated — all 179 cases have a frozen twin) — **DECIDED by the
        human 2026-09-02: option (a).** The agent reviews and promotes `output` → `checked`
        case by case as normal; **the human then mass-verifies `checked` → `verified` in one
        pass afterwards.** So `einmo_gate_verified` IS EXPECTED TO BE RED from Movement III
        until that re-attestation — that is accepted, not a defect to chase. **Never
        `#[ignore]` it** (AGENTS.md). And note the human's mass-verify presumes a real per-case
        review has already happened; it does not replace one.
        (2026-09-02 10:15)
  - [x] Also fix `ubca_snapshot_tester.rs`'s stale `einmo_gate_verified` doc comment (it says
        `verified/` "is still empty here" and the failure "is intentional"). Comment only.
        (2026-09-02 20:36)
- [x] Create worktree at
      `/home/agent/yolo/foolish_worktrees/foop-36-foolish-rendering-sequencer` with branch
      `foop-36-foolish-rendering-sequencer`:
      `git worktree add -b foop-36-foolish-rendering-sequencer /home/agent/yolo/foolish_worktrees/foop-36-foolish-rendering-sequencer`
      (2026-09-02 20:36)
- [x] **All work from here happens in the worktree** — including edits to `docs/foop/`
      (`foop.md` §worktree discipline).
      (2026-09-02 20:36)

---

## Phase 0.5 — (moved to FOOP-56)

*This phase was the NYES-group vocabulary work: the four predicates, the five hand-rolled
`matches!` replacements, and qualifying every bare "settled". It became its own FOOP —
**FOOP-56**, scheduled to land **before** this one.*

- [x] Confirm FOOP-56 has merged, and that `foolish-ubca2` therefore provides
      `NyesExt::is_preconstanic()` (with `is_nye()` as its alias), `is_constanic()`,
      `is_constantew()` and `is_conclusive()`. §3's rule is written in that vocabulary; use the
      predicates rather than hand-rolling `matches!(…, Nyes::Constant | Nyes::Independent)`.
      (2026-09-02 20:36)
- [ ] If FOOP-56 has **not** merged, this FOOP still works — §3's rule is stated in §0's
      vocabulary regardless of what the code calls things. Note it and proceed; do not do
      FOOP-56's work here.

---

## Phase 1 — Resolve Q2 before writing any rendering code

*This phase writes no rendering code. It confirms that every written form can be reconstructed
directly from ubca2's arena, before any lossy compatibility conversion (§Open Questions Q2).*

- [x] (read §3 and §FIR Impact of `FOOP-36.md`)
      (2026-09-02 20:51)
- [x] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run unit tests: `foolish-core::sequencer_tests`, `foolish-ubca2::identifier`. No einmo
      cases yet — this phase adds no rendering.
      (2026-09-02 20:51 — 28 `sequencer_tests` and 15 `identifier`-filtered tests passed.)
- [x] For each §3 table row, inspect `FirSpec`, `FirCursor`, `foolish_children`, and
      `ubc_children` in `foolish-ubca2/src/fvm_storage.rs` and record in this plan, one line per
      row: **can the canonical written form be reconstructed from the arena?**
      (2026-09-03 07:30 — yes; Q2 resolved without any `foolish-core` change.)
  - [x] Operator written form — glyph + operands from `hs_operator`
        (2026-09-02 20:51 — reconstructible.)
  - [x] Search written form — `?` / `~` / `.` / `&`-forms from `FirSpec::Search`'s pattern,
        `forward`, `anchored`, `is_value_search`, and `contexted`, plus its written children.
        (2026-09-03 07:30 — reconstructible. An anchored value-search miss such as `B?=5`
        retains `B` and `5` in `foolish_children` with no `ubc_children` result. Equivalent
        spellings are canonicalized; the renderer does not use `proto_to_core_fir`.)
  - [x] Index written form — `#N` / `^` / `$` from `hs_index`'s offset + anchored
        (2026-09-02 20:51 — reconstructible; `offset` and optional anchor suffice.)
  - [x] Concatenation written form — juxtaposition from `hs_concatenation`'s elements
        (2026-09-02 20:51 — reconstructible; elements and optional merged brane are exposed.)
  - [x] SF / SFF written form — `<`/`>`, `<<`/`>>` + interior
        (2026-09-02 20:51 — reconstructible from the distinct wrapper accessors and their child.)
- [x] **Q5 — DISSOLVED by the human 2026-09-02: "constants should always be rendered in
      Foolish."** A conclusive search IS its value, so `result = {y = 1;}?y` rendering `result = 1`
      is correct and complete — the search disappearing is the evaluator succeeding, not
      information lost. The FIR already separates the two cases structurally (resolved →
      CONSTANT, no `SearchFir` left; unresolved → the `SearchFir` survives), so **no provenance
      marking, no new accessor, nothing to decide.** §3 was rewritten accordingly.
      (2026-09-02 14:05)
- [x] **Confirm §0.1.1 — which NYES states a `settled_constanic_result` slot actually holds.** The gate
      (`fvm_storage.rs:639`) tests `is_constanic()` on the owner, but two mechanisms narrow
      what lands in the slot: `Nyes::transform_for_clone` preserves only CONSTANT/INDEPENDENT/NK
      (= **constantew**) and turns everything else EMBRYONIC; and `push_ubc_child` (line 151)
      queues a non-constanic child as a task so it gets stepped. Instrument a run over the
      corpus and record the observed distribution of result NYES. **This tells you which arms
      of §3's predicate the corpus exercises** — if ECONSTANIC/WOCONSTANIC results turn out to
      be rare or absent, say so, because the `einmo_suite2` cases must then cover them
      deliberately rather than incidentally.
      (2026-09-03 07:31 — across all 179 inputs: Independent 839, Constant 424,
      Woconstanic 51, Nk 43, Prembrionic 22, Econstanic 19, Embryonic 4. No Braning result
      slot was observed. The corpus exercises both inconclusive constanic states directly;
      explicit Phase 3 cases still cover every state.)
  - [x] Confirm §3's dispatch on real FIRs — cheap, and the basis of everything downstream:
        evaluate `misc/search_with_multiple_matches` (`r = b?a.*`, anchored) and
        `misc/undeclared_identifier` (`x = non_existent`, unanchored) and record what reaches
        the sequencer.
        (2026-09-03 07:31 — `b?a.*` is a Constant search with result `3`, so it renders
        `r = 3`; `nonˍexistent` is an Econstanic search with no result, so it renders the
        canonical search. This corrects the stale pre-clarification expectation above.)
- [x] **Q7 — does a trailing use site render its value, or revert to a search?** Evaluate
      `misc/sff_resolves_on_each_use` (`{a=1; b=2; s=<<a+b>>; a=10; s;}`) and inspect the FIR
      at the trailing `s;`. If it is a `SearchFir` for `s`, §3 says it renders `s`; if it is
      the constant `12`, it renders `12`. The committed baseline shows `12` but was produced by
      the OLD renderer, which collapsed searches to values regardless — so it does not settle
      the question. **Record the answer in the plan**: it fixes how a whole family of
      trailing-use-site lines renders across the corpus, and it is the single largest
      determinant of what the 179 migrated baselines will look like. Neither answer is a
      problem; guessing is.
      (2026-09-03 07:31 — the trailing anonymous body is a Constant `Search("^s$")` whose
      first UBC child is the Constant `+` result, value 12. It therefore renders `12`.)
- [x] **Decision point. Q2 resolved: no accessor needed.** The human directed canonical
      standardization on 2026-09-03. Foolish mode renders directly from the complete arena;
      the lossy core-FIR conversion is downstream and used only by `Detailed` mode.
      (2026-09-03 07:30)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-09-03 07:37 — `cargo test --workspace` passed; sibling
      `einmo_gate_checked` passed. No doubts.)

---

## Phase 2 — The sequencer skeleton and `Detailed` mode

*Smallest thing that compiles and proves the delegation contract. No `Foolish` rendering yet.*

- [x] (read §1 and §6 of `FOOP-36.md`) (2026-09-03 07:41)
- [x] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run unit tests: `foolish-ubca2::sequencer`. Run this subset frequently while
      implementing; add new tests to this list as they are written. (2026-09-03 07:41)
- [x] Create `foolish-ubca2/src/sequencer.rs` with `SequenceMode` (`Foolish` default,
      `Detailed`) and `Ubca2Sequencer::format(&FVMStorage, FirPointer, SequenceMode) -> String`,
      exactly as §1 gives the signature. The whole of Phase 2 is this much code:

      ```rust
      use crate::fvm_storage::{FVMStorage, FirPointer, proto_to_core_fir};

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
          pub fn format(storage: &FVMStorage, fir: FirPointer, mode: SequenceMode) -> String {
              match mode {
                  // Phase 2: Foolish temporarily delegates too, so the crate
                  // compiles and every baseline still passes. Phase 3 replaces
                  // this arm with the real renderer.
                  SequenceMode::Foolish => Self::format_detailed(storage, fir),
                  SequenceMode::Detailed => Self::format_detailed(storage, fir),
              }
          }

          fn format_detailed(storage: &FVMStorage, fir: FirPointer) -> String {
              let core_fir = proto_to_core_fir(storage, fir);
              foolish_core::FirSequencer::format(&core_fir)
          }
      }
      ```

      `Detailed` deliberately uses the existing compatibility conversion; `Foolish` replaces
      that arm in Phase 3 and never goes through it. (2026-09-03 07:41)
- [x] Register the module in `foolish-ubca2/src/lib.rs` (`pub mod sequencer;` plus the
      `pub use`), matching the existing module-doc style of that file. (2026-09-03 07:41)
- [x] Implement `SequenceMode::Detailed` as **pure delegation** to
      `foolish_core::FirSequencer::format`. Not a reimplementation. (2026-09-03 07:41)
- [x] Implement `SequenceMode::Foolish` as a temporary delegation to `Detailed`, so the crate
      compiles and every existing baseline still passes while Phase 3 fills it in. (2026-09-03 07:41)
- [x] **T1 (delegation half)** — unit tests asserting `format(storage, fir, Detailed)` is
      byte-equal to formatting `proto_to_core_fir(storage, fir)` with
      `foolish_core::FirSequencer` for at least: an int, a brane, an operator, a resolved
      search, an NK. This pins §1's contract so it cannot silently drift. (2026-09-03 07:41)
- [x] Run all tests — old and new — and make sure they all pass correctly. **No einmo baseline
      may move in this phase** — if one does, `Detailed` is not pure delegation.
      (2026-09-03 07:41 — `cargo test --workspace` passed; sibling `einmo_gate_checked`
      passed independently. No einmo baseline moved; no doubts.)

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

- [x] (read §2, §2.1, §3, §3.1, §4, §5 of `FOOP-36.md` — ALL of them; you are about to write
      their expected output by hand) (2026-09-03 07:42)
- [x] Create `foolish-ubca2/einmo_suite2/` with `input/`, `output/`, `checked/`, `verified/`,
      mirroring `einmo_suite/`'s layout. (2026-09-03 07:45)
- [x] Copy `foolish-ubca2/einmo_suite/einmo.toml` and adapt the header comment. Keep
      `[signing.output] passphrase = ""`. Use a DISTINCT `[signing.checked]` passphrase
      (e.g. `foolish-ubca2-suite2`) so the two suites' stamps cannot be confused.
      Leave `verified` unconfigured, exactly as `einmo_suite/` does. (2026-09-03 07:45)
- [x] **Separator: use `①` + LF** (U+2460), the same two-character sequence as
      `foolish-ubca2/einmo_suite`, so both ubca2 suites are configured alike and a case can move
      between them unchanged. That is einmo's default, so `TestConfig::new(...)` gives it —
      do **not** call `foolish_separator()`. (2026-09-03 07:45)
- [x] Fix the stale comment in `foolish-ubca2/einmo_suite/einmo.toml`: it claimed the suite
      used "the Foolish line-comment separator … set in code via
      `TestConfig::foolish_separator()`", which is not what happens — the tester calls plain
      `TestConfig::new(...)` and the artifacts carry `separator=①\n`. Comment corrected to
      describe the `①` default and to warn that `foolish-ubca` differs. Separator itself
      untouched; `einmo_gate_checked` re-run and passing.
      (2026-09-02 09:58)
- [x] Add a `README.md` in `einmo_suite2/` stating what the suite is for: **the hand-authored
      rendering contract for FOOP-36**, one case, expectations written before the code.
      (2026-09-03 07:45)
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

- [x] Write `foolish-ubca2/einmo_suite2/input/foop/36/rendering_contract.foo` as ONE top-level
      brane whose members are named sub-branes, one per group below. Use Unicode operator forms
      (`⬤`, `<̲`, `=̲=̲`) per AGENTS.md, and `!!` comments to say what each group asserts.
      **Discovered discrepancy (2026-09-03):** the current lexer rejects U+0332 and has no
      implementation for `<̲`/`=̲=̲`, despite AGENTS.md requiring those spellings. Repairing
      `foolish-parser` is outside this rendering FOOP, so the contract expresses comparison
      through the supported `{1, 2, 'lt}$` system-brane idiom and still expects `'True`. This
      also resolves the `operators_written` bullet's literal "a comparison using `<̲`" request
      below — the contract's `comparison = {1, 2, 'lt}$` case is the supported substitute; `⬤`
      IS used (`creation = ⬤`).
- [x] **Apply §4.2's comment style throughout this file** — it is the first input written under
      the new rules and becomes the worked example others copy:
  - [x] Each sub-brane group gets a `!!!` fenced heading, blank line before AND after
  - [x] Within a group, a full-line `!!` comment marks the cases below it: blank line before,
        NO blank line after
  - [x] Per-case remarks are short inline comments trailing the statement
  - [x] **No line contains `①`** (U+2460) — the suite's separator; einmo refuses to serialize
        a section containing it. `!!` is unrestricted here.
        (2026-09-03 — confirmed by `foolish_annotations_are_separator_safe`, which asserts
        `!contract.contains('①')` directly against the committed file, plus the file having
        serialized and signed successfully at all — a `①` collision is a hard write-time error.)
- [x] Every group below gets a sub-brane. **Write the case, and write what you expect it to
      render as, in a `!!` comment beside it** — the comment is the prediction, the einmo
      OUTPUT is the check.

  - [x] **`leaves`** — the trivially-stable renderings (§3):
        integer `7`; negative integer; a bare creation `⬤`; a named creation `'True`
        (FOOP-33 original name); an empty brane `{}`; a brane of constants.
  - [x] **`names`** — statement and identifier rendering (§3):
        plain `a = 1`; an underscore name `my_var = 2` (must render `myˍvar`, U+02CD, and
        re-lex — the round-trip hazard); a Unicode name (Greek/Cyrillic/Chinese per AGENTS.md);
        a characterized brane `a'b'{…}`; a null-characterized name `'k = 1`.
        (2026-09-03 — present and correct; `λ` covers the Unicode-identifier requirement.
        Cyrillic/Chinese specifically are not in THIS contract but are exercised elsewhere in
        the corpus (`unicode_identifiers_basic` in the sibling `einmo_suite`) — not a gap in
        what this contract needs to prove about the RENDERER's dispatch, which is
        script-agnostic.)
  - [x] **`operators_written`** — the same predicate on operators: **when the result is an
        inconclusive constanic, render the op operating on its parameters**; when it is
        conclusive, render the VALUE. Cover both sides:
        - `3 + 4` → `7` (result conclusive — the operator is spent)
        - `a + b` with ECONSTANIC operands → `a + b` (no value was reached)
        - `1/0` → `1/0` with `!! NK: …` (result NK — §5's operator instance)
        Plus: a nested `1 + 2 * 3` (precedence must survive the round trip); unary minus;
        a comparison using `<̲`.
        (2026-09-03 — all present: `sum`/`unresolved`/`divide_by_zero`/`nested`/`unary`/
        `comparison`, matching this list exactly modulo the `<̲` substitution noted above.)
  - [x] **`searches_written`** — §3's predicate, which keys on the search's **`result()`**, not
        on the search's own NYES: **when the result is an inconclusive constanic (§0), render
        the original search**; when it is conclusive, render its value. Cover both sides:
        - result **conclusive** (CONSTANT/INDEPENDENT) → collapses to the value
        - **Add the test line 816's rule lacks**: an operator with an ECONSTANIC operand must
          still queue it as a task. The existing
          `operator_pushes_tasks_for_inconclusive_operands` covers PREMBRYONIC operands, and
          `operator_pushes_tasks_for_econstanic_operand` proves the remaining distinction:
          an ECONSTANIC operand is constanic but must remain queued because
          `all_foolish_children_conclusive` gates on `is_conclusive()`.
        - result ECONSTANIC (unanchored miss) → renders the search
        - result WOCONSTANIC → renders the search
        - result NK (anchored miss) → renders the search, NOT `???`
        - result absent → renders the search
        And both anchoring shapes: unanchored → the search alone (`?x`, `nonexistent`);
        anchored → anchor then search (`b?a.*`, `a.field`), the anchor rendered by these same
        rules. Include the regex-unwrap case (stored `'^a$'` must render `a`).
        (2026-09-03 — the original contract was missing a genuinely distinct case: result
        **WOCONSTANIC**. `nyes_from_found` (`fvm_storage.rs`) maps a FOUND statement's
        ECONSTANIC/WOCONSTANIC value to a WOCONSTANIC search result — different from an
        unanchored MISS settling ECONSTANIC. Verified with a probe before adding: `found =
        woconstanic_base?y` where `y` is itself ECONSTANIC renders `woconstanicˍhit =
        woconstanicˍbase?y  !! WOCONSTANIC` — the search reverts to written form exactly as
        ECONSTANIC/NK do, per §0's inconclusive-constanic predicate. Added `woconstanic_base` /
        `woconstanic_hit`; promoted after review — one clean addition, no other line's value
        changed. Anchored-vs-unanchored, regex-unwrap, and conclusive/ECONSTANIC/NK were
        already present and confirmed correct.)
  - [x] **`indexes_written`** — `#-1`; `#0`; `^`; `$`; the attached form `A =$ B` (FOOP-75 §4).
        **Note on `#-1`**: `render_index`'s marker match (`sequencer.rs`) canonicalizes EVERY
        anchored offset `-1` to `$` — there is no code path that ever prints a literal `#-1`,
        so a dedicated `#-1` case would be indistinguishable from the `$` case already present.
        Covered: `^`/`#0` (documented as sharing one arena shape), the two attached forms
        (`=^`, `=$`), and — in the `nk` group, not duplicated here — an INCONCLUSIVE index
        reverting to written form (`not_a_brane = 4$` renders `notˍaˍbrane =$ 4  !! NK: 4 is
        not a brane`, proving §3's Index row for the non-conclusive case; every other index
        case here is conclusive and collapses to a value). (2026-09-03)
  - [x] **`sf_sff`** — §3 + §3.1 together:
        `<x>` and `<<x>>` as NAMED statements (render written forms, delimiters kept);
        an SFF with an operator interior `<<a + b>>`; a nested `<<a + <<b>>>>`.
        (2026-09-03 — all four present: `sf`, `sff`, `operator_interior`, `nested`.)
  - [x] **`substitution`** — **§3.1's load-bearing group; the reason this FOOP has a §3.1.**
        The exact case `{x = 1; sf = <x>; sff = <<x>>; x = 10; sf; sff;}` — the named
        statements must render `<x>` / `<<x>>`, and the two TRAILING use sites must render
        `1` and `10` respectively. Two identical-looking output lines here means §3.1 is
        implemented wrong.
        Add a second, simpler substitution case: a resolved search used at a later position.
        (2026-09-03 — added `simple_substitution = {base={y=5;};found=base?y;later=found;};`,
        which was genuinely missing from the original contract; renders
        `simpleˍsubstitution = {base = {y = 5}; found = 5; later = 5}` — the resolved search's
        captured value survives to the later use site. Promoted to `checked/` after review:
        this was the only diff against the prior baseline, one clean addition, no other line
        touched.)
  - [x] **`concatenation`** — §3.2's two-way split: a **merged** concatenation
        (`{{a=1}{b=2}{c=3}}` → `{{a=1, b=2, c=3}}`); an **unmerged** one rendering the
        juxtaposition with each constituent recursively simplified — include the spec's worked
        case `{f=3; a={a=1,aa=f}{b=notfound}not_found_brane{d=f}}`, which exercises a resolving
        constituent, a non-resolving one, a bare unresolved name, and a second resolving one in
        a single statement; concatenation of empty branes; and `⨃` appearing nowhere in any of
        it.
        **Deferred by the human (2026-09-03):** the exact worked case currently stores `aa=f`
        and `d=f` as ECONSTANIC searches with no UBC result because bare concatenation branes
        are built under SFF. It belongs to FOOP-46's BraneConcatOp ergonomics work, where the
        evaluator and this sequencer will be updated together. This FOOP's hand contract instead
        covers the simpler `{f=3, a=2, b={f1=f, f2=a, f3=not_found}}` nested-brane case.
  - [x] **`nk`** — §5: NK **reverts to the written Foolish**, it is NOT rendered `???`.
        `1/0` renders `1/0` with `!! NK: DIV-BY-ZERO: …` beside it (flag on). An NK *search*
        result renders as the SEARCH, not as NK — e.g. an anchored miss renders
        `miss = b?nonexistent  !! NK: …`. The ONLY `???` in output is where the Foolisher
        wrote `???` in source (the no-no literal renders as itself). Also cover: `4 =$ x`
        (the "4 is not a brane" reason); an over-length reason (60-char cap and `…`); a
        multi-line reason (newline and `①` collapse to a space).
        (2026-09-03 — `divide`/`miss`/`no_no`/`not_a_brane` cover the four written-form cases
        directly through the real evaluator. The 60-char truncation and multi-line/`①`-collapse
        rules live in the pure, already-isolated `sanitize_reason()` and are unit-tested
        directly (`foolish_annotations_are_separator_safe`) rather than via a contrived
        evaluator-produced long/multi-line reason: no natural Foolish program in this suite
        produces one, and the contract's job is proving the renderer's per-kind DISPATCH under
        §3/§4/§5, not re-proving a leaf formatting function that already has its own direct
        test.)
  - [x] **`flags`** — **could not be expressed as suite INPUT, by the same structural reason as
        `preconstanic` below.** The einmo adapter (`ubca_snapshot_tester2.rs`) calls
        `Ubca2Sequencer::format(…, SequenceMode::Foolish)` unconditionally, which resolves to
        `SequenceOptions::default()` — there is exactly ONE OUTPUT per INPUT case, and §4.1
        requires it: "baselines are rendered at 108 and nothing else, or the corpus would not
        be reproducible." A `flags` sub-brane could show the DEFAULT-flag rendering (which
        every other group already exercises) but not a second rendering of the same case under
        `comment_nk: false` or a non-default `width` — the suite mechanism has no second
        OUTPUT slot to hold it. Coverage lives in unit tests instead:
        `foolish_flags_change_only_annotations_and_layout` (T9, `comment_nk` on/off with the
        expression identical either way, plus a narrow `width`) and
        `foolish_suppress_sequencing_comments_overrides_comment_nk_and_state_annotations` (the
        override). (2026-09-03)
  - [x] **`econstanic`** — §4 + FOOP-23: an unanchored miss `x = nonexistent` renders the
        SEARCH with `!! ECONSTANIC`, **not** `???`. This is the pair to the `nk` group and the
        distinction the einmo reviewer most needs to see side by side.
        (2026-09-03 — `econstanic = {unfound = nowhere}` present and confirmed; sits directly
        beside `nk` in the file for the side-by-side contrast.)
  - [x] **`woconstanic`** — a statement whose searches all found but whose dependencies are
        themselves constanic; renders written form + `!! WOCONSTANIC`.
        (2026-09-03 — `woconstanic = {sum = nowhere + absent}` present, an operator whose
        operand is ECONSTANIC; also now doubly covered by `searches_written`'s new
        `woconstanic_hit` case, a search whose own result is WOCONSTANIC.)
  - [x] **`preconstanic`** — §2.1: at least one PREMBRYONIC, one EMBRYONIC and one BRANING
        node, rendered written-form with the state in `!!`. If a bounded-step case cannot be
        expressed as suite INPUT (the suite steps to settlement), record that here and cover
        these three in the T2b **unit** tests instead — say which, do not silently drop them.
        **Could not be expressed as suite INPUT**: `ubca_snapshot_tester2.rs` steps every case
        to full settlement before rendering (`step_to_constanic`), so a genuinely pre-constanic
        snapshot cannot survive as a `checked/` baseline — by the time OUTPUT is written the
        state has moved on. Moved to Phase 4's T2b (see that checkbox for the two tests and
        what each covers). (2026-09-03)
  - [x] **`comments`** — §4's placement rule under pressure: several annotated statements
        adjacent; an annotated multi-line construct (comment on the OPENING line); a statement
        that is both annotated and trailing (no `;`); confirm exactly two spaces before `!!`.
        (2026-09-03 — `first`/`second`/`trailing` cover adjacent annotations and an
        annotated-and-trailing statement (`trailing = 1 / 0` with no `;`, since it is the
        brane's last statement), all with exactly two spaces before `!!` per
        `annotate()`'s hard-coded `"  !! "`. **The "annotated multi-line construct" requirement
        turned out to be narrower than it reads**: after §4.0 (this session), the ONLY
        multi-line construct this renderer ever produces is `render_brane` — every other kind
        (`render_operator`, `render_search`, `render_index`, `render_stay`) always returns a
        single joined line — and a brane's own opening line is now annotated in exactly one
        case, the direct-alarm exception. So the concrete instance of this requirement IS the
        `foolish_iteration_alarm_is_a_parseable_nk_annotation` test (Phase 3d), not a separate
        contract case: `{  !! NK: Iteration exceeded 9999\n  f1 = {...` — the alarm's comment
        on the brane's opening line, exactly the shape this bullet asks for.)
  - [x] **`no_state_tokens`** — a brane that formerly rendered `{WOCONSTANIC` and now must
        render a bare `{`. The negative assertion: **no NYES token appears as syntax anywhere.**
        (2026-09-03 — this bullet's premise is now the DEFAULT everywhere, not a special case:
        §4.0 (this session) means every brane's opening line is bare unless it carries a direct
        alarm. `no_state_tokens = {unfound = nowhere}` renders `noˍstateˍtokens = {\n  unfound
        = nowhere  !! ECONSTANIC\n}` — bare opening brace, member's own annotation intact. The
        negative assertion is also enforced globally by
        `foolish_preconstanic_rendering_parses_without_state_syntax` and
        `foolish_mid_step_snapshot_renders_without_settling`, which scan every rendered line for
        all five non-syntax state tokens.)
  - [x] **`width`** — §4.1's 108-char budget: a brane whose single-line form would exceed 108
        (must break, body indented); the same construct nested one level deeper (budget reduced
        by the indent, so it breaks sooner); and one case of each stated exception rendering
        intact rather than mangled — an unsplittable long identifier, an annotated line pushed
        over by its `!!`, and an echoed over-width source statement.
        (2026-09-03 — the contract's `width` group covers "must break at 108" and "unsplittable
        long identifier renders intact" directly through the real evaluator at the default
        width. The nesting-reduces-the-effective-budget requirement needs a NON-default
        (narrow) width to demonstrate cheaply and directly — reaching it via nesting depth
        alone at 108 columns would need an unwieldy contract case for no added proof, since the
        underlying mechanism (`line_hint` reduced by indent at each level) is identical either
        way. Covered instead by the existing unit test
        `foolish_width_preserves_atoms_and_indents_nested_branes` (width: 24, asserting the
        nested brane breaks, its body is indented, and the long identifier is preserved whole)
        — same "flag/width variation can't live in one suite baseline" reasoning already used
        for the `flags` group above. An annotated line pushed over by its `!!` is implicit in
        several contract lines already (e.g. `unresolved = missingˍleft + missingˍright;  !!
        WOCONSTANIC` and the DIV-BY-ZERO lines) but not isolated as its own dedicated case;
        judged sufficient since §4.1's exception list is a "must not mangle" requirement and
        every annotated line in the whole 200+-line OUTPUT already demonstrates the annotation
        is simply appended, never causing a break.)

### 3c — Hand-write the expected OUTPUT

- [x] **Before running anything**, write the expected OUTPUT for the whole case by hand, from
      the spec alone. Put it in the plan (or a scratch file committed alongside) so the
      prediction is on record and cannot be quietly revised after seeing real output.
      (2026-09-03 07:46 — recorded in `expected/rendering_contract.out` before invoking einmo
      or the new renderer.)
- [x] Install it as the suite's `checked/` artifact, signed with `einmo_suite2`'s configured
      checked passphrase. **The OUTPUT section is subject to the same collision rule** — if the
      hand-written expectation contains `①` anywhere, einmo will refuse to serialize it and the
      failure will look like a tooling bug rather than a typo. (2026-09-03 07:47)
- [x] **This is the FOOP's own acceptance test.** If writing this by hand proves impractical,
      STOP and report — that is the FOOP failing at its stated purpose, and it is far cheaper
      to learn it here than after Phase 6. (2026-09-03 07:55 — stopped at the first blocking
      arena/spec contradiction; see the `concatenation` item above.)

### 3d — Write the suite runner, then make it pass

- [x] Write `foolish-ubca2/src/ubca_snapshot_tester2.rs` (or an added module in the existing
      tester) with einmo gates for `einmo_suite2`, modelled on `ubca_snapshot_tester.rs`:
      `einmo_suite2_gate_output` and `einmo_suite2_gate_checked`. Its adapter uses
      **`Ubca2Sequencer::format(…, SequenceMode::Foolish)`** — the new renderer, from the
      start. (2026-09-03 07:49)
  - [x] Take the SAME `GATE_LOCK` discipline as the existing gates, and confirm whether the
        lock must also serialize against `einmo_suite`'s three gates — they write different
        `output/` directories, so it may not; **verify rather than assume**, and write down
        which it is and why. (2026-09-03 07:49 — suite2's two gates share one local lock;
        no cross-suite lock because their output trees are disjoint.)
  - [x] Do **not** add an `einmo_suite2_gate_verified` yet: `verified/` is empty and AGENTS.md
        forbids an agent marking a Verified-tier test `#[ignore]`. Adding a gate that must fail
        is a decision for the human — raise it, do not make it. (2026-09-03 07:49)
- [x] **Now implement the renderer** (§3, §3.1, §4, §5) until this one case passes. This is the
      whole development loop for the sequencer: one hand-authored target, iterate against it.
      (2026-09-03 — `einmo_suite2_gate_checked` passes: the renderer reproduces the
      hand-authored target exactly, all groups green, all children below complete.)
      The first comparison against the frozen target corrected these non-blocking predictions
      on 2026-09-03: the enclosing root/operator/search/comment branes settle WOCONSTANIC rather
      than inheriting a descendant NK; the `names` and `indexes_written` branes fit on one line;
      `#0` and `^` have the same arena representation and standardize to attached `=^`; the
      supported comparison idiom ends in an attached tail and therefore standardizes to `=$`;
      and the written no-no reason is exactly `??? literal`. These revisions are recorded in
      the committed hand-target scratch file. The old bare-concatenation worked case is deferred
      to FOOP-46 by human direction; its replacement nested-brane case is green here.
  - [x] Establish relevant tests. Use [these instructions](../../README.md#running-specific-tests)
        to run unit tests: `foolish-ubca2::sequencer`; run einmo case:
        `foop/36/rendering_contract` in `einmo_suite2`.
        (2026-09-03 09:56)
  - [x] Implement group by group, in the order listed in 3b. Each group's sub-brane going green
        is a checkpoint.
        (2026-09-03 — all groups green: `einmo_suite2_gate_checked` matches the hand-authored
        target byte-for-byte, including the two groups added this session
        (`simple_substitution`, `woconstanic_hit`).)
  - [x] While implementing the `concatenation` group (§3.2): split on whether the merge
        SUCCEEDED — `hs_concatenation()` returns `(elements, merged)`, and `merged.is_some()`
        is exactly that question. **Merged** → render the merged brane
        (`{{a=1}{b=2}{c=3}}` → `{{a=1, b=2, c=3}}`). **Unmerged** → render the juxtaposition
        `A B` with **each constituent rendered recursively through this same sequencer** — the
        simplest rendering of `foolish_children` under §3, element by element. Never emit `⨃`
        in either case; it is not input syntax. The bare-concatenation worked case that requires
        `aa=f` and `d=f` to resolve is deferred to FOOP-46 (human, 2026-09-03); this FOOP pins
        the ordinary nested-brane replacement case instead.
        (2026-09-03 — implemented as `render_concatenation`/`render_concat_element`;
        `concatenation` group green, `⨃` confirmed absent from all output.)
  - [x] While implementing `searches_written` AND `operators_written` (§3): it is ONE predicate
        on the **result**, applied to both kinds — render the original expression when the
        result is absent or an **inconclusive constanic**; render the value when the result is
        **conclusive**. `hs_search()` and `hs_operator()` each hand you what you need.
        **Key on the RESULT's NYES, never on the node's own** — that is the single easiest
        mistake to make here, and it renders the wrong thing for any node whose result is a
        plain value. Write the predicate ONCE and share it between the two arms.
        (2026-09-03 — implemented as the shared `render_process_or_result` helper, called by
        both the Operator and Search dispatch arms; confirmed keying on the RESULT's NYES via
        the new `woconstanic_hit` case, whose SEARCH's own NYES is WOCONSTANIC because of what
        it found, not what it itself is.)
  - [x] While implementing the `width` group (§4.1): set the budget to **108** and thread it
        as `line_hint`, reduced by indent at each level — mirror
        `foolish-core/src/sequencer.rs`'s existing `line_hint` plumbing rather than inventing
        new logic. Never split an atom; Foolish has no line-continuation syntax and splitting
        one would break Property 1.
        (2026-09-03 — `LINE_BUDGET = 108`, threaded as the `width` parameter through every
        `render_*` method; unsplittable-atom behavior confirmed by
        `foolish_width_preserves_atoms_and_indents_nested_branes`.)
  - [x] While implementing the `nk` group (§5): reason is ONE line — newlines AND the einmo
        separator `①` (U+2460) collapse to a space; truncate to 60 chars with a trailing `…`;
        prefix the `Alarm` code when present (`!! DIV-BY-ZERO: division by zero`).
        (2026-09-03 — implemented as `sanitize_reason` + `NK_REASON_LIMIT = 60`; unit-tested
        directly by `foolish_annotations_are_separator_safe`.)
  - [x] While implementing the `econstanic` / `woconstanic` / `preconstanic` groups (§4):
        annotation only — written-form rendering is already done by §3. **No comment for
        CONSTANT / INDEPENDENT.** **No NYES token may appear as syntax**, only inside `!!`.
        Comment placement is §4's rule, exactly: one per rendered line, after the `;` (or after
        the last token of a trailing statement), separated by **exactly two spaces**;
        multi-line constructs annotate their OPENING line — **except a `Brane`/`ConcatHelper`
        node's OWN rollup state, which is never annotated at all (§4.0, human direction
        2026-09-03), with the single exception of a DIRECT alarm on the brane itself** (the
        step-cap case, which has no member line to carry the reason).
  - [x] **Every difference between the hand-written expectation and real output must be
        accounted for**, one at a time: either the renderer is wrong (fix it) or the
        prediction was wrong (fix it AND record why the spec misled you — that is a spec
        defect worth reporting).
        (2026-09-03 — the five prediction corrections from the first comparison are recorded
        just above (root/operator/search/comment branes settling WOCONSTANIC not inherited-NK;
        one-line `names`/`indexes_written`; `#0`/`^` sharing one arena shape; the attached-tail
        comparison idiom; `??? literal`'s exact wording). This session's own brane-suppression
        change and its two follow-on baseline diffs were each reviewed line-by-line before
        promotion — see the dedicated addenda above and the T1/searches_written/substitution
        entries — with every changed line accounted for and no unexplained value change in
        any of them.)
- [x] T1 unit tests alongside, per §Test Plan T1: exact rendered strings per §3 row, and the
      `Detailed`-delegation byte-equality tests from Phase 2. (2026-09-03 09:56 — 15 focused
      sequencer tests cover conclusive/inconclusive process results, canonical indexes, leaves,
      names, branes, concatenation, SF/SFF, annotations, and Detailed delegation.)
- [x] **Human-directed refinement (2026-09-03): brane opening lines no longer annotate their
      own rollup state.** During review the human observed that a brane containing an NK member
      duplicated that member's exact reason text onto the brane's own opening `{` (e.g.
      `{  !! NK: DIV-BY-ZERO: division by zero\n  x = 1 / 0;  !! NK: DIV-BY-ZERO: division by
      zero\n}`), and directed that no brane state — not just NK, all six — be marked on the
      bracket, "for purpose of rendering... they can be recompiled and stepped to the same
      stage" (§2.1's round-trip argument). Implemented as new §4.0: `annotate()` now skips
      `Brane`/`ConcatHelper` nodes entirely, with one narrow exception — a DIRECT
      `alarm_reason` on the brane itself (not borrowed from a child) still renders, because the
      step-cap/`ITERATION-EXCEEDED` alarm is set only on the composed root and no member line
      carries it; omitting it there would be a genuine information loss, not a redundant echo.
      Also added `SequenceOptions::suppress_sequencing_comments` (human-named, to distinguish
      this renderer's own annotations from any comment a future feature might echo through from
      source) as an override sitting above `comment_nk`. Updated: `annotate()` in
      `sequencer.rs`; `SequenceOptions`; three existing tests
      (`foolish_flags_change_only_annotations_and_layout`,
      `foolish_renders_branes_and_the_supported_nested_concatenation_case`,
      `foolish_iteration_alarm_is_a_parseable_nk_annotation`) whose expected strings carried the
      old duplicated bracket comments; one new test
      (`foolish_suppress_sequencing_comments_overrides_comment_nk_and_state_annotations`).
      `einmo_suite2/checked/foop/36/rendering_contract.foo.einmo` re-promoted — 12 lines
      changed, every one a sub-brane or the top-level brane's opening `{` losing its rollup
      comment, reviewed line-by-line against `checked_body.txt`/`output_body.txt`: no value
      changed, and every removed reason is still present, verbatim, on the responsible member's
      own line. FOOP-36.md §4 updated (worked EMBRYONIC example corrected; new §4.0 added).
      (2026-09-03 — see also the `annotate` doc comment in `sequencer.rs` for the code-level
      statement of this rule.)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      `einmo_suite/`'s gates are still on the OLD rendering and must still pass — Phase 5 is
      where they move.
      (2026-09-03 — `cargo test -p foolish-ubca2 --lib`: 161/161 (both einmo_suite2 gates and
      all three einmo_suite gates green). `cargo test -p foolish-ubca --lib --
      einmo_gate_checked`: still passes, unchanged — sibling scope guard honored.)

## Phase 4 — Movement II: feature completion against the hand-written target

*The renderer now reproduces one hand-authored case. This phase finishes it — proving the
properties the whole design rests on — while `einmo_suite/` remains untouched and green on the
OLD rendering. Nothing here generates a baseline.*

- [x] (read §2 and §2.1 of `FOOP-36.md`) (2026-09-03 09:56)
- [x] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run unit tests: `foolish-ubca2::sequencer`, `foolish-ubca2::round_trip`; run einmo
      case: `foop/36/rendering_contract` in `einmo_suite2` (must stay green throughout).
      (2026-09-03 — `cargo test -p foolish-ubca2 --lib -- sequencer` run repeatedly through
      this phase's work; `einmo_suite2_gate_checked` reconfirmed green throughout.)
- [x] **T2 — round-trip properties (§2).** Write the test as the six literal steps, since that
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
      (2026-09-03 09:56 — `foolish_rendering_round_trips_constanic_programs` covers constants,
      ECONSTANIC/WOCONSTANIC, NK, searches, indexes, SF/SFF, and both concatenation forms.)
  - [x] **The T2 input must instrument a VARIETY of constanic states**, not just constants:
        CONSTANT and INDEPENDENT values; ECONSTANIC searches (unanchored misses); WOCONSTANIC
        statements; and NK expressions (`1/0`, an anchored miss). NK is **constantew** (§5.1) so
        it re-settles NK on the second pass and `R2 == R1` holds; ECONSTANIC is
        **non-constantew**, so it is the interesting one to watch — a rendered ECONSTANIC search
        re-read in a new context may resolve differently, and T2 is what proves the rendering
        is nonetheless stable.
        (2026-09-03 — verified via probe against the ten T2 sources' root NYES: CONSTANT
        (`{x=3+4;}`), INDEPENDENT (`{x={a=1}{b=2};}` — a self-contained merged concatenation),
        WOCONSTANIC (`{r=missing;}`, the SF/SFF case, the nested-brane concatenation case), NK
        (`{x=1/0;}`, `{x=???;}`, the anchored-miss case). `{r=missing;}`'s `missing` member is
        itself the ECONSTANIC (non-constantew) case the sub-bullet is most concerned with, and
        `R2 == R1` holding for it is exactly the empirical proof the sub-bullet asks for.)
- [x] **T2b — pre-constanic rendering (§2.1).** Build FIRs stepped a bounded number of steps
      (not to settlement) and assert: renders, **parses**, contains no NYES token as syntax,
      state appears only inside `!!`. Cover at least one PREMBRYONIC, one EMBRYONIC, one
      BRANING, and one case halted by an `ALARM:` mid-step. **Do not assert idempotence** —
      §2.1 explicitly does not require it of pre-constanic FIR. If Phase 3b's `preconstanic`
      group could not be expressed as suite INPUT, this is where those three states get their
      coverage.
      (2026-09-03 — Phase 3b's `preconstanic` group was not written as suite INPUT: the suite
      steps every case to settlement, so a genuinely pre-constanic snapshot cannot survive as a
      `checked/` baseline. Coverage lives here instead:
      `foolish_preconstanic_rendering_parses_without_state_syntax` incrementally steps three
      programs and records the first observed PREMBRYONIC, EMBRYONIC and BRANING state each,
      asserting Property 1 and no leaked state token for each. The fourth requirement — a
      mid-step `ALARM:` case — was genuinely missing (the existing
      `foolish_iteration_alarm_is_a_parseable_nk_annotation` test runs the self-referential
      program to full settlement, i.e. the iteration cap firing and NK being reached, not a
      snapshot mid-way there); added `foolish_mid_step_snapshot_renders_without_settling`,
      which steps the same self-referential program 5 times (well short of the 10,000-step
      cap), asserts the root is still pre-constanic, and checks Property 1 without asserting
      idempotence.)
- [x] **T8 — comment style and separator safety (§4.2).** Assert the renderer never emits `①`
      (U+2460) anywhere — chiefly via an NK reason containing one, which §5 collapses to a
      space. Also check the §4.2 layout rules hold across every `.foo` input this FOOP authors.
      (2026-09-03 09:56 — focused sanitizer and contract-layout tests.)
- [x] **T9 — flags (§4.1, §5).** `comment_nk` off renders `a = 1/0;` with no annotation, on
      renders `a = 1/0;  !! NK: …` — the EXPRESSION identical under both, only the annotation
      moving. A non-default `width` changes where lines break. Confirm the einmo adapter uses
      the defaults, so the corpus is reproducible.
      (2026-09-03 09:56 — direct `SequenceOptions` coverage; the suite adapter calls
      `SequenceMode::Foolish`.)
- [x] **T7 — line width (§4.1).** Unit tests that a construct over 108 chars at its indent
      breaks with its body indented, that nesting reduces the budget by the indent, and that
      the three exceptions (unsplittable atom, annotated line, echoed over-width source) render
      intact. **Not** a corpus-wide width assertion.
      (2026-09-03 — `foolish_width_preserves_atoms_and_indents_nested_branes` covers breaking
      + nested-indent-reduces-budget + the unsplittable-atom exception. The remaining two
      exceptions were genuinely untested (only structurally guaranteed by `render_statement`
      calling `annotate` AFTER the width-aware line-breaking decision, never influencing it) —
      added `foolish_width_exceptions_render_intact_not_mangled`, proving directly that a
      narrow-width annotation renders intact rather than split/truncated, and that a
      128-character echoed identifier operand (exceeding even the default 108) renders whole.
      Neither is a corpus-wide assertion — both are single-construct unit tests.)
- [x] Confirm every §3 table row has at least one T1 unit test and one `einmo_suite2` group.
      Name any row that does not, and close the gap.
      (2026-09-03 — checked all 13 rows against the actual test file and contract. 11 rows
      already had both. Two rows — **Creation** and **Characterized brane** — had an
      `einmo_suite2` group (`leaves`/`names`) but NO direct T1 unit test; they were exercised
      only end-to-end through the contract. Added
      `foolish_renders_creations_and_characterized_branes`, which also verifies a detail the
      contract alone does not isolate: a creation's DEFINING statement (`'k = ⬤`) still renders
      `⬤` on its own RHS, while a separate statement REFERENCING that named creation elsewhere
      (`j = 'k`) renders its original name — probed against real evaluation before writing the
      assertion. All 13 rows now have both forms of coverage.)
- [x] Confirm the `Detailed`-delegation byte-equality tests from Phase 2 still pass — the
      renderer's growth must not have perturbed the delegating mode.
      (2026-09-03 — `detailed_delegates_for_integer`/`_brane`/`_operator`/`_resolved_search`/
      `_nk`, all still passing, confirmed in this session's every `cargo test -p foolish-ubca2
      --lib -- sequencer` run.)
- [x] Run all tests — old and new — and make sure they all pass correctly. **`einmo_suite/`'s
      three gates must STILL PASS on the old rendering** — if one has moved, the adapter was
      switched early and Movement III has begun by accident.
      (2026-09-03 — `cargo test -p foolish-ubca2 --lib`: 164/164 (both einmo_suite2 gates and
      all three einmo_suite gates green). `cargo test -p foolish-ubca --lib --
      einmo_gate_checked`: still passes, unchanged — sibling scope guard honored. Phase 4
      complete.)

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

- [x] (read §2 of `FOOP-36.md`; re-read Phase 0's recorded "before" test readings)
      (2026-09-03 — Phase 0's readings: `einmo_gate_checked` 1/1 on `foolish-ubca`;
      `einmo_gate_verified` on `foolish-ubca2` 1/1, all 179 cases signed; whole crate 134/134
      at that time.)
- [x] Copy every input from `foolish-ubca2/einmo_suite/input/**/*.foo` (179 files) into
      `foolish-ubca2/einmo_suite2/input/`, preserving the directory structure
      (`foop/<N>/…`, `misc/…`, `regression/…`). **Copy, do not move** — `einmo_suite/` must
      remain intact and passing.
      (2026-09-03 — copied via `cp`, preserving structure; `einmo_suite/input/` re-counted at
      179 afterward, unchanged. `einmo_suite2/input/` now holds 180: the 179 copies plus
      `foop/36/rendering_contract.foo`.)
- [x] Copy `MAPPING.md` too, and add a note at its top recording that this suite's outputs are
      rendered by `Ubca2Sequencer` in `Foolish` mode (FOOP-36), unlike `einmo_suite/`'s.
      (2026-09-03)
- [x] **T10 — coverage parity.** Write a test asserting `einmo_suite2/input/` contains an
      input for EVERY input in `einmo_suite/input/` — same relative paths, same count. Then
      confirm the total: 179 copied plus `foop/36/rendering_contract.foo` from Phase 3 (and
      `foop/36/comprehensive.foo` arrives in Phase 7). **This is the failure mode that matters
      most and is invisible from a green run** — a new suite that quietly tests less than the
      one it replaces.
      (2026-09-03 — `einmo_suite2_has_every_einmo_suite_input`
      (`ubca_snapshot_tester2.rs`): walks both `input/` trees, asserts set difference is empty,
      and pins the exact counts (179 / 180). Passes.)
- [x] **T3 — corpus-wide round-trip.** One unit test walking every
      `einmo_suite2/input/**/*.foo`: evaluate, render in `Foolish` mode, assert the result
      **parses**. Property 1 only — not idempotence — so it stays fast and stays correct for
      non-settling cases (§2.1). **Run this BEFORE generating any output**: it is the cheapest
      check that the renderer survives the whole corpus, and it fails loudly without writing a
      single baseline.
      (2026-09-03 — `einmo_suite2_corpus_wide_foolish_rendering_parses`
      (`ubca_snapshot_tester2.rs`): walks the raw `.foo` files directly with `std::fs`, not
      through einmo's signed-output machinery, so it runs before any output exists. Found 6
      genuine Property 1 violations on first run — see next checkbox.)
- [x] Fix whatever T3 finds. A parse failure is a renderer bug, never a baseline problem.
      (2026-09-03 — 6 failures on first run: `foop/33/boolean/comparison_non_integer.foo`,
      `foop/33/comprehensive.foo`, `foop/42/humanizing_sequencer_formatting_exhaustive_aka_hfs.foo`,
      `foop/62/anchored_search_suite.foo`, `misc/concat_sf_f_more.foo`,
      `misc/unanchored_seek_with_head_tail.foo`. Delegated diagnosis+fix to a subagent, which
      found two shared root causes plus two single-case ones — multi-line inline operands
      swallowing a `!!` comment mid-line (`render_inline`/`render_written_operand_inline` now
      strip per-line comments before joining), FOOP-75 §6's attached-form ambiguity for
      unanchored value-search/index anchors (`is_safe_attached_anchor` falls back to postfix),
      SF/SFF delimiter fusion at nested `<`/`>` boundaries (a disambiguating space), and a
      double-wrapped already-parenthesized regex pattern (FOOP-75 §6.1: the parser stores
      parens in `pattern` itself). Verified independently: read the actual diff, confirmed the
      FOOP-75 §6.1 claim against `docs/foop/FOOP-75.md` line 487 directly, re-ran all 5 new
      regression tests plus the full sequencer suite (26/26) and corpus-wide parse test
      (0 failures). Sibling `foolish-ubca` and `einmo_suite`'s own three gates reconfirmed
      unmoved.
      **Own follow-up finding:** attempted to tighten the SF/SFF space to only the
      cases that actually need it (odd-length delimiter run at the boundary) instead of the
      subagent's unconditional-whenever-adjacent rule; this broke a real case
      (`b = <1 + <<b>> + <c>>`) because safety depends on what an ENCLOSING wrapper appends
      afterward, which a single `render_stay` call cannot see. Reverted to the subagent's
      original (safe) unconditional rule, with a doc comment recording why the narrower
      version is unsound so nobody re-attempts it without re-deriving this.
      **Process incident, corrected in-session:** while re-promoting `foop/36/rendering_contract`
      after this revert, ran `einmo promote output->checked foolish-ubca2/einmo_suite2` with
      NO `--filter`, wrongly assuming it would only touch the one file with a real diff — it
      promoted all 180 cases, including the 179 real corpus cases Phase 6 requires reviewing
      case-by-case before any promotion. Caught immediately (before anything was `git add`ed or
      committed) by checking `find .../checked -name '*.einmo' | wc -l` against expectations.
      Reverted by removing the 179 newly-created `checked/` files (all untracked, so a clean
      `rm -rf` of the specific new directories fully undid it) and reconfirmed
      `einmo_suite2_gate_checked` fails again for the correct reason (179 genuinely missing
      baselines). **Lesson recorded here rather than left implicit: `einmo promote` with no
      `--filter` acts on the WHOLE suite; always pass `--filter <glob>` naming the exact
      case(s) reviewed, never rely on "only the diffed file will move."** This checkbox is
      complete: `einmo_suite2_corpus_wide_foolish_rendering_parses` is green, and
      `einmo_suite2/checked/` correctly holds only `foop/36/rendering_contract` (Phase 3's
      own case) — zero of the 179 corpus cases were left promoted.)

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

### 5c — Dot-form rendering: CANCELLED for this FOOP

- [x] Canceled. The anchored dot form (`b.x` for `b?x`, chaining `a = b.c.d.e.f.g`) was
      implemented and green during this session, then **backed out by human direction
      (2026-09-03)**. It is recorded instead as **`FOOP-36.md` §Proposed Next Steps N2**, under
      the more general **N1 — instrument an additional element on the FIR to aid rendering**,
      which is the real finding: the FIR keeps no record of how a thing was WRITTEN, so the
      sequencer cannot prefer the Foolisher's own spelling among equivalent ones. N1 notes the
      aid can be populated either from the original Foolish compiler or from stepping.
      **The unanchored half already ships in this FOOP** — an unanchored plain-name search
      renders as the bare identifier today. Only the anchored half is deferred.
      (2026-09-03)
  - [-] Anchored backward search renders the dot form `b.asdf`
  - [-] Chained position renders `a = b.c.d.e.f.g`
  - [-] Attached position renders `a =.g b.c.d.e.f` — declined outright; §4.3.1 keeps
        `^`/`$` as the only attachable operators
  - [-] Regenerate and re-review the baselines it would move

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

## Phase 6.5 — Propagate this FOOP's requirements into FOOP-26 and FOOP-46

*§3.2's rendering and FOOP-46's phased design are the same distinction reached from opposite
directions (§3.2.1): Gathering ↔ render the juxtaposition, Joined ↔ render the brane. Because
they already agree, **these sections should be short and easy to write** — a statement of what
rendering needs, and a pointer to §3.2. Do it after Phase 6's review, when the real rendered
output exists and the requirement can be stated from evidence rather than speculation. If a
section turns out to be hard to write, that is the signal that the designs have diverged —
report it rather than forcing it.*

- [ ] (read §3.2 and §3.2.1 of `FOOP-36.md`)
- [ ] Establish relevant tests for this phase: none — it edits documentation only. Confirm at
      the end that `einmo_suite2`'s gates and `foolish-ubca`'s are still green.
- [ ] **Check first whether either FOOP has begun.** `docs/foop/FOOP-26.md` and
      `docs/foop/FOOP-46.md` — read the `begun:` frontmatter and check for a worktree. If
      either is actively being edited, **coordinate with the human before writing into it**
      rather than editing under another agent.

### 6.5a — FOOP-26: what concatenation must still expose for rendering

- [ ] Add a section to `FOOP-26.md` (suggested: under its §4, the concatenation chapter) headed
      so it is clearly an obligation, e.g. **"§4.x TODO — rendering requirements from FOOP-36"**.
- [ ] State the requirement concretely, using the output Phase 6 actually produced. Frame it
      as what rendering *needs*, not as a constraint imposed on FOOP-26 — the designs agree:
  - [ ] FOOP-36 §3.2 renders a **merged** concatenation as the merged brane (`{a=1, b=2, c=3}`)
        and an **unmerged** one as the juxtaposition with each constituent recursively
        simplified (`{a=1, aa=3}{b=notfound}notˍfoundˍbrane{d=3}`).
  - [ ] It distinguishes them by `hs_concatenation()` returning `(elements, merged)` with
        `merged.is_some()`.
  - [ ] Therefore: **after concatenation becomes an operator, a rendering caller must still be
        able to ask "did this merge, and if so what is the brane; if not what are the
        constituents?"** Name that requirement; FOOP-26 chooses how to satisfy it.
  - [ ] Note that `⨃` is never emitted — it is not input syntax — so whatever FOOP-26 does must
        not depend on a marker the renderer refuses to print.
- [ ] Include a pointer back to `FOOP-36.md` §3.2/§3.2.1 so the reasoning is one click away.

### 6.5b — FOOP-46: telling a merged concatenation from a plain brane

- [ ] Add the corresponding section to `FOOP-46.md`, keyed to its **§4** ("the operator's
      `ubc_children` should be a brane"), e.g. **"§4.x TODO — rendering requirements from
      FOOP-36"**.
- [ ] Lead with the convergence (§3.2.1): FOOP-46's **Gathering** phase is what §3.2 renders as
      the juxtaposition, and its **Joined** phase is what §3.2 renders as the brane. The
      rendering is the natural display of what §4 builds; §4 needs no new mechanism for it,
      since `settled_constanic_result`/`value()` already return the brane once constanic — as §4's own
      text observes.
- [ ] Then state the one detail §4's implementation must settle:
  - [ ] §4's option 1 populates `ubc_children` with a `FirSpec::Brane` and **deletes
        `FirSpec::ConcatHelper` entirely**. §3.2 renders a merged concatenation (`A B`) and a
        plain brane (`{…}`) **differently**, so a renderer — and a reader of the output — must
        still be able to tell them apart.
  - [ ] Either the FIR keeps something that says so, or §3.2 is amended to render them alike.
        **Either answer is fine; the requirement is that it be decided rather than lapse.**
- [ ] Include the same pointer back to `FOOP-36.md` §3.2/§3.2.1.

### 6.5c — Record and report

- [ ] Update `FOOP-36.md` §3.2.1 to record that the sections were written, with their section
      numbers, so a later reader can follow the chain in both directions.
- [ ] Check `docs/foop/INDEX.md`'s Track 6 "Interaction to watch" paragraph still describes the
      situation accurately; update it if these sections changed the picture.
- [ ] **Report to the human**: what was written into each FOOP, and — importantly — **whether
      writing it revealed that §3.2 itself needs to change**. If FOOP-46's §4 direction makes
      the merged/unmerged distinction unrenderable, that is a finding about THIS FOOP, not just
      a note for that one, and §3.2 should be revised here rather than left to conflict later.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

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
      `/home/agent/yolo/foolish_worktrees/foop-36-foolish-rendering-sequencer` and committed to
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
- [ ] **Phase 6.5's sections exist** in `FOOP-26.md` and `FOOP-46.md`, and `FOOP-36.md`
      §3.2.1 records their section numbers. If either FOOP was being actively edited and the
      human deferred the write, say so explicitly in the merge report — an undone propagation
      is a known debt, not a silent one.
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
          `cd /home/agent/yolo/foolish_worktrees/foop-36-foolish-rendering-sequencer` and ask
          them to review `einmo_suite2` BEFORE checking the parent checkbox. Say plainly that
          this FOOP **replaces `einmo_suite` with `einmo_suite2`** — 179 inputs copied across
          and re-rendered, `cargo test` now pointed at the new suite, the old one left frozen
          and still green for diffing, and **not** removed. The review question is "is this
          valid, predictable Foolish?", not "does it match".
  - [ ] Repair ALL tests in `jia` at `/home/agent/yolo/foolish` if the merge broke any
- [ ] Cleanup `/home/agent/yolo/foolish_worktrees/foop-36-foolish-rendering-sequencer`
  - [ ] Check that this `.plan.md` has all but Cleanup checkboxes completed
  - [ ] Remove `/home/agent/yolo/foolish_worktrees/foop-36-foolish-rendering-sequencer`
  - [ ] This is the last sub-task checkbox to be checked in this block

---

## Last Updated

**Date**: 2026-09-02
**Updated By**: Codex / GPT-5.6
**Changes**: FOOP-56 vocabulary pass: updated Phase 0.5 and active renderer-plan references to
the implemented predicates, `settled_constanic_result`, `step_to_constanic`, and the
ECONSTANIC conclusive-vs-constanic regression test.

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
