# UBC2 Design Specification

The Unicellular Brane Computer Mark 2 (UBC2) is the reference implementation for establishing the
expected behavior of a brane computer. This document specifies its design.

For the engineering reference of the current UBC1 implementation, see
[UBC Engineering Reference](ubc_engineering.md).

---

## Table of Contents

- [What Is the UBC2](#what-is-the-ubc2)
- [Message-Passing Architecture](#message-passing-architecture)
- [Literal Values and Proto-Branes](#literal-values-and-proto-branes)
  - [Literal Values Are Always INDEPENDENT](#literal-values-are-always-independent)
  - [Proto-Branes: Boundary-Less Expressions](#proto-branes-boundary-less-expressions)
  - [The value() Method](#the-value-method)
  - [Operators as Syntactic Sugar](#operators-as-syntactic-sugar)
- [Branes: Full Boundary Expressions](#branes-full-boundary-expressions)
- [FIR Lifecycle Stages](#fir-lifecycle-stages)
  - [PRE_EMBRYONIC](#pre_embryonic)
  - [EMBRYONIC](#embryonic)
  - [BRANING](#braning)
  - [CONSTANIC Terminal States](#constanic-terminal-states)
- [SF and SFF Markers](#sf-and-sff-markers)
- [SearchFir: Search Resolution and Dereferencing](#searchfir-search-resolution-and-dereferencing)
- [Constanic Cloning and Re-Coordination](#constanic-cloning-and-re-coordination)
- [Brane Communication Protocol](#brane-communication-protocol)
- [LUID: Locally Unique Identifiers](#luid-locally-unique-identifiers)
- [Search Resolution and Precedence](#search-resolution-and-precedence)
- [NK vs CONSTANIC: When Is a Search Truly Failed?](#nk-vs-constanic-when-is-a-search-truly-failed)
- [Brane Concatenation](#brane-concatenation)
- [System Operators](#system-operators)
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

## Literal Values and Proto-Branes

### Literal Values Are Always INDEPENDENT

Foolish literal values (integers, strings) are always in the **INDEPENDENT** state. They do not
transition through any lifecycle stages. They have no searches to resolve, no dependencies to wait
for, and no parent to communicate with. They simply exist. Their `value()` method returns the
literal value itself.

### Proto-Branes: Boundary-Less Expressions

**Proto-branes** are expressions that participate in the full FIR lifecycle (PRE_EMBRYONIC →
EMBRYONIC → BRANING → CONSTANIC) but **do not define a search boundary**. This category includes:

- Binary operations: `a + b`, `x * y`
- Unary operations: `-x`, `!flag`
- Searches: `identifier`, `pattern$`, `{#-1}`
- Assignments (RHS expressions): `name = expression`

Proto-branes differ from full branes (curly-brace branes `{...}`) in two fundamental ways:

1. **No search boundary.** When a search starts inside a proto-brane, it begins in the **parent
   brane** where the proto-brane is located. There is no local scope to search first. Searches
   pass directly through to the containing brane.

2. **`.value()` returns a literal.** The result of evaluating a proto-brane is a scalar value
   (integer, string, boolean), not a brane. Proto-branes compute values; full branes compute
   branes.

Proto-branes are "everything we've described except for search boundary." They implement the
complete lifecycle but without establishing their own namespace.

### The value() Method

Every FIR in UBC2 has a `value()` method that returns its evaluated result:

| FIR Type | `value()` Returns | Notes |
|----------|------------------|-------|
| **Literal** (INDEPENDENT) | The literal value (int, string) | Immediate; no evaluation needed |
| **Proto-brane** (CONSTANIC) | Scalar value or search result | For integer literal proto-branes: returns `int`. For search proto-branes: returns the found FIR or CONSTANIC marker |
| **Brane** (CONSTANIC) | The brane itself | Full branes are first-class values |

The `value()` method is called by the VM when an expression reaches a terminal constanic state
and the result is needed for further computation.

### Operators as Syntactic Sugar

**All unary and binary operators are syntactic sugar for brane concatenations.**

The expression `1 + 2` is not a special "addition operation" — it is syntactic sugar for
concatenating branes:

```
1 + 2   →   {1}{2}{🧠+}
```

The curly braces `{...}` in this explanation indicate proto-branes, not Foolish syntax. The three
proto-branes concatenate due to proximity, producing:

```
{1, 2, 🧠+}
```

This is a single proto-brane containing three statements. The system operator `🧠+` references
library code that the VM/hardware implements. When this proto-brane reaches BRANING stage, the
`🧠+` operator accesses the two preceding values in its parent brane and computes their sum.

The `🧠` prefix marks system operators. When the VM encounters these symbols, their dedicated FIR
implementations are initialized. The actual FIR implementation for `🧠+` might be implemented
naively as:

```java
a, b = parent.getValuesBeforeMe(me);
myValue = a + b;
```

See [System Operators](#system-operators) for the complete list and their semantics.

### Characterization of Proto-Branes

For typing purposes, proto-branes can be characterized. For example, an addition operation might
be expressed as:

```foolish
add = $'{ <#-1> + <#-2> }
```

The characterization `$'` signals that this is a proto-brane whose result is a scalar value
extracted from its computation. This notation is for future reference; the UBC2 reference
implementation does not require explicit characterizations to handle proto-branes.

---

## Branes: Full Boundary Expressions

**Branes** (curly-brace branes `{...}`) are proto-branes that **define a search boundary**.
Everything about the proto-brane lifecycle applies, with these additions:

1. **Search boundary.** Searches that start inside a brane search the brane's named statements
   first before escalating to parent branes. The brane establishes a local namespace.

2. **`.value()` returns the brane.** A brane is a first-class value. Its `value()` method returns
   the brane itself, not a scalar.

Branes derive from proto-branes conceptually. The proto-brane lifecycle is the foundation; branes
add boundary semantics on top.

**Note:** UBC2 removes if-then-else (`IfFiroe`) entirely. Path selection in UBC2 is search-based,
not conditional-branching-based. The IfFiroe implementation in UBC1 had persistent infinite-loop
bugs and a fundamentally fragile recursive `step()` design. UBC2 replaces this with a mechanism
that fits the brane model natively. See [Design TODO](#design-todo) for the planned approach.

---

## FIR Lifecycle Stages

The significant reframing in UBC2 is what happens between Foolish code and CONSTANIC state. The
UBC2 follows a conservative implementation style that identifies four explicit stages of FIR
processing.

### PRE_EMBRYONIC

A FIR in PRE_EMBRYONIC stage holds a Foolish code AST. The UBC2 performs the following actions **as
a single atomic step**. The individual operations listed are not separate steps; they occur
together during the PRE_EMBRYONIC → EMBRYONIC transition:

**Action 1 — Count lines.**
Count the number of lines (statements) in the brane. **This is required because branes have finite,
known size.** The line count establishes the size of the statement array and is used for anchored
searches like `{#-1}` (previous line) and `{#N}` (line N).

**Action 2 — Establish statement array.**
Create the array of statements from the AST. Each statement occupies one slot, indexed from 0.

**Action 3 — Build search cache.**
Walk all *named* statements (assignments with LHS identifiers) and cache the fully characterized
identifier name for regex searching. Unnamed statements (those assigned to `???`) are skipped.
**The cache is just an array of strings** with fully-characterized names, enabling fast pattern
matching during search resolution.

Each `IdentifyingFiroe` caches its fully characterized identifier string so that searches can
match against it without re-traversing the characterization chain.

**Action 4 — Instantiate RHS FIRs.**
For each statement, instantiate the RHS expression's FIR **in PRE_EMBRYONIC stage** without taking
the PRE_EMBRYONIC step. This means the RHS FIRs are **readied** for the steps being defined here,
but they have not yet performed their own PRE_EMBRYONIC actions. They hold AST and await their
turn.

**Action 5 — Find all searches in RHS expressions.**
Walk the RHS expression FIRs and find all search operations, **stopping at brane boundaries**
(searches inside nested branes belong to those branes, not this one). Maintain the searches in the
order they were written to establish precedence: **first-to-write-first-to-find**.

**Note:** Brane-bounded search resolution occurs during the EMBRYONIC stage (see below), not during
PRE_EMBRYONIC. PRE_EMBRYONIC only identifies which searches exist; it does not attempt to resolve
them.

**Action 7 — Append child branes.**
Append PRE_EMBRYONIC branes found in RHS expressions to the braneMind, in writing order (the order
in which they appear in the source code). These child branes will be stepped during the BRANING
stage.

**Action 0 — Establish LUID.**
Each expression receives a Locally Unique Identifier. The LUID is composed of the statement number
followed by a disambiguator that distinguishes this expression from other expressions on the same
statement. The LUID is unique within the brane and is used for message routing.

**→ Transition to EMBRYONIC.**

**Important Note:** The PRE_EMBRYONIC actions occur as a single step. There is no intermediate
state between Actions 1-7. The proto-brane transitions atomically from holding an AST to being
ready for EMBRYONIC communication.

### EMBRYONIC

During the EMBRYONIC stage, the proto-brane actively resolves its searches through a combination of
local search, dereferencing, and message-passing communication with parent and child branes.

**IMPORTANT**: Identifiers on the RHS are **SearchFir** instances. There is no "variable
dereferencing" in the traditional sense. When you write `a = b`, the `b` is a SearchFir that
performs a search operation. All identifier resolution is search resolution.

#### Search Resolution Process

EMBRYONIC performs search resolution in steps:

1. **Call `resolve()` on each SearchFir** (up to search bandwidth limit per step)
2. **SearchFir.resolve() performs**:
   - Local brane-bounded search for the identifier
   - Dereferencing loop to chase through search chains
   - State determination based on what was found
3. **Add unresolved searches to wait-for queue** if targets are NYE (still stepping)
4. **Dispatch FulfillSearch messages** to parent for searches that need inter-brane resolution
5. **Receive and route messages** from parent and children

See [SearchFir: Search Resolution and Dereferencing](#searchfir-search-resolution-and-dereferencing)
for the complete SearchFir protocol.

#### Search Bandwidth and Dispatch

The proto-brane tracks **outstanding search requests** — searches that have been sent to parents
and await responses. Each EMBRYONIC step processes a **constant number of searches**. This constant
is the **search bandwidth** (e.g., 4 or 8 searches per step).

For each step:
1. Take up to `search_bandwidth` searches from the queue
2. Execute each search locally using the cursor/query system
3. For searches that succeed: bind the result
4. For searches that fail locally: dispatch a `FulfillSearch` message to the parent brane
5. Increment the **outstanding search count** for each dispatched search
6. Remove dispatched searches from the local tracking queue (the parent now owns them)

**The parent brane responds** with `RespondToSearch` messages. When a response arrives:
- Decrement the outstanding search count
- Apply the result to the requesting expression
- If the result is NYE (not-yet-evaluated), add to the wait-for queue
- If the result is CONSTANIC or WOCONSTANIC, trigger constanic cloning

#### Communication

- **Outbound to parent:** The proto-brane sends `FulfillSearch` messages for searches that cannot
  be resolved locally (up to `search_bandwidth` per step).
- **Inbound from children:** The proto-brane receives `FulfillSearch` messages from its child
  branes. It attempts to resolve them using its own cursor/query system. If it cannot, it forwards
  them to its own parent (subject to its own search bandwidth).
- **Inbound from parent:** The proto-brane receives `RespondToSearch` messages. It applies the
  results to its own unresolved searches and forwards results to children that requested them.

#### The braneMind Heap

The **braneMind** is a min-heap keyed by `<state, cur_rank, intra_statement_order, fir>`:

- **state**: The FIR's current lifecycle state (PRE_EMBRYONIC < EMBRYONIC < BRANING < CONSTANIC)
- **cur_rank**: Initially the statement number; increases by `statement_count` each time a FIR is
  stepped and needs to requeue. This implements round-robin stepping.
- **intra_statement_order**: Position within the statement (for expressions that share a statement)
- **fir**: The FIR reference

Heap sort is smallest-on-top. If the top of the heap is not in the current state, then it appears
the current state is complete in terms of stepping descendant FIRs.

#### Wait-For Queue Re-Checking

Every EMBRYONIC step, the proto-brane **re-checks the wait-for queue**. Searches in the wait-for
queue are waiting for their targets to finish stepping (NYE → constanic transition).

For each search in the wait-for queue:
1. Call `search.resolve()` again (resolve is idempotent)
2. If target changed state (e.g., NYE → CONSTANIC), search will detect and update its own state
3. Remove finalized searches (CONSTANIC, WOCONSTANIC, CONSTANT) from wait-for queue

**The dereferencing loop extends automatically** when targets change state, because `resolve()` is
called again and re-performs dereferencing based on current target states.

#### Transition Criterion

**→ Transition to BRANING** when:
- All SearchFirs have been resolved (reached terminal constanic states)
- Wait-for queue is empty
- All non-search expressions are resolved

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

### BRANING

In the BRANING state, the proto-brane continues to execute its child branes and handle
communication until all work is complete.

During BRANING:
- **Step child branes:** The proto-brane steps each child brane from the braneMind heap (see
  EMBRYONIC section for heap structure). Children may be in any state (PRE_EMBRYONIC, EMBRYONIC,
  BRANING, or constanic).
- **Continue communication:** The proto-brane continues to handle inbound and outbound
  communication:
  - Receives and forwards `FulfillSearch` messages from children to parent
  - Receives `RespondToSearch` messages from parent and routes them to children
  - Resolves searches locally when possible
- **Monitor wait-for queue:** Expressions waiting on constanic dependencies are periodically
  checked. When dependencies reach constanic states, constanic cloning is triggered.

#### Communication Bandwidth

The proto-brane processes a constant number of communication events per BRANING step, subject to
the same **search bandwidth** limit as EMBRYONIC. This prevents a single proto-brane from
monopolizing the message channel.

#### Transition Criterion

The BRANING state persists until **all work is complete**:
- All child branes have reached constanic states (CONSTANIC, WOCONSTANIC, CONSTANT, or INDEPENDENT)
- All expressions in the wait-for queue have resolved or reached constanic states
- No outstanding search requests remain
- The braneMind heap contains only constanic FIRs

**→ Transition to a constanic terminal state** (CONSTANIC, WOCONSTANIC, CONSTANT, or INDEPENDENT)
depending on the resolution status of the proto-brane's own searches and dependencies.

### CONSTANIC Terminal States

A FIR that is not stepping is **constanic** (lowercase). This is the generic term for any of the
terminal states:

| State | Meaning |
|-------|---------|
| **CONSTANIC** | Constant In Context. Search was performed and **nothing was found**. The expression has unresolved searches with no binding. May gain value when recoordinated into a new context that provides the missing bindings. |
| **WOCONSTANIC** | Waiting On CONSTANICs. Every search was found, but one or more of the found results are themselves CONSTANIC or WOCONSTANIC. The expression is waiting for its dependencies to settle. When those dependencies gain value (through recoordination), this expression can progress. |
| **CONSTANT** | Fully evaluated. All searches resolved to CONSTANT or INDEPENDENT values. Immutable. |
| **INDEPENDENT** | Promoted from CONSTANT. Detached from parent — although the parent may still hold a reference to it. Reserved for future development. |

#### The Constanic Predicate

The constanic predicate is defined as:

```
is_constanic = at_constanic || at_woconstanic || at_constant || at_independent
```

The English phrase "is constanic" has the same meaning. A FIR stops stepping when it reaches any
terminal state.

**Important Note:** Some WOCONSTANIC FIRs may have CONSTANIC children (or other constanic states
that are earlier in the ordinal ordering). This violates the strict ordinal hierarchy assumption
that "children's states are >= parent state." Therefore, **ordinal comparisons should still be
used for efficiency in checking constanic states**, but implementations must handle the edge case
where WOCONSTANIC parents have CONSTANIC (or CONSTANT) children.

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

## SearchFir: Search Resolution and Dereferencing

**SearchFir** is the FIR type that represents identifier searches on the RHS of assignments. When
you write `a = b`, the `b` is a SearchFir instance. SearchFir is **not** a proto-brane — it has no
independent lifecycle. Instead, it is driven by the proto-brane that contains it.

### SearchFir Structure

```java
class SearchFir extends FIR {
    Query query;           // Pattern, anchored/unanchored, scope
    FIR target;            // The found result (after dereferencing)
    Nyes state;            // CONSTANIC, WOCONSTANIC, CONSTANT (terminal states)
                           // Or NYE (still resolving)
}
```

### The resolve() Method

SearchFir has a single key method: `resolve()`. This method is called by the parent proto-brane
during its EMBRYONIC stage. The `resolve()` method performs:

1. **Search** for the identifier in the parent brane
2. **Dereferencing loop** to chase through search chains
3. **State determination** based on the final dereferenced result

#### Pseudocode for resolve()

```python
def resolve(parent_brane):
    # If already finalized, don't re-resolve
    if self.at_constanic() or self.at_woconstanic() or self.at_constant():
        return

    # Step 1: Perform search
    cur_res = parent_brane.perform_search(self.query)

    if cur_res is None:
        # Search failed - identifier doesn't exist
        self.state = CONSTANIC
        return

    # Step 2: Dereferencing loop
    # Chase through searches until we hit a barrier
    depth = 0
    while cur_res.is_search():
        depth += 1
        if depth > MAX_DEREFERENCE_DEPTH:
            raise CircularReferenceException()

        # Check state of the found search
        if cur_res.at_constanic():
            # STOP: CONSTANIC is a dereferencing barrier
            # Search failed in original context, won't change without re-coordination
            self.state = WOCONSTANIC
            self.target = cur_res
            return

        if cur_res.at_woconstanic():
            # STOP: WOCONSTANIC search hasn't performed its search yet
            # It's waiting for dependencies - it has no value to dereference
            self.state = WOCONSTANIC
            self.target = cur_res
            return

        if cur_res.at_constant():
            # STOP: Found fully resolved search
            # The search itself becomes CONSTANT - dereference to the final value
            self.state = CONSTANT
            self.target = cur_res  # Store reference to CONSTANT search
            return

        if cur_res.is_pre_constanic():  # NYE
            # STOP: Target still stepping, wait for it
            parent_brane.add_to_wait_for_queue(self)
            self.state = NYE
            return

    # Step 3: Found a non-search (brane)
    if cur_res.at_constanic() or cur_res.at_woconstanic():
        self.state = WOCONSTANIC
        self.target = cur_res
        return

    if cur_res.at_constant() or cur_res.at_independent():
        self.state = CONSTANT
        self.target = cur_res
        return

    if cur_res.is_pre_constanic():
        parent_brane.add_to_wait_for_queue(self)
        self.state = NYE
        return
```

### Key Dereferencing Rules

1. **WOCONSTANIC searches are barriers (NOT transparent)** - Stop dereferencing and become WOCONSTANIC
2. **CONSTANIC searches are barriers** - Stop dereferencing and become WOCONSTANIC
3. **CONSTANT searches are endpoints** - Stop dereferencing and become CONSTANT (search succeeded, has final value)
4. **Branes are opaque** - Don't dereference through branes (WOCONSTANIC or CONSTANIC)

**Why is WOCONSTANIC a barrier for searches?**

A WOCONSTANIC search has **not yet performed its search**. It's waiting for its dependencies to
resolve before it can search. The search has no value yet — it doesn't know what it will find.
Therefore, we cannot dereference through it. The original search must wait for the WOCONSTANIC
search to resolve.

**Why is CONSTANIC a barrier?**

CONSTANIC means "search failed in current context and will NEVER change" unless the FIR is
re-coordinated into a new context. When dereferencing encounters a CONSTANIC search, there's no
point waiting for it to "become CONSTANT" in the current context — that will never happen. The
search should stop and become WOCONSTANIC (waiting for re-coordination).

**Why are branes opaque?**

Branes are first-class values. A WOCONSTANIC or CONSTANIC brane has unresolved searches inside it,
but the brane itself is the value we found. We don't dereference "into" branes — we treat them as
endpoints.

### SearchFir Lifecycle Integration

SearchFir does not have its own independent state machine. It is driven by the proto-brane's Nyes
state transitions:

| Proto-Brane State | SearchFir Action |
|------------------|------------------|
| **PRE_EMBRYONIC** | SearchFir is instantiated (holds query pattern) but not resolved |
| **EMBRYONIC** | Proto-brane calls `searchFir.resolve(this)` - SearchFir performs search and dereferencing |
| **EMBRYONIC (wait-for)** | Proto-brane re-calls `searchFir.resolve(this)` when checking wait-for queue - SearchFir re-resolves if target changed state |
| **BRANING** | SearchFir is finalized (CONSTANIC, WOCONSTANIC, or CONSTANT) - receives messages about target state changes |
| **CONSTANIC** | SearchFir is in terminal state along with parent proto-brane |

### Message-Based State Change Notification

When a SearchFir becomes WOCONSTANIC (waiting on a WOCONSTANIC or CONSTANIC target), it needs to
be notified when the target's state changes. This is handled through the **messaging system**
(not callbacks).

#### Registration for State Change Messages

When `resolve()` determines that a SearchFir should become WOCONSTANIC:

1. SearchFir transitions to WOCONSTANIC
2. SearchFir stores reference to target
3. **SearchFir registers interest** in target's state changes by adding itself to the target's
   **state change listener list**
4. When target transitions to a new state, target sends **StateChange** messages to all registered
   listeners

#### StateChange Message Processing

State change messages are processed during the proto-brane's `step()` method. The proto-brane
processes messages at a **finite rate** (message bandwidth) to ensure fairness across all FIRs.

**Message format:**
```
StateChange {
    source_fir: FIR           // The FIR whose state changed
    old_state: Nyes           // Previous state
    new_state: Nyes           // Current state
}
```

**When a WOCONSTANIC SearchFir receives a StateChange message:**

1. Check if `source_fir` is the target this search is waiting on
2. If yes, re-call `resolve()` to re-evaluate dereferencing
3. Possible outcomes:
   - Target became CONSTANT → SearchFir may become CONSTANT (dereference to final value)
   - Target became CONSTANIC → SearchFir remains WOCONSTANIC (still waiting)
   - Target became WOCONSTANIC again → SearchFir remains WOCONSTANIC (still waiting)

**Message-based triggering ensures:**
- No direct callback mechanism (avoids tight coupling)
- Finite message processing rate (fairness, no monopolization)
- Messages delivered during `step()` (clear control flow)
- Proto-brane orchestrates all message handling (centralized)

### SearchFir Terminal States

A SearchFir reaches one of three terminal states after dereferencing:

| State | Meaning | Sequencer Output |
|-------|---------|-----------------|
| **CONSTANIC** | Search itself found nothing (identifier doesn't exist) | `?` |
| **WOCONSTANIC** | Dereferencing chain ended at CONSTANIC or WOCONSTANIC barrier | `??` |
| **CONSTANT** | Dereferencing chain ended at CONSTANT or INDEPENDENT result | *(value)* |

### Anchored Search into CONSTANIC or WOCONSTANIC Branes

When an anchored search targets a CONSTANIC or WOCONSTANIC brane, the search becomes WOCONSTANIC
(waiting for the brane to resolve):

```foolish
{
    b = {x = y;};  // b is CONSTANIC (y not found)
    c = b.x;       // Anchored search: b.x
}
```

**Resolution:**
1. `SearchFir(b.x).resolve()`: anchored search into brane `b`
2. Brane `b` is CONSTANIC (has unresolved searches)
3. **Can we search inside a CONSTANIC brane?** Yes! The brane's statement array exists.
4. Search for `x` within brane `b` → finds `x` at statement 0
5. `x` is bound to SearchFir(y), which is CONSTANIC
6. SearchFir(b.x) becomes WOCONSTANIC (found `x`, but `x` is CONSTANIC)

**Result:**
```
{
＿b = {＿＿x = ?;};
＿c = ??;       // WOCONSTANIC (found b.x, but b.x is CONSTANIC)
}
```

**If the anchor itself is WOCONSTANIC:**
```foolish
{
    b = {x = y; z = w;};  // b is WOCONSTANIC (waiting on y and w)
    c = b.x;              // Anchored search: b.x
}
```

**Resolution:**
1. `SearchFir(b.x).resolve()`: anchored search into brane `b`
2. Brane `b` is WOCONSTANIC (anchor is not fully resolved)
3. SearchFir(b.x) becomes WOCONSTANIC immediately (anchor is WOCONSTANIC)
4. **Cannot search inside WOCONSTANIC brane** — the brane hasn't finished stepping

**Result:**
```
{
＿b = ??;
＿c = ??;       // WOCONSTANIC (anchor b is WOCONSTANIC)
}
```

### Example: Search Chain

```foolish
{
    a = x;      // x not found
    b = a;      // search for 'a'
    c = b;      // search for 'b'
}
```

**PRE_EMBRYONIC:**
- Statement 0: `a = SearchFir(x)` (not resolved)
- Statement 1: `b = SearchFir(a)` (not resolved)
- Statement 2: `c = SearchFir(b)` (not resolved)

**EMBRYONIC Step 1:**
- `SearchFir(a).resolve()`: searches for 'x' → not found → CONSTANIC
- `SearchFir(b).resolve()`: searches for 'a' → finds SearchFir(a) (NYE) → wait-for queue
- `SearchFir(c).resolve()`: searches for 'b' → finds SearchFir(b) (NYE) → wait-for queue

**EMBRYONIC Step 2 (re-check wait-for queue):**
- `SearchFir(b).resolve()`: searches for 'a' → finds SearchFir(a) (CONSTANIC) → STOP (barrier) → WOCONSTANIC
- `SearchFir(c).resolve()`: searches for 'b' → finds SearchFir(b) (WOCONSTANIC) → STOP (barrier) → WOCONSTANIC

**Result:**
```
{
＿a = ?;        // CONSTANIC
＿b = ??;       // WOCONSTANIC (waiting on CONSTANIC 'a')
＿c = ??;       // WOCONSTANIC (waiting on WOCONSTANIC 'b')
}
```

**Note:** SearchFir(c) does **not** dereference through SearchFir(b) because WOCONSTANIC searches
are barriers (they haven't performed their search yet, so have no value to dereference).

### SearchFir value() Method

```java
public Object value() {
    if (at_constant()) {
        return target.value();  // Dereference to final value
    }
    if (at_woconstanic()) {
        return target;  // Return the FIR we're waiting on
    }
    if (at_constanic()) {
        return this;  // Return self as CONSTANIC marker
    }
    throw new IllegalStateException("Cannot get value of NYE SearchFir");
}
```

---

## Constanic Cloning and Re-Coordination

**Constanic cloning** is the definitive mechanism for re-coordinating proto-branes into new
contexts. When a CONSTANIC or WOCONSTANIC expression is referenced in a new context, it is cloned
and given a chance to resolve in that context. This replaces UBC1's CMFir wrapper approach.

### The Re-Coordination Problem

Consider:
```foolish
{
    fn = {a = x; b = a;};  // fn is WOCONSTANIC (x not found)
    x = 42;
    result = fn;           // How does fn gain value?
}
```

When `fn` is assigned to `result`, the WOCONSTANIC brane `fn` needs to be **re-coordinated** into
the new context where `x = 42` exists. The mechanism: **constanic cloning**.

### When Constanic Cloning Occurs

Constanic cloning is triggered when:
1. A CONSTANIC or WOCONSTANIC FIR is found during search resolution
2. The FIR needs to be relocated into a new parent brane
3. The clone will be re-evaluated in the new context

**Key timing**: Cloning occurs **after** the source FIR reaches a constanic state (CONSTANIC,
WOCONSTANIC, CONSTANT, or INDEPENDENT). The wait-for mechanism ensures we never try to clone NYE
FIRs.

### Parent Chain Update

**Critical**: When a FIR is constanic-cloned, the clone receives a **new parent reference**. This
ensures that:
- CONSTANIC searches re-resolve in the new context (search the new parent brane)
- FulfillSearch messages are sent to the new parent (not the original parent)
- The clone is fully integrated into the new brane's hierarchy

The parent chain update happens during the cloning process, before the clone begins stepping.

### Constanic Cloning State Transition Table

| Source State | Clone Action | Clone Initial State | Rationale |
|-------------|-------------|-------------------|-----------|
| **CONSTANT** | Reference (no copy) | CONSTANT | Immutable value, safe to share. No cloning needed. |
| **INDEPENDENT** | Reference (no copy) | INDEPENDENT | Immutable and detached, safe to share. No cloning needed. |
| **CONSTANIC** | Recursive clone | **EMBRYONIC** | Search failed in original context. Re-search in new context: local scope first, then beyond boundary. New parent may provide missing bindings. |
| **WOCONSTANIC** | Recursive clone | **BRANING** | Already found all symbols, waiting on dependencies. Don't re-search. Wait for cloned children to resolve, then re-evaluate dereferencing chain. Receives messages from children. |
| **NYE** | Exception | N/A | Constanic cloning must only be called on constanic FIRs. Wait-for mechanism ensures source is constanic before cloning. |

### Detailed Rationale

#### CONSTANT and INDEPENDENT: No Cloning

CONSTANT and INDEPENDENT FIRs are **immutable**. Once a FIR reaches CONSTANT, its value never
changes. INDEPENDENT FIRs are similarly immutable and detached from context. Both can be safely
shared across multiple contexts without cloning.

**Implementation**: The clone is simply a reference to the original FIR. No recursion, no copying.

#### CONSTANIC: Re-Search in New Context

A CONSTANIC FIR has unresolved searches. In the original context, the searches failed (identifiers
not found). When constanic-cloned into a new context, the clone gets a fresh chance to find those
identifiers.

**Why EMBRYONIC?**
- The clone needs to perform search resolution from scratch
- Searches are performed locally first (brane-bounded), then beyond boundary if needed
- The new parent brane may contain the missing identifiers
- EMBRYONIC is the standard stage for search resolution

**Cloning process:**
1. Clone the FIR structure recursively
2. Clone all sub-FIRs (children) using constanic cloning rules
3. Update parent reference to new parent brane
4. Transition clone to EMBRYONIC
5. Clone begins search resolution via `searchFir.resolve()`

**Example:**
```foolish
{
    fn = {a = x;};  // fn is CONSTANIC (x not found)
    x = 42;
    result = fn;    // Constanic-clone fn
}
```

Cloning process:
1. Clone brane `fn` → new instance
2. Clone SearchFir(x) → new instance, parent = new brane clone
3. New brane clone transitions to EMBRYONIC
4. SearchFir(x).resolve() searches in new context → finds x=42 → CONSTANT
5. New brane clone transitions to BRANING → CONSTANT

#### WOCONSTANIC: Wait for Children, Then Re-Dereference

A WOCONSTANIC FIR has **already resolved all its searches**. It found all the identifiers it was
looking for. It's just waiting for those identifiers to finish evaluating (they're CONSTANIC or
WOCONSTANIC themselves).

**Why BRANING?**
- The clone doesn't need to re-search (it already found everything)
- It needs to wait for its children (dependencies) to re-resolve in the new context
- When children change state, the clone re-evaluates its dereferencing chain
- BRANING is the state for "I'm done with my work, waiting for children"

**Cloning process:**
1. Clone the FIR structure recursively
2. Clone all sub-FIRs (children) using constanic cloning rules:
   - CONSTANT/INDEPENDENT children → reference (no copy)
   - CONSTANIC children → recursive clone, transition to **EMBRYONIC**
   - WOCONSTANIC children → recursive clone, transition to **BRANING**
3. Update parent reference to new parent brane
4. Transition clone to BRANING
5. Clone waits for messages from children indicating state changes
6. When children change state, clone re-calls `resolve()` to re-evaluate dereferencing

**Example:**
```foolish
{
    fn = {a = x; b = a;};  // a is CONSTANIC, b is WOCONSTANIC (waiting on a)
    x = 42;
    result = fn;           // Constanic-clone fn
}
```

Cloning process:
1. Clone brane `fn` → new instance
2. Clone SearchFir(a) → CONSTANIC, transitions to **EMBRYONIC**
3. Clone SearchFir(b) → WOCONSTANIC, transitions to **BRANING**
4. New brane clone transitions to BRANING (waiting for children)

Resolution in new context:
1. SearchFir(a).resolve() searches for 'x' → finds 42 → CONSTANT
2. SearchFir(a) sends message to SearchFir(b): "I resolved to CONSTANT"
3. SearchFir(b) receives message, re-calls `resolve()`
4. SearchFir(b).resolve() searches for 'a' → finds SearchFir(a) (CONSTANT) → dereferences → CONSTANT
5. All children CONSTANT → brane transitions to CONSTANT

### Special Case: SearchFir WOCONSTANIC Cloning

When a WOCONSTANIC SearchFir is constanic-cloned:

1. **Clone transitions to BRANING** (not EMBRYONIC)
2. **Clone's target is constanic-cloned** using the same rules
3. **Clone waits for target to re-resolve** in new context
4. **When target changes state**, clone re-calls `resolve()` to re-evaluate dereferencing

**Why this works:**
- WOCONSTANIC SearchFir already found the identifier (search succeeded)
- The identifier it found is CONSTANIC or WOCONSTANIC (needs re-coordination)
- When the target is cloned and transitions to EMBRYONIC or BRANING, it will re-resolve
- The SearchFir clone receives notification and re-dereferences
- The dereferencing chain extends automatically through the message-passing system

### Recursive Cloning

Constanic cloning is **recursive**. When a brane is constanic-cloned:

1. Clone the brane structure (statement array, named statement cache, etc.)
2. **Update parent reference** to new parent brane
3. For each statement in the brane:
   - Clone the LHS identifier (if named)
   - Clone the RHS expression using constanic cloning rules
   - Sub-expressions are constanic-cloned recursively
4. Determine clone's initial state (EMBRYONIC for CONSTANIC, BRANING for WOCONSTANIC)
5. Clone begins stepping in its initial state

**Important**: CONSTANT and INDEPENDENT sub-expressions are **not** recursively cloned — they are
referenced directly. Only CONSTANIC and WOCONSTANIC sub-expressions require cloning.

### Coordination After Cloning

After cloning, the clone participates in the brane's normal lifecycle:

**CONSTANIC clone (in EMBRYONIC):**
1. Performs search resolution via SearchFir.resolve()
2. Sends FulfillSearch messages to new parent for unresolved searches
3. Receives RespondToSearch messages with results
4. Transitions through EMBRYONIC → BRANING → constanic based on resolution results

**WOCONSTANIC clone (in BRANING):**
1. Waits for children to finish re-resolving
2. Receives messages from children when they transition to constanic states
3. Re-evaluates dereferencing chains when children change state
4. Transitions to CONSTANT when all dependencies resolve, or remains WOCONSTANIC if dependencies
   stay constanic

**The fundamental operation**: A constanic brane gains value when associated with a new context.
Constanic cloning makes this possible by giving the clone a fresh chance to resolve in the new
parent's context.

### Key Differences from UBC1

| Aspect | UBC1 (CMFir) | UBC2 (Constanic Clone) |
|--------|-------------|----------------------|
| Mechanism | Wrap FIR in CMFir, two-phase eval | Clone and replace in slot |
| When triggered | During evaluation | Only when object is constanic |
| Wrapper overhead | CMFir delegates state | No wrapper; direct replacement |
| Re-evaluation | CMFir Phase B steps clone | Clone re-enters EMBRYONIC or BRANING |
| CONSTANT handling | CMFir short-circuits | Reference to immutable; no clone |
| Parent chain update | Unclear/inconsistent | Explicit parent reference update during cloning |
| WOCONSTANIC handling | Not distinguished from CONSTANIC | BRANING state, wait for children |

---

## Brane Communication Protocol

The UBC2 uses a **message-passing architecture** where branes, proto-branes, and SearchFirs
communicate through discrete messages. Messages are processed during `step()` at a **finite rate**
(message bandwidth) to ensure fairness.

### Message Types

#### FulfillSearch

Sent from child to parent when a search cannot be resolved locally.

```
FulfillSearch {
    source_luid: LUID       // Who is asking (SearchFir's LUID)
    source_fir: FIR         // Reference to the SearchFir
    query: Query            // What to search for (pattern, anchored/unanchored, scope)
    precedence: int         // Writing-order position (lower = higher priority)
}
```

#### RespondToSearch

Sent from parent to child with search result.

```
RespondToSearch {
    target_luid: LUID        // Who asked (SearchFir's LUID)
    target_fir: FIR          // Reference to the requesting SearchFir
    query: Query             // What was searched for
    result: FIR | NotFound   // The answer (FIR reference or NotFound marker)
}
```

#### StateChange

Sent from a FIR to all registered listeners when its state changes.

```
StateChange {
    source_fir: FIR          // The FIR whose state changed
    old_state: Nyes          // Previous state (e.g., NYE, WOCONSTANIC)
    new_state: Nyes          // Current state (e.g., CONSTANT, CONSTANIC)
}
```

**When sent:**
- When any FIR transitions between Nyes states
- Particularly important for WOCONSTANIC → CONSTANT transitions
- Triggers re-resolution of dependent searches

**Who receives:**
- SearchFirs that registered interest in this FIR (added themselves to state change listener list)
- Parent proto-brane (if it has wait-for queue entries depending on this FIR)

### Message Processing

Proto-branes process messages during their `step()` method. Message processing is **bounded** by
**message bandwidth** — a constant limit on the number of messages processed per step.

**Message processing order:**

1. **Inbound messages from parent** (RespondToSearch, StateChange)
2. **Inbound messages from children** (FulfillSearch)
3. **Outbound messages to parent** (FulfillSearch for unresolved searches)
4. **Outbound messages to children** (RespondToSearch with results, StateChange notifications)

**Message bandwidth enforcement:**

```java
void step() {
    int messagesProcessed = 0;

    // Process inbound messages (up to MESSAGE_BANDWIDTH)
    while (messagesProcessed < MESSAGE_BANDWIDTH && !inboundQueue.isEmpty()) {
        Message msg = inboundQueue.dequeue();
        processMessage(msg);
        messagesProcessed++;
    }

    // ... other step logic (search resolution, etc.)
}
```

**Typical MESSAGE_BANDWIDTH**: 8-16 messages per step (configurable).

### Message Routing

#### FulfillSearch Routing

1. Child SearchFir calls `resolve()`, cannot resolve locally
2. Child adds FulfillSearch to parent's inbound message queue
3. Parent processes FulfillSearch during `step()`:
   - If parent can resolve: create RespondToSearch, add to child's inbound queue
   - If parent cannot resolve: forward FulfillSearch to grandparent

#### RespondToSearch Routing

1. Parent resolves search, creates RespondToSearch
2. Add RespondToSearch to child's inbound message queue (addressed by LUID)
3. Child processes RespondToSearch during `step()`:
   - Locate SearchFir by LUID
   - Update SearchFir.target with result
   - SearchFir may trigger state change (NYE → WOCONSTANIC or CONSTANT)

#### StateChange Routing

1. FIR transitions to new state (e.g., WOCONSTANIC → CONSTANT)
2. FIR sends StateChange to all registered listeners:
   - Each listener is a SearchFir that is WOCONSTANIC waiting on this FIR
   - StateChange added to listener's parent proto-brane's inbound queue
3. Listener's proto-brane processes StateChange during `step()`:
   - Re-call `searchFir.resolve()` to re-evaluate dereferencing
   - SearchFir may transition to new state based on updated target state

### Communication Flow Example

```
Child Brane                    Parent Brane                   Grandparent
    |                              |                              |
    |--- FulfillSearch(q) -------->|                              |
    |                              |                              |
    |                        (can resolve locally?)               |
    |                              |                              |
    |                         NO - forward:                       |
    |                              |--- FulfillSearch(q) -------->|
    |                              |                              |
    |                              |<-- RespondToSearch(q,r) -----|
    |                              |                              |
    |<-- RespondToSearch(q,r) -----|                              |
    |                              |                              |
(SearchFir receives result)        |                              |
    |                              |                              |
(If result is WOCONSTANIC, register for StateChange)              |
    |                              |                              |
(Later: result transitions WOCONSTANIC → CONSTANT)                |
    |                              |                              |
    |<-- StateChange --------------|                              |
    |                              |                              |
(SearchFir re-resolves, may become CONSTANT)                      |
```

### Address Model

Each proto-brane or brane has:

- **Parent address:** The brane that contains this brane. Root brane has no parent. Used for
  sending FulfillSearch messages upward.
- **Child addresses:** The sub-branes appended during PRE_EMBRYONIC. Used for sending
  RespondToSearch and StateChange messages downward.
- **LUID-based lookup table:** Maps LUID to FIR for routing messages to specific SearchFirs.

These addresses are established during the PRE_EMBRYONIC → EMBRYONIC transition and remain stable
for the brane's lifetime.

### Message Organization in Proto-Brane

Proto-branes track message-related state:

```java
class ProtoBrane {
    // Message queues
    Queue<Message> inboundQueue;     // Messages from parent and children
    Queue<Message> outboundQueue;    // Messages to send to parent/children

    // Search tracking
    Map<LUID, SearchFir> searchIndex;  // Lookup SearchFir by LUID
    Set<SearchFir> waitForQueue;       // Searches waiting on NYE targets

    // State change listeners
    Map<FIR, List<SearchFir>> stateChangeListeners;  // FIR → interested SearchFirs

    // Message bandwidth
    final int MESSAGE_BANDWIDTH = 8;
}
```

**Message triggers SearchFir functionality:**

- **FulfillSearch received:** Trigger local search, possibly forward to parent
- **RespondToSearch received:** Trigger SearchFir.resolve() to process result
- **StateChange received:** Trigger SearchFir.resolve() to re-evaluate dereferencing

The proto-brane organizes messages and triggers appropriate SearchFir methods, but does not
duplicate SearchFir logic.

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

## Brane Concatenation

**Brane concatenation** is a fundamental Foolish operation that combines multiple branes into a
single brane. Whether explicit `{...}{...}` or implicit `a b c` or mixed `{z+y}a{t+u}b{}{}c d` or
even grouped `(a$) (b$) ((c d e)$)$`, they are all wrapped inside a **ConcatenationBrane**.

Concatenation is **the mechanism** by which operators work: `a + b` desugars to `{a}{b}{🧠+}`,
which concatenates into a single proto-brane `{a, b, 🧠+}`.

**Note:** Grouping `()` must work before concatenation is fully implemented. See prioritization in
[Design TODO](#design-todo).

### ConcatenationBrane Structure

A ConcatenationBrane is a wrapping FIR that:
- Holds the branes being concatenated as **direct children**
- Does **not** pass search queries from children to the parent (search isolation)
- Executes its children's searches until they reach constanic states
- Merges children into a contiguous brane once all are ready

```java
class ConcatenationBrane extends ProtoBrane {
    List<FIR> children;           // The branes being concatenated
    Brane merged;                 // The merged result brane
    boolean searchIsolation;      // Don't forward FulfillSearch to parent
}
```

### Concatenation Lifecycle

#### PRE_EMBRYONIC

The ConcatenationBrane is instantiated with references to its child branes (the branes being
joined). Each child is in PRE_EMBRYONIC or CONSTANIC state.

**Actions:**
1. Store references to children
2. Set `searchIsolation = true` (do not forward messages to parent)
3. Transition to EMBRYONIC

#### EMBRYONIC

The ConcatenationBrane steps each child brane until all children are either:
- **PRE_EMBRYONIC or CONSTANIC** (not actively stepping), AND
- **BRANE-valued** (full branes, not proto-branes)

**Message handling:**
- **FulfillSearch from children:** Do **not** forward to parent. Search isolation is enforced.
  Children must resolve locally or reach CONSTANIC.
- **StateChange from children:** Monitor for transitions to constanic states
- **RespondToSearch:** Not sent to children (no forwarding from parent)

**Transition criterion:**
All children are (PRE_EMBRYONIC or constanic) AND all children are BRANE-valued → transition to
BRANING

#### BRANING

Once all child branes are available (PRE_EMBRYONIC or CONSTANIC) and BRANE-valued:

**Step 1: Constanic-copy statements**

For each child brane, constanic-copy its statements into a new contiguous statement array:
- CONSTANT and INDEPENDENT statements → reference (no copy)
- CONSTANIC and WOCONSTANIC statements → recursively clone
- **Update parent references** on cloned statements to point to merged brane

**Step 2: Create merged brane**

The constanic-copied statements become the merged brane's statement array. The merged brane is a
new first-class brane with:
- Statement count = sum of all children's statement counts
- Named statement cache = concatenated from all children (in order)
- Parent reference = ConcatenationBrane's parent

**Step 3: Merged brane executes PRE_EMBRYONIC**

The merged brane now executes the full Brane PRE_EMBRYONIC actions:
1. Count lines (already known from sum)
2. Build statement array (already built)
3. Build search cache (concatenate children's caches)
4. Instantiate RHS FIRs (already instantiated via constanic-copy)
5. Find searches (walk RHS, already done)
6. Append child branes (none for merged brane, it's flat)

Transition merged brane to EMBRYONIC.

**Step 4: ConcatenationBrane steps merged brane**

ConcatenationBrane transitions to BRANING (stepping child merged brane). The merged brane:
- Performs search resolution in its combined statement array
- Sends FulfillSearch to ConcatenationBrane's parent (search isolation ends after merge)
- Steps through EMBRYONIC → BRANING → constanic

**Transition criterion:**
Merged brane reaches constanic state → ConcatenationBrane reaches same constanic state

#### CONSTANIC Terminal State

When the merged brane reaches a constanic state (CONSTANIC, WOCONSTANIC, or CONSTANT), the
ConcatenationBrane reaches the same state. The ConcatenationBrane's `value()` returns the merged
brane.

### Search Isolation Enforcement

**Critical:** During EMBRYONIC, the ConcatenationBrane enforces **search isolation**:

```java
void processMessage(Message msg) {
    if (msg instanceof FulfillSearch) {
        // Do NOT forward to parent
        // Respond with NotFound or local resolution only
        FulfillSearch req = (FulfillSearch) msg;
        FIR result = this.performLocalSearch(req.query);
        if (result == null) {
            // Search failed - respond with NotFound
            sendToChild(new RespondToSearch(req.source_luid, req.query, NotFound));
        } else {
            // Found locally - respond with result
            sendToChild(new RespondToSearch(req.source_luid, req.query, result));
        }
    }
}
```

Children cannot search through the concatenation boundary into the parent. This ensures:
- Concatenation does not leak scope
- Child branes are isolated during the concatenation process
- After merging, the merged brane can search parent normally

### Writing-Order Precedence

When concatenating multiple branes, **writing order** determines statement order in the merged
brane:

```foolish
{a=1}{b=2}{c=3}
```

Merged statement array: `[a=1, b=2, c=3]` (left-to-right order)

Searches within the merged brane follow **backward search from cursor position**, so later
statements can reference earlier statements:

```foolish
{a=1}{b=a+1}{c=b+1}
```

Merged: `{a=1; b=a+1; c=b+1;}` → `c` finds `b`, `b` finds `a`

### Concatenation and Operators

Binary and unary operators desugar to brane concatenation:

```foolish
result = 1 + 2 * 3
```

Desugars to (assuming standard precedence):
```foolish
result = {1}{2}{3}{🧠*}{🧠+}
```

Concatenates to:
```foolish
result = {1, 2, 3, 🧠*, 🧠+}
```

The system operators `🧠*` and `🧠+` are proto-branes that access their parent's statement array
to retrieve operands (see [System Operators](#system-operators)).

### Constanic Cloning and Concatenation

When a ConcatenationBrane is constanic-cloned:

1. **Clone each child** using constanic cloning rules
2. **Create new ConcatenationBrane** with cloned children
3. **Re-merge** in the new context (children may have different states after cloning)
4. Merged brane re-resolves searches in new parent context

This allows concatenated branes to gain value through re-coordination.

### Implementation Priority

Concatenation depends on grouping `()` working correctly. The implementation priority is:

1. **Grouping:** Implement `()` for expression precedence
2. **Proto-branes and Branes:** Ensure the basic lifecycle works
3. **Concatenation:** Implement ConcatenationBrane after the above are stable

### Open Design Questions

1. **How does concatenation interact with liberation branes `[...]`?**
   - Does `[a]{x=a}{y=a}` liberate `a` in both branes before concatenation?
   - Per NAMES_SEARCHES_N_BOUNDS.md: liberation is removed before concatenation

2. **What if children are not all BRANE-valued?**
   - Can we concatenate proto-branes (scalars) with branes?
   - Probably not — concatenation requires BRANE-valued children
   - Proto-branes that are scalars cannot be concatenated

3. **Performance: when to trigger merging?**
   - Eager: merge as soon as all children are available
   - Lazy: merge only when the merged brane is referenced
   - Current design: eager merging during BRANING

---

## System Operators

System operators are built-in operations prefixed with the **🧠** symbol (U+1F9E0, "brain"). When
the VM encounters a system operator, it initializes a dedicated FIR that implements the operation.

### System Operator Table

| Operator | Symbol | Unicode | Operation | Naive Implementation |
|----------|--------|---------|-----------|---------------------|
| Add | 🧠+ | U+1F9E0 U+002B | Addition | `a, b = parent.getValuesBeforeMe(me, 2); myValue = a + b;` |
| Subtract | 🧠- | U+1F9E0 U+002D | Subtraction | `a, b = parent.getValuesBeforeMe(me, 2); myValue = a - b;` |
| Multiply | 🧠* | U+1F9E0 U+002A | Multiplication | `a, b = parent.getValuesBeforeMe(me, 2); myValue = a * b;` |
| Divide | 🧠/ | U+1F9E0 U+002F | Division | `a, b = parent.getValuesBeforeMe(me, 2); if (b == 0) throw ArithmeticException; myValue = a / b;` |
| Negate | 🧠− | U+1F9E0 U+2212 | Unary negation | `a = parent.getValueBeforeMe(me); myValue = -a;` |
| Not | 🧠! | U+1F9E0 U+0021 | Logical NOT | `a = parent.getValueBeforeMe(me); myValue = !a;` |

### How System Operators Work

When Foolish source code is parsed, binary and unary operators are **desugared** into system
operator proto-branes:

**Source:**
```foolish
result = 1 + 2
```

**Desugared (conceptual):**
```
result = {1}{2}{🧠+}  →  {1, 2, 🧠+}
```

The `🧠+` proto-brane concatenates with the literal proto-branes `1` and `2`, forming a single
proto-brane with three statements. When the `🧠+` reaches BRANING stage, it:

1. Calls `parent.getValuesBeforeMe(me, 2)` to retrieve the two preceding values (1 and 2)
2. Computes `1 + 2 = 3`
3. Sets `myValue = 3`
4. Transitions to CONSTANT

The `value()` method for the `🧠+` proto-brane returns `3` (an int).

### Adding System Operators to Symbol Table

The 🧠 symbol must be added to the symbol table documented in
[SYMBOL_TABLE.md](SYMBOL_TABLE.md). Each system operator requires:
- Symbol rendering (🧠+, 🧠-, etc.)
- Unicode codepoints
- HTML entities (for documentation)
- Reference to this section

### Implementation Notes

System operator FIRs are **proto-branes** (not full branes). They:
- Have no search boundary
- Return scalar values via `value()`
- Access parent brane's statement array to retrieve operands
- Follow the standard proto-brane lifecycle

The "naive implementation" examples above are conceptual. The actual FIR implementation would:
- Validate operand types
- Handle CONSTANIC operands (wait-for mechanism)
- Produce NK for arithmetic errors (division by zero, type mismatches)
- Use the standard proto-brane step-based evaluation

---

## Design TODO

The following features are required for UBC2 but not yet fully specified in this document. They
are listed in priority order.

### 1. Grouping and Operator Precedence

**Priority: Highest.** Parentheses `()` for grouping and operator precedence must work before brane
concatenation can be implemented. This establishes the parse tree structure that concatenation
operates on.

Design must specify:
- How `(a + b) * c` groups differently from `a + b * c`
- How grouping interacts with proto-brane boundaries
- Precedence rules for binary operators (+, -, *, /, etc.)
- How grouped expressions are represented in the AST

### 2. Brane Concatenation

**Priority: Highest (after grouping).** Brane concatenation is now fully specified in
[Brane Concatenation](#brane-concatenation). Implementation tasks remaining:

- Implement ConcatenationBrane FIR type
- Test concatenation of CONSTANT + CONSTANIC branes
- Test concatenation of proto-branes vs full branes
- Verify search isolation (children cannot search through concatenation boundary to parent)
- Test writing-order precedence in concatenated branes
- Implement the three-level precedence rules from NAMES_SEARCHES_N_BOUNDS.md: left-associate
  liberations, liberation right-associates with branes, brane free association

### 3. Detachment (Liberation Branes)

**Priority: High (after concatenation).** Detachment semantics depend on how concatenation
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

### 4. Search-Based Path Selection (Replacing If-Then-Else)

**Priority: High (after proto-branes are stable).** UBC2 does not implement `if-then-else`.
Control flow is expressed through search operations that select paths based on pattern matching
and value conditions. The specific mechanism is to be designed. UBC1's IfFiroe had infinite-loop
bugs and a fragile recursive `step()` design that is not carried forward.

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

### 10. Implement Finer-Grained Proto-Brane Steps

UBC1's step implementation was often too coarse, leading to non-deterministic behavior when
multiple FIRs tried to step simultaneously. UBC2's PRE_EMBRYONIC actions are atomic (single step)
but clearly documented. EMBRYONIC and BRANING have explicit bounded work per step (search
bandwidth, communication bandwidth).

### 11. Use World-Stopping Exceptions or Alarms for Errors

UBC1 often silently converted errors to NK values, hiding bugs. UBC2 should use **world-stopping
exceptions** for programming errors (NullPointerException, ClassCastException, etc.) and
**Alarms** for domain errors (division by zero, depth exceeded, etc.). Alarms propagate through
the message channel; exceptions halt evaluation and surface to the user immediately.

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

**Date**: 2026-02-20
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-5-20250929
**Changes**: **Microstates and messaging refinements.** Corrected dereferencing rule: WOCONSTANIC
searches are barriers (NOT transparent) — they haven't performed their search yet, have no value to
dereference. Added message-based state change notification (StateChange messages instead of
callbacks). Added anchored search into CONSTANIC/WOCONSTANIC branes specification. Completely
rewrote Brane Communication Protocol section with detailed message formats (FulfillSearch,
RespondToSearch, StateChange), message processing with bandwidth limits, message routing,
organization in proto-brane. Significantly expanded Brane Concatenation section with lifecycle
details, search isolation enforcement, writing-order precedence, constanic cloning interaction,
open design questions. Added terminology note: "Nyes" is proper noun (capitalized), "nyes" is
lowercase when referring to the nyes of a FIR. "Microstate" = any state finer than Nyes.
**Previous session:** Major design on search dereferencing and constanic cloning. SearchFir section,
CONSTANIC/WOCONSTANIC cloning state transitions, parent chain updates, operators as syntactic
sugar, Proto-Branes/Branes hierarchy, System Operators, Lessons Learned.
