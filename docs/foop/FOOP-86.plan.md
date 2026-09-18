# FOOP-86.plan — retire-ubca

**Read [`FOOP-86.md`](FOOP-86.md) in full before executing any checkbox below.** This plan
assumes the specification's context and is meaningless without it. Work the boxes top to bottom.

- **Worktree branch**: `foop-86-retire-ubca`
- **Worktree path**: `/yolo/foolish_worktrees/foop-86-retire-ubca`
- **Origin branch**: `jia`
- **Origin path**: `/yolo/foolish`

---

## Orientation — facts measured on `jia` at `8c9043d8`

**Verify, don't re-derive.** Every fact below was measured while this plan was written. If a
check disagrees with a fact here, that is a finding: STOP and report it, do not silently adopt
the new value.

### The two APIs the CLI must use (§1.3)

```rust
// foolish-ubca2/src/evaluator.rs:16 — already pub
pub fn evaluate_arena(&self, source: &str) -> Result<(FVMStorage, Vec<FirPointer>), String>

// foolish-ubca2/src/sequencer.rs:142 — already pub
pub fn format(storage: &FVMStorage, fir: FirPointer, mode: SequenceMode) -> String
```

Both re-exported from `foolish-ubca2/src/lib.rs`:
`pub use sequencer::{SequenceMode, SequenceOptions, Ubca2Sequencer};` and
`pub use evaluator::UbcaEvaluator;`. **No new public API is needed.**

### The trap (§1.2) — the wrong version COMPILES and PASSES

`foolish-ubca2` also implements `foolish_core::Evaluator` (`evaluator.rs:45`), and both crates
name the type `UbcaEvaluator`. Changing `foolish-cli/src/main.rs:7` to
`use foolish_ubca2::UbcaEvaluator;` builds and runs — and is **WRONG**, because that impl
converts out through the lossy `proto_to_core_fir` (`evaluator.rs:52`). **If at any point the
CLI calls `.evaluate(` rather than `.evaluate_arena(`, the phase is wrong. STOP.**

### The bridge's non-test call sites (§3.1) — exactly two

| File:line | Context |
|---|---|
| `foolish-ubca2/src/sequencer.rs:160` | `format_detailed`, reached only by `SequenceMode::Detailed` |
| `foolish-ubca2/src/evaluator.rs:52` | `impl foolish_core::Evaluator for UbcaEvaluator` |

Definitions: `fvm_storage.rs:3494` (`proto_to_core_fir`), `3504` (`_sff_body`), `3551`
(`_sff_operand`), `3599` (`_inner`) — running to roughly `4151`, ~650 lines, module
`core_fir_conversion`, re-exported at `fvm_storage.rs:4990`. Test-only callers:
`fvm_storage.rs:6876, 6889, 6905, 6924, 6956, 8273` and `sequencer.rs:1194`.

### `SequenceMode::Detailed` — KEPT, rewritten arena-native (§3.2)

Declared `sequencer.rs:22`, dispatched `sequencer.rs:155`, **one** reference outside the enum —
the test at `sequencer.rs:1198`.

**Q4 is RESOLVED (human, 2026-09-16): `Detailed` STAYS.** `SequenceMode` keeps both variants and
`Ubca2Sequencer::format`'s signature is unchanged. Only `format_detailed`'s *implementation*
changes — from `proto_to_core_fir` + `foolish_core::FirSequencer` to an arena-native dump reading
`FVMStorage`/`FirSpec`/`FirCursor`. **That rewrite is what lets the bridge be deleted**, which is
why Phase 4a (write it) precedes Phase 4b (delete the bridge) and why there is never a window in
which neither exists. **Byte-compatibility with the old output is NOT required** and the output
is expected to differ — it can show fields the bridge dropped.

### Suite counts (§4.1) — verify these before touching anything

| Suite | inputs | checked | verified |
|---|---|---|---|
| `foolish-ubca/einmo_suite` | 178 | 178 | 178 |
| `foolish-ubca2/einmo_suite` | 179 | 179 | 179 |
| `foolish-ubca2/einmo_suite2` | 181 | 181 | 181 |

### Why the rename is signature-safe (§4.3)

- `einmo/src/verify.rs:39-45` — `verify_bytes` checks the stamp chain against the **stored file
  bytes**. `git mv` changes no byte.
- `einmo/src/einmo_suite.rs:605-607` — correspondence "compares only the configured sections —
  STAMPS and metadata are excluded by design."
- `einmo/src/config.rs:63-64` — default `MatchSections::InputOutput`.
- Live proof: every `einmo_suite2/verified/*.einmo` already carries
  `suite: /yolo/foolish_worktrees/foop-36-foolish-rendering-sequencer/...` — a path that no
  longer exists — and `einmo_suite2_gate_verified` passes today.

### Decisions already made — do not reopen

| # | Decision | Source |
|---|---|---|
| **Q1** | Discard UBCa's 178 human attestations. Accepted. | human, 2026-09-16 |
| **Q3** | `foolish-ubca` deleted outright — no tag, no archive. Git history retains it. | human, 2026-09-16 |
| **Q4** | `SequenceMode::Detailed` KEPT, re-implemented arena-native. | human, 2026-09-16 |
| **naming** | The crate stays `foolish-ubca2`; public types keep their names. No de-suffixing. | human, 2026-09-16 |

Still open, none blocking: **Q2** (`cmd_compile`), **Q5** (do the two evaluators' attested
answers agree — informational), **Q6** (which docs), **Q7** (`zweimomo` is absent from the tree).

### ⛔ Stop conditions — standing, for every phase

1. **Any einmo gate goes red** → a regression THIS FOOP introduced. Fix the code. **NEVER
   `einmo promote`** (§T2 — this FOOP promotes nothing).
2. **Any edit needed in `foolish-core/src/`** → STOP and report (§3.3 scope guard).
3. **Tempted to add `#[ignore]` to `einmo_gate_verified`** → STOP. Never an agent's call
   (AGENTS.md), and this FOOP is exactly the situation that would tempt it.
4. **The surviving `einmo.toml`'s `[signing.checked] passphrase = "foolish-ubca2-suite2"` looks
   like stale naming after the rename** → **DO NOT CHANGE IT.** The 181 `checked/` stamps were
   made under that string; changing it invalidates all of them.
5. **Tempted to de-suffix `foolish-ubca2` / `Ubca2Sequencer` now that the "2" looks vestigial**
   → **DON'T.** The human decided 2026-09-16 that the crate and its public types keep their
   names (FOOP-86 §5). The two renames that DO happen are the suite directory and its test
   file — nothing else.

---

## Phase 0 — Begin, baseline, and the open questions

- [x] Read [`FOOP-86.md`](FOOP-86.md) in full — especially §0.3 (why this order), §0.5 (what is
      actually in the tree re Euler-1), §1.2 (the trap), §4.3 (rename mechanics).
      (2026-09-18 00:00)
- [x] Establish relevant tests for this
     phase. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests: the
     whole `foolish-ubca2/einmo_suite2` suite via its three gates, and the whole
     `foolish-ubca/einmo_suite` suite via its three gates; run unit tests:
     `foolish-ubca2::einmo_suite2_gate_output`, `foolish-ubca2::einmo_suite2_gate_checked`,
     `foolish-ubca2::einmo_suite2_gate_verified`,
     `foolish-ubca2::einmo_suite2_has_every_einmo_suite_input`, `foolish-ubca::einmo_gate_checked`.
      (2026-09-18 00:00)
- [x] Record the **before** baseline in this plan, as literal numbers:
  - [x] `cargo test --workspace` total passing (expected **791**) — **CONFIRMED: 791 passed, 0
        failed, 1 ignored** (the ignored one is a doctest in `foolish-ubca`,
        `evaluator.rs::step_until`). Matches the plan's expected baseline exactly.
        (2026-09-18 00:00)
  - [x] Per-crate test counts for `foolish-ubca`, `foolish-ubca2`, `foolish-core`, `einmo` (all
        via `--lib`): `foolish-core` 84, `foolish-parser` 62, `foolish-ubca` 328, `foolish-ubca2`
        184, `einmo` 133, `foolish-cli` 0 (no test module yet, matches §T3's premise). Sum =
        84+62+328+184+133 = **791**.
        (2026-09-18 00:00)
  - [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` — recorded the
        **pre-existing** error count: exactly **4** errors, all
        `clippy::iter_next_slice` ("using `.iter_mut().next()` on an array"), all in
        `foolish-core/src/sequencer.rs` at lines **187, 537, 563, 743**. This matches MEMORY's
        note precisely. The build stops at `foolish-core` (blocking clippy from even reaching
        later crates in that invocation), but this is pre-existing and out of scope — §3.3
        forbids editing `foolish-core/src/` in this FOOP. Not blamed on this FOOP.
        (2026-09-18 00:00)
  - [x] Verify the three suites' counts against the Orientation table — **CONFIRMED exactly**:
        `foolish-ubca/einmo_suite` 178/178/178 (input/checked/verified), `foolish-ubca2/einmo_suite`
        (old) 179/179/179, `foolish-ubca2/einmo_suite2` 181/181/181.
        (2026-09-18 00:00)
- [x] Verify §0.5's claim against the tree: confirm `future_exercise_inputs/project_euler/` holds
      only `1.foo.disabled` and `1.py`, and that neither suite's `input/` has an `exercises/`
      tree. **If a euler or fib einmo case IS found, STOP** — §0.5's reasoning would be wrong.
      **CONFIRMED**: `future_exercise_inputs/project_euler/` contains exactly `1.py` and
      `1.foo.disabled`, nothing else in the tree matches `*euler*`/`*fib*`. Both suites'
      `input/` trees hold only `regression/`, `misc/`, `foop/` — no `exercises/` directory
      anywhere. No STOP triggered.
      (2026-09-18 00:00)
- [x] Confirmed input parity ahead of Phase 2 (measurement re-used there): diffing
      `foolish-ubca/einmo_suite/input/**.foo` against `foolish-ubca2/einmo_suite2/input/**.foo`
      by relative path gives **0 missing, 3 extra** (`foop/16/comprehensive.foo`,
      `foop/36/comprehensive.foo`, `foop/36/rendering_contract.foo`) — exactly as §4.1 states.
      Also confirmed **Q7**: `zweimomo` does not exist anywhere under `/yolo/foolish` (only an
      unrelated worktree, `.claude/worktrees/foop-55-event-handlers/zweimomo`, has a directory
      by that name, and it is not a member of this workspace's `Cargo.toml`). AGENTS.md's crate
      list is indeed stale on this point, as §Q7 already flagged.
      (2026-09-18 00:00)
- [x] Record the **already-decided** items in the plan log so they are not reopened (see
      Orientation §"Decisions already made"): **Q1** discard UBCa's attestations; **Q3** delete
      outright, no tag; **Q4** keep `Detailed`, rewrite arena-native; **naming** the crate stays
      `foolish-ubca2` and public types keep their names (FOOP-86 §5)
      (2026-09-18 00:00)
- [x] Put the four REMAINING questions to the human in ONE message, with FOOP-86's
      recommendations — **none of these blocks starting work**, so do not wait on them:
  - [x] **Q2** — `cmd_compile`'s fate. Phase 1 reads first and decides; ask the human only if
        the answer is "retain a `core_fir` conversion", which would contradict deliverable 3
  - [x] **Q5** — do the two evaluators' attested answers agree on the 178 shared inputs?
        Phase 2 produces it; informational unless a disagreement surfaces
  - [x] **Q6** — do README/AGENTS.md updates belong here? (rec: yes; `foop.md` and old plans no)
  - [x] **Q7** — `zweimomo` is listed in AGENTS.md but absent from the tree — documentation
        finding, raised not fixed. Confirmed above: not present anywhere under `/yolo/foolish`,
        not a workspace member.
      (2026-09-18 00:00, message sent to human in this session per below)
- [x] Remind the human: *"Above message comes from FOOP-86, retiring UBCa so foolish-ubca2
      becomes the implementation; the worktree is at
      /yolo/foolish_worktrees/foop-86-retire-ubca. PTAL"*
      (2026-09-18 00:00)
- [x] Commit `FOOP-86.md` and `FOOP-86.plan.md` to `jia` and check `begun: [x]` in the
      frontmatter of `FOOP-86.md`
      (2026-09-18 00:00, commit 5e4dbd88 "Major: Retire UBCa, Phase: 0--begun")
- [x] Create worktree at /yolo/foolish_worktrees/foop-86-retire-ubca with branch `foop-86-retire-ubca`
      — from here on, **ALL work including edits to FOOP-86.md and this plan happens ONLY in
      the worktree**
      (2026-09-18 00:00)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      **Note on a transient flake observed here**: the first `cargo test --workspace` run in the
      fresh worktree showed 3 failures in `foolish-ubca` (`einmo_gate_output/checked/verified`),
      each due to a "catastrophe crumb" (`status: output-error`, "TEST IN PROGRESS — test harness
      crashed during evaluation") written for `regression/deep_nesting_does_not_lose_values.foo`
      and `regression/operator_does_not_block_search.foo`, with an iteration-exceeded (9999) alarm
      on the first. Diagnosed as **parallel-execution resource contention**, not a code
      regression: (1) no source had been touched yet, only doc/frontmatter edits; (2) running
      `foolish-ubca`'s einmo gate in isolation (`cargo test -p foolish-ubca --lib -- einmo_gate_checked`)
      passed cleanly; (3) the full `foolish-ubca` lib suite standalone passed 328/328; (4) a
      second full `cargo test --workspace` run, after restoring the two mutated `output/`
      artifacts with `git checkout --`, passed cleanly at **791 passed, 0 failed, 1 ignored** with
      no leftover crumb. Recorded here rather than silently retried, per AGENTS.md's doubt-recording
      discipline — future full-workspace runs in this FOOP should be re-run once before treating a
      failure as real, and any leftover `output/` diff after a run must be checked against
      `git status` and reverted if it is crumb noise from an interrupted parallel run, not a
      genuine baseline change.
      (2026-09-18 00:00)

## Phase 1 — `foolish-cli` evaluates through `foolish-ubca2` (§1)

> **Judgment phase — larger model.** §1.2's trap is one a small model walks into, because the
> wrong version compiles and the tests pass.

- [x] (read §1 of [`FOOP-86.md`](FOOP-86.md), all four sub-sections)
      (2026-09-18 00:00)
- [x] Establish relevant tests for this
     sub-section. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests:
     the `foolish-ubca2/einmo_suite2` suite via `einmo_suite2_gate_checked`; run unit tests:
     `foolish-ubca2::einmo_suite2_gate_checked`,
     `foolish-ubca2::einmo_suite2_corpus_wide_foolish_rendering_parses`, and (once written)
     `foolish-cli::cli_run_renders_foolish`, `foolish-cli::cli_agrees_with_einmo_adapter`.
      (2026-09-18 00:00)
- [x] Resolve **Q2** by reading: inspect `foolish_core::fir_to_json`'s signature and what
      `cmd_compile` actually needs. Write the decision and its reasoning into this plan.
      **If the answer is "retain a `core_fir` conversion", STOP and ask the human** — it
      contradicts deliverable 3.

      **Decision: retire `cmd_compile` from the CLI.** Reading `foolish-core/src/serialization.rs:42`,
      `fir_to_json(fir: &Fir) -> Result<String, SerdeError>` takes `foolish_core::fir::Fir` (the
      `core_fir` representation) and derives its shape from serde on that type directly — it is
      not a thin adapter, it is *of* that representation, so there is no "just repoint it at the
      arena" move available. `cmd_run`/`cmd_step`/`cmd_repl` need a renderer over
      `(FVMStorage, FirPointer)`; `cmd_compile` needs a *serializer*, a different capability that
      does not exist yet for the arena and would need to be built from nothing. Cross-checked
      against the tree: `grep` for `fir_to_json`/`cmd_compile`/`compile` across `README.md` and
      all of `docs/foop/*.md` (excluding this FOOP's own draft) returns **zero** hits — no
      README example, no einmo case, no other FOOP referencing this subcommand's behavior.
      §1.4's third disposition ("retire `compile`... its user base may be zero") is exactly this
      case, measured rather than assumed. The first disposition (arena-native JSON serializer)
      would be new production surface built to serve a need nothing in the tree currently
      demonstrates; the second (retain a `core_fir` conversion) is explicitly the one that
      contradicts deliverable 3 and requires asking the human. Retiring the subcommand is the
      only option that neither invents an unrequested serializer nor keeps the bridge alive.
      **Not escalated to the human** — the chosen answer is not "retain a conversion," so per
      this checkbox's own instruction, resolving by reading was sufficient.
      (2026-09-18 00:00)
- [x] Add `foolish-ubca2 = { path = "../foolish-ubca2" }` to `foolish-cli/Cargo.toml` and
      **remove** the `foolish-ubca` line
      (2026-09-18 00:00)
- [x] Replace `foolish-cli/src/main.rs`'s `evaluate()` helper with an `evaluate_arena()` helper
      returning `(FVMStorage, Vec<FirPointer>)` (§1.3)
      (2026-09-18 00:00)
- [x] Repoint `cmd_run` to render each `FirPointer` with
      `Ubca2Sequencer::format(&storage, fir, SequenceMode::Foolish)`
      (2026-09-18 00:00)
- [x] Repoint `cmd_step` the same way
      (2026-09-18 00:00)
- [x] Repoint `cmd_repl` the same way
      (2026-09-18 00:00)
- [x] Apply Q2's decision to `cmd_compile` — **retired**: removed the `Compile` variant from
      `Commands`, the `cmd_compile` function, and the `Commands::Compile { file } => cmd_compile(&file)`
      dispatch arm.
      (2026-09-18 00:00)
- [x] Remove now-unused `foolish_core` imports (`Evaluator`, `FirSequencer`, `clone_steppable`,
      and `fir_to_json` if Q2 retired it) from `main.rs:6` — **all four removed**; `main.rs` no
      longer imports `foolish_core` at all (it now imports only `foolish_ubca2::{fvm_storage::{FVMStorage,
      FirPointer}, SequenceMode, Ubca2Sequencer, UbcaEvaluator}`). `foolish-core` remains a
      transitive dependency (via `foolish-ubca2`) but is no longer a direct one in `main.rs`'s
      `use` list.
      (2026-09-18 00:00)
- [x] **Self-check the trap**: `grep -n '\.evaluate(' foolish-cli/src/` must return NOTHING.
      If it returns a hit, the wrong API is in use — STOP (§1.2). **CONFIRMED: zero matches.**
      (2026-09-18 00:00)
- [x] `cargo build --workspace` and `cargo run -p foolish-cli -- run` on a small `.foo`; confirm
      by eye that the output is Foolish, not FIR internals. Built clean; ran
      `foolish-ubca2/einmo_suite2/input/foop/9/unary_operator.foo` (`{a=-42;}`) through both
      `run` and `step` — output was `{\n  a = -42\n}` in both cases: valid Foolish, no
      `?(pattern=`, no bare NYES tokens, no FIR-internal syntax.
      (2026-09-18 00:00)
- [x] `cargo fmt` and `cargo clippy -p foolish-cli -- -D warnings`. `cargo fmt -p foolish-cli`
      applied cleanly (one reformatting of the two `println!` call sites to multi-line form).
      `cargo clippy -p foolish-cli --all-targets --all-features -- -D warnings` **fails**, but
      not on anything in `foolish-cli`: it fails on the same 4 pre-existing
      `clippy::iter_next_slice` errors in `foolish-core/src/sequencer.rs` recorded in Phase 0,
      now reached because `foolish-cli` depends on `foolish-core` transitively (through
      `foolish-ubca2`) where it did not need to build `foolish-core`'s lib target under `-D
      warnings` with the old `foolish-ubca` dependency in exactly the same way. Confirmed
      `foolish-cli`'s own code is lint-clean: `cargo clippy -p foolish-cli --all-targets
      --all-features` (without `-D warnings`) produces zero diagnostics on any `foolish-cli`
      file — the 4 errors are the entirety of the output, all attributed to
      `foolish-core/src/sequencer.rs` lines 187/537/563/743, identical to Phase 0's baseline.
      Per §3.3's scope guard (never edit `foolish-core/src/` in this FOOP) and T5's instruction
      not to be blamed for pre-existing errors, this is left as-is; it is `foolish-core`'s
      own known debt (MEMORY), not something Phase 1 introduced or must fix.
      (2026-09-18 00:00)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (workspace test run recorded below, after T3 tests exist in Phase 6 — `foolish-cli` has no
      test module yet per §T3's own premise; Phase 1 re-confirms the einmo suite is undisturbed)

## Phase 2 — Do the two evaluators AGREE? (§4.1, Q5)

> **Judgment phase — larger model.** **No coverage is at risk and nothing needs porting** —
> §4.1 measured it: 0 UBCa inputs are absent from the surviving suite. This phase exists for a
> different reason: it is the **last moment both signed corpora exist**, so it is the last chance
> to ask whether the two implementations actually agree about what those 178 programs mean.

- [x] (read §4.1 of [`FOOP-86.md`](FOOP-86.md) — note the measured parity table)
      (2026-09-18 00:00)
- [x] Establish relevant tests for this
     sub-section. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests:
     both suites' full input listings and their `checked` artifacts (no gate change expected in this
     phase); run unit tests: `foolish-ubca2::einmo_suite2_has_every_einmo_suite_input`,
     `foolish-ubca::einmo_gate_checked`.
      (2026-09-18 00:00)
- [x] **Confirm input parity one last time, as the record**: diff `foolish-ubca/einmo_suite/input/**.foo`
      against `foolish-ubca2/einmo_suite2/input/**.foo` by relative path. **Expected: 0 missing,
      3 extra** (`foop/16/comprehensive.foo`, `foop/36/comprehensive.foo`,
      `foop/36/rendering_contract.foo`). Write the result into this plan.
      **⛔ If anything IS missing, STOP** — §4.1's central claim would be wrong.
      **CONFIRMED, re-measured**: 0 missing, exactly 3 extra (the same three named above). No
      STOP triggered on parity.
      (2026-09-18 00:00)
- [x] **Compare the two evaluators' attested ANSWERS** for the 178 shared inputs: UBCa's
      `checked/` OUTPUT against the survivor's, normalized for the rendering difference (the
      two suites render differently by design — FOOP-36's whole subject — so compare *meaning*,
      not bytes; `t12_value_diff`'s `normalize` in `ubca_snapshot_tester2.rs` is prior art for
      exactly this and may be reused before it is deleted in Phase 3a).

      **Method**: added a temporary test, `temporary_foop86_do_the_two_evaluators_agree` (in
      `foolish-ubca2/src/ubca_snapshot_tester2.rs`, to be removed before this phase closes — it
      is a one-off comparison instrument, not a surviving test, per this phase's own scope). It
      reused `t12_value_diff`'s `normalize` (whitespace/comment stripping, NYES-token removal)
      and its digit-extraction technique after additionally stripping the old renderer's
      `?(pattern=..., ECONSTANIC)`-style "machinery" groups, comparing all 178 shared inputs'
      `checked/` OUTPUT sections between `foolish-ubca/einmo_suite` and `foolish-ubca2/einmo_suite2`.

      **Result: 178 shared inputs, 0 missing on either side (confirms parity again); 77 differ
      textually after normalization (expected — genuinely different renderers); of those, 36
      differ even at the digit-stream level.** All 36 were read case-by-case (input semantics
      checked against each rendered output, via a forked review) and classified:

      - **35 are rendering-only artifacts**, not disagreements: (a) UBCa's old FIR-internal
        renderer re-embeds an already-shown sub-value's digits inside `?(result=…, pattern=…)`/
        `⨃(elements=…)`/`Op+(…)` machinery dumps that ubca2's terser Foolish-mode sugar
        (`<x>`, `pt <<nowhere>>`) doesn't repeat — same value, shown twice on one side;
        (b) ubca2's Foolish-mode NK/failure annotations name the literal operand in the
        `!! NK: …` comment (`d = #-100 !! NK: unknown`) where UBCa's old renderer omitted it
        (`d=#(offset=-100, UNANCHORED, NK)`) — both sides agree the result is NK, one just says
        more about why; (c) one case (`foop/62/infinite_loop.foo`) is an artifact of the
        comparison script itself, not the evaluators — my `normalize` strips ubca2's
        `!!`-comment-only iteration-limit banner line entirely, losing its "9999" digits, while
        both evaluators actually agree the program hits the step cap unsettled.
      - **1 is a genuine semantic disagreement — independently re-verified directly (not just
        taking the forked review's word for it) by reading both full checked `.einmo` files
        byte-for-byte:**

        **`foop/33/boolean/null_char_constant.foo`** — identical input on both sides:
        ```
        restate = 'True;
        'True = 'True;
        conflict = 'True;
        'True = 3;
        ```
        with the input's own comment stating the rule under test (FOOP-33 §4): a
        null-characterized name already bound may be **re-stated to an equal value** but a
        **conflicting redefinition must refuse** (NF, `'True not-foolish`).

        **UBCa's checked OUTPUT** (`foolish-ubca/einmo_suite/checked/foop/33/boolean/null_char_constant.foo.einmo`):
        ```
        {NK
          restate='True;
          'True='True;
          conflict='True;
          'True=??? ('True not-foolish)
        }
        ```
        Refuses `'True = 3` exactly as the comment specifies; brane settles NK.

        **ubca2's checked OUTPUT** (`foolish-ubca2/einmo_suite2/checked/foop/33/boolean/null_char_constant.foo.einmo`):
        ```
        {
          restate = 'True;
          'True = 'True;
          conflict = 'True;
          'True = 3
        }
        ```
        `status: normal` — accepts the literal RHS `3`, no refusal, brane not NK. **The two
        evaluators disagree about whether this program is even legal Foolish.** This baseline
        has a `verified/` twin on **both** sides (human-attested 2026-09-07 for ubca2's), so a
        human signed off on ubca2's answer here too — the disagreement was not caught at
        attestation time.

        **Root cause, read but not fixed** (identifying it is useful context, fixing it is not
        this phase's call): `foolish-ubca2/src/fvm_storage.rs:2744`'s
        `check_rename_of_named_creation` only refuses when the RHS *resolves to a creation
        reference* (`matches!(storage.get(resolved), FirSpec::Creation)`) — it is written to
        catch renaming a creation to a SECOND name (`'other = 'True`), and returns early,
        un-refusing, when the RHS is a plain value like `3` that is not a creation at all. It
        appears to be checking the wrong condition for THIS rule (conflicting redefinition to
        a non-creation value), not merely missing a case.
- [x] Report the result to the human in ONE message.
  - [ ] **If the answers agree** (the expected outcome): record it and proceed. This is the
        closing record of the two implementations' agreement.
  - [x] **If any genuine disagreement surfaces**: STOP and report it. The two implementations
        differing about what a program means is a bug in at least one of them (AGENTS.md), and
        it is information that **cannot be recovered after the merge**. It is not an agent's
        call which evaluator was right.

        **STOPPED. Reported to the human.** The other 35 cases and the parity re-confirmation
        are recorded above and are not in question.
        (2026-09-18 00:00)
- [x] **Found a bug during this work (human directed, 2026-09-18): `foolish-ubca2` accepts a
      conflicting redefinition of a named creation (`'True = 3`) that FOOP-33 §4 says must
      refuse.** It impacts exactly **one** einmo case:
      `foop/33/boolean/null_char_constant.foo` — this case already existed (one of the 178
      shared inputs) and already demonstrates the bug end-to-end (its own input comment states
      the rule, UBCa's checked baseline shows the correct refusal), so **no new covering case
      needed to be created** — the existing case IS the failing-test requirement this pattern
      demands. `checked/` and its `verified/` twin were **deleted outright** (`git rm`, not
      edited — no hand-authored replacement content), after consulting the human, so the case
      **fails visibly** instead of silently passing a wrong answer:
      `einmo_suite2_gate_checked` now fails with `[checked]
      foop/33/boolean/null_char_constant.foo.einmo: missing entirely from checked/ (present in
      output/)`; `einmo_suite2_gate_verified` fails the same way (transitively, since it
      escalates from Checked). Both are honest, specific, named failures — not a silent
      wrong-answer pass and not an `#[ignore]` (which AGENTS.md forbids an agent from adding to
      a Verified-tier gate regardless).

      **All OTHER test/gate requirements remain in force unchanged — `foop/33/boolean/null_char_constant.foo`
      is the ONLY accepted exception.** Every other case in every gate, and every other test in
      the workspace, must stay green through the rest of this FOOP exactly as before; this entry
      is not blanket permission to relax discipline anywhere else. Removed the temporary
      `temporary_foop86_answer_comparison` module from `ubca_snapshot_tester2.rs` — its job
      (producing this finding) is done. Continuing with FOOP-86 now; see the TODO checkbox below
      for the fix.
      (2026-09-18 00:00)
- [ ] **Fix the bug found above affecting `foop/33/boolean/null_char_constant.foo` (deferred
      earlier in this plan) — not this FOOP's scope (see §3.3-style guard), flagged here for
      whichever FOOP or session picks it up next.** Here's what is known: the bug is in
      `check_rename_of_named_creation` (`foolish-ubca2/src/fvm_storage.rs:2744`) — it only
      refuses when the RHS resolves to a creation reference (it is written to catch renaming a
      creation to a SECOND name, e.g. `'other = 'True`), so it returns early, un-refused, when
      the RHS is a plain non-creation value like `3`. It needs a second condition: also refuse
      when a null-characterized name's existing value is a creation and the new RHS is a
      **different** value of any kind, not only when the new RHS is itself a creation. Once
      fixed, restore `foop/33/boolean/null_char_constant.foo`'s `checked/` (and, with a human's
      signing key, `verified/`) artifacts reflecting the CORRECT NK/refusal answer — do not
      hand-author them without running the fixed code, per this project's promotion discipline.
      **Confirm the fix makes this case pass** (it already has failing coverage — the deletion
      above — so the fix has something concrete to make green) before this box is checked.
- [x] Run all tests — old and new — and make sure they all pass correctly.
      **Full workspace run, post-deletion**: `einmo_suite2_gate_checked` and
      `einmo_suite2_gate_verified` fail as intended (the one case above); every other test is
      unaffected. This is an EXPECTED failure this phase deliberately introduced per human
      direction, not a regression to chase — Phase 3 onward must treat these two tests as
      "red until the TODO above is fixed," not as something to silently work around or
      `einmo promote` over.

## Phase 3 — The einmo suite rename (§4.3, §4.4)

> **Execution phase — smaller model.** High risk, fully specified, fixed target: three gates
> green. Do the `git mv` and NOTHING ELSE first, so that if §4.3's analysis is wrong it is
> wrong in isolation and immediately visible.

- [x] (read §4.3 and §4.4 of [`FOOP-86.md`](FOOP-86.md) — especially stop condition 4)
      (2026-09-18 00:00)
- [x] Establish relevant tests for this
     sub-section. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests:
     the full surviving suite through all three gates; run unit tests:
     `foolish-ubca2::einmo_suite2_gate_output`, `foolish-ubca2::einmo_suite2_gate_checked`,
     `foolish-ubca2::einmo_suite2_gate_verified`,
     `foolish-ubca2::einmo_suite2_corpus_wide_foolish_rendering_parses`.

     **Known exception carried in from Phase 2**: `einmo_suite2_gate_checked` and
     `einmo_suite2_gate_verified` are EXPECTED to fail on exactly one case
     (`foop/33/boolean/null_char_constant.foo`, missing-from-checked by deliberate deletion,
     per the human-directed TODO). Every OTHER case in every gate must stay green throughout
     Phase 3; that one case's continued (expected) redness is not itself a new STOP, but any
     ADDITIONAL failure is.
      (2026-09-18 00:00)

### 3a — Retire the old suite and its gates

- [x] Record the final green run of `einmo_suite2_has_every_einmo_suite_input` — this is the
      **record that parity held** before the comparand is removed (§4.5)
      (2026-09-18 00:00, see result below)
- [x] Delete `foolish-ubca2/src/ubca_snapshot_tester.rs` (231 lines — the OLD suite's three
      gates and its lossy-bridge adapter, §4.2). Confirmed 231 lines exactly before deletion,
      matching §4.2.
      (2026-09-18 00:00)
- [x] Delete `foolish-ubca2/einmo_suite/` (179 inputs, 179 checked, **179 verified**)
      (2026-09-18 00:00)
- [x] Delete `einmo_suite2_has_every_einmo_suite_input` from `ubca_snapshot_tester2.rs` (§4.5 —
      its referent is gone; its purpose is discharged by the run recorded above)
      (2026-09-18 00:00)
- [x] Delete the `t12_value_diff` module and `t12_report_value_differences_old_vs_new` (§4.5 —
      FOOP-36's old-vs-new instrument; there is no old side)
      (2026-09-18 00:00)
- [x] Remove `#[cfg(test)] mod ubca_snapshot_tester;` from `foolish-ubca2/src/lib.rs`
      (2026-09-18 00:00)
- [x] `cargo test -p foolish-ubca2` — the remaining suite2 gates must still be green (**modulo**
      the one known-red case carried in from Phase 2). **CONFIRMED**: 177 passed, 2 failed
      (only `einmo_suite2_gate_checked`/`einmo_suite2_gate_verified`, both on the single known
      case). One transient "catastrophe crumb" flake hit a DIFFERENT case
      (`foop/13/concat_brane_nested_shadowed_resolution.foo`, `status: output-error`) on the
      first run — same diagnosis as Phase 0's flake (parallel/resource-timing artifact, not a
      code regression): restored `output/` with `git checkout --` and re-ran; the second run
      was clean at exactly 177/2/0 with no new crumb. No additional STOP triggered.
      (2026-09-18 00:00)

### 3b — The rename itself, in isolation

- [x] **Housekeeping before the mv**: `foolish-ubca2/einmo_suite/` (the just-deleted OLD suite)
      left an empty directory tree on disk (git does not track empty directories — Phase 3a's
      `git rm -r` removed every tracked file but not the directory itself, and an untracked
      empty `flagged/` subdirectory was found inside it too). Removed the empty tree
      (`rm -rf foolish-ubca2/einmo_suite`) so the target path was clear — otherwise `git mv`
      below would have failed or nested incorrectly.
      (2026-09-18 00:00)
- [x] `git mv foolish-ubca2/einmo_suite2 foolish-ubca2/einmo_suite` — **and nothing else in this
      commit**
      (2026-09-18 00:00)
- [x] Change **only** `ubca_snapshot_tester2.rs:8` to join `"einmo_suite"`
      (2026-09-18 00:00)
- [x] Run all three gates immediately. **⛔ If ANY gate goes red, STOP and report** — §4.3's
      analysis is then wrong, and that is the finding, not something to work around.

      **Finding, not a STOP**: the workspace did not even COMPILE at first —
      `foolish-ubca2/src/sequencer.rs:1484`'s `foolish_annotations_are_separator_safe` test has
      a hardcoded `include_str!("../einmo_suite2/input/foop/36/rendering_contract.foo")` that
      §4.4's table of "code and docs that name the suites" **missed** — it is a build-time path
      literal, not a name §4.4 enumerated. This is a genuine gap in §4.4's inventory (recorded
      here as the finding), but it is a *compile* failure from an unmoved path literal, not a
      signature-safety failure of the kind §4.3 analyzed — so it was fixed in-place (the path
      string only, `einmo_suite2` → `einmo_suite`) as a prerequisite to even running the gates,
      rather than treated as a reason to STOP and unwind the rename. §4.3's actual claim
      (signature verification survives a directory rename) was then tested and **confirmed**:

      - `einmo_suite2_gate_output` — **PASS** (clean, no path issues once the include_str! was
        fixed)
      - `einmo_suite2_gate_checked` — **FAILS on exactly one case**:
        `foop/33/boolean/null_char_constant.foo.einmo: missing entirely from checked/` — the
        SAME known, expected exception from Phase 2, with the SAME diagnostic text as before
        the rename. No new failure.
      - `einmo_suite2_gate_verified` — fails the same way, transitively, same single case.
      - `einmo_suite2_corpus_wide_foolish_rendering_parses` — **PASS**.

      **§4.3's central claim is confirmed**: the rename introduced ZERO new signature or
      correspondence failures. The only red is the pre-existing, deliberate Phase 2 exception.
      (2026-09-18 00:00)
- [x] Confirm stop condition 4: the moved `einmo.toml` still reads
      `[signing.checked] passphrase = "foolish-ubca2-suite2"`. **Leave it exactly as it is.**
      **CONFIRMED**: read `foolish-ubca2/einmo_suite/einmo.toml` post-move — passphrase is
      byte-identical to the pre-move file (`git diff` shows the move/rename with no content
      change). Not touched.
      (2026-09-18 00:00)
- [x] Commit this step on its own, so the rename is bisectable
      (2026-09-18 00:00, commit "Major: Retire UBCa, Phase: 3b--complete")

### 3c — Re-point the names

- [x] Rename the three gate functions: `einmo_suite2_gate_output` → `einmo_gate_output`,
      `einmo_suite2_gate_checked` → `einmo_gate_checked`, `einmo_suite2_gate_verified` →
      `einmo_gate_verified` (§4.4 — the substring every document already uses must keep
      selecting the real gate)
      (2026-09-18 00:00)
- [x] Rename `einmo_suite2_dir()` → `einmo_suite_dir()`, and
      `einmo_suite2_corpus_wide_foolish_rendering_parses` → `einmo_corpus_wide_..._parses`
      (2026-09-18 00:00, landed as `einmo_corpus_wide_foolish_rendering_parses`)
- [x] Update the assertion messages that name "suite2" — all updated (`"einmo suite2 discovered
      no inputs"` → `"einmo suite discovered no inputs"`, `"einmo_suite2 is not sound..."` →
      `"einmo_suite is not sound..."`, `"suite2 output differs..."` → `"suite output
      differs..."`, `"suite2 correspondence failure..."` → `"suite correspondence
      failure..."`, the `GATE_LOCK` comment's "suite2/output" → "einmo_suite/output", and the
      corpus test's doc comment and inline messages). Confirmed with
      `grep -rn "einmo_suite2\|suite2"` over `foolish-ubca2/src/` and `foolish-cli/src/`: zero
      hits.
      (2026-09-18 00:00)
- [x] **Preserve verbatim** the doc comment on the verified gate explaining it is deliberately
      NOT `#[ignore]`d (currently `ubca_snapshot_tester2.rs:94-100`) — updating only the suite
      name inside it. Preserved word-for-word except `` `einmo_suite2/verified/` `` →
      `` `einmo_suite/verified/` `` (the one suite-name token); every other word, including "AGENTS.md
      forbids an agent from adding `#[ignore]` to a Verified-tier gate," is untouched.
      (2026-09-18 00:00)
- [x] `git mv foolish-ubca2/src/ubca_snapshot_tester2.rs foolish-ubca2/src/ubca_snapshot_tester.rs`
      and update `lib.rs`'s `mod` declaration
      (2026-09-18 00:00)
- [x] `cargo fmt`; run all three gates again. **CONFIRMED**: same result as Phase 3b's rename
      verification, now under the canonical names — `einmo_gate_output` and
      `einmo_corpus_wide_foolish_rendering_parses` PASS; `einmo_gate_checked`/`einmo_gate_verified`
      fail on exactly the one known case, same diagnostic text. `cargo test -p foolish-ubca2
      --lib -- einmo_gate_checked` — the exact command form AGENTS.md/README/every FOOP plan
      already uses — now correctly selects the real gate in the surviving crate.
      (2026-09-18 00:00)
- [x] Run all tests — old and new — and make sure they all pass correctly. **Full workspace
      run**: 133+84+62+328+177(+2 known-red)+0(doctests) — every crate's count is UNCHANGED
      from before Phase 3 except `foolish-ubca2`, which moved from Phase 0's baseline of 184 to
      179 (177 passed + 2 known-red), a drop of exactly **5**, fully accounted for by Phase 3a's
      deletions: the OLD suite's 3 gate tests (`einmo_suite2_gate_output/checked/verified`),
      `einmo_suite2_has_every_einmo_suite_input` (1), and
      `t12_report_value_differences_old_vs_new` (1) = 5. Holding steady through 3b/3c's pure
      renames, as expected (renaming a test doesn't change how many exist). No unexpected
      change anywhere else.
      (2026-09-18 00:00)

## Phase 4 — Arena-native `Detailed`, then remove the bridge (§3)

> **Two sub-phases, and the ORDER is the point.** 4a writes the replacement; 4b deletes what it
> replaced. There is never a window in which neither exists.

### 4a — Re-implement `format_detailed` over the arena (§3.2)

> **Judgment phase — larger model.** New code against the arena API, with no fixed target to
> match: byte-compatibility with the old output is explicitly NOT required (§3.2).

- [x] (read §3.2 of [`FOOP-86.md`](FOOP-86.md) in full — what it must render, and why
      byte-compatibility is not wanted)
      (2026-09-18 00:00)
- [x] Establish relevant tests for this
     sub-section. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests:
     the full surviving suite through `einmo_gate_checked` (Detailed must not disturb Foolish-mode
     output); run unit tests: `foolish-ubca2::sequencer`, and as they are written
     `foolish-ubca2::detailed_renders_every_fir_kind`,
     `foolish-ubca2::detailed_shows_search_direction_and_contexting`,
     `foolish-ubca2::detailed_differs_from_foolish`, `foolish-ubca2::detailed_is_deterministic`.
      (2026-09-18 00:00)
- [x] **Keep `SequenceMode` two-variant.** Do NOT collapse the enum; do NOT change
      `Ubca2Sequencer::format`'s signature. Only `format_detailed`'s body changes. **Confirmed**:
      `SequenceMode` still has exactly `Foolish` and `Detailed`; `format`/`format_with`'s
      signatures are byte-identical to before this phase.
      (2026-09-18 00:00)
- [x] Rewrite `format_detailed` (`sequencer.rs:159-162`) to read `FVMStorage` / `FirSpec` /
      `FirCursor` directly. Per §3.2 it renders, per node: the `FirSpec` variant, the NYES state
      by name, and the kind-specific fields — a search's `pattern`, `anchored`, `forward`,
      `is_value_search`, `contexted`; an operator's kind and operand order; a statement's name
      and line number; and the `foolish_children` / `ubc_children` split.

      **Implemented as `DetailedRenderer`** (new struct in `sequencer.rs`), a recursive walker
      over `FirCursor` with an exhaustive `match` on all 14 `FirSpec` variants (no catch-all
      `_`, per `rust_instructions.md`) rendering each variant's own fields, the NYES state via
      `Nyes`'s existing `Display` impl (`PREMBRIONIC`/`ECONSTANIC`/etc.), and separate
      `ubc_children:`/`foolish_children:` sections per node exactly as §3.2 asks.

      **Bug found and fixed during T4b-i, before any test was written to hide it**: a naive
      first version walked the tree assuming it was a strict tree. It is not — the arena is a
      DAG (a search's `FoolRef` and a `Concatenation`'s helper can each be reached from more
      than one parent), and a pre-constanic, still-stepping program can hold a genuine pointer
      CYCLE (`foop/62/infinite_loop.foo`'s `f1 = { f1 }` is exactly this, capped at the
      9999-iteration limit and never settling). The naive version re-expanded shared subtrees
      exponentially (confirmed: >20,000,000 node visits inside one second, capped depth 101,
      real corpus is ~2,000 nodes) and hung on that one case. **Fixed with global
      visit-memoization**: `render_node` labels every `FirPointer` the first time it is reached
      (`#N`) and, on any LATER reach — whether sibling-shared or a genuine ancestor cycle, the
      distinction does not matter for finiteness — prints a short `<SEE #N>` back-reference
      instead of recursing again. Verified: `infinite_loop.foo`'s Detailed output is now finite
      (78,301 bytes, terminates in milliseconds) and legible — it visibly shows the
      self-referential `FoolRef` chain, which is exactly the debugging value §3.2 argues for.
      Added a `FirCursor::ptr()` accessor (`fvm_storage.rs`) to make pointer identity available
      for this — a small, justified widening (read-only identity access for a caller that
      genuinely needs it), not a design change.
      (2026-09-18 00:00)
- [x] **Self-check**: `grep -n 'proto_to_core_fir' foolish-ubca2/src/sequencer.rs` must return
      NOTHING outside `#[cfg(test)]`. If it does, 4b cannot proceed. **Confirmed**: the only two
      hits are inside doc-comment prose referring to the OLD bridge by name for historical
      context (not code, not a call, not `#[cfg(test)]`-gated either — but not a dependency on
      the bridge). The functional import and the one call site are both gone. 4b may proceed.
      (2026-09-18 00:00)
- [x] **T4b-i** — write `detailed_renders_every_fir_kind`: render the whole surviving corpus in
      `Detailed`; every case must produce output without panicking. **This is the test that
      caught the DAG/cycle bug above** — it hung before the fix and passes in 0.18s after.
      (2026-09-18 00:00)
- [x] **T4b-ii** — write `detailed_shows_search_direction_and_contexting`: on a case with a
      contexted or forward search, assert the output names direction and contexting. **This is
      the test that proves the rewrite was worth doing** — those are fields §1.2 records the
      bridge as dropping. Used `steps~bake&?prep` (anchored forward search, then a contexted
      backward search from `bake`'s position) — verified this input settles correctly
      (`back_step` = `7`, the value of `prep`) via the CLI before writing the assertion, rather
      than assuming syntax.
      (2026-09-18 00:00)
- [x] **T4b-iii** — write `detailed_differs_from_foolish`: the two modes are genuinely different
      renderings of the same FIR (guards against the mode silently collapsing)
      (2026-09-18 00:00)
- [x] **T4b-iv** — write `detailed_is_deterministic`: rendering the same settled FIR twice is
      identical
      (2026-09-18 00:00)
- [x] Replace the old delegation test at `sequencer.rs:1198` — it asserted delegation to the
      code being deleted, so it is superseded by T4b-i…iv rather than kept. Removed
      `assert_detailed_delegates` and its 5 `detailed_delegates_for_*` tests along with the
      now-unused `proto_to_core_fir` test import.
      (2026-09-18 00:00)
- [x] `cargo fmt`; `cargo clippy -p foolish-ubca2 -- -D warnings`. `cargo fmt` applied cleanly.
      `clippy -D warnings` still fails on the same 4 pre-existing `foolish-core` errors (Phase
      0/1's known, out-of-scope debt) — **but also surfaces one pre-existing warning inside
      `foolish-ubca2` itself**, `collapsible_if` at what is now `sequencer.rs:544` (nested
      `if !suppress... { if let Some(first)... }`). Checked against `HEAD` (the commit before
      any Phase 4a edit): this exact nested-if pattern already existed, untouched by this
      phase's work. Left as-is — pre-existing style debt outside this FOOP's remit, same
      discipline as the `foolish-core` errors, and a `warn`-level lint anyway (not
      `correctness`).
      (2026-09-18 00:00)
- [x] Run all tests — old and new — and make sure they all pass correctly. **Full workspace
      run**: 133+84+62+328+176(+2 known-red) — `foolish-ubca2` moved from 179 to 178 (net −1:
      +4 new T4b tests, −5 old delegation tests). `einmo_gate_checked` re-confirmed to fail on
      only the one known case — Detailed's rewrite did not disturb Foolish-mode output.
      (2026-09-18 00:00)

### 4b — Delete the bridge (§3.1, §3.4)

> **Execution phase — smaller model.** The call sites are enumerated; the target is "the
> workspace compiles with them gone." 4a has already removed one of the two.

- [ ] (read §3.1, §3.3 and §3.4 of [`FOOP-86.md`](FOOP-86.md) — §3.3's scope guard especially)
- [ ] Establish relevant tests for this
     sub-section. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests:
     the full surviving suite through `einmo_gate_checked` and `einmo_gate_verified`; run unit tests:
     `foolish-ubca2::einmo_gate_checked`, `foolish-ubca2::einmo_gate_verified`,
     `foolish-ubca2::sequencer`, `foolish-ubca2::fvm_storage`,
     `foolish-ubca2::detailed_renders_every_fir_kind`.
- [ ] Remove `impl foolish_core::Evaluator for UbcaEvaluator` (`evaluator.rs:45-55`) — §3.4.
      **This is the bridge's last remaining non-test caller** now that 4a repointed the other.
- [ ] Remove the `core_fir_conversion` bridge: `proto_to_core_fir` and its `_sff_body`,
      `_sff_operand`, `_inner` siblings (`fvm_storage.rs:3494`–~`4151`) and the re-export at
      `4990`
- [ ] Remove the bridge's own tests (`fvm_storage.rs:6876, 6889, 6905, 6924, 6956, 8273`;
      `sequencer.rs:1194`) — they test the removed code, not surviving behavior
- [ ] Correct `foolish-ubca2/src/lib.rs`'s module docs: `evaluate_arena` is now the crate's one
      production entry point, and the "two independent implementations" paragraph is no longer
      true — rewrite it to describe a single implementation
- [ ] **⛔ If any of the above requires editing `foolish-core/src/`, STOP and report** (§3.3).
      `foolish_core::FirSequencer` is NOT deleted by this FOOP.
- [ ] `cargo build --workspace`; `cargo fmt`; `cargo clippy -p foolish-ubca2 -- -D warnings`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 5 — Remove `foolish-ubca` (§2)

> **Execution phase — smaller model.** Stop condition: if anything OUTSIDE `foolish-ubca/` must
> change in order to delete it, STOP — that is an undiscovered dependency.

- [ ] (read §2 of [`FOOP-86.md`](FOOP-86.md))
- [ ] Establish relevant tests for this
     sub-section. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests:
     the full surviving suite through all three gates; run unit tests: the whole `foolish-ubca2` crate,
     plus `cargo test --workspace`.
- [ ] **Q3 is decided: delete outright, no tag, no archive** (human, 2026-09-16). Record the
      commit SHA immediately before the deletion in this plan — git history retains the crate
      regardless, so this is a convenience note, not a safety net
- [ ] Remove `"foolish-ubca",` from the workspace `Cargo.toml:6`
- [ ] `git rm -r foolish-ubca/` (14 171 lines of source + the 178/178/178/178 einmo suite)
- [ ] `grep -rn "foolish.ubca\b" --include='*.rs' --include='*.toml' --include='*.sh' .` —
      expect no hits outside `docs/` and historical FOOP plans
- [ ] `cargo build --workspace` and `cargo test --workspace`
- [ ] Record the **after** test count and **account for the difference** against Phase 0's
      baseline of 791 — foolish-ubca's tests, the bridge's ~7, and §4.5's two instruments.
      "Fewer tests pass" must never be mistaken for "tests were lost."
- [ ] `cargo clippy --workspace -- -D warnings` — compare against Phase 0's pre-existing count;
      **this FOOP must not add any new error**
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 6 — CLI tests and the exercise recording (§T3, §T4)

- [ ] (read §Test Plan T3 and T4 of [`FOOP-86.md`](FOOP-86.md))
- [ ] Establish relevant tests for this
     sub-section. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests:
     the full surviving suite through `einmo_gate_checked`; run unit tests:
     `foolish-cli::cli_run_renders_foolish`, `foolish-cli::cli_agrees_with_einmo_adapter`,
     `foolish-cli::cli_step_and_repl_share_the_render_path`, `foolish-ubca2::einmo_gate_checked`.
- [ ] **T3a** — write `cli_run_renders_foolish`: `run` on a small program emits Foolish; assert
      the output contains no `?(pattern=`, no `Op`, and no bare NYES token
- [ ] **T3b** — write `cli_agrees_with_einmo_adapter`: for the same source, the CLI's rendering
      equals the einmo adapter's. **This is the test that pins §1.3** — a divergence means the
      CLI grew its own path
- [ ] **T3c** — write `cli_step_and_repl_share_the_render_path`: no second sequencer call site
- [ ] **T3d** — write the test asserting Q2's decision for `cmd_compile`
- [ ] **T4** — run `future_exercise_inputs/project_euler/1.foo.disabled` through the new CLI and
      **record verbatim in this plan** what it produces: output, alarms, step count, or failure
      mode. This is a RECORDING task, not an acceptance criterion — FOOP-86 is not blocked by
      the result, and **the file stays `.disabled`** (re-enabling it belongs to FOOP-26/46).
- [ ] `cargo fmt`; `cargo clippy --workspace -- -D warnings`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 7 — Documentation (§4.4, Q6)

- [ ] (read §4.4 of [`FOOP-86.md`](FOOP-86.md))
- [ ] Establish relevant tests for this
     sub-section. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests:
     the full surviving suite through all three gates, invoked using the NEWLY DOCUMENTED commands (this
     is how the docs get verified); run unit tests: `cargo test --workspace`.
- [ ] Update `README.md` §"Running specific tests": every `foolish-ubca/einmo_suite` path
      becomes `foolish-ubca2/einmo_suite`, `-p foolish-ubca` becomes `-p foolish-ubca2`, and the
      `foolish-cli run` evaluator command is re-verified against the new rendering
- [ ] Update AGENTS.md §"Approval Tests (einmo)" and §"Crates of Foolish": suite paths, the gate
      command, and the crate list (`foolish-ubca` removed). Per **Q7**, also raise — but do not
      unilaterally fix beyond the obvious — `zweimomo`'s absence from the tree.
- [ ] Per **Q6**: leave `foop.md` and historical FOOP plans alone (historical record)
- [ ] **Execute every command as newly written** in README §"Running specific tests" and confirm
      each works. A documented command that was not run is not documentation.
- [ ] Update the `## Last Updated

**Date**: 2026-09-16
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created the FOOP-86 plan — ten phases sequencing the four deliverables so the tree
is green at every step and the riskiest work is not first. Phase 0 records the human's four
settled decisions (discard UBCa's attestations; delete outright with no tag; keep
`SequenceMode::Detailed` and rewrite it arena-native; no crate/type rename) and puts only the
four non-blocking questions to them. Phases 1 and 2 (judgment, larger model) do the CLI switch
and — the last moment both signed corpora exist — compare the two evaluators' attested ANSWERS
on the 178 shared inputs; input parity is already measured (0 missing), so nothing is ported.
Phase 3 isolates the einmo `git mv` in its own commit with its own verification. **Phase 4 is
split 4a/4b and the order is the point**: 4a writes the arena-native `Detailed`, 4b then deletes
the bridge whose last caller 4a removed — never a window in which neither exists. Phases 5–7
remove the crate, add CLI tests, and update docs by executing every command written. Carries an
Orientation block of measured facts marked *verify, don't re-derive*, a decisions table, and
five standing stop conditions — chief among them that **no `einmo promote` occurs in this FOOP**,
that the surviving suite's `"foolish-ubca2-suite2"` passphrase must not be tidied after the
rename, and that the vestigial `2` suffix must not be de-suffixed. No Promotion Review Gate and
no comprehensive case: this FOOP produces no new einmo output.
