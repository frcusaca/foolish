# FOOP-42 Implementation Plan

## Goal

Update the Humanizing FIR Sequencer (HFS) in `foolish-core/src/sequencer.rs` so
that both UBC (`foolish-core`) and UBCb (`foolish-ubcb`) snapshot tests produce
output matching the FOOP-42 formatting specification. The canonical acceptance
test is `foop42_humanizing_sequencer_formatting_exhaustive.foo` — a single large
snapshot input that exercises every formatting rule.

## Strategy

**Iterate on the HFS, not on snapshots.** Make changes to the formatter,
run `cargo test -p foolish-core --lib` to regenerate the `.snap.new` for
the foop42 test. Inspect the output, fix discrepancies, repeat until the foop42
test output is **exactly correct** per FOOP-42. Do NOT accept any snapshots until
the human reviews the final output.

## Pre-implementation Baseline

- 84 unit tests pass
- `foop42_humanizing_sequencer_formatting_exhaustive.foo` compiles and evaluates
- `.snap.new` exists with current (old-format) output at
  `foolish-core/snapshot_tests/approved/foop42_humanizing_sequencer_formatting_exhaustive.foo.snap.new`
- UBC and UBCb share the same HFS code (`sequencer.rs`)
- Both have their own `snapshot_tests/input/` and `snapshot_tests/approved/`

## Worktree

- [ ] Create worktree at `${HOME}/tmp/foolish-worktrees/hfs-formatting-foop-42` with branch `foop/foop-42-humanizing-fir-sequencer`

## Implementation Tasks

### Phase 0: Rename HS → HFS

- [ ] Rename `Sequencer` struct to `FirSequencer` in `foolish-core/src/sequencer.rs`
- [ ] Rename `HumanizingSequencerRef` to `HumanizingFirSequencerRef`
- [ ] Update all public re-exports in `foolish-core/src/lib.rs`:
  - `pub use sequencer::{Sequencer, HumanizingSequencerRef}` → `{FirSequencer, HumanizingFirSequencerRef}`
- [ ] Update snapshot header: `\`\`\`hssnap` → `\`\`\`hfssnap` in `snapshot_suite.rs`
- [ ] Update signature label: `HS signature` → `HFS signature` in `signature.rs` and all `.snap` files
- [ ] Update all references in UBC evaluator (`ubc_snapshot_tester.rs`)
- [ ] Update all references in UBCb evaluator (`ubcb_snapshot_tester.rs`)
- [ ] Update all unit tests in `sequencer_tests.rs`: `Sequencer::` → `FirSequencer::`, `HS` → `HFS`
- [ ] Update `foolish-cli` and `foolish-ubcb-cli` if they reference the old names
- [ ] Run `cargo check --workspace` to find any remaining old-name references
- [ ] Verify: `cargo test -p foolish-core --lib` passes with new names (ignoring snapshot mismatch)

### Phase 1: HFS Core Rewrite

- [ ] Implement `(prefix_count, text)` pair model — formatters return `[(prefix, text), ...]`,
  not strings with embedded spaces. Only outermost level materializes spaces.
- [ ] Implement `proto_brane_formatter(pbid, opener, closer, open_indent, close_indent, state, inline, body)`:
  - Compute `internal_indent = len(pbid + opener)`
  - Compute `body_indent = min(open_indent + internal_indent + B_DENT, 3 × B_DENT)`
  - Opening line: `pbid + opener` at column `open_indent` with inline args + state
  - Body lines: children at `body_indent`
  - Closing line: `closer` at column `close_indent` (usually = `open_indent`)
- [ ] Implement `proto_brane_formatter_with_result(...)` for Search/HeadTail/Index:
  - Generate non-result items first (pattern, anchor, state)
  - Generate result last with `close_indent = parent_body_indent`
  - Result child receives `open_indent = body_indent + len("result=") + len(pbid + opener)`
- [ ] Configure proto-brane formatter for each FIR variant (source-trigger pbids):
  - Brane: `("", "{", "}", ...)`
  - Search ←: `("?", "(", ")", ...)`
  - Search →: `("/", "(", ")", ...)`
  - HeadTail (head): `("^", "(", ")", ...)`
  - HeadTail (tail): `("$", "(", ")", ...)`
  - Index: `("#", "(", ")", ...)`
  - Operator: `(op_str, "(", ")", ...)`
  - StayFoolish: `("", "<", ">", ...)`
  - StayFullyFoolish: `("", "<<", ">>", ...)` — 2-char delimiters
  - Concatenation: `("⨃", "(", ")", ...)`
- [ ] Replace `[STATE]` bracket syntax with bare state tokens
  - `[WOCONSTANIC]` → `WOCONSTANIC`
  - `[ECONSTANIC]` → `ECONSTANIC`
  - `[NK]` → omit (NK state implicit from `???` display)
  - CONSTANT and INDEPENDENT: omit entirely
- [ ] Brane state: place immediately after opening `{`, no space
  - `{WOCONSTANIC` not `{ WOCONSTANIC`
  - `{` for constant branes
- [ ] Integer literals: render as value only (already done)
- [ ] NK: render as `??? (reason)` without state suffix
- [ ] Operator: transparent when constant, proto-brane formatter when non-constant
- [ ] Search/HeadTail/Index: proto-brane with source-trigger pbid, inline args, optional body
- [ ] Statement `;` separators: append to all non-last lines
- [ ] Named statement `{` merging: stmt outputs `name=` at body_indent, child's `{` goes inline,
  `opener_indent = len(name) + 1` passed to child
- [ ] Implement single-lining: parent collapses short child bodies onto one line when within budget
- [ ] Implement `line_hint` parameter chain with 128-char screen width

### Phase 2: Iterative Refinement

- [ ] Run `cargo test -p foolish-core --lib` to regenerate foop42 `.snap.new`
- [ ] Inspect output for correctness against FOOP-42 spec
- [ ] Fix any discrepancies found
- [ ] Repeat: run → inspect → fix until output is exactly correct
  - [ ] Verify: flat branes (1 level) render correctly
    - Short names: `a=42` on one line
    - Long names: proper body_indent from open_indent chain
  - [ ] Verify: deeply nested branes (up to 5 levels) render correctly
    - `body_indent = min(open_indent + internal + B_DENT, 3×B_DENT)` at each level
    - Closer at `close_indent` (back to parent's indent)
    - `;` separators on all but last statement
  - [ ] Verify: proto-brane formatter produces correct opening line for each variant
    - Search ←: `?(pattern=..., UNANCHORED/ANCHORED)` with state at end
    - HeadTail head: `^(STATE)` or `^(result=X, STATE)`
    - Index: `#(offset=N, UNANCHORED/ANCHORED, STATE)`
    - StayFoolish: `<STATE` or `<` with `<`/`>` delimiters
    - StayFullyFoolish: `<<STATE` or `<<` with `<<`/`>>` delimiters
    - Concatenation: `⨃(elements=N, STATE)`
    - Operator: `+(operands, STATE)` with `(`/`)`
  - [ ] Verify: NK values render as `??? (reason)` without `[NK]` bracket
  - [ ] Verify: Search/HeadTail/Index with result use deferred generation — result last, closer at `close_indent`
  - [ ] Verify: Empty branes render as `{}` on one line
  - [ ] Verify: Transparent (constant) operators show computed value, no FIR wrapper
  - [ ] Verify: `body_indent` cap at `3 × B_DENT` prevents runaway indentation
  - [ ] Verify: closing delimiters align with parent-provided indent (prefix `0`)
  - [ ] Verify: Unicode identifier names and underscore→ˍ substitution preserved
  - [ ] Verify: single-lining collapses short bodies: `{a={r=1}}` stays on one line
  - [ ] Verify: `line_hint` propagates correctly; 128-char screen width

### Phase 3: UBCb Integration

- [ ] Run `cargo test -p foolish-ubcb --lib` to regenerate UBCb `.snap.new`
- [ ] Verify UBCb output matches UBC output for structurally equivalent FIRs
- [ ] UBCb uses the same `FirSequencer::format()` — changes are shared automatically
- [ ] Update UBCb snapshot header to `hfssnap`

### Phase 4: Unit Test Repair

- [ ] Update `sequencer_tests.rs` for new format expectations and HFS naming
- [ ] Add new unit tests:
  - Empty brane rendering: `{}`
  - Single-element brane: `{ x=1 }`
  - Deeply nested brane (5 levels) indentation computation
  - `body_indent` cap at `3 × B_DENT`
  - `(prefix, text)` pair accumulation through parent indenter chain
  - StayFullyFoolish with `<<`/`>>` 2-char opener/closer
  - Search with `?` pbid (backward) and `result=` body
  - HeadTail with `^`/`$` pbid and `result=` body
  - Index with `#` pbid
  - Named statement `opener_indent = len(name) + 1`
  - Closer always at prefix `0`
  - Single-lining: verify `{a={r=1}}` collapses; `{a={r=1; s=2}}` does not
  - `line_hint` budget computation and 128-char limit

### Phase 5: Final Verification

- [ ] `cargo test -p foolish-core --lib` — all unit tests pass
- [ ] `cargo test -p foolish-ubcb --lib` — all unit tests pass
- [ ] STOP! STOP!! STOP!!! ASK HUMAN to review `.snap.new` files before accepting
- [ ] After human approval: accept snapshots (human runs `cargo insta review`)
- [ ] Verify signatures with `./target/debug/verify_signatures`

### Phase 6: Cleanup

- [ ] Verify all work is complete in `${HOME}/tmp/foolish-worktrees/hfs-formatting-foop-42` and committed to `foop/foop-42-humanizing-fir-sequencer`
- [ ] Merge `foop/foop-42-humanizing-fir-sequencer` to alpha
- [ ] Cleanup `${HOME}/tmp/foolish-worktrees/hfs-formatting-foop-42`
  - [ ] Check that plan has all but Cleanup checkboxes completed
  - [ ] Remove the worktree directory
  - [ ] This is the last checkbox to be checked in this plan

## Acceptance Criteria (foop42 test file)

The output of `foop42_humanizing_sequencer_formatting_exhaustive.foo` must:

1. Use `hfssnap` header (not `hssnap`)
2. Show all FIR types present in the evaluated output with source-trigger pbids
3. Use bare `{`/`}` for brane delimiters (no `Brane` keyword)
4. Use character-aligned indentation: `body_indent = min(opener_indent + B_DENT, 3 × B_DENT)`
5. Display state tokens without `[` `]` brackets
6. Omit state for CONSTANT and INDEPENDENT FIRs
7. Show NK as `??? (reason)` without state suffix
8. Show `;` separators on all non-last statements
9. Close `)` / `}` / `>` / `>>` at parent-provided indent (prefix `0`)
10. Handle empty branes as `{}` (single line)
11. Single-line short bodies when within 128-char budget
12. Preserve the original `.foo` source in the INPUT section (already works)

## Notes

- The foop42 test file has 76 lines of Foolish source and produces output with
  all 10 FIR variants in various states
- Underscores in variable names are rendered as `ˍ` (modifier letter low macron)
  by the compiler — this is existing behavior, not part of FOOP-42
- The `seek_unanchored` case produces `??? (constanic_clone called on NYE FIR)`
  — this is a known UBC edge case, verify if FOOP-42 should handle it differently
- Both UBC and UBCb snapshot suites will change simultaneously since they share
  the same `FirSequencer::format()`
- Phase 0 (rename HS → HFS) must complete first — Phase 1 writes new code under
  the new names

## References

- Specification: `docs/foop/FOOP-42.md`
- HFS implementation: `foolish-core/src/sequencer.rs`
- HFS unit tests: `foolish-core/src/sequencer_tests.rs`
- FIR types: `foolish-core/src/fir.rs` (FirQueryable trait, all FIR structs)
- UBC evaluator: `foolish-core/src/ubc_snapshot_tester.rs`
- UBCb evaluator: `foolish-ubcb/src/ubcb_snapshot_tester.rs`
- Snapshot suite: `foolish-core/src/snapshot_suite.rs`
- Signature tool: `foolish-core/src/signature.rs`
- Test input: `foolish-core/snapshot_tests/input/foop42_humanizing_sequencer_formatting_exhaustive.foo`
- Current baseline output: `foolish-core/snapshot_tests/approved/foop42_humanizing_sequencer_formatting_exhaustive.foo.snap.new`

## Last Updated

**Date**: 2026-06-04
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Updated Phase 1 to use two-indent model (`open_indent`, `close_indent`).
Added `proto_brane_formatter_with_result` task for deferred result generation in
Search/HeadTail/Index. Updated Phase 2 verification items: closer at `close_indent`,
HeadTail without ANCHORED arg, operator state at end after operands, deferred result
generation.