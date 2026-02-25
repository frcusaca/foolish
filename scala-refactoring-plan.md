# Scala Refactoring Plan

**Date**: 2026-02-24
**Target**: Match Java implementation fixes for concatenation and state management

---

## Implementation Status

| # | Change | File | Status |
|---|--------|------|--------|
| 1 | Fix `UnanchoredSeekFiroe.isConstanic()` | UnanchoredSeekFiroe.scala | ✅ Done |
| 2 | Simplify `FIR.unwrapConstanicable()` | FIR.scala | ⏳ Pending |
| 3 | Fix `AssignmentFiroe.cloneConstanic()` + private constructor | AssignmentFiroe.scala | ⏳ Pending |
| 4 | Fix `AssignmentFiroe.step()` result detection | AssignmentFiroe.scala | ⏳ Pending |
| 5 | Fix `IdentifierFiroe.step()` CHECKED state handling | IdentifierFiroe.scala | ⏳ Pending |
| 6 | Add `BraneFiroe.stream()` method | BraneFiroe.scala | ⏳ Pending |
| 7 | Fix `BraneFiroe.cloneConstanic()` CONSTANT vs CONSTANIC | BraneFiroe.scala | ⏳ Pending |
| 8 | Fix `FiroeWithBraneMind.step()` EVALUATING to CONSTANT | FiroeWithBraneMind.scala | ⏳ Pending |
| 9 | Implement `ConcatenationFiroe.scala` | NEW FILE | ⏳ Pending |
| 10 | Update `FIR.createFiroeFromExpr()` for Concatenation | FIR.scala | ⏳ Pending |
| 11 | Fix `BraneMemory.get()` parent search position | BraneMemory.scala | ⏳ Pending |

---

## Change 1: Fix `UnanchoredSeekFiroe.isConstanic()` ✅

**File**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/UnanchoredSeekFiroe.scala`

**Problem**: Method checks `value == null` before checking state, causing incorrect CONSTANIC detection.

**Java Fix**:
```java
public boolean isConstanic() {
    if (getNyes() != Nyes.CONSTANIC && getNyes() != Nyes.CONSTANT) {
        return false;
    }
    if (value == null) {
        return true;  // Resolved to nothing (out of bounds)
    }
    return value.isConstanic();
}
```

**Scala Fix Applied**:
```scala
override def isConstanic(): Boolean =
  if getNyes != Nyes.CONSTANIC && getNyes != Nyes.CONSTANT then
    false
  else if value == null then
    true  // Resolved to nothing (out of bounds)
  else
    value.isConstanic()
```

**Test Impact**: All search-related tests

---

## Change 2: Simplify `FIR.unwrapConstanicable()`

**File**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/FIR.scala`

**Java Fix**:
```java
public static FIR unwrapConstanicable(FIR fir) {
    FIR current = fir;
    while (current != null) {
        if (current instanceof Constanicable constanicable) {
            FIR result = constanicable.getResult();
            if (result == null || result == current) return current;
            current = result;
        } else {
            return current;
        }
    }
    return current;
}
```

**Scala Fix** (pending):
```scala
def unwrapConstanicable(fir: FIR): FIR =
  var current = fir
  while current != null do
    if current.isInstanceOf[Constanicable] then
      val result = current.asInstanceOf[Constanicable].getResult
      if result == null || result == current then return current
      current = result
    else
      return current
  current
```

**Test Impact**: Concatenation tests

---

## Change 3: Fix `AssignmentFiroe.cloneConstanic()` + Private Constructor

**File**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/AssignmentFiroe.scala`

**Java Fix - New Private Constructor**:
```java
private AssignmentFiroe(AST.Assignment assignment, FIR newParent, CharacterizedIdentifier lhs) {
    super(assignment, null);
    setParentFir(newParent);
    this.lhs = lhs;
    this.result = null;
    this.ordinated = false;
    this.indexLookup = new java.util.IdentityHashMap<>();
    storeExprs(assignment.expr());
}
```

**Java Fix - cloneConstanic**:
```java
protected FIR cloneConstanic(FIR newParent, Optional<Nyes> targetNyes) {
    if (!isConstanic()) {
        throw new IllegalStateException(...);
    }
    if (isConstant()) {
        return this;  // Share CONSTANT assignments completely
    }
    AssignmentFiroe copy = new AssignmentFiroe((AST.Assignment) this.ast, newParent, this.lhs);
    copy.result = null;
    copy.setInitialized();
    copy.nyes = this.nyes;
    if (targetNyes.isPresent()) {
        copy.nyes = targetNyes.get();
    }
    return copy;
}
```

**Scala Fix** (pending):
```scala
class AssignmentFiroe(assignment: AST.Assignment) extends FiroeWithBraneMind(assignment):
  val lhs = CharacterizedIdentifier(assignment.identifier())
  private var result: Option[FIR] = None

  // New private constructor for cloneConstanic
  private def this(assignment: AST.Assignment, newParent: FIR, lhs: CharacterizedIdentifier) =
    super(assignment, None)
    setParentFir(newParent)
    this.lhs = lhs
    this.result = null
    this.ordinated = false
    this.indexLookup = new java.util.IdentityHashMap[FIR, Int]()
    enqueueExprs(assignment.expr())

  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    if !isConstanic() then
      throw IllegalStateException(...)
    if isConstant() then
      return this
    val copy = new AssignmentFiroe(ast.asInstanceOf[AST.Assignment], newParent, lhs)
    copy.result = null
    copy.setInitialized()
    copy.nyes = this.nyes
    targetNyes.foreach(copy.nyes = _)
    copy
```

**Test Impact**: CMFir tests, concatenation tests

---

## Change 4: Fix `AssignmentFiroe.step()` Result Detection

**File**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/AssignmentFiroe.scala`

**Java Fix**:
```java
// Check if we can get the final result
if (!super.isNye && !braneMemory.isEmpty) {
    val res = braneMemory.get(0);
    result = Some(res);
    if (res.isConstanic()) {  // Changed from res.atConstanic()
        setNyes(Nyes.CONSTANIC);
    }
}
```

**Scala Fix** (pending):
```scala
// Check if we can get the final result
if !super.isNye && !braneMemory.isEmpty then
  val res = braneMemory.get(0)
  result = Some(res)
  if res.isConstanic() then  // Changed from res.atConstanic()
    setNyes(Nyes.CONSTANIC)
```

**Test Impact**: Assignment tests

---

## Change 5: Fix `IdentifierFiroe.step()` CHECKED State Handling

**File**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/IdentifierFiroe.scala`

**Java Fix**:
```java
case CHECKED -> {
    if (value.atConstanic()) {
        Nyes targetNyes = value.getNyes() == Nyes.CONSTANIC || value.getNyes() == Nyes.CONSTANT
            ? value.getNyes() : Nyes.INITIALIZED;
        value = value.cloneConstanic(this, Optional.of(targetNyes));
        storeFirs(value);
        if (targetNyes != Nyes.CONSTANIC && targetNyes != Nyes.CONSTANT) {
            braneEnqueue(value);
        }
        setNyes(Nyes.PRIMED);
    } else if (value.atConstant()) {
        storeFirs(value);
        setNyes(Nyes.CONSTANT);
    } else {
        storeFirs(value);
        braneEnqueue(value);
        setNyes(Nyes.PRIMED);
    }
    return 1;
}
```

**Simplified Java Fix** (current):
```java
case CHECKED -> {
    if (value.isConstanic()) {
        storeFirs(value);
        setNyes(value.atConstanic() ? Nyes.CONSTANIC : Nyes.CONSTANT);
    } else {
        storeFirs(value);
        braneEnqueue(value);
        setNyes(Nyes.PRIMED);
    }
    return 1;
}
```

**Scala Fix** (pending):
```scala
case Nyes.CHECKED =>
  if value.isConstanic() then
    storeFirs(value)
    setNyes(value.atConstanic() match
      case true => Nyes.CONSTANIC
      case false => Nyes.CONSTANT
    )
  else
    storeFirs(value)
    braneEnqueue(value)
    setNyes(Nyes.PRIMED)
  return 1
```

**Test Impact**: Identifier resolution tests

---

## Change 6: Add `BraneFiroe.stream()` Method

**File**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/BraneFiroe.scala`

**Java Fix**:
```java
public Stream<FIR> stream() {
    return braneMemory.stream();
}
```

**Scala Fix** (pending):
```scala
def stream: Iterator[FIR] = braneMemory.stream
```

**Test Impact**: Concatenation tests

---

## Change 7: Fix `BraneFiroe.cloneConstanic()` CONSTANT vs CONSTANIC

**File**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/BraneFiroe.scala`

**Java Fix**:
```java
protected FIR cloneConstanic(FIR newParent, Optional<Nyes> targetNyes) {
    if (!isConstanic()) {
        throw new IllegalStateException(...);
    }
    if (isConstant()) {
        return this;  // Share CONSTANT branes completely
    }
    // CONSTANIC: create fresh copy from AST
    BraneFiroe copy = new BraneFiroe((AST.Brane) this.ast, newParent);
    // ... copy memory items ...
    if (targetNyes.isPresent()) {
        copy.nyes = targetNyes.get();
    } else {
        copy.nyes = this.nyes;
    }
    return copy;
}
```

**Scala Fix** (pending):
```scala
override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
  if !isConstanic() then
    throw IllegalStateException(...)
  if isConstant() then
    return this
  val copy = new BraneFiroe(ast.asInstanceOf[AST.Brane])
  // Copy memory items - clone each statement if CONSTANIC
  this.braneMemory.stream.forEach { stmt =>
    if stmt.isConstanic() then
      val cloned = stmt.cloneConstanic(copy, stmt.isConstant() ? None : Some(Nyes.INITIALIZED))
      copy.braneMemory.put(cloned)
    else
      copy.braneMemory.put(stmt)  // CONSTANT - share
  }
  targetNyes.foreach(copy.nyes = _)
  copy
```

**Test Impact**: Concatenation tests

---

## Change 8: Fix `FiroeWithBraneMind.step()` EVALUATING to CONSTANT

**File**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/FiroeWithBraneMind.scala`

**Java Fix**:
```java
case EVALUATING ->
    // Step everything including sub-branes
    if (braneMind.isEmpty()) {
        // All expressions evaluated, transition to CONSTANT
        setNyes(Nyes.CONSTANT);
        return;
    }
    // ... step logic ...
```

**Scala Fix** (already present, verify):
```scala
case Nyes.EVALUATING =>
  if braneMind.isEmpty then
    setNyes(Nyes.CONSTANT)
    return
  // ... step logic ...
```

**Test Impact**: All evaluation tests

---

## Change 9: Implement `ConcatenationFiroe.scala` (NEW FILE)

**File**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/ConcatenationFiroe.scala` (NEW)

**Java Implementation** (278 lines):
- Stage A: Step source FIRs to CONSTANIC using ExecutionFir
- Stage B: Clone and re-parent with `performJoin()`
- Stage C: Normal evaluation
- `performJoin()`: Iterate over brane statements, clone each, store in memory

**Scala Implementation** (pending): Full port of Java implementation

**Test Impact**: concatenationBasics, concatenationResolution, concatenationSearch, concatenationResolutionAdv

---

## Change 10: Update `FIR.createFiroeFromExpr()` for Concatenation

**File**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/FIR.scala`

**Java Fix**:
```java
case concatenation: AST.Concatenation => ConcatenationFiroe(concatenation)
```

**Scala Fix** (pending):
```scala
case concatenation: AST.Concatenation => ConcatenationFiroe(concatenation)
```

**Test Impact**: All concatenation tests

---

## Change 11: Fix `BraneMemory.get()` Parent Search Position

**File**: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/BraneMemory.scala`

**Current Scala Code** (verified - already correct):
```scala
val parentPos = if owningBrane != null then
  owningBrane.getMyBraneIndex
else
  myPos.getOrElse(parent.size - 1)
```

**Test Impact**: All tests with parent search

---

## Summary

| # | Status | Estimate |
|---|--------|----------|
| 1 | ✅ Done | - |
| 2-11 | ⏳ Pending | ~4-6 hours |

**Total New Code**: ~300 lines
**Total Modified**: ~50 lines

---

## Notes

- Changes 1-8 are small, localized fixes (10-30 lines each)
- Change 9 (ConcatenationFiroe) is the largest - a full new implementation
- All changes should be testable individually via `mvn test -pl foolish-core-scala`
