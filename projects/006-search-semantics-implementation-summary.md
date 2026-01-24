# Search Semantics Implementation Summary

## Overview

This document summarizes the implementation and testing of search semantics in Foolish, specifically addressing the distinction between `???` (NK - Not Known) and `CONSTANIC` states for anchored vs level-skipping searches.

## Implementation Status

### ✅ Completed

1. **Documentation Created**
   - `docs/search-semantics.md`: Comprehensive documentation of search semantics
   - Updated `AbstractSearchFiroe.java` JavaDoc with search semantics overview

2. **Test Cases Implemented**
   - 6 new approval tests covering anchored and level-skipping search scenarios
   - All tests passing (48 total UBC approval tests)

3. **Test Coverage**

   **Anchored Searches:**
   - `anchoredSearchOnConstant.foo`: Searches on CONSTANT branes succeed
   - `anchoredSearchFailsOnConstant.foo`: Searches fail and return `???`
   - `anchoredSearchOnConstanic.foo`: Chained searches on `???` anchors

   **Level-Skipping Searches:**
   - `levelSkippingSearchFound.foo`: Identifier found, CONSTANT result
   - `levelSkippingSearchNotFound.foo`: Identifier not found, CONSTANIC state
   - `levelSkippingSearchConstanic.foo`: Found value is CONSTANIC, propagates CONSTANIC

## Key Findings

### Anchored Search Behavior (Verified)

**Returns `???` (NK) when:**
1. ✅ Anchor brane is empty (e.g., `{}^` or `{}$`)
2. ✅ Anchor itself is `???` (chained searches on NK)
3. ✅ Anchor brane is CONSTANT but identifier not found (e.g., `{α=100}?γ`)
4. (Implicit) Searching on non-brane values

**Current Implementation:**
- `OneShotSearchFiroe.executeSearch()`: Returns NK if brane is empty
- `RegexpSearchFiroe.executeSearch()`: Returns NK if pattern not found
- `AbstractSearchFiroe.performSearchStep()`: Returns NK if anchor is constanic or unwrapping fails

### Level-Skipping Search Behavior (Verified)

**Important**: Only unanchored searches are level-skipping:
- `x` - identifier reference (searches current brane, then parents)
- `?pattern` - unanchored regex search (NOT `B?pattern` which is anchored to B)

**Uses CONSTANIC when:**
1. ✅ Identifier not found in scope chain (rendered as `⎵⎵`)
2. ✅ Found identifier has CONSTANIC value (propagates CONSTANIC)
3. ✅ Found identifier becomes CONSTANT (completes normally)

**Current Implementation:**
- `IdentifierFiroe.step()`: Sets state to CONSTANIC if identifier not found via `braneMemory.get()`
- `IdentifierFiroe.isConstanic()`: Returns true if value is null or value.isConstanic()
- Level-skipping is inherent in `BraneMemory.get()` which traverses parent branes

## Semantic Clarifications Documented

### 1. Anchored Searches Can Return CONSTANT `???` Even on Constanic Branes

When a brane B is still Constanic but its identifiers have been accumulated (idFIRs created), an anchored search on B can return CONSTANT `???` rather than remaining Constanic. This is correct behavior - the search knows definitively that the identifier is not present.

**Example Scenario (documented as TODO):**
```foolish
{
  b = {
    x = 10;
    y = ?;  // Unspecified parameter - makes b Constanic
  };

  result = b?x;  // Should be CONSTANT 10, even though b is Constanic
}
```

### 2. Level-Skipping CONSTANIC is Intentionally Alarming

When an identifier is not found in the scope chain, returning CONSTANIC (rather than NK) is intentional. The semantics of Foolish dictate that identifier resolution is precisely known by the UBC. The CONSTANIC state signals to the coder that they may have forgotten to declare an attachment or detachment.

**Exception:** Universal constants like `pi` should be available without detachment (future feature).

### 3. CMFir Wrapping for CONSTANIC Values

When a level-skipping search finds an identifier whose value is CONSTANIC, the result is wrapped in a CMFir (Context Manipulation FIR) to properly handle the "Stay Foolish" behavior and re-evaluation semantics.

## Code Locations

**Search Implementation:**
- `AbstractSearchFiroe.java`: Common search logic and state machine
- `OneShotSearchFiroe.java`: Head/tail search (B^, B$)
- `RegexpSearchFiroe.java`: Pattern-based search (B?pattern, B.pattern)
- `IdentifierFiroe.java`: Identifier reference (level-skipping)

**Support Classes:**
- `NKFiroe.java`: Not-known value representation (???)
- `CMFir.java`: Context manipulation for CONSTANIC values
- `BraneMemory.java`: Memory search with parent traversal
- `Query.java`: Search query patterns (StrictlyMatchingQuery, RegexpQuery)

**Tests:**
- `foolish-core-java/src/test/resources/org/foolish/fvm/inputs/anchored*.foo`
- `foolish-core-java/src/test/resources/org/foolish/fvm/inputs/levelSkipping*.foo`
- `UbcApprovalTest.java`: Test runner

## TODOs and Future Work

### TODO: Partial Parameter Specification Test

**Status:** Documented but not yet implemented

**Description:** A test case where we partially specify parameters in a brane and query for their values while the brane is still Constanic. This should demonstrate that anchored searches can return CONSTANT results even when the anchor brane is Constanic.

**Rationale:** The current implementation already supports this behavior (searches become CONSTANT when identifiers are accumulated), but we don't have an explicit test demonstrating it.

**Proposed Test Case:**
```foolish
{
  b = {
    x = 10;
    y = ?;  // Unspecified parameter
  };

  result = b?x;  // Should be CONSTANT 10
  missing = b?z; // Should be CONSTANT ???
}
```

### TODO: Resolve Stage Semantics

**Status:** Needs clarification

The RESOLVED state currently acts as a transitional state before search execution, but its relationship to CONSTANIC needs better documentation. Questions:
- What exactly does RESOLVED mean for a search?
- Can a search remain RESOLVED indefinitely?
- How does RESOLVED relate to the anchor's constancy?

### TODO: Recursive Search Behavior

**Status:** Not yet specified or tested

Searches that reference themselves (recursive searches) may need special handling:
- Should recursive searches be allowed?
- How should cycles be detected and handled?
- What state should cyclic searches have (CONSTANIC? Error?)

### TODO: Multi-Level Search Constancy Rules

**Status:** Needs refinement

When level-skipping searches traverse parent branes, the constancy rules need clarification:
- What happens if an intermediate parent is CONSTANIC?
- Should the search continue to higher parents or stop?
- How does CMFir wrapping interact with multi-level searches?
- Does the search become CONSTANIC if any parent in the chain is CONSTANIC?

## Test Statistics

- **Total UBC Approval Tests:** 48
- **New Search Tests:** 6
- **Test Status:** ✅ All passing
- **Test Execution Time:** ~1.4 seconds

## Verification Commands

```bash
# Run all approval tests
mvn test -Dtest=UbcApprovalTest

# Run only search-related tests
mvn test -Dtest=UbcApprovalTest -Dfoolish.test.filter=Search

# Run anchored search tests
mvn test -Dtest=UbcApprovalTest -Dfoolish.test.filter=anchoredSearch

# Run level-skipping tests
mvn test -Dtest=UbcApprovalTest -Dfoolish.test.filter=levelSkipping
```

## References

- **Main Documentation:** `docs/search-semantics.md`
- **Code Documentation:** `AbstractSearchFiroe.java` (lines 7-47)
- **Test Inputs:** `foolish-core-java/src/test/resources/org/foolish/fvm/inputs/`
- **Approved Outputs:** `foolish-core-java/src/test/resources/org/foolish/fvm/ubc/*.approved.foo`

---

## Last Updated

**Date**: 2026-01-25
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-5-20250929
**Changes**: Clarified search operators. `B/pattern` is forward anchored search (not yet implemented). Unanchored `/pattern` is NOT in syntax. Find-all `??` and `//` are reserved for future.
