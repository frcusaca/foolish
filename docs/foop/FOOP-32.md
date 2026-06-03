---
foop: 23
title: Repair rudimentary FVM evaluation and Sequencer formatting bugs found in snapshot review
author: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
status: Final
type: Bugfix
created: 2026-06-01
phase: phase-2
supersedes: []
---

# FOOP-32: Repair rudimentary FVM evaluation and Sequencer formatting bugs

## Abstract

Fix six bugs discovered during human review of snapshot test outputs. Three are FVM
evaluation bugs (scope resolution across brane boundaries, operator precedence between
search and concatenation, asymmetric boundary clamping for negative seeks). Three are
Sequencer formatting bugs (missing `{}` delimiters for anonymous branes, misplaced
`search_result` child formatting, and a related scope resolution failure inside nested
branes).

## Motivation

These bugs were found when reviewing `.snap.new` files in
`foolish-core/snapshot_tests/approved/`. The human reviewer annotated specific lines
with `@Agent` comments indicating incorrect output. The bugs affect both correctness
(scope resolution, precedence, boundary handling) and readability (HSSnap formatting).

A detailed bug catalog exists at `docs/foop/FOOP-32.bugs.md` (moved from the temporary
`FOOP-22.bugs.md`). This FOOP specifies the fixes; the plan file tracks implementation.

## Specification

### Bug A: Forward reference resolves across two brane boundaries (Critical)

**File:** `complex_forward_refs_in_nested_branes.foo.snap.new`
**Input:** `{nested = {inner = {val = x}}; x = 42;}`

**Current (wrong):** `val` resolves to `Int(42)`.
**Expected:** `val` should be a `Search` FIR in WOCONSTANIC/ECOSTANIC state — `x` is
blocked by two brane boundaries (`nested` and `inner`) and appears AFTER the reference
in source order.

**Fix:** The FVM search must respect brane depth. When searching for an identifier from
within a nested brane, the search should not penetrate more than one brane boundary into
the parent scope unless the identifier was defined BEFORE the nested brane in source
order. The exact rule depends on the AB/IB recoordination semantics — the search should
fail when the target is both (a) in a parent brane AND (b) defined after the nested
brane that contains the reference.

### Bug B: Search has higher precedence than concatenation (Critical)

**File:** `complex_search_and_concatenation.foo.snap.new`
**Input:** `{target = {a=1; c=2; c={a=1, b=2, c=3}}; b1 = {x=10}; result = b1 target.c;}`

**Current (wrong):** `result` is `{x=10}` — only `b1`, `target.c` is lost.
**Expected:** `result` should be `{x=10; a=1; b=2; c=3}` — concatenation of `b1` and
`target.c`.

**Fix:** The expression `b1 target.c` should parse as two operands `(b1)` and `(target.c)`
being implicitly concatenated. Currently, search (`target.c`) has higher precedence than
concatenation, causing `b1 target.c` to be interpreted as a search operation on `b1`
rather than two independent operands. The parser or evaluation order needs adjustment so
that implicit concatenation binds at the correct level. Parenthesizing as `b1 (target.c)`
is suspected to work — confirming this will isolate whether the fix is in the parser
(precedence) or the evaluator (order of operations).

### Bug C: Search for `sum` fails inside nested brane (Critical)

**File:** `complex_full_program_with_all_features.foo.snap.new`
**Input:** `{a = 10; b = 20; sum = a + b; nested = {inner = sum / 2}; result = nested.inner;}`

**Current (wrong):** `sum` search inside `nested` is WOCONSTANIC — `sum = Int(30)` is
visible in the parent brane but not found.
**Expected:** `sum` should resolve to `Int(30)`, `inner` should be `Int(15)`.

**Fix:** The search for `sum` from within the `nested` brane should cross into the parent
brane and find `sum = Int(30)`. This is the opposite failure mode of Bug A — here the
search fails to go outward when it should succeed. The distinction: `sum` is defined
BEFORE `nested` in source order, so it should be visible. The fix likely involves the
same AB/IB recoordination logic as Bug A — the search boundary is too restrictive for
identifiers defined before the nested brane.

### Bug D: Negative seek out-of-bounds clamps to first element instead of NK (High)

**File:** `anchored_seek_negative_boundary.foo.snap.new`
**Input:** `{b = {10; 20; 30}; last = b#-1; second = b#1; first = b#-3; oob = b#-4;}`

**Current (wrong):** `oob = Int(10)` — `b#-4` on a 3-element brane returns the first element.
**Expected:** `oob` should be `NK` or an `Index` FIR in NK state.

**Contrast:** `anchored_seek_positive_boundary.foo.snap` correctly produces
`Index(offset=3, ANCHORED, [NK])` for `b#3` on a 3-element brane.

**Fix:** The negative seek clamping logic is asymmetric. Positive OOB correctly produces
NK; negative OOB incorrectly clamps to index 0 (first element). The fix is in the Index
FIR evaluation — apply the same bounds check for negative offsets as for positive offsets.

### Bug E: Brane HSSnap output missing `{}` curly bracket enclosures (Medium)

**Files:** `anchored_seek_positive_boundary`, `anchored_seek_negative_boundary`,
`anchored_seek_positive_negative` (and likely others)

**Current (wrong):**
```
  Brane
    Int(10)
    Int(20)
```

**Expected:**
```
  Brane{
    Int(10)
    Int(20)
  }
```

**Fix:** The Sequencer's `format_fir` for `NormalBraneFir` must emit `{` after `Brane`
and `}` after the last child element. Named branes (with `name =` entries) may omit
braces if the convention is to only show them for anonymous/positional branes, but the
human reviewer indicated that multi-line brane output should always show delimiters.

### Bug F: Search `search_result` appears outside `()` instead of as indented child (Medium)

**File:** `chained_undeclared.foo.snap.new`

**Current (wrong):** The nested `search_result` appears as a separate line after the
parent Search's closing `)`, not as an indented child.
**Expected:** The `search_result` (target) should be formatted as an indented child of
the Search FIR, consistent with how `Operator` formats its operands.

**Fix:** In `sequencer.rs`, the `hs_search` formatting already calls `format_fir_q` on
the target with `depth + 1`. The issue is likely that the output appears on the same
line as the parent Search (concatenated) rather than on a new indented line. Check the
newline/indent logic in `format_fir_q` when called from the Search formatter.

## FIR Impact

None. These are evaluation and formatting fixes, not FIR structure changes.

## UBC Step Impact

Bugs A, B, C, D affect evaluation steps:
- **Bug A & C:** Search step — brane boundary crossing rules
- **Bug B:** Parser precedence or evaluation order for implicit concatenation
- **Bug D:** Index step — bounds checking for negative offsets

Bugs E & F affect only the Sequencer (HSSnap formatting), not evaluation.

## Test Plan

Each bug has a corresponding `.snap.new` file that documents the current (wrong) output.
After each fix:

1. Run `cargo insta test -p foolish-core --lib` to regenerate the affected `.snap.new` files.
2. Verify the new output matches the expected behavior described above.
3. Human reviews the updated `.snap.new` files.
4. On approval (`@Agent, LGTM`), agent promotes to `.snap` (stripping approval line).
5. Run `cargo test -p foolish-core --lib` — all tests pass.

Additionally:
- Add unit tests in `sequencer_tests.rs` for Bugs E and F (formatting).
- Add unit tests in `unit_tests.rs` for Bug D (negative seek bounds).
- Bugs A, B, C may require new unit tests in the UBC evaluation module.

## Rejected Alternatives

### A. Fix all bugs in one commit

Bugs A-C (scope resolution) are interrelated but Bug D (bounds clamping) and Bugs E-F
(formatting) are independent. Splitting by category reduces merge risk and makes each
fix independently verifiable.

### B. Defer scope resolution bugs (A, B, C) to a later FOOP

These are rudimentary correctness bugs — the FVM produces wrong answers for basic
programs. Deferring them would leave the test suite with known-wrong snapshots, which
violates the project's rule: "NEVER start large project segment work WHEN ANY tests are
broken." Fix them now.

### C. Change the HSSnap format to use different delimiters (e.g., `[` and `]`)

The human reviewer specifically requested `{` and `}` to match Foolish's brane syntax.
Other delimiters would be inconsistent with the language.

## Open Questions

- **Bug A vs Bug C:** These appear to be opposite failure modes of the same underlying
  mechanism (brane boundary crossing in search). The fix may be a single change to the
  AB/IB recoordination logic. Confirm by investigating the search implementation.
- **Bug B:** Is the fix in the parser (precedence table) or the evaluator (order of
  operations)? Test with `b1 (target.c)` to isolate.
- **Bug E:** Should named branes also show `{}` or only anonymous/positional branes?
  The reviewer's comments suggest all multi-line branes should show delimiters.

## References

- Bug catalog: `docs/foop/FOOP-32.bugs.md`
- Affected snapshot files: `foolish-core/snapshot_tests/approved/*.foo.snap.new`
- Sequencer: `foolish-core/src/sequencer.rs`
- UBC evaluation: `foolish-core/src/ubc.rs`
- Search implementation: `foolish-core/src/search.rs`
- Related: FOOP-7 (Constanic Clone — recoordination contract)
