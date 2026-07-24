---
foop: D54
title: Parenthetical search chaining and contexted search after parenthetical
author: Sisyphus <sisyphus@foolish.dev>
status: Draft
type: Standards
created: 2026-07-24
phase: phase-2
supersedes: []
begun: [ ] 
---

# FOOP-45: Parenthetical search chaining and contexted search after parenthetical

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

## Abstract

This FOOP investigates and fixes issues with parenthetical search chaining and contexted search (`&#`) after parenthetical expressions in the UBCa FVM. Currently, `(expr)~name1~name2` fails to chain searches correctly, and `(expr)~name&#1` fails to apply contexted search. The root cause appears to be parser behavior treating `~name1~name2` as a single pattern rather than chained searches. This FOOP proposes fixing the parser to correctly handle parenthetical search results as proper anchors for subsequent searches.

## Motivation

### The Problem

When navigating complex brane hierarchies (like biological taxonomies), users need to chain multiple searches together. The natural syntax would be:

```foolish
(Magnoliopsida~Fagales)~Fagaceae~Quercus~English_Oak~Acorns
```

This should:
1. Search forward in Magnoliopsida for Fagales → returns Fagales brane
2. Search forward in Fagales for Fagaceae → returns Fagaceae brane
3. Search forward in Fagaceae for Quercus → returns Quercus brane
4. Search forward in Quercus for English_Oak → returns English_Oak brane
5. Search forward in English_Oak for Acorns → returns value 0

However, the current implementation fails at step 2, returning NK (Not Knowable) instead of the Fagaceae brane.

### Current Behavior (Incorrect)

From the `search_based_navigation.foo` snapshot test:

```foolish
!! Parenthetical grouping of searches
path_19 = (Magnoliopsida~Fagales)~Fagaceae;                                     !! finds Fagaceae brane ✓
path_20 = (Magnoliopsida~Fagales)~Fagaceae~Quercus;                             !! NK ✗ (expected Quercus brane)
path_21 = (Magnoliopsida~Fagales)~Fagaceae~Quercus~English_Oak;                 !! NK ✗ (expected English_Oak brane)
path_22 = (Magnoliopsida~Fagales)~Fagaceae~Quercus~English_Oak~Acorns;          !! NK ✗ (expected 0)
```

### Workaround (Double Parenthetical)

Using double parenthetical grouping works:

```foolish
!! Nested parenthetical grouping
path_23 = ((Magnoliopsida~Fagales)~Fagaceae)~Quercus;                           !! finds Quercus brane ✓
path_24 = ((Magnoliopsida~Fagales)~Fagaceae)~Quercus~English_Oak;               !! NK ✗ (still fails with multiple chains)
path_25 = (((Magnoliopsida~Fagales)~Fagaceae)~Quercus)~English_Oak;             !! finds English_Oak brane ✓
```

### Contexted Search Issue

Contexted search (`&#`) after parenthetical also fails:

```foolish
!! Parenthetical with contexted search
path_26 = (Magnoliopsida~Fagales)~Fagaceae~Quercus~English_Oak~Acorns&#1;       !! NK ✗ (expected 1 - Cupules)
path_27 = (Magnoliopsida~Fagales)~Fagaceae~Quercus~English_Oak~Acorns&#2;       !! NK ✗ (expected 2 - Lobed_leaves)
path_28 = (Magnoliopsida~Fagales)~Fagaceae~Quercus~English_Oak~Acorns&#-1;      !! NK ✗ (expected NK - no previous)
```

But contexted search with dot access works:

```foolish
!! Mixed: dot access then search then contexted
path_29 = Magnoliopsida.Fagales.Fagaceae.Quercus.English_Oak~Acorns&#1;         !! 1 ✓ (Cupules)
path_30 = Magnoliopsida.Fagales.Fagaceae.Quercus.English_Oak~Lobed_leaves&#-1;  !! 1 ✓ (Cupules)
```

### Root Cause Analysis

The parser appears to be treating `(expr)~name1~name2` as searching for the pattern `name1~name2` rather than chaining two separate searches. This is incorrect because:

1. `~` is a search operator, not part of a name pattern
2. Parenthetical expressions should return a brane that can be used as an anchor for subsequent searches
3. Contexted search (`&#`) should work after any search result, including parenthetical searches

### Impact

This issue affects:
- Complex navigation through deep brane hierarchies
- Any workflow that chains multiple searches together
- Contexted search after parenthetical expressions
- Value search (`~=` and `?=`) after parenthetical expressions

## Specification

### Issue 1: Parenthetical Search Chaining

**Current behavior**: `(expr)~name1~name2` is parsed as searching for pattern `name1~name2`

**Expected behavior**: `(expr)~name1~name2` should be parsed as:
1. Evaluate `(expr)` → returns brane B1
2. Search B1 for `name1` → returns brane B2
3. Search B2 for `name2` → returns result

**Grammar change**: The parser needs to recognize that `~` is a search operator that terminates the current search pattern and starts a new search.

### Issue 2: Contexted Search After Parenthetical

**Current behavior**: `(expr)~name&#1` is parsed incorrectly

**Expected behavior**: `(expr)~name&#1` should be parsed as:
1. Evaluate `(expr)` → returns brane B1
2. Search B1 for `name` → returns result R1 with position P1
3. Contexted search from P1 with offset 1 → returns result R2

**Grammar change**: The parser needs to recognize `&#` as a contexted search operator that applies to the result of the previous search.

### Issue 3: Value Search After Parenthetical

**Current behavior**: `(expr)~name~=0` is parsed incorrectly

**Expected behavior**: `(expr)~name~=0` should be parsed as:
1. Evaluate `(expr)` → returns brane B1
2. Search B1 for `name` → returns brane B2
3. Value search B2 for value 0 → returns result

**Grammar change**: The parser needs to recognize `~=` as a value search operator that applies to the result of the previous search.

### Proposed Parser Changes

The parser should implement the following rules:

1. **Search operator termination**: When encountering `~`, `?`, `~=`, `?=`, `&#`, `&~`, `&?`, `&~=`, `&?=`, the current search pattern is complete and a new search begins.

2. **Parenthetical result anchoring**: The result of a parenthetical expression can be used as an anchor for subsequent searches.

3. **Contexted search chaining**: Contexted search operators (`&#`, `&~`, `&?`, `&~=`, `&?=`) apply to the result of the immediately preceding search.

### Grammar Fragment

```ebnf
search_expression :=
    | anchor '~' name_pattern                    !! forward name search
    | anchor '?' name_pattern                    !! backward name search
    | anchor '~=' value_pattern                  !! forward value search
    | anchor '?=' value_pattern                  !! backward value search
    | search_expression '~' name_pattern         !! chained forward search
    | search_expression '?' name_pattern         !! chained backward search
    | search_expression '~=' value_pattern       !! chained forward value search
    | search_expression '?=' value_pattern       !! chained backward value search
    | search_expression '&' '~' name_pattern     !! contexted forward search
    | search_expression '&' '?' name_pattern     !! contexted backward search
    | search_expression '&' '#' offset           !! contexted index search
    | search_expression '&' '~=' value_pattern   !! contexted forward value search
    | search_expression '&' '?=' value_pattern   !! contexted backward value search
    | '(' search_expression ')'                  !! parenthetical grouping

anchor :=
    | name                                       !! simple name
    | anchor '.' name                            !! dot access
    | '(' expression ')'                         !! parenthetical expression
```

## FIR Impact

This change primarily affects the parser, not the FIR structure. The FIR types for search results remain the same:

```rust
pub struct SearchFir {
    pub(crate) anchor: FirRef,
    pub(crate) pattern: SearchPattern,
    pub(crate) direction: SearchDirection,
    pub(crate) state: Nyes,
}

pub enum SearchPattern {
    Name(String),
    Value(FirRef),
    NameValue { name: String, value: FirRef },
    Index(i64),
    Head,
    Tail,
}
```

## UBC Step Impact

The evaluator step rules for search operations need to be updated to handle chained searches correctly. The key change is:

**Before**: `(expr)~name1~name2` is evaluated as a single search with pattern `name1~name2`

**After**: `(expr)~name1~name2` is evaluated as:
1. Evaluate `(expr)` → brane B1
2. Evaluate `B1~name1` → brane B2
3. Evaluate `B2~name2` → result

The step rules for search evaluation need to recognize when a search result is being used as an anchor for a subsequent search.

## Test Plan

### Unit Tests

1. **Parser tests**: Verify that `(expr)~name1~name2` is parsed as chained searches, not a single search with pattern `name1~name2`

2. **Evaluator tests**: Verify that chained searches after parenthetical work correctly

3. **Contexted search tests**: Verify that `&#`, `&~`, `&?`, `&~=`, `&?=` work after parenthetical searches

### Approval Tests

1. **Update `search_based_navigation.foo`**: Fix the expected values for paths 20-28, 43-45, 48, 50-51

2. **New test `foop_45_comprehensive.foo`**: Comprehensive test covering:
   - Single parenthetical with chained searches
   - Double parenthetical with chained searches
   - Contexted search after parenthetical
   - Value search after parenthetical
   - Mixed dot access and parenthetical searches
   - Edge cases: empty branes, NK results, nested parenthetical

### Regression Tests

1. **Existing tests**: Verify that all existing search tests still pass
2. **Dot access tests**: Verify that dot access still works correctly
3. **Contexted search tests**: Verify that contexted search with dot access still works

## Rejected Alternatives

### A. Do Nothing

**Description**: Leave the current behavior as-is and document the workaround (double parenthetical).

**Reason for rejection**: The workaround is cumbersome and unintuitive. Users expect `(expr)~name1~name2` to work naturally. The current behavior violates the principle of least surprise and makes complex navigation unnecessarily difficult.

### B. Require Explicit Parenthetical for Each Search

**Description**: Require `((expr)~name1)~name2` for all chained searches.

**Reason for rejection**: This is even more cumbersome than the double parenthetical workaround and makes the syntax overly verbose. The natural chaining syntax should work as expected.

### C. Change Search Operator Precedence

**Description**: Make `~` bind tighter than parenthetical grouping.

**Reason for rejection**: This would break existing behavior where `(expr)~name` works correctly. The issue is not precedence but parser recognition of search operator termination.

## Open Questions

- Should the parser support unlimited chaining of searches, or should there be a limit?
- Should contexted search operators (`&#`, `&~`, etc.) have different precedence than regular search operators?
- How should the parser handle mixed chains like `(expr)~name1&#1~name2`?
- Should there be a special syntax for "search and then search within result" vs "search and then search from position"?

## References

- Prior FOOPs: FOOP-23 (Value search and contexted search)
- Code locations: 
  - `foolish-ubca/src/ubca_snapshot_tester.rs` (snapshot test runner)
  - `foolish-ubca/snapshot_tests/input/search_based_navigation.foo` (test file with issues)
- External docs: None

## Last Updated

**Date**: 2026-07-24
**Updated By**: Sisyphus / xiaomi/mimo-v2.5-pro
**Changes**: Initial draft of FOOP-45 investigating parenthetical search chaining issues
