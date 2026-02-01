# Concatenation Project

## Overview

Brane concatenation allows multiple branes to be combined into a single evaluation context. This document specifies the semantics and implementation of `ConcatenationFiroe`.

## Semantics Examples

The semantics can be confusing. Consider these cases:

### Example 1: Anonymous branes inline
```foo
{ OB={ a=2; {a=1}{b=1}{c=a}; };
  result = OB$.c }  !! result==1
```
The `{a=1}{b=1}{c=a}` concatenates into a single brane where `c=a` resolves `a` from the first sub-brane (value 1), not from the outer scope (value 2).

### Example 2: Named branes concatenated
```foo
{ OB={ a=2; A={a=1}; B={b=1}; C={c=a}; A B C };
  result = OB$.c }  !! result==2
```
At join time (when `A B C` is instantiated), A, B, C are identifiers that need to be resolved. When C was defined (`C={c=a}`), the `a` inside resolved from the outer scope where `a=2`. The concatenation `A B C` joins already-evaluated branes.

### Example 3: Order matters
```foo
{ OB={ A={a=1}; B={b=1}; C={c=a}; a=2; A B C };
  result = OB$.c }  !! result==1
```
When C is defined, it looks backward and finds `A={a=1}` before reaching any `a=2`. So `c=a` resolves to 1.

### Key Semantic Insight

The difference between Examples 1 and 2:
- **Example 1**: Anonymous branes `{a=1}{b=1}{c=a}` are parsed as a single `AST.Branes` expression. They haven't been evaluated yet, so concatenation controls their shared context.
- **Example 2**: Named branes `A`, `B`, `C` are identifiers referencing already-assigned branes. By the time `A B C` executes, each brane has already resolved its internal references in its original context.

## ConcatenationFiroe Design Specification

### AST Source

- Created from `AST.Branes` (grammar rule: `branes: brane+`)
- Currently `AST.Branes` falls through to `NKFiroe` in `FIR.createFiroeFromExpr()`
- Implementation adds: `case AST.Branes branes -> { return new ConcatenationFiroe(branes); }`

### Class Structure

```java
public class ConcatenationFiroe extends FiroeWithBraneMind {
    private final List<AST.Characterizable> sourceBranes;  // Original brane ASTs
    private final List<FIR> pendingFirs;                   // FIRs waiting to reach constanic
    private boolean joinComplete = false;
    private boolean searchLocked = true;                   // Prevent searches during Stage A
}
```

### LhsSearchable Concept

**LhsSearchable** indicates when a brane is ready for left-hand-side identifier searches (variable resolution). For this implementation, `isLhsSearchable()` is equivalent to `isPrimed()` (at PRIMED state or beyond). Later iterations may improve upon this with more nuanced search readiness criteria.

- **Normal BraneFiroe**: Becomes LhsSearchable when it reaches PRIMED (braneMemory is populated, braneMind is ready)
- **ConcatenationFiroe**: Becomes LhsSearchable when it reaches PRIMED (all component branes cloned and stitched together)

Future work will add **RhsSearchable** for value-based searches (e.g., searching for branes by their evaluated values).

### Stage A: UNINITIALIZED → INITIALIZED → CHECKED

**Purpose:** Wait for all component branes/identifiers to reach constanic before joining.

**LhsSearchable Status:** ❌ NOT searchable during this stage

1. **UNINITIALIZED → INITIALIZED**
   - Create FIRs from `sourceBranes` ASTs
   - Store in `pendingFirs` (NOT in braneMemory yet)
   - Brane is NOT LhsSearchable yet

2. **INITIALIZED → CHECKED**
   - Step all `pendingFirs` until each `isConstanic()`
   - **CRITICAL:** LHS searches into this ConcatenationFiroe are blocked during this stage
   - This prevents premature identifier resolution before the join is complete
   - Still NOT LhsSearchable

### Stage B: CHECKED → PRIMED

**Purpose:** Clone and re-parent all component branes into unified memory.

**LhsSearchable Status:** ✅ Becomes searchable at end of this stage

1. For each FIR in `pendingFirs`:
   - If it's an IdentifierFiroe, unwrap to get the referenced brane
   - Call `cloneConstanic(this, Optional.of(Nyes.INITIALIZED))` to create fresh copy
   - The clone's braneMemory parent is set to this ConcatenationFiroe's braneMemory
   - Add clone to `braneMemory` via `storeFirs()`

2. Clear `pendingFirs`
3. Set `joinComplete = true`
4. Call `prime()` to enqueue non-constant items into braneMind
5. **Brane is now LhsSearchable** (isPrimed() returns true)

### Stage C: PRIMED → EVALUATING → CONSTANT

**Purpose:** Normal evaluation with shared memory context.

**LhsSearchable Status:** ✅ Remains searchable through evaluation

- Standard `FiroeWithBraneMind.step()` behavior
- All cloned branes share the same parent memory (this ConcatenationFiroe)
- Later branes can see assignments from earlier branes in the sequence
- Evaluation completes when all items reach constant/constanic
- LhsSearchable throughout (isPrimed() remains true)

### LhsSearchable Implementation

```java
public boolean isLhsSearchable() {
    // For this implementation, LhsSearchable is equivalent to isPrimed
    // Later iterations may refine this condition
    return isPrimed();
}

@Override
public Optional<Pair<Integer, FIR>> search(Query query, int fromLine) {
    if (!isLhsSearchable()) {
        // Searches not allowed until PRIMED
        return Optional.empty();
    }
    return super.search(query, fromLine);
}
```

This protects against searches during Stage A when the brane structure is still being assembled.

## Key Implementation Notes

1. **At joining time, referenced branes are NOT all constanic yet** - This is why Stage A exists
2. **Must wait for all to reach constanic before cloning** - Ensures stable state for cloneConstanic
3. **Search locking during Stage A prevents premature resolution** - Critical for correct semantics
4. **After Stage B, concatenated branes share same parent** - Enables cross-brane identifier resolution
5. **This is why parent access restriction is needed** - CMFir and ConcatenationFiroe both change parent chains

## Prerequisites

The following cleanup work (from the plan) enables safe ConcatenationFiroe implementation:

| Cleanup Item | Why Needed |
|--------------|------------|
| at/is state conventions | Clear semantics for "wait until isConstanic" |
| Constraint C5 (empty braneMind at constanic) | Critical for cloneConstanic in Stage B |
| FoolishIndex documentation | Debugging parent chain changes |
| AlarmSystem with FoolishIndex | Error reporting during Stage A/B |
| braneMemory/braneMind accessors | Control parent mutation during Stage B |

## Test Cases

### Test 1: Anonymous concatenation
```foo
{ result = {a=1}{b=2}{c=a+b}$.c }  !! result==3
```

### Test 2: Named concatenation with outer scope
```foo
{ x=10; A={a=x}; B={b=a+1}; result = (A B)$.b }  !! result==11
```

### Test 3: Shadowing in concatenation
```foo
{ a=100; result = {a=1}{b=a}$.b }  !! result==1 (not 100)
```

### Test 4: Search across concatenated branes
```foo
{ OB = {x=1}{y=2}{z=x+y}; result = OB$.z }  !! result==3
```

## ExecutionFir Design

### Problem Summary

**Issue 1: Rendering**
Concatenation output should show flat brane contents when fully evaluated, or `{...}{...}` format when not. Currently showing nested structures incorrectly.

**Issue 2: Stepping**
`pendingFirs` doesn't use proper breadth-first stepping. Need coordinated stepping of multiple FIRs to specific milestones.

### Design: ExecutionFir

A new reusable `FiroeWithBraneMind` that coordinates stepping multiple FIRs to target states.

#### API Concept

```java
ExecutionFir.of(fir1, fir2, fir3)
    .setParent(false)           // don't re-parent these FIRs to ExecutionFir
    .stepUntil(Nyes.PRIMED)     // target state for all FIRs
    .onComplete(firs -> {...})  // callback when all reach target
    .onStuck(firs -> {...})     // callback if any stuck at CONSTANIC < target
```

#### Use Cases

1. **Concatenation Stage A**: Step identifiers/searches to PRIMED without re-parenting
2. **Concatenation Stage C**: Step joined branes after re-parenting
3. **Future patterns**: Any coordinated multi-FIR stepping

#### ExecutionFir Behavior

- Maintains list of FIRs to step
- Each `step()` call steps one FIR from its internal queue (breadth-first)
- Removes FIR from queue when it reaches target state (or beyond)
- Tracks completion: all reached target vs some stuck at CONSTANIC
- Reports final status for caller to decide next action

#### State Transitions

```
ExecutionFir states:
- EVALUATING: Still stepping FIRs toward target
- CONSTANT: All FIRs reached target state → success
- CONSTANIC: Some FIRs stuck at CONSTANIC before target → caller decides
```

### Rendering Decision

Rendering depends on Nyes state:
- If concatenation is fully evaluated (CONSTANT), render as flat brane contents from braneMemory
- If not fully evaluated, render as `{...} {...}` with single-space separators to show proximity

The single-space separator is critical - it visually reinforces that elements are associated in a concatenation, showing their proximity in the source code.

### Re-parenting Control

The `setParent(false)` option is important:
- During Stage A, we step identifiers/searches to PRIMED but don't want to re-parent them
- They retain their original parent relationships for proper identifier resolution
- After the join (Stage B), the cloned branes ARE re-parented to the concatenation

## Future Considerations

- **P-branes with concatenation**: How does `[+a]{b=a}` interact with concatenation?
- **Detachment and concatenation**: Can you detach identifiers from a concatenated brane?
- **Nested concatenation**: What happens with `{A B} {C D}`?

---

## Last Updated

**Date**: 2026-02-01
**Updated By**: Claude Code v1.0.0 / claude-opus-4-5-20251101
**Changes**: Added ExecutionFir Design section documenting the new coordinated FIR stepping utility. Includes problem summary (rendering and stepping issues), API concept with builder pattern, use cases, behavior description, state transitions, rendering decisions (flat brane vs `{...}{...}` format), and re-parenting control explanation.
