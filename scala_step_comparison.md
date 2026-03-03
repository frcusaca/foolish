# Step() Method Comparison: Java vs Scala UBC

## Summary
This document catalogs the differences between Java and Scala step() implementations that affect step counting in the UBC approval tests.

## Key Differences Affecting Step Count

### 1. BraneFiroe initialization handling
**Java:**
```java
public int step() {
    if (!isInitialized()) {
        initialize();
        return 1;  // Returns immediately, nyes stays UNINITIALIZED
    }
    return super.step();  // Next call handles UNINITIALIZED -> INITIALIZED transition
}
```

**Scala:**
```scala
override def step(): Int =
  if !isInitialized then
    initialize()
    setNyes(Nyes.INITIALIZED)  // Sets nyes explicitly
    // Falls through to super.step() - processes INITIALIZED state in SAME call
  super.step()
```

**Impact:** Java takes 2 steps to initialize (step 1: init + return 1, step 2: super processes UNINITIALIZED). Scala may take fewer steps by combining initialization and first state processing.

---

### 2. FiroeWithoutBraneMind state recovery
**Java:**
```java
public int step() {
    if (getNyes() != Nyes.CONSTANT) {
        setNyes(Nyes.CONSTANT);
        return 1;  // Counted as work to fix state
    }
    return 0;
}
```

**Scala:**
```scala
def step(): Int = 0  // Always returns 0, no state recovery
```

**Impact:** If a ValueFiroe is ever reset to non-CONSTANT (via cloneConstanic), Java returns 1 to fix it; Scala returns 0 and leaves it broken.

---

### 3. UnaryFiroe/BinaryFiroe initialization
**Java:**
```java
case UNINITIALIZED, INITIALIZED, CHECKED, PRIMED -> {
    return super.step();  // Returns super's value
}
```

**Scala:**
```scala
case Nyes.UNINITIALIZED | ... =>
    super.step()
    1  // Always returns 1, ignoring super's return value!
```

**Impact:** Scala always counts 1 step even when super.step() returns 0 (e.g., when already evaluated). This OVERCOUNTS steps.

---

### 4. IfFiroe state handling
**Java:** Does NOT handle UNINITIALIZED/INITIALIZED/CHECKED/PRIMED in override - relies on custom logic.

**Scala:** Properly delegates to super for these states.

**Impact:** Different execution paths. Java's custom logic may take more/fewer steps depending on state.

---

### 5. IfFiroe EVALUATING loop
**Java:** Processes ONE conditional per step() call, always returns 1.

**Scala:** While loop can process MULTIPLE conditionals per step(), returns actual work count.

**Impact:** Scala may complete evaluation in fewer steps by batching work.

---

### 6. FiroeWithBraneMind EVALUATING completion detection
**Java:** Only checks `isBraneEmpty()` at START of EVALUATING case. If braneMind becomes empty after stepping, NEXT call detects it.

**Scala:** After stepping, checks `if braneMind.isEmpty` and immediately transitions.

**Impact:** Java counts one extra step to detect completion; Scala transitions immediately without counting that step.

---

## Differences Not Directly Affecting Step Count

### 7. UnaryFiroe initialize()
**Java:** `storeExprs()` - stores to braneMemory only.

**Scala:** `enqueueExprs()` - enqueues directly to braneMind (deprecated method).

**Impact:** Different timing of evaluation, may affect step order but not necessarily count.

---

### 8. IdentifierFiroe CHECKED state
**Java:** Clones constanic values with `cloneConstanic()`.

**Scala:** Does NOT clone, keeps reference to original.

**Impact:** May affect evaluation if cloning triggers extra steps.

---

### 9. IdentifierFiroe state coverage
**Java:** Explicitly handles all 7 states.

**Scala:** Only handles INITIALIZED, CHECKED; delegates rest to super.

**Impact:** Different control flow, may affect step counts in edge cases.

---

## Priority Fixes to Match Java Step Counts

**High Priority:**
1. Fix UnaryFiroe/BinaryFiroe to return `super.step()` instead of always 1
2. Fix FiroeWithBraneMind EVALUATING to NOT immediately transition when empty
3. Fix BraneFiroe to match Java's initialization pattern
4. Fix FiroeWithoutBraneMind to match Java's state recovery

**Medium Priority:**
5. Fix IfFiroe state handling to match Java
6. Fix IdentifierFiroe CHECKED cloning behavior

**Low Priority:**
7. Fix UnaryFiroe initialize() to use storeExprs() instead of enqueueExprs()
8. Fix IdentifierFiroe state coverage to match Java
