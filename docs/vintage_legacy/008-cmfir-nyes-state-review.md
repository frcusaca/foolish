# CMFir Nyes State Review and Fixes

## Overview

Reviewed and fixed CMFir (Context Manipulation FIR) implementation after recent changes to the Nyes state machine. Found and fixed critical bugs in state checking logic.

## Critical Bugs Fixed

### Bug 1: Phase A Using `isConstanic()` Instead of `atConstanic()`

**File**: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/CMFir.java`
**Lines**: 37, 49

**Problem**:
The original code used `isConstanic()` which returns true for both CONSTANIC and CONSTANT states (checks `nyes.ordinal() >= Nyes.CONSTANIC.ordinal()`). This caused Phase B to start even when `o` reached CONSTANT state, which is incorrect behavior.

**Before (BUGGY)**:
```java
// Phase A: Step o until it is CONSTANT or CONSTANIC
if (!phaseBStarted) {
    if (o.getNyes() == Nyes.CONSTANT) {
        if (o.isConstanic()) {  // BUG: Always true when CONSTANT!
            startPhaseB();
            return 1;
        }
        setNyes(Nyes.CONSTANT);
        return 1;
    }
    o.step();
    return 1;
}
```

**After (FIXED)**:
```java
// Phase A: Step o until it is CONSTANT or CONSTANIC
if (!phaseBStarted) {
    // Check if o is CONSTANIC *before* stepping.
    if (o.atConstanic()) {
        startPhaseB();
        return 1;
    }

    try {
        o.step();
    } catch (Exception e) {
        throw e;
    }

    // After stepping, check if o reached CONSTANIC
    if (o.atConstanic()) {
        startPhaseB();
        return 1;
    }

    if (o.getNyes() == Nyes.CONSTANT) {
        // O reached CONSTANT without being CONSTANIC
        // This means it's fully resolved (not abstract)
        setNyes(Nyes.CONSTANT);
        return 1;
    }
    return 1;
}
```

**Correct Behavior**: Phase B should only start when `o` is in the CONSTANIC state (not fully resolved). When `o` reaches CONSTANT without being CONSTANIC, it means the expression is fully resolved in the original context and doesn't need re-evaluation in a new context.

### Bug 2: `getValue()` Using Complex Logic Instead of `atConstant()`

**File**: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/CMFir.java`
**Lines**: 103-121

**Problem**:
The original code used `isConstanic()` as a guard and then checked `getNyes() != Nyes.CONSTANT`, which was unnecessarily complex. Should directly check `!atConstant()` to verify full evaluation.

**Before (COMPLEX)**:
```java
public long getValue() {
    if (phaseBStarted) {
        if (o2.isConstanic()) {
             if (o2 instanceof AssignmentFiroe af && af.getResult() instanceof NKFiroe nk) {
                 throw new IllegalStateException("Cannot get value from NK (not-known): " + nk.getNkComment());
             }
             if (o2.getNyes() != Nyes.CONSTANT) {
                throw new IllegalStateException("CMFir not fully evaluated (o2 not constant)");
             }
             // If CONSTANT but still constanic (e.g. NK directly)
             if (o2 instanceof NKFiroe nk) {
                 throw new IllegalStateException("Cannot get value from NK (not-known): " + nk.getNkComment());
             }
        }
        return o2.getValue();
    } else {
        return o.getValue();
    }
}
```

**After (SIMPLIFIED)**:
```java
public long getValue() {
    if (phaseBStarted) {
        // Check for NK (not-known) cases before attempting getValue
        if (o2 instanceof AssignmentFiroe af && af.getResult() instanceof NKFiroe nk) {
            throw new IllegalStateException("Cannot get value from NK (not-known): " + nk.getNkComment());
        }
        if (o2 instanceof NKFiroe nk) {
            throw new IllegalStateException("Cannot get value from NK (not-known): " + nk.getNkComment());
        }
        // Verify o2 is fully evaluated
        if (!o2.atConstant()) {
            throw new IllegalStateException("CMFir not fully evaluated (o2 not constant, state: " + o2.getNyes() + ")");
        }
        return o2.getValue();
    } else {
        return o.getValue();
    }
}
```

## Re-enabled CMFir Unit Tests

**File**: `foolish-core-java/src/test/java/org/foolish/fvm/ubc/CMFirUnitTest.java`

The CMFir unit tests were previously disabled with `@Disabled("CMFir dynamic scoping implementation needs further investigation")`. After fixing the bugs, we re-enabled them.

### Test Fix: Parent Chain Evaluation

**Problem**: The `evaluateFully()` helper only stepped items in the immediate context, not the parent chain. This caused the `testShadowing` test to fail because items in the parent scope (`root`) weren't evaluated.

**Before (BUGGY)**:
```java
private void evaluateFully(BraneMemory context, FIR fir) {
    // Ensure context items are stepped
    context.stream().forEach(f -> {
        int steps = 0;
        while (f.isNye() && steps < 100) {
            f.step();
            steps++;
        }
    });

    int steps = 0;
    while (fir.isNye() && steps < 1000) {
        fir.step();
        steps++;
    }
    if (steps >= 1000) {
         throw new RuntimeException("Evaluation timed out or stuck: " + fir);
    }
}
```

**After (FIXED)**:
```java
private void evaluateFully(BraneMemory context, FIR fir) {
    // Ensure all context items in the entire parent chain are stepped
    BraneMemory current = context;
    while (current != null) {
        current.stream().forEach(f -> {
            int steps = 0;
            while (f.isNye() && steps < 100) {
                f.step();
                steps++;
            }
        });
        current = current.getParent();
    }

    int steps = 0;
    while (fir.isNye() && steps < 1000) {
        fir.step();
        steps++;
    }
    if (steps >= 1000) {
         throw new RuntimeException("Evaluation timed out or stuck: " + fir);
    }
}
```

## Test Results

### Before Fixes
- 118 tests passing
- 3 tests skipped (CMFirUnitTest disabled)

### After Fixes
- **121 tests passing**
- **0 tests skipped**
- All CMFir tests now work:
  - `testSimpleDynamicScoping` - PASS
  - `testReEvaluationInDifferentScope` - PASS
  - `testShadowing` - PASS

## CMFir State Checking Summary

After review, here's the correct usage of state-checking methods in CMFir:

| Method | Returns True When | Use Case |
|--------|------------------|----------|
| `atConstant()` | `nyes == Nyes.CONSTANT` | Check if exactly CONSTANT (fully evaluated) |
| `atConstanic()` | `nyes == Nyes.CONSTANIC` | Check if exactly CONSTANIC (paused, awaiting context) |
| `isConstant()` | `nyes >= Nyes.CONSTANT` | Check if at least CONSTANT (just CONSTANT) |
| `isConstanic()` | `nyes >= Nyes.CONSTANIC` | Check if at least CONSTANIC (CONSTANIC or CONSTANT) |

**Key Insight**: Use `atConstant()` and `atConstanic()` for precise state checks when control flow depends on distinguishing between the two states. Use `isConstanic()` only when you want to catch "at least CONSTANIC" and then refine with additional checks.

## Related Files

**Source Code**:
- `foolish-core-java/src/main/java/org/foolish/fvm/ubc/CMFir.java` - Fixed lines 37, 49, 103-121
- `foolish-core-java/src/main/java/org/foolish/fvm/ubc/FIR.java:71-77` - State checking methods

**Tests**:
- `foolish-core-java/src/test/java/org/foolish/fvm/ubc/CMFirUnitTest.java` - Re-enabled and fixed

## What CMFir Does

CMFir implements "Stay Foolish" behavior - evaluating code defined in one scope within a different scope (dynamic scoping).

### Two-Phase Evaluation

**Phase A**: Step the original FIR (`o`) until it reaches a final state:
- If `o` reaches **CONSTANIC**: Start Phase B (needs re-evaluation in new context)
- If `o` reaches **CONSTANT**: Done (fully resolved in original context)

**Phase B**: Clone and re-evaluate in new context:
1. Create `stayFoolishClone(o)` → `o2`
2. Link `o2`'s braneMemory parent to CMFir's braneMemory
3. Step `o2` in the new context
4. Mirror `o2`'s state to CMFir

### Example

```foolish
// Define expression where a, b are not yet known
r = a + b

// Evaluate in scope where a=1, b=2
// Phase A: r is CONSTANIC (needs a, b)
// Phase B: Clone r, evaluate with a=1, b=2 → 3
```

---

## Last Updated

**Date**: 2026-01-26
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-5-20250929
**Changes**: Documented CMFir review findings and fixes. Fixed critical bugs in Phase A state checking and `getValue()` logic. Re-enabled and fixed all CMFir unit tests. All 121 Java tests now passing.
