# UBC2 Design Specification

The Unicellular Brane Computer Mark 2 (UBC2) is the reference implementation for establishing the
expected behavior of a brane computer. This document specifies its design.

For the engineering reference of the current UBC1 implementation, see
[UBC Engineering Reference](ubc_engineering.md).

---

## Table of Contents

- [What Is the UBC2](#what-is-the-ubc2)
- [Message-Passing Architecture](#message-passing-architecture)
- [Every Expression Is a Brane](#every-expression-is-a-brane)
- [FIR Lifecycle Stages](#fir-lifecycle-stages)
  - [PRE_EMBRYONIC](#pre_embryonic)
  - [EMBRYONIC](#embryonic)
  - [BRANING](#braning)
  - [CONSTANIC Terminal States](#constanic-terminal-states)
- [SF and SFF Markers](#sf-and-sff-markers)
- [Constanic Cloning and Coordination](#constanic-cloning-and-coordination)
- [Brane Communication Protocol](#brane-communication-protocol)
- [LUID: Locally Unique Identifiers](#luid-locally-unique-identifiers)
- [Search Resolution and Precedence](#search-resolution-and-precedence)
- [NK vs CONSTANIC: When Is a Search Truly Failed?](#nk-vs-constanic-when-is-a-search-truly-failed)
- [Sequencing: Human-Readable Output](#sequencing-human-readable-output)
- [FIR Type Hierarchy](#fir-type-hierarchy)
- [Design TODO](#design-todo)
- [Open Issues](#open-issues)
- [Relationship to UBC1](#relationship-to-ubc1)
- [Lessons Learned from UBC1](#lessons-learned-from-ubc1)
- [Future Optimizations](#future-optimizations)

---

## What Is the UBC2

The UBC2 is a small computer that does things to a brane. It is analogous to the stack of a program
on a Xeon processor: when a brane is loaded, we know exactly what to do. The UBC2 takes Foolish
source code, parses it into AST, converts AST into Foolish Internal Representations (FIRs), and
steps those FIRs through a well-defined lifecycle until they reach a terminal constanic state.

The UBC2 is a **reference implementation**. Its purpose is to establish the expected behavior of a
brane computer — not to be the fastest or most clever implementation, but to be the most
understandable and correct one. A conservative implementation style identifies explicit stages of
FIR processing where each stage has clear entry conditions, work to perform, and exit conditions.

### Design Principles

1. **Every expression is a brane.** All non-literal expressions — binary ops, unary ops, searches,
   curly-brace branes — are branes. Literal values are INDEPENDENT. One lifecycle handles
   everything that requires identifier resolution. (Note: UBC2 does not implement if-then-else;
   path selection is search-based. See [Design TODO](#design-todo).)
2. **Conservative staging.** Each stage of the FIR lifecycle does a bounded, well-defined amount of
   work. No stage tries to be clever.
3. **Message passing.** Branes communicate with parent and children through a message protocol.
   Scope resolution is expressed as message exchange, not implicit traversal.
4. **Writing-order precedence.** When multiple search results are possible, first-to-write wins.
   The order in which the programmer wrote the code is the order in which things are found.
5. **Explicit lifecycle.** The stages PRE_EMBRYONIC → EMBRYONIC → BRANING → CONSTANIC make visible
   what the UBC is doing at each point and why.

---

## Message-Passing Architecture

The UBC2 is a message-passing architecture. Each brane has addresses for sending and receiving
messages from its parent brane and its sub-branes (children). The message protocol is simple:

| Message | Direction | Meaning |
|---------|-----------|---------|
| **Fulfill search** | Child → Parent | "I have an unresolved search. Can you find this?" |
| **Respond to search** | Parent → Child | "Here is the result of your search request." |

A brane that cannot resolve a child's search forwards it to its own parent. Messages flow upward
until resolved or until the root brane is reached (the unresolved search produces CONSTANIC — the
search was performed and nothing was found).

Each brane maintains a **braneMind** — the set of FIRs and pending communications it is actively
working on. The braneMind aggregates unresolved searches for outbound communication and monitors
for inbound search results.

### Circular Message-Passing Sanity Check

Every FIR in the brane tree has a **depth** — the number of brane boundaries between it and the
root. Top-level INDEPENDENT FIRs (literals) have depth 0. The message-passing system enforces a
structural invariant:

> **A FulfillSearch message sent to a parent must always originate from a strictly deeper node.**

When a brane receives a FulfillSearch message, it checks that the sender's depth is greater than
its own. If this invariant is violated, the message is malformed — it indicates a circular parent
reference or a broken tree structure. The brane raises a PANIC alarm and drops the message.

This check is cheap (one integer comparison per message) and prevents:
- Infinite message forwarding loops from circular parent chains
- Messages traveling sideways or downward in the tree
- Corrupted tree structures from cloning or reparenting bugs

---

## Every Expression Is a Brane

In UBC2, **every expression that is not a literal value is a brane**. Binary operations, unary
operations, searches — all are branes. This is the central unification: there is only one lifecycle
implementation, and it handles everything that requires identifier resolution.

**Note:** UBC2 removes if-then-else (`IfFiroe`) entirely. Path selection in UBC2 is search-based,
not conditional-branching-based. The IfFiroe implementation in UBC1 had persistent infinite-loop
bugs and a fundamentally fragile recursive `step()` design. UBC2 replaces this with a mechanism
that fits the brane model natively. See [Design TODO](#design-todo) for the planned approach.

### Literal Values Are Always INDEPENDENT

Foolish literal values (integers, strings) are always in the **INDEPENDENT** state. They do not
transition through any lifecycle stages. They have no searches to resolve, no dependencies to wait
for, and no parent to communicate with. They simply exist.

### Boundary-Less Branes (Proto-Branes)

Expressions like binary operations (`a + b`) and unary operations (`-x`) are branes, but they
differ from curly-brace branes `{...}` in two ways:

1. **No boundary.** A boundary-less brane does not define a search scope. When a search starts
   inside a boundary-less brane, it begins in the **parent brane** where the expression is located.
   In contrast, a curly-brace brane `{...}` defines a boundary — searches inside it search within
   the brane first before going to parents.

2. **`.value()` returns a literal.** The result of evaluating a boundary-less brane is a literal
   value (integer, string), not a brane. A curly-brace brane's `.value()` returns the brane itself.

### Characterization of Proto-Branes

For typing purposes, boundary-less branes could be characterized. For example, an addition
operation might be expressed as:

```foolish
add = $'{ <#-1> + <#-2> }
```

The characterization `$'` signals special treatment of this brane — it is a proto-brane whose
result is a scalar value extracted from its computation. This notation is for future reference; the
UBC2 reference implementation does not require explicit characterizations to handle proto-branes.

### One Lifecycle

Because every non-literal expression is a brane, the UBC2 implements **one set of lifecycle
stages** (PRE_EMBRYONIC → EMBRYONIC → BRANING → CONSTANIC) that handles all expressions uniformly.
The differences between boundary-less branes and curly-brace branes are expressed through their
search scope rules, not through different lifecycle implementations.

---

## FIR Lifecycle Stages

The significant reframing in UBC2 is what happens between Foolish code and CONSTANIC state. The
UBC2 follows a conservative implementation style that identifies four explicit stages of FIR
processing.

### PRE_EMBRYONIC

A FIR in PRE_EMBRYONIC stage holds a Foolish code AST. The UBC2 performs the following steps in
order:

**Step 0 — Establish LUID.**
Each expression receives a Locally Unique Identifier. The LUID is composed of the statement number
followed by a disambiguator that distinguishes this expression from other expressions on the same
statement. The LUID is unique within the brane.

**Step 1 — Establish line count.**
Count the number of lines (statements) in the brane.

**Step 2 — Establish statement array.**
Create the array of statements from the AST. Each statement occupies one slot, indexed from 0.

**Step 3 — Gather characterized identifiers.**
Walk all *named* statements (assignments with LHS identifiers) and cache the fully characterized
identifier name for searching. Unnamed statements (those assigned to `???`) are skipped. The
named statements form a linked list pointing into the statement array.

Each `IdentifyingFiroe` caches its fully characterized identifier string so that searches can
match against it without re-traversing the characterization chain.

**Step 4 — Instantiate RHS FIRs.**
For each statement, instantiate the RHS expression's FIR in PRE_EMBRYONIC stage. This creates the
FIR tree for the brane's expressions without evaluating anything.

**Step 5 — Find all searches in RHS expressions.**
Walk the RHS expression FIRs and find all search operations, **stopping at brane boundaries**
(searches inside nested branes belong to those branes, not this one). Maintain the searches in the
order they were written to establish precedence: **first-to-write-first-to-find**.

**Step 6 — Resolve intra-brane searches.**
Attempt to resolve each RHS search within this brane. For searches that can be resolved locally
(the target identifier exists in this brane's named statements), bind the search to its result.
For searches that cannot be resolved locally, aggregate them into the braneMind for communication
with the parent brane and for monitoring incoming results.

**Step 7 — Append child branes.**
Append PRE_EMBRYONIC branes found in RHS expressions to the braneMind, in writing order (the order
in which they appear in the source code).

**→ Transition to EMBRYONIC.**

### EMBRYONIC

During the EMBRYONIC stage, the brane actively communicates to resolve its remaining searches.

#### Communication

- **Outbound:** The brane sends "Fulfill search" messages to its parent brane for all unresolved
  searches aggregated in step 6.
- **Inbound from children:** The brane receives "Fulfill search" messages from its child branes.
  It attempts to resolve them locally. If it cannot, it forwards them to its own parent.
- **Inbound from parent:** The brane receives "Respond to search" messages. It applies the results
  to its own unresolved searches and forwards results to children that requested them.

#### Wait-For Mechanism

When a search result resolves to a non-constanic expression (the target is still being evaluated),
the UBC2 implements a **wait-for mechanism**: the statement that depends on that expression is
enqueued to wait for its dependencies to become constanic.

For simplicity, the initial implementation uses **busy-checking**: the waiting statement
periodically checks whether its dependencies have reached a constanic state. A callback-based
mechanism is a future refinement.

#### Dependency Resolution Rules

When all of a statement's search dependencies become constanic, the statement can be finalized.
Special care is required depending on the terminal states of those dependencies:

1. **All dependencies CONSTANT.** The statement evaluates to CONSTANT. No cloning is needed —
   CONSTANT values are immutable and can be referenced directly.

2. **One or more dependencies CONSTANIC (while brane is EMBRYONIC).** The UBC2 must
   **constanic-clone** and relocate the CONSTANIC result into the current brane before the brane
   transitions to BRANING. The cloned expression replaces the current object occupying its slot in
   the statement array. See [Constanic Cloning and Coordination](#constanic-cloning-and-coordination)
   for the full mechanism.

   This is a departure from UBC1, which wrapped FIRs in a CMFir and waited for re-evaluation.
   In UBC2, the messaging system triggers `cloneConstanic` only when the object is already
   constanic — the clone replaces the slot directly.

The brane remains EMBRYONIC as long as it has non-brane expressions that are not yet CONSTANIC.

**→ Transition to BRANING** when all of this brane's non-brane expressions are CONSTANIC.

### BRANING

In the BRANING state, the brane is both itself a brane *and* does work on executing other branes.
It steps its children branes — the sub-branes that were appended to the braneMind in step 7.

During BRANING:
- The brane continues to handle inbound and outbound communications
- It steps each child brane, which may produce further search messages
- Children may themselves transition through EMBRYONIC → BRANING → constanic

The BRANING state persists until everything in the braneMind is CONSTANIC — all children branes
and all expressions have reached a terminal state.

**→ Transition to CONSTANIC.**

### CONSTANIC Terminal States

A FIR that is not stepping is **constanic** (lowercase). This is the generic term for any of the
terminal states:

| State | Meaning |
|-------|---------|
| **CONSTANIC** | Constant In Context. Search was performed and **nothing was found**. The expression has unresolved searches with no binding. May gain value when recoordinated into a new context that provides the missing bindings. |
| **WOCONSTANIC** | Waiting On CONSTANICs. Every search was found, but one or more of the found results are themselves CONSTANIC or WOCONSTANIC. The expression is waiting for its dependencies to settle. When those dependencies gain value (through recoordination), this expression can progress. |
| **CONSTANT** | Fully evaluated. All searches resolved to CONSTANT or INDEPENDENT values. Immutable. |
| **INDEPENDENT** | Promoted from CONSTANT. Detached from parent — although the parent may still hold a reference to it. Reserved for future development. |

The predicate `is_constanic` means
`at_constanic || at_woconstanic || at_constant || at_independent`. A FIR stops stepping when it
reaches any terminal state.

#### CONSTANIC vs WOCONSTANIC

The distinction between CONSTANIC and WOCONSTANIC is important:

- **CONSTANIC**: The expression itself has searches that returned no result. It needs to be placed
  in a new context where those symbols exist. Constanic cloning transitions the clone back to
  EMBRYONIC for fresh search resolution.
- **WOCONSTANIC**: The expression found everything it was looking for, but some of those things
  are not yet fully resolved. It is structurally complete but waiting on its dependencies. No new
  context is needed — the dependencies just need to finish evaluating.

This distinction matters for SF and SFF markers (see [SF and SFF Markers](#sf-and-sff-markers)):
SFF-marked expressions are always CONSTANIC (nothing searched yet), while SF-marked expressions
are typically WOCONSTANIC (symbols found but not resolved further).

#### INDEPENDENT

INDEPENDENT is a future-facing state. When a brane is promoted from CONSTANT to INDEPENDENT, it
detaches from its parent brane. The parent may still hold a reference to the INDEPENDENT brane, but
the INDEPENDENT brane no longer participates in its parent's lifecycle. The semantics of INDEPENDENT
are reserved for future development and are not exercised in the initial UBC2 implementation.

---

## SF and SFF Markers

Stay-Foolish (SF) and Stay-Fully-Foolish (SFF) markers control how eagerly an expression resolves
its identifiers. They interact with the lifecycle stages and terminal states in specific ways.

### SFF Marker: `<<expression>>`

An SFF-marked expression (Stay-Fully-Foolish) is instantiated in **PRE_EMBRYONIC** and transitions
**straight to CONSTANIC** — it performs no search resolution at all.

A reference like `<<d>>` does not even resolve `d` within its own brane. The expression is frozen
in its original AST form with all identifiers unresolved.

**Lifecycle shortcut:**
```
PRE_EMBRYONIC → CONSTANIC
```

SFF-marked expressions are always **CONSTANIC** (not WOCONSTANIC) because no searches were
performed — everything is unfound.

### SF Marker: `<expression>`

An SF-marked expression (Stay-Foolish) participates in its parent's **EMBRYONIC** stage. It tries
to find symbols as best as it can with these restrictions:

1. **Resolves its own symbols.** The SF-marked proto-brane participates in the parent brane's
   EMBRYONIC communication. It sends FulfillSearch messages for its own identifiers and receives
   RespondToSearch results.

2. **Does not forward children's messages.** An SF-marked expression **refuses** to transmit
   messages from its children to anywhere. It does not act as a relay for child search requests.
   It resolves only its own symbols.

3. **Does not resolve found results further.** When an SF-marked expression finds what `a`, `b`,
   and `c` refer to (in `<a + b + c>`), it binds those references but does **not** step them
   further. It records what was found without evaluating the found expressions.

**Lifecycle:**
```
PRE_EMBRYONIC → EMBRYONIC (parent's) → CONSTANIC or WOCONSTANIC
```

Once the SF-marked expression has resolved all its own searches to constanic targets, it becomes
constanic itself:

- If some searches were **not found**: the expression is **CONSTANIC**.
- If all searches were found but the found targets are themselves constanic: the expression is
  **WOCONSTANIC**.
- If all searches were found and all targets are CONSTANT/INDEPENDENT: the expression reaches
  **CONSTANT** (though this is unusual for SF-marked expressions, since the point of SF is
  typically to preserve constanic state).

### SF vs SFF Summary

| Aspect | SF `<expr>` | SFF `<<expr>>` |
|--------|-------------|----------------|
| Search resolution | Yes, own symbols only | None |
| Child message forwarding | No | N/A (no children active) |
| Resolves found results | No | N/A |
| Typical terminal state | WOCONSTANIC | CONSTANIC |
| Lifecycle | PRE_EMBRYONIC → EMBRYONIC → constanic | PRE_EMBRYONIC → CONSTANIC |

### Examples

```foolish
{
    a=1; b=2; c=3; d=4;
    f  = [a,b,c,d]{r = a+b+c+d};

    !! SFF: <<f>> is CONSTANIC — 'f' is not even looked up
    f_sff = <<f>>;

    !! SF: <f> finds 'f' (it refers to the brane above), but does not
    !! resolve f's internals. f_sf is WOCONSTANIC — it found 'f' but
    !! f itself is CONSTANIC (due to liberation of a,b,c,d).
    f_sf = <f>;
}
```

---

## Constanic Cloning and Coordination

When a search result during the EMBRYONIC stage is CONSTANIC or WOCONSTANIC, the UBC2 must copy
and relocate that expression into the current brane so it can be re-evaluated in the new context.
This is the **constanic cloning** mechanism, and it replaces UBC1's approach of wrapping FIRs in
CMFir.

### How Constanic Cloning Works

1. The messaging system determines that a search result is constanic.
2. If the result is **CONSTANT** or **INDEPENDENT**: the clone is simply a reference to the
   immutable object. CONSTANT elements never change, so sharing is safe. INDEPENDENT elements are
   similarly immutable and detached.
3. If the result is **CONSTANIC** or **WOCONSTANIC**: the expression is recursively
   `cloneConstanic`'d. For branes, the RHS expressions are constanically cloned. CONSTANT and
   INDEPENDENT sub-expressions within the clone are references to the immutable originals (no copy).
   Only CONSTANIC and WOCONSTANIC sub-expressions are recursively cloned.
4. The clone **replaces** the current object occupying the slot in the statement array. This is a
   direct replacement, not a wrapper.
5. The cloned CONSTANIC objects — and only the objects at CONSTANIC (not WOCONSTANIC) — transition
   back to **EMBRYONIC** stage and experience search resolution again in their new context.
   WOCONSTANIC clones do not need fresh search resolution (they already found everything); they
   wait for their cloned dependencies to settle.

### Key Differences from UBC1

| Aspect | UBC1 (CMFir) | UBC2 (Constanic Clone) |
|--------|-------------|----------------------|
| Mechanism | Wrap FIR in CMFir, two-phase eval | Clone and replace in slot |
| When triggered | During evaluation | Only when object is constanic |
| Wrapper overhead | CMFir delegates state | No wrapper; direct replacement |
| Re-evaluation | CMFir Phase B steps clone | Clone re-enters EMBRYONIC |
| CONSTANT handling | CMFir short-circuits | Reference to immutable; no clone |

### Cloning Rules by State

| Source State | Clone Action |
|-------------|-------------|
| CONSTANT | Reference to immutable object (no copy) |
| INDEPENDENT | Reference to immutable object (no copy) |
| CONSTANIC | Recursive `cloneConstanic`: clone expression tree, sub-expressions cloned recursively. Clone transitions to **EMBRYONIC** for fresh search resolution. |
| WOCONSTANIC | Recursive `cloneConstanic`: clone expression tree, sub-expressions cloned recursively. Clone remains **WOCONSTANIC** — it already found everything, just waiting on dependencies. |
| NYE (any pre-constanic) | Not cloned — the wait-for mechanism ensures cloning only happens after the source reaches constanic. |

### Coordination After Cloning

After a **CONSTANIC** clone is placed in its new slot and transitions to EMBRYONIC:

1. It participates in the brane's normal EMBRYONIC communication — sending FulfillSearch messages
   for its unresolved searches.
2. The new brane context may resolve searches that were unresolvable in the original context.
3. If all searches resolve to CONSTANT/INDEPENDENT, the clone reaches CONSTANT.
4. If all searches are found but some results are constanic, the clone reaches WOCONSTANIC.
5. If some searches remain unresolved, the clone reaches CONSTANIC in the new context.

After a **WOCONSTANIC** clone is placed in its new slot:

1. It does not re-enter EMBRYONIC — its searches were already resolved.
2. It monitors its cloned dependencies. As those dependencies settle (reach CONSTANT), the
   WOCONSTANIC expression may itself reach CONSTANT.
3. If dependencies remain constanic, the expression remains WOCONSTANIC.

This is the mechanism by which "a constanic brane can gain value when associated with a new
context" — the fundamental operation of brane coordination.

---

## Brane Communication Protocol

### Message Types

```
FulfillSearch {
    source_luid: LUID       // Who is asking
    query: Query             // What to search for
    precedence: int          // Writing-order position (lower = higher priority)
}

RespondToSearch {
    target_luid: LUID        // Who asked
    query: Query             // What was searched for
    result: FIR | NotFound   // The answer
}
```

### Communication Flow

```
Child Brane                    Parent Brane                   Grandparent
    |                              |                              |
    |--- FulfillSearch(q) -------->|                              |
    |                              |-- (can resolve locally?) --->|
    |                              |   YES: bind result           |
    |                              |   NO:                        |
    |                              |--- FulfillSearch(q) -------->|
    |                              |                              |
    |                              |<-- RespondToSearch(q,r) -----|
    |<-- RespondToSearch(q,r) -----|                              |
```

### Address Model

Each brane has:
- **Parent address:** The brane that contains this brane. Root brane has no parent.
- **Child addresses:** The sub-branes appended during PRE_EMBRYONIC step 7.

These addresses are established during the PRE_EMBRYONIC → EMBRYONIC transition and remain stable
for the brane's lifetime.

---

## LUID: Locally Unique Identifiers

Each expression within a brane receives a LUID (Locally Unique Identifier):

```
LUID = statement_number : expression_disambiguator
```

- **statement_number**: 0-based index of the statement in the brane's statement array.
- **expression_disambiguator**: A unique number distinguishing this expression from other
  expressions on the same statement (e.g., in `a = b + c`, the expressions `b`, `c`, and `b + c`
  each get distinct disambiguators).

LUIDs are unique within a brane. They are used in the message-passing protocol to route search
responses back to the requesting expression.

---

## Search Resolution and Precedence

### Intra-Brane Resolution (PRE_EMBRYONIC Step 6)

During PRE_EMBRYONIC, the brane resolves what it can locally:

1. Walk searches in writing order
2. For each search, check the brane's named-statement linked list
3. If found: bind the search to its target
4. If not found: add to unresolved set for parent communication

### Writing-Order Precedence

When multiple searches exist, they are processed in the order they were written:
**first-to-write-first-to-find**. This establishes a deterministic precedence that the programmer
controls by the order of their code.

### Inter-Brane Resolution (EMBRYONIC)

Unresolved searches are sent to the parent brane as FulfillSearch messages. The parent attempts
resolution in its own scope (its named statements, then its own parent). Results flow back as
RespondToSearch messages.

---

## NK vs CONSTANIC: When Is a Search Truly Failed?

This is one of the most important semantic decisions in the UBC design. UBC1 struggled with this
distinction across many iterations (the commit history shows clarification attempts as late as
February 2026). UBC2 establishes a clear rule.

### The Rule

> **A search result is NK (`???`) only if the UBC can prove that the search will fail in ALL
> possible future executions — including when CONSTANIC expressions are discovered by search and
> coordinated elsewhere. Otherwise, the result is CONSTANIC.**

In other words: NK means "definitively not findable." CONSTANIC means "not found yet, but might
be found in a future context."

### Why This Matters: Brane Concatenation Changes Everything

In UBC1, a plain identifier on the RHS (e.g., `x` in `{a = x;}`) was treated as an intra-brane
search. If `x` was not found in the brane or its parents, the result was NK (`???`). But this
was wrong in the presence of brane concatenation.

Consider:

```foolish
{
    f = {a = x;};
    g = {x = 42;};
    h = g f;        !! Concatenation prepends g's contents to f
}
```

When `f` is first evaluated, `x` is not found — but when `f` is later concatenated with `g`, the
search for `x` succeeds. The permitted Foolish language operation of concatenation means that a
brane's contents can be extended. Therefore, a search that fails locally **could succeed later**
when the brane is concatenated with another brane that provides the missing binding.

### The Change from UBC1

This reverses the previous UBC1 decision that made certain intra-brane searches produce NK. For
example, `{#-1}` (seek the previous line) was NK in UBC1 when there was no previous line. In
UBC2, this is CONSTANIC — because a future concatenation could prepend content that provides a
previous line.

**UBC1 behavior (superseded):**
```foolish
{
    r = {#-1};   !! r = ???  (NK — no previous line)
}
```

**UBC2 behavior (correct):**
```foolish
{
    r = {#-1};   !! r is CONSTANIC (might gain value via concatenation)
}
```

### NK Is Reserved for Provably Unfindable Cases

NK (`???`) should only be produced when the UBC can **prove** the search will never succeed:

- **Anchored search on a CONSTANT brane that doesn't contain the target**: The brane is fully
  evaluated and immutable. It will never gain new members. NK is correct.
- **Explicit `???` in source code**: The programmer wrote NK deliberately.
- **Arithmetic errors**: Division by zero, type mismatches, etc.
- **Depth limit exceeded**: The brane tree is too deep to evaluate.

### Tests That May Need Updating

The following UBC1 approval tests may produce NK where UBC2 should produce CONSTANIC:

- Any test where an intra-brane identifier search fails and produces `???`
- Any test where `{#-N}` produces `???` due to insufficient preceding lines
- Any test where unanchored seek (`↑`, `←`) produces `???` in a brane that could be concatenated

These tests should be audited during UBC2 implementation. The specific test files in
`foolish-core-java/src/test/resources/org/foolish/fvm/inputs/` and their corresponding
`.approved.foo` outputs will need review.

---

## Sequencing: Human-Readable Output

The UBC2 sequencer (Sequencer4Human) renders the final state of a brane tree as human-readable
text. This is the primary way a Foolisher inspects program results, and the format used by
approval tests. UBC2 updates the rendering to distinguish all constanic states clearly.

### Output Symbols

| Symbol | State | Meaning |
|--------|-------|---------|
| `???` | NK | **Not Known.** The UBC has proven this search will fail in all possible future contexts. Definitively unfindable. |
| `?` | CONSTANIC | **Constanic.** Search was performed and nothing was found, but the result might change in a future context (e.g., via brane concatenation). |
| `??` | WOCONSTANIC | **Waiting On Constanics.** All searches were found, but one or more results are themselves CONSTANIC or WOCONSTANIC. Structurally complete, waiting on dependencies. |
| *(value)* | CONSTANT | The fully evaluated result is shown directly. |
| *(expanded)* | INDEPENDENT | Shown as value. Indistinguishable from CONSTANT in output. |

### Rendering Rules

#### Literal Values (INDEPENDENT / CONSTANT)

Rendered as their value:

```
＿x = 42;
＿greeting = "hello";
```

#### NK Values

Rendered as `???`, optionally with a comment explaining why:

```
＿empty_head = ???;
＿div_zero = ???;              !! Division by zero
```

#### CONSTANIC Expressions

Rendered as `?` — a single question mark indicates "not found yet, might be found later":

```
＿x = ?;                      !! 'x' searched, not found in any context
```

#### WOCONSTANIC Expressions

Rendered as `??` — a double question mark indicates "found everything, waiting on dependencies":

```
＿sum = ??;                   !! Found a and b, but they are constanic
```

#### CONSTANIC and WOCONSTANIC Branes (Expanded)

When a brane is CONSTANIC or WOCONSTANIC, its contents are expanded to show internal state.
This lets the reader see *what* is resolved and *what* is stuck:

```
＿f = {
＿＿a = 42;                   !! CONSTANT — fully resolved
＿＿b = ?;                    !! CONSTANIC — 'b' not found
＿＿c = ??;                   !! WOCONSTANIC — found, waiting on dependency
＿＿result = ??;              !! WOCONSTANIC — depends on b and c
＿};
```

#### WOCONSTANIC Searches

When a search operation itself is WOCONSTANIC (e.g., `A$` when `A` is WOCONSTANIC), it renders
as `??`:

```
＿A = {
＿＿x = ?;
＿＿y = 10;
＿};
＿tail = ??;                  !! A$ — A is WOCONSTANIC, so tail is WOCONSTANIC
```

### Indentation

UBC2 continues UBC1's indentation system using the full-width underscore `＿` (U+FF3F) as the
default tab character. Each nesting level adds one `＿`:

```
{                             !! depth 0
＿x = 42;                    !! depth 1
＿inner = {                  !! depth 1
＿＿y = 100;                 !! depth 2
＿＿nested = {               !! depth 2
＿＿＿z = 200;               !! depth 3
＿＿};
＿};
}
```

### Sequencing Format

The full output for an approval test follows this structure:

```
!!INPUT!!
{input Foolish code}

!!!
PARSED AST:
{formatted AST}

UBC EVALUATION:
Steps taken: [count]

FINAL RESULT:
{sequenced output with ?, ??, ??? symbols}

COMPLETION STATUS:
Complete: [true/false]
!!!
```

### Complete Example

Input:
```foolish
{
    a = 1;
    b = {x = unknown_var; y = a;};
    c = b$;
}
```

UBC2 output:
```
{
＿a = 1;
＿b = {
＿＿x = ?;                   !! CONSTANIC — unknown_var not found
＿＿y = 1;                   !! CONSTANT — 'a' resolved to 1
＿};
＿c = ??;                    !! WOCONSTANIC — b$ found b, but b is not CONSTANT
}
```

---

## FIR Type Hierarchy

UBC2's "every expression is a brane" principle leads to a shallow type hierarchy. Instead of UBC1's
deep hierarchy with many specialized FIR types (BinaryFiroe, UnaryFiroe, IfFiroe, AssignmentFiroe,
etc.), UBC2 has a small number of types with behavior differences expressed through the lifecycle
steps they perform.

### Proposed Hierarchy

```
FIR (abstract base — state machine, depth, LUID, parent reference)
 ├── IndependentFir     — Literal values. No lifecycle. Always INDEPENDENT.
 └── ProtoBrane         — Everything else. One lifecycle.
      │
      │  Behavior varies by which steps are active/inactive:
      │
      ├── (boundary: yes)   — Curly-brace branes {…}. Full lifecycle.
      │                       Defines search scope. .value() returns brane.
      │
      ├── (boundary: no)    — Binary ops, unary ops, searches, assignments.
      │                       No search scope boundary. .value() returns literal.
      │                       Searches start in parent brane.
      │
      ├── (detachment: yes) — Liberation branes [...]. Modifies search behavior.
      │                       Added after concatenation is designed.
      │
      └── (sf/sff: yes)     — SF/SFF-marked expressions. Modified lifecycle
                              (SF: no child forwarding; SFF: skip to CONSTANIC).
```

The key insight is that these are **not separate classes** — they are a single `ProtoBrane` class
with flags or traits that enable/disable specific steps in the lifecycle:

| Trait | Effect on Lifecycle |
|-------|-------------------|
| `hasBoundary` | If true: searches within this brane check local statements first. If false: searches start in parent. |
| `isDetachment` | If true: modifies search results passing through this brane (blocks/unblocks identifiers). |
| `sfMode` | NONE: normal. SF: resolve own symbols only, no child forwarding. SFF: skip straight to CONSTANIC. |

This is **additive/subtractive**: the default ProtoBrane does the full lifecycle. Traits subtract
steps (SFF subtracts all of EMBRYONIC) or add behavior (detachment adds search filtering).

### Why Not Deeper Hierarchy?

UBC1's deep hierarchy (15+ FIR types each with their own `step()`) led to:
- `instanceof` chains in search unwrapping (10-15 cases)
- Inconsistent state transitions across types
- New types silently breaking existing code
- Each type reimplementing the same lifecycle with minor variations

UBC2 avoids this by having one `step()` implementation that consults traits, not types.

---

## Design TODO

The following features are required for UBC2 but not yet fully specified in this document. They
are listed in priority order.

### 1. Brane Concatenation

**Priority: Highest.** Brane concatenation is a core Foolish language operation and affects search
semantics fundamentally (see [NK vs CONSTANIC](#nk-vs-constanic-when-is-a-search-truly-failed)).

Design must specify:
- How concatenation interacts with PRE_EMBRYONIC (does the concatenated brane re-do steps 1-7?)
- How concatenation interacts with EMBRYONIC (search messages from a brane that was just extended)
- Concatenation of CONSTANT + CONSTANIC branes
- Writing-order precedence in concatenated branes (which brane's statements come first?)
- The three-level precedence rules from NAMES_SEARCHES_N_BOUNDS.md: left-associate liberations,
  liberation right-associates with branes, brane free association

### 2. Detachment (Liberation Branes)

**Priority: Highest (after concatenation).** Detachment semantics depend on how concatenation
works, because detachment branes left-associate with each other and right-associate with branes
before brane concatenation happens.

Design must specify:
- How detachment branes interact with ProtoBrane's lifecycle
- Detachment as a trait on ProtoBrane (the `isDetachment` flag) rather than a separate class
- How the search filter chain works: `[a][b][+a]` left-associates, then result right-associates
  with the following brane
- Free variable semantics (not permanent blocking — UBC1's DetachmentBraneFiroe got this wrong)
- Forward search liberation (`[~pattern]`, `[#N]`)
- P-branes (`[+...]`) for selective undetachment
- Detachment removal: when a brane is concatenated, the liberation is removed before
  identification, ordination, and coordination

### 3. Search-Based Path Selection (Replacing If-Then-Else)

**Priority: High.** UBC2 does not implement `if-then-else`. Control flow is expressed through
search operations that select paths based on pattern matching and value conditions. The specific
mechanism is to be designed. UBC1's IfFiroe had infinite-loop bugs and a fragile recursive
`step()` design that is not carried forward.

---

## Open Issues

The following issues from the UBC1 review remain relevant to UBC2. They are organized by priority,
updated to reflect decisions already made in this document. Items 1–7 from the original review
have been addressed (NK vs CONSTANIC clarified, IfFiroe removed, concatenation and detachment
added to Design TODO, CMFir superseded, state machine redesigned, search churn acknowledged in
Lessons Learned).

### High Priority — Affects UBC2 Design

**8. Search Operators: Forward Anchored Search (`B/pattern`) Not Implemented**

UBC1 never completed forward anchored search. `RegexpSearchFiroe` handles backward local, forward
local, and global, but the grammar support and integration for `B/pattern` (forward from start of
brane B) is incomplete. UBC2 needs this for search-based path selection (the replacement for
if-then-else). Additionally, `Query.java` auto-anchors patterns with `^...$` which breaks patterns
that already contain anchors — this logic needs fixing.

*Priority: High. Required for search-based path selection.*

**9. SF and SFF Markers Not Implemented in Code**

`FIR.java` stubs SFF with `NKFiroe("SFF marker (<<==>> syntax) not yet implemented")`. The UBC2
design now fully specifies SF/SFF lifecycle behavior (SF: resolve own symbols, no child forwarding;
SFF: PRE_EMBRYONIC → CONSTANIC). This is a fresh implementation — no UBC1 code to salvage.

*Priority: High. SF/SFF are central to the constanic cloning and coordination mechanism.*

**10. Identifier Resolution via Message Passing**

UBC1's `IdentifierFiroe.isAbstract()` returns `true` and `getValue()` throws
`UnsupportedOperationException`. Under UBC2, identifier resolution goes through FulfillSearch /
RespondToSearch messages. The ProtoBrane's EMBRYONIC stage handles this uniformly. This is a
clean reimplementation.

*Priority: High. Core to the message-passing architecture.*

**11. Recursive Search / `↑` Operator Semantics Incomplete**

The `↑` operator materializes one step of recursion by substituting the AST of the expression
that created the current brane. UBC1 has `SearchUpFiroe` but no cycle detection and incomplete
interaction with brane concatenation (`UnanchoredSeekFiroe` has a TODO for this). UBC2's depth-
based message sanity check helps, but the semantics of `↑` in the presence of concatenation and
detachment need explicit specification.

*Priority: High. Recursion is fundamental to Foolish.*

### Medium Priority — Implementation Quality

**12. Exception Swallowing in Arithmetic**

UBC1's `BinaryFiroe` catches all `Exception` and wraps in NKFiroe, hiding programming bugs. UBC2
should only catch `ArithmeticException` (division by zero, overflow) and produce NK with a
descriptive comment. Other exceptions should propagate as alarms.

*Priority: Medium. Code quality; addressed by Lessons Learned item 3.*

**13. Parent Chain Integrity**

UBC1 had no cycle detection on parent FIR chains. The depth-based message sanity check in UBC2
addresses the symptom (circular messages), but the ProtoBrane should also assert that
`setParent()` does not create a cycle. A simple depth comparison (new parent must be shallower)
is sufficient.

*Priority: Medium. Defensive check; partially addressed by circular message sanity check.*

**14. Depth Limiting Should Be Communicated**

UBC1's `EXPRMNT_MAX_BRANE_DEPTH = 96_485` silently makes a brane CONSTANT with `???` when the
limit is hit. In UBC2, depth exhaustion should produce an alarm through the message channel so
the parent brane is aware evaluation was truncated. The depth limit value should also be
configurable rather than hardcoded.

*Priority: Medium. Observability improvement.*

**15. Approval Tests Were Force-Approved**

Git commit `0032e47`: "Temporarily approve some of these." Some `.approved.foo` baselines may be
incorrect. These must be audited before UBC2 uses them as reference. The new `?` / `??` / `???`
output symbols also mean all existing approval tests will need regeneration.

*Priority: Medium. Correctness of test baselines.*

### Lower Priority — Cleanup and Future Features

**16. Old Detachment Tests Test Wrong Semantics**

Five test files from UBC1 (`detachmentBlockingIsApproved.foo`, `detachmentChainingIsApproved.foo`,
`detachmentFilterChainIsApproved.foo`, `detachmentNonDistributionIsApproved.foo`,
`pbraneSelectiveBindingIsApproved.foo`) test permanent-blocking semantics. The correct semantics
are free-variable liberation. These tests should be removed or rewritten when detachment is
implemented in UBC2.

*Priority: Lower. Blocked by detachment design (Design TODO item 2).*

**17. Missing Test: Partial Parameter Specification**

A test is needed where `b?x` returns CONSTANT 10 even though `b` is WOCONSTANIC — demonstrating
that anchored search can succeed on partially-evaluated branes. This validates the CONSTANIC vs
WOCONSTANIC distinction in practice.

*Priority: Lower. Validates the state split.*

**18. Scala Cross-Validation Lagging**

UBC2 should be fully stable in Java before Scala work begins (Lessons Learned item 8). The
cross-validation module (`foolish-crossvalidation`) enforces byte-identical output, so any change
to output symbols (`?`, `??`, `???`) cascades to Scala. Defer Scala until Java UBC2 passes all
approval tests.

*Priority: Lower. Sequencing concern.*

**19. Sequencer Rendering Updates**

The Sequencer4Human must be updated for:
- `?` for CONSTANIC (was `⎵⎵`)
- `??` for WOCONSTANIC (new state, no prior rendering)
- `???` for NK (unchanged)
- Expanded rendering of CONSTANIC/WOCONSTANIC branes (show internal state)
- Remove or update `(CONSTANIC)` state suffix display (the symbol itself now carries the meaning)

*Priority: Lower. Implementation task after design is stable.*

**20. Depth Search Not Designed**

`NAMES_SEARCHES_N_BOUNDS.md` mentions extending search "into the computation graph" but provides
no design. This is a future language feature, not required for UBC2 initial implementation.

*Priority: Future. Not blocking.*

**21. Corecursion / Coordinated State Updates**

`ADVANCED_FEATURES.md` asks: "How do we centrally organize several coordinated state updates in a
single line of code?" Foolish is entirely functional so far; coroutine-like patterns are not yet
designed. UBC2's message-passing architecture may provide a natural foundation for this.

*Priority: Future. Research topic.*

**22. Future Language Features**

`000-TODO_FEATURES.md` lists: mutable branes, characterization system, loops/iterators,
traceability, program generation/differentiation, native AI facilities, async execution, library
ecosystem integration. None have designs. UBC2's ProtoBrane architecture should not preclude these
but they are not in scope for initial implementation.

*Priority: Future. Architecture should remain extensible.*

---

## Relationship to UBC1

The UBC2 design is an evolution of UBC1, not a replacement of its semantics:

| UBC1 | UBC2 | Notes |
|------|------|-------|
| Many FIR types with separate step() | Every expression is a brane; one lifecycle | Unified model; literals are INDEPENDENT |
| BinaryFiroe, UnaryFiroe, IfFiroe, etc. | ProtoBrane with traits | Shallow hierarchy; IfFiroe removed (search-based path selection) |
| UNINITIALIZED → INITIALIZED → CHECKED | PRE_EMBRYONIC (steps 0–7) | Explicit substeps replace implicit initialization |
| Implicit search resolution during evaluation | EMBRYONIC with message passing | Search resolution becomes visible communication |
| CMFir wrapping for re-evaluation | Constanic cloning with slot replacement | Clone replaces slot; re-enters EMBRYONIC |
| EVALUATING | BRANING | Explicit child-brane stepping with communication |
| CONSTANIC / CONSTANT | CONSTANIC / WOCONSTANIC / CONSTANT / INDEPENDENT | CONSTANIC split: unfound vs waiting-on |
| `BraneMemory.get()` parent chain traversal | FulfillSearch / RespondToSearch messages | Same semantics, different mechanism |
| `braneMind` as work queue | `braneMind` as work queue + message buffer | Extended to carry communication state |

The Nyes state values are extended to represent the new stages (PRE_EMBRYONIC, EMBRYONIC, BRANING),
the split of CONSTANIC into CONSTANIC/WOCONSTANIC, and the INDEPENDENT terminal state. The
constanic predicate `is_constanic` encompasses CONSTANIC, WOCONSTANIC, CONSTANT, and INDEPENDENT.

---

## Lessons Learned from UBC1

The UBC2 state machine and architecture are informed by specific problems encountered during 12
weeks of UBC1 development (November 2025 – February 2026). These lessons are documented here so
that UBC2 implementation does not repeat them.

### 1. Don't Use Ordinal Comparisons for State Transitions

UBC1's `FiroeWithBraneMind.step()` used `nyes.ordinal() >= target.ordinal()` to check state
progress. This made the state machine fragile — reordering enum values broke the entire system.
UBC2 must use explicit named-state checks (`at_embryonic()`, `at_braning()`, etc.), never ordinal
arithmetic.

### 2. Don't Let FIR Types Implement Their Own `step()`

UBC1 had 15+ FIR types each with a custom `step()` method. This led to inconsistent state
transitions, forgotten state checks, and `instanceof` chains in code that needed to handle all
types uniformly (search unwrapping had 10-15 cases). UBC2's ProtoBrane has one `step()` with
trait-based behavior selection.

### 3. Don't Swallow Exceptions as NK Values

UBC1's BinaryFiroe caught all `Exception` and wrapped them in NKFiroe. This hid programming bugs
(NullPointerException, ClassCastException) behind "errors as values." UBC2 should only catch
domain-specific errors (ArithmeticException for division by zero) and let programming errors
propagate.

### 4. Don't Use Reflection to Access Private Fields

UBC1's ConcatenationFiroe used Java reflection to access `braneMemory` and `indexLookup` from
`FiroeWithBraneMind`. This indicates the abstraction was wrong. UBC2's ProtoBrane exposes what
concatenation needs through its public interface, not through reflection hacks.

### 5. Don't Rename Core Concepts During Implementation

UBC1 renamed "Constantic" → "Constanic" on day 63, and "Brain" → "Brane" on day 79. Late renames
cause confusion across code, documentation, commit messages, and agent instructions. UBC2's
terminology (PRE_EMBRYONIC, EMBRYONIC, BRANING, CONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT,
ProtoBrane) should be finalized before implementation begins.

### 6. Don't Wrap When You Can Clone-and-Replace

UBC1's CMFir was a wrapper that delegated state queries to an inner FIR. This broke transparency
(code checking `this.nyes` directly bypassed the delegation) and required complex two-phase
evaluation logic. CMFir was implemented, then reversed (with/without braneMind), and had critical
state-checking bugs (`isConstanic()` vs `atConstanic()`). UBC2's constanic cloning replaces the
slot directly — no wrapper, no delegation, no phase confusion.

### 7. Don't Force-Approve Tests

UBC1 had a commit "Temporarily approve some of these" where approval test baselines were accepted
without verification. This creates false confidence. UBC2 approval tests must only be approved
after manual review of the diff.

### 8. Stabilize in One Language First

UBC1 ported features to Scala while they were still unstable in Java, creating a two-front war
where cross-validation failures masked Java bugs. UBC2 should be fully stable in Java before any
Scala implementation begins.

### 9. Validate Parent Chain Integrity

UBC1 had no cycle detection on parent FIR chains. The constraint "parent FIR should not be
reassigned" was documented but not enforced. CMFir and ConcatenationFiroe both modified parent
references. UBC2's depth-based message sanity check (see
[Circular Message-Passing Sanity Check](#circular-message-passing-sanity-check)) catches this
class of bug structurally.

---

## Future Optimizations

These are noted in the design but deferred to later iterations:

1. **Direct forwarding.** Under certain conditions, a brane may ask its communication channel to
   directly forward messages from a child to its parent, bypassing its own processing. This is an
   optimization for cases where the brane knows it cannot resolve the search and would simply
   forward it anyway.

2. **Parallel child stepping.** In BRANING state, children that do not share search dependencies
   could be stepped in parallel.

3. **Search result caching.** Repeated identical searches from different children could share
   cached results.

---

## Last Updated

**Date**: 2026-02-16
**Updated By**: Claude Code v1.0.0 / claude-opus-4-6
**Changes**: Initial creation and iterative refinement in single session. Major sections: message-
passing architecture with circular sanity check; every-expression-is-a-brane unification; FIR
lifecycle stages (PRE_EMBRYONIC → EMBRYONIC → BRANING → CONSTANIC); EMBRYONIC wait-for mechanism;
constanic cloning (replacing CMFir); CONSTANIC/WOCONSTANIC split; SF/SFF markers; INDEPENDENT
state. Added NK vs CONSTANIC clarification: searches are CONSTANIC (not NK) when brane
concatenation could provide the missing binding — reverses UBC1 decision on `{#-1}`. Removed
if-then-else from UBC2 (search-based path selection instead). Added FIR type hierarchy proposal
(IndependentFir + ProtoBrane with traits). Added Design TODO (brane concatenation, detachment,
search-based path selection). Added Lessons Learned from UBC1 (9 items from code review and git
history analysis).
