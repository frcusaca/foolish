# FOOP-42: Improve Humanizing Sequencer — literal Int() output and consistent brane formatting

## TL;DR

> **Quick Summary**: Add `Int()` type tags to constant integer output in the Humanizing Sequencer and document brane formatting rules.
>
> **Deliverables**: Updated `sequencer.rs` (2 lines), new tests, regenerated snapshots
>
> **Estimated Effort**: Quick
> **Parallel Execution**: NO — sequential
> **Critical Path**: Code change → tests → snapshot regeneration → human review

---

## Context

### Original Request
"Let's update Humanizing sequencer. the way it is is still bad." + "Can you describe in the new FOOP42 how brane formatting works? Add literal int output"

### Current State
The Humanizing Sequencer (`format_fir_q` / `format_fir_q_inline` in `sequencer.rs`) has two issues:

1. **ConstantInt omits type tag** — line 73 outputs bare `42` instead of `Int(42)`
2. **Inconsistent with `format_fir_simple_indent`** — line 314 correctly outputs `Int({})`

### Sample Output (Current vs Expected)

**Current (wrong):**
```
Brane{
    x = 42
}
```

**Expected:**
```
Brane{
    x = Int(42)
}
```

---

## Work Objectives

### Core Objective
Make the Humanizing Sequencer output `Int(42)` instead of bare `42` for all constant integers, and document the brane formatting rules.

### Concrete Deliverables
- `sequencer.rs` lines 73 and 190: add `Int()` wrapper
- `sequencer_tests.rs`: new tests for `Int()` output
- All 134 `.foo.snap` files regenerated with new format

### Must Have
- `Int()` tag on all constant integer output in `format_fir_q` and `format_fir_q_inline`
- New unit tests verifying the change
- Snapshot regeneration via proper `.snap.new` → human review → `.snap` flow

### Must NOT Have
- No `INSTA_UPDATE=always` — snapshots must go through review
- No changes to evaluation logic
- No changes to `format_fir_simple_indent` (already correct)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (cargo test, insta snapshots)
- **Automated tests**: Tests after implementation
- **Framework**: `cargo test -p foolish-core --lib`

### QA Policy
- Unit tests in `sequencer_tests.rs` for `Int()` output
- Snapshot regeneration for all 134 files
- Human review of `.snap.new` files before promotion

---

## Execution Strategy

### Sequential Tasks

```
Task 1: Update sequencer.rs (lines 73, 190)
Task 2: Add unit tests in sequencer_tests.rs
Task 3: Run cargo test — verify unit tests pass
Task 4: Run cargo insta test — generate .snap.new files
Task 5: Human review .snap.new files
Task 6: On approval, promote .snap.new → .snap
Task 7: Run cargo test — all tests pass
```

---

## TODOs

- [ ] 1. Update `sequencer.rs` — add `Int()` wrapper to constant int output

  **What to do**:
  - Line 73: Change `let _ = writeln!(buf, "{}{}", indent, value);` to `let _ = writeln!(buf, "{}Int({})", indent, value);`
  - Line 190: Change `let _ = writeln!(buf, "{}", value);` to `let _ = writeln!(buf, "Int({})", value);`
  - Verify: `cargo check -p foolish-core` passes

  **Must NOT do**:
  - Do not change `format_fir_simple_indent` (line 314 already correct)
  - Do not change evaluation logic

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 2
  - **Blocked By**: None

  **References**:
  - `foolish-core/src/sequencer.rs:73` — `format_fir_q` constant int output
  - `foolish-core/src/sequencer.rs:190` — `format_fir_q_inline` constant int output
  - `foolish-core/src/sequencer.rs:314` — `format_fir_simple_indent` (already uses `Int()`)

  **Acceptance Criteria**:
  - `cargo check -p foolish-core` passes
  - `format_fir_q` outputs `Int(42)` not `42`
  - `format_fir_q_inline` outputs `Int(42)` not `42`

  **Commit**: YES
  - Message: `foolish-core(sequencer): add Int() type tag to constant integer output`

- [ ] 2. Add unit tests in `sequencer_tests.rs`

  **What to do**:
  - Add `test_format_constant_int_with_tag` — creates a ConstantInt FIR, formats it, asserts output contains `Int(42)`
  - Add `test_format_brane_with_int_values` — creates a brane with int values, verifies `Int()` tags appear
  - Verify: `cargo test -p foolish-core --lib -- sequencer_tests` passes

  **Must NOT do**:
  - Do not change existing tests

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 3
  - **Blocked By**: Task 1

  **References**:
  - `foolish-core/src/sequencer_tests.rs:61` — `test_format_constant_int` (existing test to follow)
  - `foolish-core/src/sequencer_tests.rs:55` — `test_format_empty_brane` (brane test pattern)
  - `foolish-core/src/fir.rs` — FIR builder patterns for test construction

  **Acceptance Criteria**:
  - `cargo test -p foolish-core --lib -- sequencer_tests` passes
  - New tests verify `Int()` tag presence

  **Commit**: YES (group with Task 1)
  - Message: `foolish-core(sequencer): add Int() type tag to constant integer output`

- [ ] 3. Run `cargo insta test -p foolish-core --lib` — generate `.snap.new` files

  **What to do**:
  - Run `cargo insta test -p foolish-core --lib` to generate all `.snap.new` files
  - Count the `.snap.new` files (expect ~134)
  - Present sample diffs to human for review

  **Must NOT do**:
  - NEVER run `INSTA_UPDATE=always` or `cargo insta accept`
  - NEVER promote `.snap.new` to `.snap` without human approval

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 4 (human review)
  - **Blocked By**: Task 2

  **Acceptance Criteria**:
  - `.snap.new` files generated for all affected snapshots
  - Sample diffs show `Int()` tags added correctly

  **Commit**: NO

- [ ] 4. Human review `.snap.new` files

  **What to do**:
  - Present sample `.snap.new` diffs (5-10 representative files)
  - Wait for explicit human approval (`@Agent, LGTM`)
  - On approval, promote `.snap.new` → `.snap` (rename files)
  - On rejection, identify issues and fix

  **Must NOT do**:
  - Do not auto-accept without human approval

  **Recommended Agent Profile**:
  - **Category**: N/A (human action)

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 5
  - **Blocked By**: Task 3

  **Acceptance Criteria**:
  - Human approves the changes
  - All `.snap.new` files promoted to `.snap`

  **Commit**: YES
  - Message: `foolish-core(snapshots): regenerate all snapshots with Int() type tags`

- [ ] 5. Run `cargo test -p foolish-core --lib` — all tests pass

  **What to do**:
  - Run full test suite
  - Verify 85/85 tests pass (or whatever the current count is)
  - Run `cargo test -p foolish-ubcb --lib` — verify UBCb tests still pass

  **Must NOT do**:
  - Do not skip UBCb verification

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Final verification
  - **Blocked By**: Task 4

  **Acceptance Criteria**:
  - All tests pass
  - No regressions

  **Commit**: NO (verification only)

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Verify `Int()` tags present in all constant int output, no evaluation changes, proper snapshot review flow.

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy -p foolish-core` — no new warnings.

- [ ] F3. **Real Manual QA** — `unspecified-high`
  Spot-check 5 random `.snap` files for correct `Int()` formatting.

- [ ] F4. **Scope Fidelity Check** — `deep`
  Verify only `sequencer.rs` and `sequencer_tests.rs` were modified (plus snapshots).

---

## Commit Strategy

- **Tasks 1-2**: `foolish-core(sequencer): add Int() type tag to constant integer output`
  - Files: `sequencer.rs`, `sequencer_tests.rs`
  - Pre-commit: `cargo test -p foolish-core --lib -- sequencer_tests`

- **Task 4**: `foolish-core(snapshots): regenerate all snapshots with Int() type tags`
  - Files: `*.foo.snap` (all 134)
  - Pre-commit: `cargo test -p foolish-core --lib`

---

## Success Criteria

### Verification Commands
```bash
cargo test -p foolish-core --lib -- sequencer_tests  # Unit tests pass
cargo test -p foolish-core --lib                     # All tests pass
cargo test -p foolish-ubcb --lib                     # UBCb tests pass
```

### Final Checklist
- [ ] `Int()` tags present in all constant int output
- [ ] New unit tests added and passing
- [ ] All snapshots regenerated and human-approved
- [ ] No evaluation logic changes
- [ ] No `INSTA_UPDATE=always` used
