# ProtoBrane Derived FIRs — Design Document

> AI agents: read [../DOC_AGENTS.md](../DOC_AGENTS.md) before editing this file.

This document is a design proposal for task D0 from
[human_todo_index.md](../todo/human_todo_index.md). It specifies the contract for each of the
four FIR roles that derive from ProtoBrane — how each role leverages the shared lifecycle while
contributing its own specialized behavior.

---

## Table of Contents

- [Background](#background)
- [Section 1: ProtoBrane — The Shared Contract](#section-1-protobrane--the-shared-contract)
- [Section 2: Role A — Normal Brane `{...}`](#section-2-role-a--normal-brane-)
- [Section 3: Role B — System Operator Brane `🧠+`, `🧠-`, etc.](#section-3-role-b--system-operator-brane--etc)
- [Section 4: Role C — ConcatenationBrane](#section-4-role-c--concatenationbrane)
- [Section 5: Role D — Detachment Brane `[...]`](#section-5-role-d--detachment-brane-)
- [Section 6: Lessons Learned from UBC1](#section-6-lessons-learned-from-ubc1)
- [Section 7: How Each Role Appears in UBC2 Terms](#section-7-how-each-role-appears-in-ubc2-terms)
- [Section 8: Walking Through Approval Tests](#section-8-walking-through-approval-tests)
- [Section 9: Design Proposal](#section-9-design-proposal)

---

## Background

Four kinds of FIR derive from ProtoBrane, ranging from the most structurally complete to the
most specialized:

- **Normal Brane `{...}`** — the full curly-brace brane. Defines a search scope, holds a
  statement array, and is itself a first-class value. This is the canonical ProtoBrane; all
  other roles are understood relative to it.
- **System Operator `🧠+`, `🧠-`, etc.** — proto-branes that compute scalar values.
  No search boundary, no local namespace. Produced by desugaring infix/prefix operators.
- **ConcatenationBrane** — a wrapping FIR that merges multiple branes into a single evaluation
  context. Has its own three-stage lifecycle (isolate → merge → evaluate).
- **Detachment `[...]`** — a search-filter wrapper that intercepts identifier resolution,
  creating free variables for late binding.

These four roles differ in traits such as
[`hasBoundary`](#section-2-role-a--normal-brane-) (whether the FIR defines a search scope),
[`isDetachment`](#section-5-role-d--detachment-brane-) (whether the FIR filters search messages),
and [`sfMode`](#sf-and-sff-markers) (how eagerly identifiers are resolved). This is intentional:
UBC2 avoids deep inheritance and repeated `step()` overrides by expressing behavioral differences
through a small number of trait flags on a single ProtoBrane class, rather than through 15+
specialized subclasses each reimplementing the same lifecycle with minor variations.

The [UBC2 design document](ubc2_design.md) establishes the general ProtoBrane contract — the
Nyes lifecycle (PREMBRYONIC → EMBRYONIC → BRANING → constanic), message-passing architecture,
and core invariants — but does not fully specify how each derived role exercises that contract.
This document fills that gap. For each role, we specify the delta: which lifecycle steps are
active or inactive, what `value()` returns, how search resolution behaves, how constanic cloning
works, and how the Nyes microstates play out differently depending on the role's traits. The
design of Nyes microstate transitions for different trait combinations is a central concern —
a system operator reaching CONSTANIC means something different from a normal brane reaching
CONSTANIC, and these differences must be explicit before implementation can proceed.

This matters because:

- **Implementation** needs to know which lifecycle steps to enable/disable for each role
- **Testing** needs to know what each role's correct output looks like
- **Concatenation** (D2) needs to know what it's merging and what invariants hold on the children
- **Detachment** (D4) needs to know how search filtering interacts with each role's behavior

### The Four Roles

| Role | Syntax | Key Traits |
|------|--------|-----------|
| **[A: Normal Brane](#section-2-role-a--normal-brane-)** | `{a=1; b=2;}` | [`hasBoundary=true`](#section-2-role-a--normal-brane-) |
| **[B: System Operator](#section-3-role-b--system-operator-brane--etc)** | `🧠+`, `🧠-`, `🧠*` | [`hasBoundary=false`](#section-3-role-b--system-operator-brane--etc) |
| **[C: ConcatenationBrane](#section-4-role-c--concatenationbrane)** | `{a=1}{b=2}` or `b1 b2` | [Three-stage lifecycle; search isolation](#section-4-role-c--concatenationbrane) |
| **[D: Detachment](#section-5-role-d--detachment-brane-)** | `[a,b]{...}` | [`isDetachment=true`](#section-5-role-d--detachment-brane-); search filtering |

---

## Section 1: ProtoBrane — The Shared Contract

Before specifying deltas, here is what ProtoBrane provides to all roles. This is the baseline.

### Lifecycle

All ProtoBranes traverse:

```
PREMBRYONIC → EMBRYONIC → BRANING → constanic terminal state
```

Where constanic terminal states are: CONSTANIC, WOCONSTANIC, CONSTANT, or INDEPENDENT.

Each stage does bounded work per step. No stage tries to be clever.

### PREMBRYONIC (Atomic Transition)

All actions occur as a single step:

1. **Count lines** — establish statement array size
2. **Establish statement array** — indexed from 0
3. **Build search cache** — array of fully-characterized identifier strings
4. **Instantiate RHS FIRs** — in PREMBRYONIC, not stepped
5. **Find searches in RHS** — stopping at brane boundaries; maintain writing order
6. **Append child branes** — to braneMind, in writing order
7. **Establish LUID** — locally unique identifier for message routing

→ Transition to EMBRYONIC.

### EMBRYONIC (Search Resolution)

- Resolve searches through local search, dereferencing, and message passing
- Dispatch FulfillSearch to parent for searches that can't be resolved locally
- Receive RespondToSearch from parent
- Search bandwidth limits work per step
- Wait-for queue for dependencies not yet constanic

→ Transition to BRANING when all SearchFirs have reached terminal states.

### BRANING (Child Execution)

- Step child branes from braneMind heap
- Continue communication (forward messages up/down)
- Monitor wait-for queue; trigger constanic cloning when dependencies settle

→ Transition to constanic terminal state when all work is complete.

### Core Invariants

| ID | Invariant | Description |
|----|-----------|-------------|
| C1 | Immutability after constanic | No fields may change once `isConstanic()` is true |
| C2 | CONSTANT tree invariant | CONSTANT FIR never has non-CONSTANT descendants |
| C3 | Constanic value stability | Value only changes via re-evaluation in new context |
| C4 | Parent chain integrity | parentFir not reassigned after setup (except cloning) |

### value() Default

Every ProtoBrane has a `value()` method. The default behavior depends on the role (see per-role
sections below).

### cloneConstanic() Default

When constanic-cloned:
- **CONSTANT** → returns `this` (immutable, safe to share)
- **CONSTANIC/WOCONSTANIC** → creates clone with new parent, resets to PREMBRYONIC for
  re-evaluation in new context

---

## Section 2: Role A — Normal Brane `{...}`

### What It Is

The curly-brace brane: the fundamental Foolish structure that holds and evaluates a sequence of
statements. This is the "full brane" — it has its own namespace, its own search scope, and its
own identity as a first-class value.

### Trait Configuration

```
hasBoundary  = true
isDetachment = false
sfMode       = NONE (default; may be overridden with SF/SFF markers)
```

### Delta from ProtoBrane

**Search boundary (the key distinction).** When a search starts inside a Normal Brane:
1. First search **local statements** (backward from cursor position, writing-order precedence)
2. If not found locally, **escalate to parent** via FulfillSearch message

This is the only role where local search happens before parent escalation. Proto-branes without
boundaries skip step 1 entirely.

**value() returns the brane itself.** A Normal Brane is a first-class value. When another
expression evaluates `b = {...}`, the RHS evaluates to the brane object, not to a scalar extracted
from it. To get a scalar out of a brane, you use search operators (`b?name`, `b^`, `b$`, `b#0`).

**Brane depth tracking.** Normal Branes maintain `EXPRMNT_brane_depth` — the nesting level from
root. Exceeding `EXPRMNT_MAX_BRANE_DEPTH` produces NK (depth alarm).

### Lifecycle Specifics

Standard full lifecycle. No steps are skipped or added:

```
PREMBRYONIC → EMBRYONIC → BRANING → constanic
```

### Example: Scope Resolution

```foolish
{
    a = 10;
    b = 20;
    outer'{
        c = 30;
        a + c;          !! a found in parent, c found locally → 40
        inner'{
            a = 40;     !! shadows outer a
            a + b + c;  !! a=40 (local), b=20 (grandparent), c=30 (parent) → 90
        };
        a + b;          !! a=10 (parent, not shadowed here), b=20 → 30
    };
}
```

Approved output:
```
{
＿a = 10;
＿b = 20;
＿outer'{
＿＿c = 30;
＿＿40;
＿＿inner'{
＿＿＿a = 40;
＿＿＿90;
＿＿};
＿＿30;
＿};
}
```

**What this demonstrates:**
- `outer'` defines a search boundary — `c` is found locally before searching parent
- `inner'` shadows `a` — the local `a=40` is found before the grandparent's `a=10`
- `b` is found in grandparent because neither `outer'` nor `inner'` define it locally
- The search escalation chain: local → parent → grandparent → ... → root

### Example: Empty and Nested Branes

```foolish
{}                      !! Empty brane — still a valid first-class value
{5;}                    !! Single-statement brane — value is the brane, not 5
{{{1};2;};3;}           !! Three levels deep — each level has its own scope
```

### Related Approval Tests

Each file is in `test-resources/org/foolish/fvm/inputs/`. Tests marked *(+B)* also exercise
Role B (system operators) as secondary.

- [`emptyBraneIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/emptyBraneIsApproved.foo) — Minimal brane `{}`; verifies brane-as-value for the degenerate case
- [`nestedBranesIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/nestedBranesIsApproved.foo) — Multi-level nesting `{5; {10; 15;}; 20;}`, each level maintains own scope
- [`deeplyNestedBranesIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/deeplyNestedBranesIsApproved.foo) — Three-level nesting `{{{1};2;};3;}`
- [`veryDeepNestingIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/veryDeepNestingIsApproved.foo) — Four-level nesting; exercises depth tracking
- [`fourLevelNestedBranesWithNamesIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/fourLevelNestedBranesWithNamesIsApproved.foo) — Named branes (`second'`, `third'`, `inner'`) with characterized identifiers
- [`complexIdentifierScopeIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/complexIdentifierScopeIsApproved.foo) — *(+B)* Multi-level scope resolution: local → parent → grandparent search chain
- [`identifierShadowingIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/identifierShadowingIsApproved.foo) — *(+B)* Sequential shadowing: `x=10; x; x=20; x;` — writing-order precedence
- [`nestedScopeIdentifierIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/nestedScopeIdentifierIsApproved.foo) — *(+B)* Identifier access across nested brane boundaries
- [`nestedScopeShadowingIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/nestedScopeShadowingIsApproved.foo) — *(+B)* Shadowing behavior in nested named branes
- [`nestedBranesWithArithmeticIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/nestedBranesWithArithmeticIsApproved.foo) — *(+B)* Arithmetic in nested branes; each brane evaluates independently

### Contract Summary: Normal Brane

| Aspect | Specification |
|--------|--------------|
| `hasBoundary` | `true` — searches check local statements first |
| `value()` | Returns the brane itself (first-class value) |
| Search escalation | Local → parent via FulfillSearch |
| Depth tracking | Yes; NK on depth exceeded |
| Statement array | Yes; holds and indexes named+unnamed statements |
| Lifecycle | Full standard: PREMBRYONIC → EMBRYONIC → BRANING → constanic |
| Cloning | Standard constanic cloning; CONSTANT shared, CONSTANIC cloned |

---

## Section 3: Role B — System Operator Brane `🧠+`, `🧠-`, etc.

### What It Is

A proto-brane produced by desugaring infix/prefix operators. `1 + 2` desugars to the
concatenation `{🧠1, 🧠2, 🧠+}` where `🧠+` is the system operator. The operator waits for its
operands to become CONSTANT, then computes the result.

### Trait Configuration

```
hasBoundary  = false
isDetachment = false
sfMode       = NONE
```

### Delta from ProtoBrane

**No search boundary.** System operators do not define a local namespace. They do not hold named
statements. Searches initiated by a system operator start directly in the **parent brane** where
the operator resides.

**value() returns a scalar.** The result of `🧠+` on operands `1` and `2` is the integer `3`, not
a brane. This is the fundamental distinction from Normal Branes: proto-branes compute scalars;
branes compute branes.

**Operand access via parent.** System operators access their operands by indexing into the parent
brane's statement array. Binary operators take the two preceding values; unary operators take one:

```java
// Binary operator (🧠+, 🧠-, 🧠*, 🧠/)
a, b = parent.getValuesBeforeMe(me, 2);
myValue = a + b;

// Unary operator (🧠−, 🧠!)
a = parent.getValueBeforeMe(me);
myValue = -a;
```

**Interior is not inspectable.** The operands (proto-branes for `1` and `2`) are implementation
details of the concatenation that produced the system operator's context. They are not accessible
from outside; only the final scalar result is visible.

**Wait-for on operands.** If an operand is not yet CONSTANT (e.g., it's CONSTANIC because an
identifier hasn't resolved yet), the system operator enters its own wait-for state. It does not
compute until all operands are available.

### Lifecycle Specifics

The lifecycle is the standard proto-brane lifecycle, but the BRANING stage has a specific action:

```
PREMBRYONIC → EMBRYONIC → BRANING → constanic
                              │
                              └─ Retrieve operand values from parent
                                 Compute result
                                 Set myValue
                                 Transition to CONSTANT (or CONSTANIC if operands unavailable)
```

### Terminal States

| Condition | Terminal State |
|-----------|---------------|
| All operands CONSTANT, computation succeeds | **CONSTANT** |
| One or more operands CONSTANIC | **WOCONSTANIC** (waiting on dependencies) |
| Arithmetic error (division by zero) | **CONSTANT** with NK value (`🧠???`) |
| Type mismatch | **CONSTANT** with NK value |

### Example: Simple Arithmetic

```foolish
{3 + 4;}
```

Desugars to `{🧠3, 🧠4, 🧠+}`. After evaluation:
```
{
＿7;
}
```

Steps: 11. The `🧠+` finds its two preceding siblings (`3` and `4`), both already INDEPENDENT
(literals). Computes `3 + 4 = 7`. Returns `value() = 7`.

### Example: Chained Operators

```foolish
{1 + 2 + 3 + 4;}
```

Parsed as left-associative: `((1 + 2) + 3) + 4`. The innermost `🧠+` resolves first, producing
`3`. Then `3 + 3 = 6`. Then `6 + 4 = 10`:

```
{
＿10;
}
```

### Example: Operator with Unresolved Operand

```foolish
{x = non_existent; y = x + 1;}
```

Here `x` resolves to CONSTANIC (identifier `non_existent` not found). The `🧠+` for `x + 1`
cannot compute because one operand is CONSTANIC. The system operator transitions to WOCONSTANIC.

### Related Approval Tests

Each file is in `test-resources/org/foolish/fvm/inputs/`. Grouped by sub-category within Role B.

**Arithmetic operators** (binary `🧠+`, `🧠-`, `🧠*`, `🧠/`; unary `🧠−`):

- [`simpleAdditionIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/simpleAdditionIsApproved.foo) — `3 + 4` → 7; basic binary operator desugaring and scalar value()
- [`simpleSubtractionIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/simpleSubtractionIsApproved.foo) — `10 - 3` → 7
- [`simpleMultiplicationIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/simpleMultiplicationIsApproved.foo) — `6 * 7` → 42
- [`simpleDivisionIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/simpleDivisionIsApproved.foo) — `15 / 3` → 5
- [`simpleUnaryMinusIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/simpleUnaryMinusIsApproved.foo) — `-42`; unary operator taking one preceding value
- [`chainedArithmeticIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/chainedArithmeticIsApproved.foo) — `1 + 2 + 3 + 4` → 10; left-associative chaining of operators
- [`negativeResultsIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/negativeResultsIsApproved.foo) — Multiple assignments producing negative arithmetic results
- [`zeroDivisionIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/zeroDivisionIsApproved.foo) — `10 / 0` → NK (`🧠???`); ArithmeticException producing terminal CONSTANT with NK value
- [`multipleArithmeticExpressionsIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/multipleArithmeticExpressionsIsApproved.foo) — Multiple independent arithmetic expressions in one brane

**Identifier resolution and assignment** (SearchFir, IdentifierFiroe, AssignmentFiroe):

- [`simpleIdentifierIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/simpleIdentifierIsApproved.foo) — `x = 42; x;` — assignment + identifier search in parent brane
- [`multipleIdentifiersIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/multipleIdentifiersIsApproved.foo) — `x * y + z` with multiple bindings; search resolution order
- [`identifierInExpressionIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/identifierInExpressionIsApproved.foo) — Identifiers used as operands in arithmetic expressions
- [`constanticRendering.foo`](../../test-resources/org/foolish/fvm/inputs/constanticRendering.foo) — `x = non_existent;` → CONSTANIC; search finds nothing, not NK (may gain value via concatenation)
- [`identifierSeparators.foo`](../../test-resources/org/foolish/fvm/inputs/identifierSeparators.foo) — Multi-script identifiers (π, Б, א, 値); verifies search cache handles Unicode

**Search operators** (`^` head, `$` tail, `#N` offset, `?pattern` regex):

- [`oneShotSearchIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/oneShotSearchIsApproved.foo) — Head (`^`) and tail (`$`) extracting scalars from branes; empty brane → NK
- [`offsetAccess.foo`](../../test-resources/org/foolish/fvm/inputs/offsetAccess.foo) — Positional access `data#0`, `data#-1`; out-of-bounds → NK
- [`unanchoredSeekBasic.foo`](../../test-resources/org/foolish/fvm/inputs/unanchoredSeekBasic.foo) — `#-1 + #-2` within brane; unanchored seek accesses preceding statements
- [`test_unanchored_oneshot.foo`](../../test-resources/org/foolish/fvm/inputs/test_unanchored_oneshot.foo) — `#-1$` combining unanchored seek with tail operator
- [`simpleRegexSearchIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/simpleRegexSearchIsApproved.foo) — `{y = 1;}?y` localized pattern search
- [`regexSearchWithPatternIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/regexSearchWithPatternIsApproved.foo) — `?(a.*)` regex matching against identifier names
- [`regexSearchNotFoundIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/regexSearchNotFoundIsApproved.foo) — Pattern search returning NK when brane is CONSTANT and pattern not found
- [`assignmentAnchor.foo`](../../test-resources/org/foolish/fvm/inputs/assignmentAnchor.foo) — Pattern search anchored to assignment `brn?a.*`
- [`testTilde.foo`](../../test-resources/org/foolish/fvm/inputs/testTilde.foo) — Complement search `~.*e$` for pattern negation

**Level-skipping search** (FulfillSearch escalation through brane hierarchy):

- [`levelSkippingSearchFound.foo`](../../test-resources/org/foolish/fvm/inputs/levelSkippingSearchFound.foo) — Search escalates through brane levels to find CONSTANT value
- [`levelSkippingSearchNotFound.foo`](../../test-resources/org/foolish/fvm/inputs/levelSkippingSearchNotFound.foo) — Undeclared identifier; search reaches root, produces CONSTANIC
- [`levelSkippingSearchConstanic.foo`](../../test-resources/org/foolish/fvm/inputs/levelSkippingSearchConstanic.foo) — Level-skipping when found value is itself CONSTANIC → WOCONSTANIC
- [`anchoredSearchOnConstant.foo`](../../test-resources/org/foolish/fvm/inputs/anchoredSearchOnConstant.foo) — `b^`, `b$`, `b?α` on fully CONSTANT brane
- [`anchoredSearchFailsOnConstant.foo`](../../test-resources/org/foolish/fvm/inputs/anchoredSearchFailsOnConstant.foo) — Pattern not found on CONSTANT brane → NK (provably unfindable)
- [`anchoredSearchOnConstanic.foo`](../../test-resources/org/foolish/fvm/inputs/anchoredSearchOnConstanic.foo) — Head/tail on CONSTANIC value
- [`test_nested_brane_boundary.foo`](../../test-resources/org/foolish/fvm/inputs/test_nested_brane_boundary.foo) — Unanchored seek respects brane boundary (does not cross into child)

### Contract Summary: System Operator Brane

| Aspect | Specification |
|--------|--------------|
| `hasBoundary` | `false` — no local namespace |
| `value()` | Returns scalar result (integer, boolean, etc.) |
| Search escalation | Starts directly in parent brane |
| Operand access | `parent.getValuesBeforeMe(me, N)` |
| Interior visibility | Not inspectable from outside |
| Arithmetic errors | Produce NK (`🧠???`), only catch `ArithmeticException` |
| Lifecycle | Standard proto-brane; computation in BRANING |
| Cloning | Standard constanic cloning |

---

## Section 4: Role C — ConcatenationBrane

### What It Is

ConcatenationBrane is a wrapping FIR that combines multiple branes into a single evaluation
context. Whether explicit `{...}{...}` or implicit `b1 b2` or operator desugaring `1 + 2` →
`{🧠1, 🧠2, 🧠+}`, all multi-brane merging goes through concatenation.

This is **the mechanism** by which operators work, by which branes gain new members, and by
which recoordination happens.

### Trait Configuration

ConcatenationBrane does not fit cleanly into the trait model because it has a **three-stage
lifecycle** that is fundamentally different from the standard proto-brane lifecycle. It is closer
to a specialized ProtoBrane subclass than a trait variation.

```
hasBoundary  = false (during stages A/B), then delegates to merged brane (which has boundary=true)
isDetachment = false
sfMode       = NONE
searchIsolation = true (during stages A/B), false (after merge)
```

### Delta from ProtoBrane

**Three-stage lifecycle replaces standard lifecycle.** ConcatenationBrane does not follow the
standard PREMBRYONIC → EMBRYONIC → BRANING flow. Instead:

```
Stage A (Isolation)     Stage B (Merge)          Stage C (Evaluation)
─────────────────────   ──────────────────────   ─────────────────────
Step children to        Clone & re-parent all    Normal brane evaluation
constanic in isolation  into merged brane        on merged result

Search isolation ON     Search isolation OFF     Parent search enabled
FulfillSearch blocked   Merged brane created     Merged brane steps
Children step           Statements flattened     through full lifecycle
independently           into contiguous array
```

**Search isolation.** During Stage A, the ConcatenationBrane **does not forward FulfillSearch
messages to its parent**. Children that cannot resolve identifiers locally reach CONSTANIC. This
is the critical design constraint: children must be evaluated in isolation so that their
CONSTANIC state accurately reflects what they're missing, which concatenation will then provide.

**value() returns the merged brane.** After merge, the ConcatenationBrane delegates to the
merged brane, which is itself a Normal Brane (Role A).

### Stage A: Isolation (PREMBRYONIC → EMBRYONIC of children)

1. Instantiate FIRs from source elements
2. Step all children until each reaches constanic **in isolation**
3. During this phase, `FulfillSearch` from children is **not forwarded** to parent
4. Children reach CONSTANIC if they have unresolved identifiers (which is expected — the
   concatenation partner will provide them)

### Stage B: Merge (BRANING transition)

1. For each child brane (in writing order), constanic-copy its statements:
   - CONSTANT/INDEPENDENT statements → reference (no copy needed)
   - CONSTANIC/WOCONSTANIC statements → recursively clone
2. Flatten all statements into a single contiguous array
3. Update parent references on cloned statements to point to merged brane
4. Build merged brane with:
   - Statement count = sum of all children's statement counts
   - Search cache = concatenated from all children (in writing order)
   - Parent reference = ConcatenationBrane's own parent

### Stage C: Evaluation (merged brane's full lifecycle)

The merged brane is now a Normal Brane (Role A) and runs the standard lifecycle:
- PREMBRYONIC → EMBRYONIC → BRANING → constanic
- Search resolution now works across the merged namespace
- FulfillSearch to parent is now allowed (search isolation is off)
- Previously-CONSTANIC expressions re-resolve in the new combined context

### Terminal State

ConcatenationBrane reaches the same constanic state as its merged brane.

### Example: Direct Concatenation

```foolish
{
   c = {a=1,b=2,c=3}{e=4,f=5,g=6};
}
```

Approved output shows `c` contains both sets of statements:
```
＿c = {
＿    ＿{
＿    ＿＿a = 1;
＿    ＿＿b = 2;
＿    ＿＿c = 3;
＿    ＿};
＿    ＿{
＿    ＿＿e = 4;
＿    ＿＿f = 5;
＿    ＿＿g = 6;
＿    ＿};
＿};
```

**Walk-through:**
1. **Stage A**: `{a=1,b=2,c=3}` and `{e=4,f=5,g=6}` are stepped in isolation. Both reach
   CONSTANT (all identifiers are literal — nothing to search for).
2. **Stage B**: Statements are merged: `[a=1, b=2, c=3, e=4, f=5, g=6]`.
3. **Stage C**: The merged brane evaluates. All statements are already CONSTANT. The merged brane
   reaches CONSTANT immediately.

### Example: Concatenation Resolving CONSTANIC

This is the motivating example for why concatenation matters:

```foolish
{
    f = {a = x;};         !! x not found → a is CONSTANIC
    g = {x = 42;};
    h = g f;              !! Concatenation: g prepended to f
}
```

**Walk-through:**
1. `f` evaluates: `a = x` searches for `x`, doesn't find it → `a` is CONSTANIC
2. `g` evaluates: `x = 42` is fully resolved → CONSTANT
3. `h = g f` triggers concatenation:
   - **Stage A**: `g` (CONSTANT) and `f` (has CONSTANIC child `a`) are stepped in isolation
   - **Stage B**: Merge → `[x=42, a=x]`. Clone `a=x` with new parent pointing to merged brane
   - **Stage C**: Re-evaluate `a = x` in merged context. Now `x` is found (it's `42`).
     `a` becomes `42`. The merged brane reaches CONSTANT.

This is why unresolved identifiers should be CONSTANIC, not NK — concatenation can provide
the missing context.

### Example: Reference Concatenation

```foolish
{
   b1={a=1,b=2,c=3};
   b2={e=4,f=5,g=6};
   c = b1 b2;
}
```

Here `b1` and `b2` are brane-valued identifiers. The concatenation `b1 b2` first resolves the
references (during Stage A), then merges the referenced branes' statements.

### Related Approval Tests

Each file is in `test-resources/org/foolish/fvm/inputs/`.

- [`concatenationBasics.foo`](../../test-resources/org/foolish/fvm/inputs/concatenationBasics.foo) — Direct concatenation `{a=1}{b=2}`, reference concatenation `b1 b2`, mixed `{...} b2`, and chained `b1 b2 b3`; exercises all three stages with CONSTANT children
- [`concatenationSearch.foo`](../../test-resources/org/foolish/fvm/inputs/concatenationSearch.foo) — Concatenation with field dereferencing and search after merge; exercises Stage C search resolution across merged namespace

Note: only 2 dedicated concatenation tests exist currently. The testing strategy (Section 9.7)
identifies additional tests needed, especially for CONSTANIC resolution through concatenation.

### Contract Summary: ConcatenationBrane

| Aspect | Specification |
|--------|--------------|
| `hasBoundary` | `false` during isolation; delegates to merged brane after |
| `value()` | Returns the merged brane |
| Search isolation | **ON** during Stages A/B; **OFF** after merge |
| FulfillSearch | Blocked during isolation; forwarded to parent after merge |
| Three stages | A (isolate), B (merge), C (evaluate) |
| Writing order | Left-to-right determines statement order in merged array |
| Constanic cloning | Clone each child, create new ConcatenationBrane, re-merge |
| Terminal state | Same as merged brane's terminal state |

---

## Section 5: Role D — Detachment Brane `[...]`

### What It Is

A detachment brane wraps a following brane and **filters search messages** passing through it.
When you write `[a]{...}`, the detachment intercepts searches for `a` heading toward the parent
and blocks them — making `a` temporarily "free" (unbound) within the wrapped brane.

This is Foolish's mechanism for free variables, parameterization, and late binding — *not*
permanent blocking. The filter is active during evaluation and becomes inactive once the wrapped
brane reaches CONSTANIC.

### Trait Configuration

```
hasBoundary  = varies (typically false for the detachment itself)
isDetachment = true
sfMode       = varies
```

### Delta from ProtoBrane

**Search filtering.** The defining behavior. Detachment adds a filter to the search path:

```
child sends FulfillSearch("a") → detachment intercepts → blocks if "a" is in filter set
```

When a search for a detached identifier would normally escalate to the parent, the detachment
brane intercepts it and responds with "not found" — even if the parent *does* have a binding for
that identifier.

**Filter is temporary.** Once the wrapped brane reaches CONSTANIC, the filter becomes permanently
inactive. Subsequent references to the brane do not re-apply the detachment. This is the M-brane
default semantics: one-time filtering.

**Left-associates with other detachments, right-associates with brane.**

```foolish
[a][b][+a]{...}
```

Parsed as: `(([a] [b]) [+a]) {…}` — the detachment chain left-associates, then the result
right-associates with the following brane.

- `[a]` — detach identifier `a` (block searches for `a`)
- `[b]` — also detach `b`
- `[+a]` — **re-attach** `a` (P-brane: selective un-detachment)

The combined filter: `a` is attached (the `[+a]` overrides the earlier `[a]`), `b` is detached.

### Detachment Filter Rules

| Syntax | Meaning |
|--------|---------|
| `[a]` or `[-a]` | Block searches for `a` (M-brane, default) |
| `[+a]` | Unblock searches for `a` (P-brane, selective re-attachment) |
| `[a,b,c]` | Block searches for `a`, `b`, and `c` |
| `[~pattern]` | Block searches matching regex pattern (forward search liberation) |
| `[#N]` | Block by position index (forward search liberation) |

### Lifecycle Specifics

Detachment follows the standard proto-brane lifecycle with one addition: the search filter
is active during EMBRYONIC and BRANING, and deactivates at constanic:

```
PREMBRYONIC → EMBRYONIC → BRANING → constanic
                 │             │
                 └─────────────┘
                 Filter active: intercept
                 FulfillSearch for detached names

                                          → constanic: filter permanently inactive
```

### Interaction with Concatenation

Per the design (NAMES_SEARCHES_N_BOUNDS.md), **detachment is removed before concatenation**:

```foolish
[a]{x = a;} {a = 42;}
```

When concatenation processes this:
1. The detachment `[a]` is applied during Stage A (isolation) — `a` is free in `{x = a;}`
2. During Stage B (merge), the detachment filter is removed
3. In Stage C (evaluation), the merged brane `{x = a; a = 42;}` resolves `a` normally

This is a critical design decision: concatenation provides fresh context, and detachment should
not persist across that boundary.

### Example: Basic Detachment

```foolish
{
    a = 10;
    b = [a]{x = a + 1;};    !! a is detached — x cannot find a
}
```

Without detachment, `x = a + 1` would find `a = 10` in the parent and produce `x = 11`.
With `[a]`, the search for `a` is blocked: `a + 1` → `a` is CONSTANIC → `x` is WOCONSTANIC.

### Example: Detachment + Concatenation (Recoordination)

```foolish
{
    template = [x]{result = x * 2;};
    instance = {x = 5;} template;
}
```

**Walk-through:**
1. `template` evaluates: `[x]` blocks `x`, so `result = x * 2` → `x` is CONSTANIC,
   `result` is WOCONSTANIC. Template is CONSTANIC.
2. `instance = {x = 5;} template` triggers concatenation:
   - Stage A: `{x = 5;}` reaches CONSTANT. `template` is already CONSTANIC.
   - Stage B: Merge → `[x=5, result=x*2]`. Detachment removed during merge.
   - Stage C: `result = x * 2` re-resolves. `x` is now found (`5`). `result = 10`. CONSTANT.

This is the Foolish equivalent of function application: detachment creates parameters,
concatenation provides arguments.

### Open Questions (Requiring Decisions in D4)

1. **Does detachment interact with `hasBoundary`?** If the detachment wraps a Normal Brane
   (which has its own search boundary), does the filter apply *before* or *after* local search?
   (Answer should be: filter applies to searches escalating *past* the wrapped brane, not to
   local searches within it.)

2. **What happens when the detachment wraps a system operator?** System operators have no
   boundary. Their searches start in parent. Detachment intercepts those searches.

3. **Re-detachment after cloning.** When a CONSTANIC detachment brane is constanic-cloned, does
   the clone get a fresh active filter? (Answer should be: yes — the clone is a fresh evaluation.)

### Related Approval Tests

No dedicated detachment approval tests exist in the current test suite. The five detachment tests
from UBC1 encode wrong semantics (see [human_todo_index.md § H10](../todo/human_todo_index.md))
and need to be rewritten. The testing strategy (Section 9.7) identifies the tests that must be
created before Role D can be implemented.

Legacy reference: [`vintage_legacy/003-Detachment_Project.md`](../vintage_legacy/003-Detachment_Project.md)

### Contract Summary: Detachment Brane

| Aspect | Specification |
|--------|--------------|
| `isDetachment` | `true` — intercepts and filters searches |
| `value()` | Returns the wrapped brane (with filter inactive) |
| Filter set | Identifiers to block/unblock; M-brane (block) or P-brane (unblock) |
| Filter lifetime | Active during EMBRYONIC/BRANING; inactive at constanic |
| Association | Left-associate detachments; right-associate result with following brane |
| With concatenation | Filter removed before merge (per NAMES_SEARCHES_N_BOUNDS.md) |
| Cloning | Clone gets fresh active filter |
| Lifecycle | Standard proto-brane + search interception |

---

## Section 6: Lessons Learned from UBC1

The following lessons from UBC1 directly inform the D0 design choices. Each lesson is stated
with its consequence for the contract specification.

### Lesson 1: Don't Let FIR Types Implement Their Own `step()`

**What happened:** UBC1 had 15+ FIR types (BinaryFiroe, UnaryFiroe, IfFiroe, AssignmentFiroe,
etc.) each with a custom `step()`. This led to inconsistent state transitions, forgotten state
checks, and `instanceof` chains (10-15 cases) in search unwrapping.

**Consequence for D0:** All four roles share one `step()` implementation in ProtoBrane. Behavior
differences are expressed through trait checks within that single `step()`, not through
method override.

### Lesson 2: Don't Swallow Exceptions as NK Values

**What happened:** UBC1's BinaryFiroe caught all `Exception` and wrapped them in NKFiroe. This
hid NullPointerExceptions and ClassCastExceptions behind "errors as values."

**Consequence for D0:** System operators (Role B) must only catch `ArithmeticException`. All
other exceptions propagate as world-stopping errors.

### Lesson 3: Don't Use Reflection to Access Private Fields

**What happened:** UBC1's ConcatenationFiroe used Java reflection to access `braneMemory` and
`indexLookup` from `FiroeWithBraneMind`.

**Consequence for D0:** ConcatenationBrane (Role C) accesses what it needs through ProtoBrane's
public interface. The contract must expose merge-necessary operations (statement iteration,
constanic cloning, parent reassignment) without reflection.

### Lesson 4: Don't Force-Approve Tests

**What happened:** UBC1 had a commit "Temporarily approve some of these" where test baselines
were accepted without verification. Five detachment tests encode wrong semantics.

**Consequence for D0:** Each role's contract must be testable through approval tests that are
manually reviewed. No approval test should be accepted without understanding why the output
is correct.

### Lesson 5: Don't Wrap When You Can Clone-and-Replace

**What happened:** UBC1's CMFir wrapper delegated state queries to an inner FIR. This broke
transparency and had critical bugs (`achievedConstanic()` vs `atConstanic()`).

**Consequence for D0:** Constanic cloning replaces the slot directly. No wrapper types. Each
role's `cloneConstanic()` produces a fresh ProtoBrane with the appropriate traits, not a wrapper.

### Lesson 6: Stabilize in One Language First

**What happened:** UBC1 ported features to Scala while unstable in Java, creating a two-front war.

**Consequence for D0:** All four role contracts are specified and tested in Java first. Scala
implementation follows only after Java is stable.

---

## Section 7: How Each Role Appears in UBC2 Terms

This section maps each role to the UBC2 denotational semantics and lifecycle terminology.

### Normal Brane in Denotational Terms

A Normal Brane `{a=1; b=2; c=a+b;}` is the canonical brane. Its denotation is:

```
🧠⟦{a=1; b=2; c=a+b;}⟧
```

The semantic bracket `🧠⟦...⟧` wraps the entire expression. The brane has:
- Three statements in its statement array
- A search cache with entries `["a", "b", "c"]`
- hasBoundary = true → searches for `a`, `b`, `c` resolve locally

### System Operator in Denotational Terms

The expression `1 + 2` has denotation:

```
🧠⟦1 + 2⟧  =  🧠1 🧠2 🧠+
```

Each token carries its own semantic bracket. The three proto-branes concatenate by proximity:

```
{🧠1, 🧠2, 🧠+}
```

The `🧠+` operator's contract: wait for siblings `🧠1` and `🧠2` to be CONSTANT, retrieve their
`value()` results (both `long`), compute the sum, set own `value()` to result.

### ConcatenationBrane in Denotational Terms

The expression `{a=1}{b=a+1}` has denotation:

```
🧠⟦{a=1}{b=a+1}⟧
```

This is a concatenation of two brane denotations. The ConcatenationBrane's contract:
- Evaluate each child in isolation (search isolation enforced)
- Merge into a single brane: `{a=1; b=a+1;}`
- Re-evaluate in merged context: `b` now finds `a`, resolves to `2`

### Detachment in Denotational Terms

The expression `[x]{f = x + 1;}` has denotation:

```
🧠⟦[x]{f = x + 1;}⟧
```

The detachment `[x]` is a modifier on the following brane. It does not have its own value — it
modifies the search behavior of the thing it wraps.

When combined with concatenation: `{x=5;} [x]{f = x + 1;}`:
1. Detachment is applied during isolation (Stage A): `x` is free in `{f = x + 1;}`
2. Detachment is removed during merge (Stage B)
3. `x=5` is available in the merged context (Stage C): `f = 6`

---

## Section 8: Walking Through Approval Tests

This section walks through existing approval tests to show how each role's contract manifests in
actual system behavior.

### Test: `constanticRendering` — Demonstrating CONSTANIC

**Input:**
```foolish
{x = non_existent;}
```

**Approved output:**
```
{
＿x = ⎵⎵ (CONSTANIC);
}
```

**What this shows:** The identifier `non_existent` is searched for but not found. The search
produces CONSTANIC (not NK `🧠???`) because in a different context (via concatenation), the
identifier might exist. The brane itself is complete (it evaluated everything it could), but `x`
is stuck waiting for a context that provides `non_existent`.

**Role contract exercised:** Role B (system operator / search FIR) — the identifier search is a
proto-brane that starts searching in its parent brane (no local boundary), finds nothing, and
becomes CONSTANIC.

### Test: `complexIdentifierScopeIsApproved` — Normal Brane Scoping

**Input:**
```foolish
{
    a = 10; b = 20;
    outer'{
        c = 30;
        a + c;
        inner'{a = 40; a + b + c;};
        a + b;
    };
}
```

**Approved output:**
```
{
＿a = 10;
＿b = 20;
＿outer'{
＿＿c = 30;
＿＿40;
＿＿inner'{
＿＿＿a = 40;
＿＿＿90;
＿＿};
＿＿30;
＿};
}
```

**Walk-through:**
- `a + c` in `outer'`: `a` not local → FulfillSearch to parent → found `a=10`. `c` is local
  → `30`. Result: `10 + 30 = 40`. ✓
- `a + b + c` in `inner'`: `a` found locally (`40`). `b` not local, not in parent `outer'` →
  FulfillSearch escalates to grandparent → found `b=20`. `c` not local → found in parent
  `outer'` (`30`). Result: `40 + 20 + 30 = 90`. ✓
- `a + b` in `outer'`: `a` not local (inner's `a=40` is in inner's scope, not outer's) →
  FulfillSearch to parent → found `a=10`. `b` not local → parent → `b=20`. Result: `30`. ✓

**Role contracts exercised:**
- Role A (Normal Brane): Each `{...}` defines a search boundary; local-first resolution
- Role B (System Operator): `+` operators access parent statement arrays, compute scalars

### Test: `concatenationBasics` — ConcatenationBrane

**Input:**
```foolish
{
   test_1={
      c = {a=1,b=2,c=3}{e=4,f=5,g=6};
   };
}
```

**Approved output:**
```
＿testˍ1 = {
＿         ＿c = {
＿         ＿    ＿{
＿         ＿    ＿＿a = 1;
＿         ＿    ＿＿b = 2;
＿         ＿    ＿＿c = 3;
＿         ＿    ＿};
＿         ＿    ＿{
＿         ＿    ＿＿e = 4;
＿         ＿    ＿＿f = 5;
＿         ＿    ＿＿g = 6;
＿         ＿    ＿};
＿         ＿};
＿};
```

**Walk-through:**
1. `{a=1,b=2,c=3}` evaluates in isolation → CONSTANT (all literals)
2. `{e=4,f=5,g=6}` evaluates in isolation → CONSTANT (all literals)
3. Merge: statements `[a=1, b=2, c=3, e=4, f=5, g=6]` in a new brane
4. Merged brane evaluates → CONSTANT (everything already resolved)
5. `c` in `test_1` receives the merged brane as its value

**Role contract exercised:** Role C (ConcatenationBrane): search isolation during Stage A,
merge in Stage B, normal evaluation in Stage C.

### Test: `oneShotSearchIsApproved` — Head/Tail Operators

**Input:**
```foolish
{
  x = { 10; 20; 30 }^;   !! head → 10
  y = { 10; 20; 30 }$;   !! tail → 30
  e = {}^;                !! empty → ???
}
```

**What this shows:**
- `^` (head) extracts the first statement's value from a brane
- `$` (tail) extracts the last statement's value
- On empty brane: NK (`🧠???`) — there is no first/last element, and no future context will
  add one (the brane literal is CONSTANT and empty)

**Role contracts exercised:**
- Role A (Normal Brane): The `{10; 20; 30}` is a Normal Brane — a first-class value that can
  be searched/indexed
- Role B (System Operator): `^` and `$` are proto-brane search operators that access the
  brane's statement array

---

## Section 9: Design Proposal

Based on the analysis above, here is the proposed design for the four FIR role contracts.

### 9.1 One ProtoBrane, Trait-Differentiated

**Proposal:** Implement ProtoBrane as a single concrete class with three trait fields:

```java
public class ProtoBrane extends FIR {
    // --- Traits ---
    private final boolean hasBoundary;       // Role A: true; Roles B,C,D: false (or varies)
    private final boolean isDetachment;      // Role D: true; Roles A,B,C: false
    private final SFMode sfMode;             // NONE, SF, or SFF

    // --- Lifecycle ---
    private Nyes nyes;                       // Current state
    private BraneMind braneMind;             // Work queue (nullable for leaf proto-branes)
    private BraneMemory braneMemory;         // Statement storage (nullable for leaf proto-branes)

    // --- Detachment (Role D only) ---
    private Set<String> detachedIdentifiers; // null when isDetachment=false
    private boolean filterActive;            // true during evaluation, false at constanic
}
```

**Rationale:** This directly implements the UBC2 design principle: "These are not separate classes
— they are a single ProtoBrane class with flags that enable/disable specific steps."

### 9.2 ConcatenationBrane as a Subclass

**Proposal:** ConcatenationBrane should be a **subclass** of ProtoBrane, not a trait variation.

```java
public class ConcatenationBrane extends ProtoBrane {
    private List<FIR> children;
    private ProtoBrane mergedBrane;
    private boolean searchIsolation;
    private ConcatStage stage;  // STAGE_A, STAGE_B, STAGE_C
}
```

**Rationale:** The three-stage lifecycle is too structurally different from the standard lifecycle
to be expressed as a trait flag. ConcatenationBrane overrides `step()` with its own staging
logic, but delegates to the merged brane (which is a standard ProtoBrane) for Stage C.

This is the **one exception** to "no separate `step()` implementations." The justification: the
concatenation lifecycle genuinely has different stages, not just different trait checks within the
same stages.

**Alternative considered:** Expressing ConcatenationBrane as a standard ProtoBrane with
`isConcatenation=true` and conditional logic in `step()`. Rejected because the three stages
would make `step()` a sprawling state machine that is harder to understand than a clean subclass.

### 9.3 Search Resolution Contract

**Proposal:** The search resolution contract depends on `hasBoundary`:

```java
// Inside ProtoBrane.resolveSearch(Query query):
if (this.hasBoundary) {
    // Role A: search local statements first
    FIR result = this.braneMemory.search(query, backwards_from_cursor);
    if (result != null) return result;
}
// All roles: escalate to parent via FulfillSearch
this.sendToParent(new FulfillSearch(query, this.luid));
```

If `isDetachment` is true on the parent chain, the detachment intercepts:

```java
// Inside ProtoBrane.processChildMessage(FulfillSearch msg):
if (this.isDetachment && this.filterActive) {
    if (this.detachedIdentifiers.contains(msg.query.name())) {
        // Block: respond NotFound to child
        this.sendToChild(new RespondToSearch(msg.sourceLuid, msg.query, NotFound));
        return; // Do NOT escalate to parent
    }
}
// Not blocked: escalate normally
this.sendToParent(msg);
```

### 9.4 value() Contract

**Proposal:** Explicit `value()` behavior per role:

| Role | `value()` When CONSTANT | `value()` When CONSTANIC |
|------|------------------------|--------------------------|
| A: Normal Brane | Returns `this` (the brane) | Returns `this` (the brane, partially resolved) |
| B: System Operator | Returns computed scalar (`long`) | Throws / returns CONSTANIC marker |
| C: ConcatenationBrane | Returns merged brane | Returns merged brane (partially resolved) |
| D: Detachment | Returns wrapped brane (filter inactive) | Returns wrapped brane (filter inactive) |

### 9.5 Constanic Cloning Contract

**Proposal:** Per-role cloning behavior:

| Role | CONSTANT Cloning | CONSTANIC Cloning |
|------|-----------------|-------------------|
| A: Normal Brane | Returns `this` (share) | Clone brane + all CONSTANIC children; reset to PREMBRYONIC |
| B: System Operator | Returns `this` (share) | Clone operator + operand refs; reset to PREMBRYONIC |
| C: ConcatenationBrane | Returns `this` (share) | Clone each child, create new ConcatenationBrane, re-merge |
| D: Detachment | Returns `this` (share) | Clone with **fresh active filter** + clone wrapped brane |

Key point for Role D: when a detachment brane is constanic-cloned, the clone gets a fresh filter
(`filterActive = true`) even though the original's filter was deactivated at constanic. This is
because the clone is being re-evaluated in a new context and should re-apply the detachment.

### 9.6 Interface Requirements for ProtoBrane

**Proposal:** ProtoBrane must expose these methods for the four roles to function:

```java
// State queries (shared by all roles)
boolean isNye();
boolean isConstanic();
boolean isConstant();
boolean atConstanic();
boolean atConstant();
Nyes getNyes();

// Value (role-specific behavior, one implementation)
Object value();        // Returns brane for Role A/C/D, scalar for Role B

// Trait queries
boolean hasBoundary();
boolean isDetachment();
SFMode sfMode();

// Lifecycle
int step();            // Single step(); trait-differentiated internally
FIR cloneConstanic(FIR newParent, Optional<Nyes> targetNyes);

// Statement access (needed by concatenation and system operators)
BraneMemory getBraneMemory();
int getStatementCount();
FIR getStatement(int index);

// Parent chain
FIR getParentFir();
void setParentFir(FIR parent);

// Communication
void sendToParent(Message msg);
void processChildMessage(Message msg);

// Detachment (Role D only, no-op for others)
boolean shouldFilter(String identifierName);
```

### 9.7 Testing Strategy

**Proposal:** Each role gets targeted approval tests:

| Role | Test Focus | Concrete Tests Needed |
|------|-----------|----------------------|
| A | Scope resolution, nesting, value-is-brane | Existing tests adequate; add depth-limit test |
| B | Scalar computation, operand wait-for, error handling | Add: CONSTANIC operand test, division-by-zero test, type mismatch test |
| C | Search isolation, merge ordering, CONSTANIC resolution | Add: cross-brane identifier resolution, writing-order precedence |
| D | Filter activation/deactivation, P-branes, concat interaction | Rewrite 5 wrong-semantics tests; add filter lifetime test |

### 9.8 Summary: The Contract Delta Table

| Aspect | ProtoBrane Default | A: Normal Brane | B: System Operator | C: Concatenation | D: Detachment |
|--------|-------------------|-----------------|-------------------|------------------|---------------|
| `hasBoundary` | — | **true** | **false** | false → true (after merge) | varies |
| `isDetachment` | — | false | false | false | **true** |
| Search start | parent | **local first** | parent | isolated → parent (after merge) | parent (filtered) |
| `value()` | — | **brane itself** | **scalar** | **merged brane** | **wrapped brane** |
| `step()` | standard | standard | standard | **three-stage override** | standard + filter |
| Cloning | — | standard | standard | **clone children + re-merge** | standard + **fresh filter** |
| Depth | — | **tracked** | inherited | delegated to merged | inherited |
| Interior | visible | **visible** | **not visible** | visible (merged) | visible (wrapped) |

---

## Last Updated

**Date**: 2026-02-25
**Updated By**: Claude Code v1.0.0 / claude-opus-4-6
**Changes**: Addressed human feedback: renamed document to "ProtoBrane Derived FIRs — Design
Document"; rewrote Background section (big-to-small role introduction, positive framing, Nyes
microstate design concerns, trait explanation); added cross-links from roles table to later
sections; added comprehensive approval test references to each role section (10 tests for Role A,
27 tests for Role B organized by sub-category, 2 tests for Role C with gap analysis, 0 for
Role D with rewrite plan noted). De-prioritized parsing-focused tests (precedence, grouping)
in favor of runtime FIR behavior tests.
**Previous (2026-02-25):** Initial creation.
