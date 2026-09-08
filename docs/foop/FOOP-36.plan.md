# FOOP-36.plan — foolish-rendering-sequencer

**Read `docs/foop/FOOP-36.md` before executing this plan.** The plan is derived from the
specification and assumes its context; section pointers below (`§1`, `§2.1`, `§3`…) refer to
that file.

**Worktree variables, expanded:**

```
WORKTREE_ORIGIN_BRANCH  = jia
WORKTREE_ORIGIN_PATH    = /home/agent/yolo/foolish
WORKTREE_BRANCH_NAME    = foop-36-foolish-rendering-sequencer
WORKTREE_FULL_FS_PATH   = /home/agent/yolo/foolish_worktrees/foop-36-foolish-rendering-sequencer
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
  - [x] Markdown File Update Protocol followed: AGENTS.md's "## Last Updated

**Date**: 2026-09-07

**Updated By**: Claude Code / claude-opus-5

**Changes**: **FOOP-36 COMPLETE — merged to `jia` as `d82a33b0`** (--no-ff, 33 commits) and the
worktree removed. The Phase 8 STOP was approved by the human after they ran the interactive
`checked → verified` promotion (181 cases, `aa22b82d`) and asked for a fault-injection sanity
check of the gates: perturbing three inputs (`3 + 4`→`3 + 9`, `6 * 7`→`6 * 8`, `5`→`99`) failed
BOTH `einmo_suite2_gate_checked` and `einmo_suite2_gate_verified`, each reporting **INPUT and
OUTPUT** divergence per case, so the suite detects change rather than passing vacuously. Inputs
restored and all gates green. `einmo_suite2_gate_verified` was added and passes with **no
`#[ignore]`** — the reason it was withheld until the human acted. Before removing the worktree,
verified `jia..foop-36-foolish-rendering-sequencer` was empty and the worktree had no uncommitted
changes. Post-merge `cargo test --workspace` on `jia`: **791 passed, 0 failed** (foolish-core 133,
foolish-parser 84, foolish-cli 62, foolish-ubca 328, foolish-ubca2 184); the sole "ignored" is the
pre-existing `evaluator::step_until` doctest. Follow-on work stays recorded in the spec's Proposed
Next Steps: **N5** (arrow indexers) and **N6** (FIR equality — its own FOOP, which also refreshes
`EQUIVALENCE.md` and brings it out of `vintage_legacy/`), with **Q9** blocking N6, not this FOOP.
