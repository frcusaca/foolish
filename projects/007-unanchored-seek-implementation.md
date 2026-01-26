# Unanchored Seek Implementation Summary

## Overview

Implemented unanchored backward seek in Foolish, allowing expressions like `#-1` and `#-2` to reference previous statements in the current brane without an anchor.

## Syntax

- **Unanchored seek**: `#-N` where N is a positive integer
- **Only negative offsets allowed**: `#-1`, `#-2`, `#-3`, etc.
- **Positive offsets not permitted**: `#0`, `#1`, etc. are reserved for anchored seeks

## Examples

```foolish
{
    a = 1;
    b = 2;
    c = #-1 + #-2  !! c = 2 + 1 = 3
}

{
    val1 = 3;
    val2 = 4;
    val3 = 5;
    sum = #-1 + #-2 + #-3  !! sum = 5 + 4 + 3 = 12
}
```

## Implementation

### Grammar Changes

**File**: `foolish-parser-java/src/main/antlr4/Foolish.g4`

Added unanchored seek to the `primary` rule:

```antlr
primary
    : characterizable
    | LPAREN expr RPAREN
    | UNKNOWN
    | HASH MINUS INTEGER  // Unanchored backward seek: #-1, #-2, etc.
    ;
```

### AST Changes

**File**: `foolish-parser-java/src/main/java/org/foolish/ast/AST.java`

1. Added `UnanchoredSeekExpr` to the sealed `Expr` interface
2. Created new record:

```java
record UnanchoredSeekExpr(int offset) implements Expr {
    public String toString() {
        return "#" + offset;
    }
}
```

**File**: `foolish-parser-java/src/main/java/org/foolish/ast/ASTBuilder.java`

Updated `visitPrimary()` to create `UnanchoredSeekExpr` when parsing `#-N`:

```java
// Handle unanchored backward seek: #-N
if (ctx.HASH() != null && ctx.MINUS() != null && ctx.INTEGER() != null) {
    int offset = -Integer.parseInt(ctx.INTEGER().getText());
    return new AST.UnanchoredSeekExpr(offset);
}
```

### FIR Implementation

**File**: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/UnanchoredSeekFiroe.java`

Created new FIR class that:
1. Extends `FiroeWithBraneMind` to access brane memory hierarchy
2. Traverses up the parent brane memory chain to find the actual brane
3. Calculates the current statement position
4. Performs the seek relative to that position

**Key Challenge**: Memory Hierarchy Traversal

The FIR hierarchy for an expression like `c = #-1 + #-2` is:

```
BraneFiroe (braneMemory = [a, b, c])
  └─ AssignmentFiroe (c) (braneMemory = [], parent = BraneFiroe's memory)
      └─ BinaryFiroe (+) (braneMemory = [], parent = AssignmentFiroe's memory)
          ├─ UnanchoredSeekFiroe (#-1) (braneMemory = [], parent = BinaryFiroe's memory)
          └─ UnanchoredSeekFiroe (#-2) (braneMemory = [], parent = BinaryFiroe's memory)
```

**Solution**: Traverse all the way up the parent chain to find the top-level `BraneFiroe`, then traverse back down to find the current position using `myPos` at each level.

**File**: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/BraneMemory.java`

Added getter methods:
- `getMyPos()`: Returns the position of this memory in its parent (-1 if not set)
- `getParent()`: Returns the parent `BraneMemory`

**File**: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/FIR.java`

Updated `createFiroeFromExpr()` to handle `UnanchoredSeekExpr`:

```java
case AST.UnanchoredSeekExpr unanchoredSeekExpr -> {
    return new UnanchoredSeekFiroe(unanchoredSeekExpr);
}
```

## Semantics

### Out of Bounds Behavior

When an unanchored seek offset goes beyond the brane start, it becomes **CONSTANIC** (not NK/???):

```foolish
{
    first = 100;
    oob = #-5      !! Out of bounds - becomes CONSTANIC (⎵⎵)
}
```

**Rationale**: The seek is "CONSTANt IN Context" - it awaits potential brane concatenation that could provide the missing statements. When branes are concatenated in the future, a previously out-of-bounds seek may become resolved.

### Brane Concatenation (Future Feature)

**IMPORTANT**: Unanchored seeks are designed to work with brane concatenation:

```foolish
{a=1}{b=#-1}  !! Future: #-1 in second brane should find 'a' from first brane
```

**TODO**: When implementing brane concatenation:
1. Unanchored seeks must re-evaluate after concatenation
2. CONSTANIC seeks should transition to CHECKED when they find their target
3. The search must use the concatenated brane's full memory, not just the original brane

## Testing

**File**: `test-resources/org/foolish/fvm/inputs/unanchoredSeekBasic.foo`

Test cases:
1. Basic unanchored seek: `c = #-1 + #-2` → 3
2. Nested branes with unanchored seek
3. Out of bounds → CONSTANIC
4. Complex expression with multiple seeks
5. User's example: nested assignment with seek

**Approved File**: `foolish-core-java/src/test/resources/org/foolish/fvm/ubc/unanchoredSeekBasic.approved.foo`

Test results:
- All tests passing
- Correct values computed
- Out-of-bounds correctly returns CONSTANIC

## Code Locations

**Grammar/Parser**:
- `foolish-parser-java/src/main/antlr4/Foolish.g4:103` - Grammar rule
- `foolish-parser-java/src/main/java/org/foolish/ast/AST.java:213-217` - AST record
- `foolish-parser-java/src/main/java/org/foolish/ast/ASTBuilder.java:242-250` - Parser logic

**FIR Implementation**:
- `foolish-core-java/src/main/java/org/foolish/fvm/ubc/UnanchoredSeekFiroe.java` - Main implementation
- `foolish-core-java/src/main/java/org/foolish/fvm/ubc/FIR.java:170-172` - Factory method
- `foolish-core-java/src/main/java/org/foolish/fvm/ubc/BraneMemory.java:37-43` - Getters

**Tests**:
- `test-resources/org/foolish/fvm/inputs/unanchoredSeekBasic.foo` - Input
- `foolish-core-java/src/test/resources/org/foolish/fvm/ubc/unanchoredSeekBasic.approved.foo` - Approved output

## Future Work

### Brane Concatenation Support

When brane concatenation is implemented:
- Update `UnanchoredSeekFiroe` to handle concatenated branes
- Ensure CONSTANIC→CHECKED transitions work correctly
- Add tests for concatenated brane scenarios

### Forward Unanchored Seek

Currently only backward seek (`#-1`, `#-2`) is supported. Future work could add forward unanchored seek, though this raises questions about what "current position" means for forward searches.

### Positive Offset Validation

The grammar currently allows `#-N`, but positive offsets like `#1`, `#2` might be confused with anchored seeks like `brane#1`. Consider adding grammar-level validation to reject `#N` (positive) at the unanchored position.

## Related Features

- **Anchored seek**: `brane#-1`, `brane#0` (already implemented)
- **Identifier resolution**: Uses similar parent traversal for scope lookup
- **Search operators**: `?`, `??`, `/`, `^`, `$` (related search features)

---

## Last Updated

**Date**: 2026-01-26
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-5-20250929
**Changes**: Implemented unanchored backward seek with `#-N` syntax. All tests passing (121 Java tests).
