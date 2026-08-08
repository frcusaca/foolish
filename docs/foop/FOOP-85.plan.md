# FOOP-85.plan — einmo-separator-block-comment-collision

Implementation plan for [FOOP-85](FOOP-85.md) — *The einmo Foolish separator
collides with Foolish block comments*. **Read `FOOP-85.md` first**; Appendix A
carries the exact diff, already generated and verified, which this plan
reapplies.

This is a **one-constant change**. It is small enough that the worktree
ceremony is optional (see Phase 1); the bulk of the plan is the review and
promotion of the baselines it regenerates.

Worktree variables, expanded (if used):

```
WORKTREE_ORIGIN_BRANCH = jia
WORKTREE_ORIGIN_PATH   = /storage1/human/hcbusy/foolish
WORKTREE_BRANCH_NAME   = foop-85-einmo-separator
WORKTREE_FULL_FS_PATH  = /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-85-einmo-separator
```

---

## Phase 0 — Preconditions

- [ ] Read `FOOP-85.md` in full, including **Appendix A** (the generated diff).
- [ ] Confirm the working tree is clean (`git status --short` empty). The
      change was **deliberately reverted** after FOOP-85 was written so the
      build returns to a known-good state; this plan reapplies it.
- [ ] Confirm the human has moved non-`.foo` files out of
      `foolish-ubca/einmo_suite/input/`. As of 2026-08-07 the remaining ones
      were `exercises/project_euler/1.foo.disabled` and
      `exercises/project_euler/1.py`. einmo discovery is extension-agnostic
      **by design** (`stage.rs:99`), so anything left in `input/` is a live
      test — see FOOP-85 §Open Questions.
- [ ] Delete stale generated output with no surviving input:
      `rm -rf foolish-ubca/einmo_suite/output/exercises/` (untracked; no
      `checked/` or `verified/` twins exist).
- [ ] Record the **baseline failure set** before changing anything, so the
      after-state can be compared honestly:
      `cargo test --workspace 2>&1 | grep -E 'test result|FAILED'`

---

## Phase 1 — Apply the change

- [ ] Decide: worktree or direct-to-`jia`. A single-constant fix that
      unblocks the whole suite is a reasonable direct commit; use a worktree
      if it will sit unreviewed for any length of time.
  - [ ] If worktree: `git worktree add -b foop-85-einmo-separator /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-85-einmo-separator`
- [ ] Apply FOOP-85 §1 — set `FOOLISH_SEPARATOR` in `einmo/src/format.rs`
      to `"\n!!!EINMO!!!\n"`, with the doc comment from Appendix A. The
      appendix diff applies cleanly to `einmo/src/format.rs@e6ca7864`.
- [ ] Add the two regression tests from §Test Plan → "To add":
  - [ ] A body containing a `!!!` block-comment line **serializes
        successfully** under `FOOLISH_SEPARATOR`. This is the guard against
        re-arming the trap.
  - [ ] A body containing a standalone `!!!EINMO!!!` line is **still
        refused** (the collision check must keep working).
- [ ] `cargo test -p einmo` — expect 133+ passing (the new tests add to it).
- [ ] `cargo fmt` and `cargo clippy -D warnings` per `rust_instructions.md`.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 2 — Review and promote the regenerated baselines

The separator change is backward compatible (§4): existing `.einmo` files
record their own separator and keep verifying. Only **newly written** files
use the new value. Regenerate, then review.

- [ ] Regenerate outputs:
      `cargo test -p foolish-ubca --lib -- run_einmo_tests`
      (or `einmo evaluate foolish-ubca/einmo_suite --command "cat"`)
- [ ] **Inspect the diff before promoting anything** — the human's requested
      first step:
      ```bash
      git diff foolish-ubca/einmo_suite/
      ```
      Expect: separator lines changing from `!!` to `!!!EINMO!!!` in
      regenerated files, and stamp/timestamp churn. **Do not expect** any
      change to a rendered OUTPUT body — the separator is envelope framing,
      not content. **An OUTPUT body change is a red flag**: stop and
      investigate rather than promote.
- [ ] **Justify the diff** per AGENTS.md §"The einmo review workflow" step 4.
      For this FOOP the justification is narrow and must be stated: *the
      changed bytes are the section separator and the signatures over it;
      no section body content changed.* If any line falls outside that
      description, it is not covered by this FOOP.
- [ ] Confirm the two **known, unrelated** failures are unchanged in nature:
  - [ ] `foop/62/infinite_loop.foo` — OUTPUT regressed
        `NK(ITERATION-EXCEEDED)` → `BRANING`. **Pre-existing** (verified by
        stashing this change and re-running on clean `jia`). Belongs to
        FOOP-62. **Do NOT promote over it** — a failing einmo test is broken
        code, not a stale baseline (AGENTS.md §Non-regression invariant).
  - [ ] Any `exercises/*` entry — resolved by Phase 0's file moves.
- [ ] Mass promote `output` → `checked`:
      ```bash
      einmo promote output to checked foolish-ubca/einmo_suite
      ```
      Add `--filter '<glob>'` to scope it if the diff review says only part
      of the tree should move.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 3 — Mass promote to `verified` (HUMAN ONLY)

`verified/` is the human-signed stage. **An agent must never promote or
re-sign a `verified/` baseline** (AGENTS.md §Non-regression invariant).

- [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing.
      UNDER NO CIRCUMSTANCES will Agent continue past this point
      automatically!!
  - [ ] Present the human with the `git diff` output from Phase 2 and the
        justification, then the commands below.
- [ ] **Human** reviews the diff:
      ```bash
      git diff foolish-ubca/einmo_suite/verified/
      ```
- [ ] **Human** mass promotes (the requested command — bare directory means
      *all files*; `--interactive` forces the passphrase prompt):
      ```bash
      einmo promote checked to verified foolish-ubca/einmo_suite --interactive
      ```
      Scope it with `--filter '<glob>'` if only part of the tree should move
      (e.g. `--filter 'foop/62/*'` to exclude, by promoting the rest).
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 4 — Documentation and close

- [ ] Add FOOP-85 to `docs/foop/INDEX.md` in the correct little-endian
      position, with a one-line summary.
- [ ] Note in FOOP-75's plan (Phase 0) that FOOP-85 resolves the separator
      half of its gate failure. The `infinite_loop` half remains FOOP-62's.
- [ ] Update the "## Last Updated" section of every `.md` touched
      (AGENTS.md §Markdown File Update Protocol — **replace** the entry, do
      not append).
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] Verify all work is committed.
- [ ] If a worktree was used: merge `foop-85-einmo-separator` to `jia`, then
      clean up
      `/storage1/human/hcbusy/foolish/../foolish_worktrees/foop-85-einmo-separator`.

---

## Note on why this FOOP exists

The change was made, verified, and then **deliberately reverted** so the
build would return to a clean state rather than carrying an uncommitted
modification across FOOP-75's work. FOOP-85 is where that change was parked.
Appendix A of `FOOP-85.md` holds the diff verbatim, so nothing was lost in
the revert.

Evidence gathered before the revert, for the record:

- `cargo test -p einmo` — 133 passed, 0 failed, with the new separator.
- `cargo test -p foolish-ubca --lib -- einmo_gate_output` — **passed**
  (previously failed).
- `cargo test --workspace` — 3 failures → 2, the remaining two unrelated
  (see Phase 2).

## Last Updated

**Date**: 2026-08-07
**Updated By**: Claude Code / claude-opus-5
**Changes**: Initial plan. Phase 2 puts `git diff` before any promotion and
states what the diff must and must not contain (envelope framing only — an
OUTPUT body change is a red flag). Phase 3 gates the `verified` mass promote
behind a human STOP, carrying the requested
`einmo promote checked to verified … --interactive` command.
