# Search Semantics in Foolish

This document describes the semantics of search operations in Foolish, particularly focusing on when searches result in `???` (NK - Not Known) vs `CONSTANIC` states, and how anchored vs level-skipping searches behave differently.

## Search Types

### Anchored Searches
Anchored searches look within a specific brane only (do not traverse parent branes):
- `B^` - head search (first element of brane B)
- `B$` - tail search (last element of brane B)
- `B#123` - indexed search (element at index 123 in brane B)
- `B/pattern` - forward anchored search (finds first match from start to end of B)
- `B?pattern` - backward anchored search (finds latest match from end to start of B)
- `B.pattern` - alias for `B?pattern` (backward anchored search)

**Note**: `B.x` behaves like object member/method access in Java/C++/Python - it finds the latest definition of x within B.

**Example**:
```foolish
{a1=1; a2=2; a3=3}/a.*  // Returns 1 (first match: a1)
{a1=1; a2=2; a3=3}?a.*  // Returns 3 (last match: a3)
{a1=1; a2=2; a3=3}.a.*  // Returns 3 (same as ?, alias)
```

### Level-Skipping Brane-Boundary Insensitive Searches
These searches traverse parent branes (unanchored):
- `x` - identifier reference by name (searches current brane, then parents)
- `?pattern` - unanchored regex search (searches backwards in current brane, then parent branes)

**Important**: Unanchored forward search `/pattern` is **not part of the syntax**. It conflicts with division operator and its semantics are too complex for the current UBC implementation.

### Find-All Searches (Not Yet Implemented)
These operators for finding all matches are **reserved for future implementation**:
- `B??pattern` - find all matches in B (local)
- `B//pattern` - find all matches in B and parents (global)
- `??pattern` - unanchored find-all (backwards, with parents)
- `//pattern` - NOT in syntax (conflicts with division)

## NK (`???`) vs CONSTANIC States

### When `???` (NK) is Returned

**Anchored searches return `???` when:**

1. **The anchor brane is empty or the search fails**
   ```foolish
   e = {}^;  // ??? - empty brane has no head
   f = {}$;  // ??? - empty brane has no tail
   ```

2. **The anchor itself evaluates to `???`**
   ```foolish
   badAnchor = {}^;
   result = badAnchor$;  // ??? - searching on ??? anchor
   ```

3. **The anchor brane is CONSTANT but search fails**
   When the anchor brane has evaluated to CONSTANT state and all its identifiers have been accumulated (all idFIRs created), but the search does not find a matching identifier:
   ```foolish
   {
     result = {α = 100; β = 200;}?notfound;  // ??? - 'notfound' not in brane
   }
   ```

   **Note**: This search can result in CONSTANT `???` even if the brane B is still Constanic. The key is that by the time the identifier search is performed, the brane's own identifiers have been accumulated.

4. **Searching on non-brane values**
   ```foolish
   x = 42;
   y = x^;  // ??? - cannot search on scalar value
   ```

### When CONSTANIC is Used

**Level-skipping searches use CONSTANIC when:**

1. **Identifier not found in any parent brane**
   ```foolish
   {
     x = undeclaredIdentifier;  // CONSTANIC - not found in scope chain
   }
   ```

   The semantics of Foolish dictate that whether an identifier can be found or not is precisely known by the UBC. This CONSTANIC state is *intentionally alarming* - it signals that the coder may have forgotten to declare an attachment or detachment.

   **Exception**: Universal constants like `pi` should ideally be available without detachment (future feature).

2. **Found value is itself CONSTANIC**
   When an identifier is found but its value is CONSTANIC, the search wraps it in a CMFir (Context Manipulation FIR):
   ```foolish
   {
     x = {y = undeclaredValue;};
     z = x;  // CONSTANIC - x found but contains CONSTANIC value
   }
   ```

3. **Found value becomes CONSTANT**
   When the identifier is found and its value evaluates to CONSTANT, the search completes normally:
   ```foolish
   {
     x = 42;
     y = x;  // CONSTANT 42
   }
   ```

## Implementation Details

### AbstractSearchFiroe State Machine

The search process follows these phases:

1. **INITIALIZED → REFERENCES_IDENTIFIED**: Step non-brane expressions
2. **REFERENCES_IDENTIFIED → ALLOCATED**: Continue stepping
3. **ALLOCATED → RESOLVED**: Continue stepping until anchor is ready
4. **RESOLVED → CONSTANT/CONSTANIC**: Perform search and evaluate result

Key method: `performSearchStep()` in AbstractSearchFiroe.java:163-263

### Anchor Unwrapping

Before searching, anchors are unwrapped:
- `IdentifierFiroe` → its `value`
- `AssignmentFiroe` → its result
- `AbstractSearchFiroe` → its search result

If any unwrapping step encounters CONSTANIC, the search returns `???` (NK).

### Search Execution

Once the anchor is unwrapped to a `BraneFiroe`, the specific search is executed:
- `OneShotSearchFiroe.executeSearch()`: Returns head/tail or NK if empty
- `RegexpSearchFiroe.executeSearch()`: Returns matched identifier or NK if not found

## Future Work and TODOs

### TODO: Test for Partial Parameter Specification
Currently not tested: A scenario where we partially specify parameters in a brane and then query for their values while the brane is still Constanic. This should demonstrate that searches can return CONSTANT results even when the anchor brane is Constanic.

Example test case needed:
```foolish
{
  b = {
    x = 10;
    y = ?;  // Unspecified parameter
  };

  result = b?x;  // Should be CONSTANT 10, even though b is Constanic
}
```

### Resolve Stage Semantics
The RESOLVED state needs clearer semantics. Currently it acts as a transitional state before search execution, but its relationship to CONSTANIC needs documentation.

### Recursive Search Behavior
Recursive searches (searches that reference themselves) may behave differently and need special handling. This is not yet fully specified or tested.

### Level-Skipping Search Constancy
When level-skipping searches traverse parent branes, the constancy rules need refinement:
- What happens if an intermediate parent is CONSTANIC?
- Should the search continue to higher parents or stop?
- How does CMFir wrapping interact with multi-level searches?

## Test Coverage

### Existing Tests
- `oneShotSearchIsApproved.approved.foo`: Tests head/tail search including empty branes
- `regexSearchNotFoundIsApproved.approved.foo`: Tests regex search returning ???
- Various incidental coverage in other approval tests

### Tests Added by This Document
See the new test files in `foolish-core-java/src/test/resources/org/foolish/fvm/inputs/`:
- `anchoredSearchOnConstant.foo`: Anchored search on CONSTANT brane
- `anchoredSearchOnConstanic.foo`: Anchored search on CONSTANIC brane
- `levelSkippingSearchNotFound.foo`: Identifier not found in scope
- `levelSkippingSearchConstanic.foo`: Identifier found but value is CONSTANIC

## References

- AbstractSearchFiroe.java: Common search logic
- OneShotSearchFiroe.java: Head/tail search implementation
- RegexpSearchFiroe.java: Pattern-based search implementation
- NKFiroe.java: Not-known value representation
- CMFir.java: Context manipulation for CONSTANIC values
- IdentifierFiroe.java: Identifier reference and level-skipping search

---

## Last Updated

**Date**: 2026-01-25
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-5-20250929
**Changes**: Clarified search operators: `B/pattern` is forward anchored search (not yet implemented). Unanchored `/pattern` is NOT in syntax (conflicts with division). Find-all operators `??` and `//` are reserved for future implementation.
