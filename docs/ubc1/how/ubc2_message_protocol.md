# UBC2 Message Protocol Design

> AI agents: read [../DOC_AGENTS.md](../DOC_AGENTS.md) before editing this file.

Date: 2026-02-27
Context: D0-D2 Design (ProtoBrane Subtypes, Grouping/Precedence, Brane Concatenation)

This document specifies the message protocol that governs communication between
branes and proto-branes in UBC2. It covers the three message types, the two
parent-attachment linkages that enable messaging, the lifecycle phases where each
message type is active, and the fate of messages during constanic cloning.

For the broader FIR lifecycle, search resolution, and constanic cloning rules,
see [UBC2 Design Specification](ubc2_design.md). For how the communication adapter
abstracts the underlying medium, see [D0.6 Communication Medium](d0_6_communication_medium.md).
This document focuses on the mechanics of message exchange.

The message protocol described here is a design-forward specification. The
current UBC1 codebase resolves searches synchronously through direct
`BraneMemory` parent-chain traversal. UBC2 replaces that mechanism with
asynchronous message passing to achieve bounded step cost, fairness across
branes, and correct behavior under concurrent evaluation.

---

## Table of Contents

- [Design Principles](#design-principles)
- [Message Types](#message-types)
  - [FulfillSearch](#fulfillsearch)
  - [RespondToSearch](#respondtosearch)
  - [StateChange](#statechange)
- [FIR Creation and Parent Attachment](#fir-creation-and-parent-attachment)
  - [The Two Linkages](#the-two-linkages)
  - [Creation and Attachment Sequence](#creation-and-attachment-sequence)
  - [What Attachment Enables](#what-attachment-enables)
  - [Attachment Precedes First Step](#attachment-precedes-first-step)
- [Search Resolution via Messages (EMBRYONIC)](#search-resolution-via-messages-embryonic)
  - [SearchFir Dispatch](#searchfir-dispatch)
  - [Parent Processing](#parent-processing)
  - [Response Routing](#response-routing)
  - [Wait-For Queue and StateChange Registration](#wait-for-queue-and-statechange-registration)
  - [Transition to BRANING](#transition-to-braning)
- [BRANING Stage Communication](#braning-stage-communication)
- [Constanic Cloning: Lifecycle and Message Fate](#constanic-cloning-lifecycle-and-message-fate)
  - [When Cloning Triggers](#when-cloning-triggers)
  - [CONSTANIC Clone Sequence](#constanic-clone-sequence)
  - [WOCONSTANIC Clone Sequence](#woconstanic-clone-sequence)
  - [CONSTANT and INDEPENDENT: No Clone](#constant-and-independent-no-clone)
  - [Message Fate During Cloning](#message-fate-during-cloning)
  - [Listener Re-Registration After Cloning](#listener-re-registration-after-cloning)
  - [Worked Example: Constanic Clone in New Context](#worked-example-constanic-clone-in-new-context)
  - [Worked Example: WOCONSTANIC Clone with Cascading Resolution](#worked-example-woconstanic-clone-with-cascading-resolution)
- [Message Routing and Address Model](#message-routing-and-address-model)
  - [Address Structure](#address-structure)
  - [Depth-Based Sanity Check](#depth-based-sanity-check)
  - [Message Bandwidth and Fairness](#message-bandwidth-and-fairness)
- [Order of Precedence: Search and Concatenation](#order-of-precedence-search-and-concatenation)
- [Anchored vs Unanchored Search: NK and CONSTANIC Outcomes](#anchored-vs-unanchored-search-nk-and-constanic-outcomes)
- [Search Isolation (ConcatenationBrane)](#search-isolation-concatenationbrane)
- [Exception Cases](#exception-cases)
  - [Cycle Detection](#cycle-detection)
  - [Stalled WOCONSTANIC](#stalled-woconstanic)
- [Open Design Questions](#open-design-questions)
- [Appendix: State Name Mapping](#appendix-state-name-mapping)

---

## Design Principles

| Principle | Explanation |
|-----------|-------------|
| Asynchronous | Children send requests and await responses. No blocking waits. |
| Unidirectional by type | FulfillSearch flows upward (child to parent). RespondToSearch flows downward (parent to child). StateChange flows to registered listeners. |
| State-driven | The parent decides what to respond based on its own Nyes state, not the child's request parameters. |
| No shared state | No parent-chain walking, no field access across brane boundaries. All inter-brane information travels as messages. |
| Bounded step cost | Each `step()` processes at most MESSAGE_BANDWIDTH messages. No unbounded work per step. |
| Isolation by default | ConcatenationBrane blocks FulfillSearch from children during EMBRYONIC. Search isolation ends after merging. |

---

## Message Types

UBC2 uses three message types. The sealed interface:

```java
public sealed interface Message permits
    FulfillSearch, RespondToSearch, StateChange {}
```

### FulfillSearch

Direction: child to parent.

Sent when a SearchFir cannot resolve an identifier locally and needs the parent
brane to search on its behalf.

```java
record FulfillSearch(
    LUID source_luid,       // who is asking (SearchFir's LUID)
    FIR source_fir,         // reference to the SearchFir
    Query query,            // what to search for
    int precedence          // writing-order position (lower = higher priority)
) implements Message {}
```

When sent:
- During EMBRYONIC, after a SearchFir's local brane-bounded search fails.
- Subject to the parent proto-brane's search bandwidth limit per step.

### RespondToSearch

Direction: parent to child.

Sent in response to a FulfillSearch, carrying the search result back to the
requesting child.

```java
record RespondToSearch(
    LUID target_luid,        // who asked (SearchFir's LUID)
    FIR target_fir,          // reference to the requesting SearchFir
    Query query,             // what was searched for
    FIR result               // found FIR, or NotFound marker
) implements Message {}
```

Response semantics:

| Result | Meaning | SearchFir action |
|--------|---------|------------------|
| FIR at CONSTANT or INDEPENDENT | Found and fully resolved | SearchFir becomes CONSTANT |
| FIR at CONSTANIC | Found but search failed in that context | SearchFir becomes WOCONSTANIC, registers for StateChange |
| FIR at WOCONSTANIC | Found but waiting on dependencies | SearchFir dereferences through to the constanic target, becomes WOCONSTANIC |
| FIR pre-constanic (nigh) | Found but still stepping | SearchFir enters wait-for queue, re-checks later |
| NotFound | Search failed at all ancestor levels | SearchFir becomes CONSTANIC |

### StateChange

Direction: from any FIR to all registered listeners.

Sent when a FIR transitions between Nyes states. This is the mechanism by which
WOCONSTANIC SearchFirs learn that their targets have made progress.

```java
record StateChange(
    FIR source_fir,          // the FIR whose state changed
    Nyes old_state,          // previous state
    Nyes new_state           // current state
) implements Message {}
```

When sent:
- When any FIR transitions between Nyes states.
- Particularly important for constanic to CONSTANT transitions (dependency resolved).
- After constanic cloning, when cloned children re-resolve in new context.

Who receives:
- SearchFirs that registered interest in the source FIR (added themselves to its listener list).
- The parent proto-brane, if it has wait-for queue entries that depend on the source FIR.

---

## FIR Creation and Parent Attachment

A FIR must be attached to its parent before it can participate in message
exchange. Attachment establishes the routing infrastructure that FulfillSearch
and RespondToSearch depend on. This section traces the exact sequence.

### The Two Linkages

Attachment consists of two independent linkages, both required for a child to
function:

Linkage 1 — the FIR parent pointer. Set by `setParentFir(parent)`. This is a
structural pointer: the child knows who its parent is. It enables upward
tree-walking for depth calculation, error reporting, and `getMyBrane()`. In
UBC2, it is also the address to which FulfillSearch messages are sent.

Linkage 2 — the BraneMemory parent chain. Set by
`ordinateToParentBraneMind(parent, position)`. This links the child's
`braneMemory.parent` to the parent's `braneMemory`. When a search within the
child fails locally, `BraneMemory.get(query, fromLine)` follows this link to
search the parent's statements. In UBC2, this chain is replaced by message
passing, but the linkage still determines which brane a FulfillSearch message
addresses.

Both linkages are established together, in a single operation, before the child
ever steps.

### Creation and Attachment Sequence

The following sequence describes what happens when a brane executes its
PREMBRYONIC step and creates its children. The brane `B` contains statements
`{a = 1; b = x;}`.

```
PREMBRYONIC step of brane B
(all actions below are atomic — no intermediate state is observable)

  Action 0 — Establish LUID for B
      B receives a locally unique identifier within its parent.

  Action 1 — Count lines
      B has 2 statements: a=1 and b=x.

  Action 2 — Establish statement array
      Create the array with 2 slots.

  Action 3 — Build search cache
      Walk named statements. Cache "a" at position 0.
      "b" at position 1. (The RHS expressions are not cached.)

  Action 4 — Instantiate RHS FIRs
      For statement 0: create ValueFiroe(1). State: INDEPENDENT.
      For statement 1: create SearchFir("x"). State: PREMBRYONIC.

      At this instant, the children exist but have:
        parentFir = null
        braneMemory.parent = null
        No LUID assigned

  Action 4a — Attach children to B (storeFirs)
      For each child FIR c:
        i.   B.braneMemory.put(c)           — c is now in B's statement list
        ii.  c.setParentFir(B)              — Linkage 1 established
        iii. B.indexLookup.put(c, position) — LUID-equivalent assigned
        iv.  c.ordinateToParentBraneMind(B, position)
                                            — Linkage 2 established

      After this step, every child can:
        - Walk parentFir upward (for depth, error context)
        - Resolve identifiers through B's braneMemory
        - Send FulfillSearch to B (in UBC2)

  Action 5 — Find all searches in RHS expressions
      Walk child FIRs. SearchFir("x") is identified as an unresolved search.
      Added to the search dispatch queue, in writing order.

  Action 7 — Append child branes to braneMind
      Any sub-branes found in RHS expressions are appended to B's braneMind
      for stepping during BRANING.

  Transition: B moves from PREMBRYONIC to EMBRYONIC.
```

### What Attachment Enables

Before attachment (between Action 4 and Action 4a), a child FIR is an orphan.
It holds AST but cannot do anything useful.

After attachment, the child has:

| Capability | Enabled by | Used when |
|------------|-----------|-----------|
| Send FulfillSearch to parent | Linkage 1 (parentFir) | EMBRYONIC, when local search fails |
| Receive RespondToSearch from parent | Linkage 1 (parentFir routes response by LUID) | EMBRYONIC, when parent responds |
| Resolve identifiers through parent chain | Linkage 2 (braneMemory.parent) | EMBRYONIC search resolution |
| Walk parent chain for depth | Linkage 1 (parentFir) | Depth-based sanity check, depth limit |
| Appear in parent's statement array | braneMemory.put() | Siblings can find this child via search |

The separation of Linkage 1 and Linkage 2 matters during constanic cloning,
where they are established in two phases (see below).

### Attachment Precedes First Step

The child receives both linkages before it ever calls `step()`. From the
child's very first step, the parent chain is available for identifier
resolution. There is no "unattached" phase where a child exists and is
stepping but cannot reach its parent.

This invariant is critical: the EMBRYONIC stage assumes that FulfillSearch has
a valid destination from the first step.

---

## Search Resolution via Messages (EMBRYONIC)

During EMBRYONIC, the proto-brane resolves its SearchFirs through a combination
of local search and message-based communication with parent and child branes.

### SearchFir Dispatch

Each EMBRYONIC step:

1. Take up to `search_bandwidth` SearchFirs from the dispatch queue.
2. For each SearchFir, call `resolve()`:
   - Perform local brane-bounded search using the cursor/query system.
   - If found locally: bind the result. No message needed.
   - If not found locally: create a FulfillSearch message. Add it to the
     parent's inbound queue. Increment the outstanding search count.
3. Remove dispatched searches from the local queue. The parent now owns them.

### Parent Processing

When a parent proto-brane processes a FulfillSearch from its inbound queue
during `step()`:

1. Search the parent's own braneMemory for the query.
2. If found: create a RespondToSearch with the result. Add it to the child's
   inbound queue (addressed by `target_luid`).
3. If not found and parent has its own parent: forward the FulfillSearch upward
   (subject to search bandwidth). The forwarded message retains the original
   `source_luid` so the response can be routed back.
4. If not found and this is the root brane: create a RespondToSearch with
   NotFound. The search has failed at all levels.

### Response Routing

When a child proto-brane processes a RespondToSearch from its inbound queue:

1. Look up the SearchFir by `target_luid` in the LUID-based search index.
2. Apply the result to the SearchFir:
   - NotFound: SearchFir becomes CONSTANIC.
   - Found at CONSTANT/INDEPENDENT: SearchFir becomes CONSTANT.
   - Found at CONSTANIC: SearchFir becomes WOCONSTANIC. Register for
     StateChange on the target (see below).
   - Found at WOCONSTANIC: Dereference through to the constanic target.
     SearchFir becomes WOCONSTANIC. Register for StateChange.
   - Found pre-constanic (nigh): Add SearchFir to the wait-for queue.
3. Decrement the outstanding search count.

### Wait-For Queue and StateChange Registration

Two mechanisms handle FIRs that are not yet ready:

The wait-for queue holds SearchFirs whose targets are nigh (still stepping).
Every EMBRYONIC step, the proto-brane re-checks entries in the wait-for queue by
calling `resolve()` again. The `resolve()` method is idempotent — if the target
changed state since last check, the SearchFir will detect it and update itself.

StateChange registration handles the case where a SearchFir becomes WOCONSTANIC
(found a constanic target). The SearchFir adds itself to the target's state
change listener list. When the target transitions, it sends a StateChange
message to all registered listeners. The listener's parent proto-brane receives
the StateChange in its inbound queue and re-calls `resolve()` on the affected
SearchFir.

The difference: the wait-for queue is for nigh targets (busy-polling until they
settle). StateChange registration is for constanic targets (event-driven
notification when they make progress, typically through constanic cloning and
re-coordination).

### Transition to BRANING

The proto-brane transitions from EMBRYONIC to BRANING when:
- All SearchFirs have reached a terminal constanic state.
- The wait-for queue is empty.
- No outstanding FulfillSearch messages remain (all responses received).

---

## BRANING Stage Communication

During BRANING, the proto-brane steps its child branes from the braneMind heap
and continues to handle message traffic:

- Receives FulfillSearch from children. Attempts local resolution. If it
  cannot resolve, forwards the FulfillSearch to its own parent.
- Receives RespondToSearch from parent. Routes the response to the child that
  originally requested it, using the LUID from the message.
- Sends StateChange to registered listeners when its own children transition.
- Monitors the wait-for queue. When constanic dependencies resolve (through
  constanic cloning triggered elsewhere), the proto-brane detects the
  transition and triggers constanic cloning of the dependency into its own
  context.

Message processing during BRANING is subject to the same bandwidth limit as
EMBRYONIC. The proto-brane processes at most MESSAGE_BANDWIDTH messages per
step.

Transition to a constanic terminal state when:
- All child branes in the braneMind heap have achieved constanicity.
- All wait-for queue entries have resolved or achieved constanicity.
- No outstanding FulfillSearch messages remain.

The proto-brane's terminal state depends on its children's states:
- All children CONSTANT or INDEPENDENT: proto-brane becomes CONSTANT.
- Any child CONSTANIC or WOCONSTANIC: proto-brane becomes CONSTANIC or
  WOCONSTANIC accordingly.

---

## Constanic Cloning: Lifecycle and Message Fate

Constanic cloning creates a copy of a constanic FIR and attaches it to a new
parent, giving the clone a chance to resolve in the new context. This section
traces the exact sequence and specifies what happens to every piece of
message-related state.

### When Cloning Triggers

Constanic cloning is triggered when a search resolution discovers a CONSTANIC
or WOCONSTANIC FIR that needs to operate in a new context. The typical trigger
is an assignment like `result = fn` where `fn` is a constanic brane. The
containing brane's wait-for mechanism detects that `fn` has achieved
constanicity and initiates the clone.

The source FIR must have achieved constanicity before cloning. Attempting to
clone a nigh FIR raises an exception. The wait-for mechanism guarantees this
precondition.

### CONSTANIC Clone Sequence

Source state: CONSTANIC (search failed in original context).
Clone initial state: EMBRYONIC (re-search in new context).

The clone needs to re-search because identifiers that were not found in the
original context may exist in the new parent.

```
Timeline: CONSTANIC clone of brane F = {a = x;} into context where x = 42

  Step 1 — Create clone structure
      clone = new ProtoBrane(original_AST)
      clone.braneMind  = empty (fresh)
      clone.braneMemory = empty (fresh)
      clone.inboundQueue  = empty (fresh)
      clone.outboundQueue = empty (fresh)
      clone.searchIndex   = empty (fresh)
      clone.waitForQueue  = empty (fresh)
      clone.stateChangeListeners = empty (fresh)

  Step 2 — Attach clone to new parent (Linkage 1)
      clone.setParentFir(newParent)
      Clone now has a parent pointer. FulfillSearch can be addressed.

  Step 3 — Recursively clone children
      For each child in the original's braneMemory:
        - If child is CONSTANT or INDEPENDENT: share by reference.
        - If child is CONSTANIC: recursively clone. Cloned child's
          parentFir = clone (not the original parent).
        - If child is WOCONSTANIC: recursively clone into BRANING state.
      Cloned children are added to clone.braneMemory.

  Step 4 — Attach clone to new context (Linkage 2)
      clone.braneMemory.parent = newParent.braneMemory
      clone.memoryPosition = position within new parent

      This is the step that enables resolution in the new context.
      Linkage 2 was null between Steps 1 and 4. During this window the
      clone does not step, so the gap is safe.

  Step 5 — Set clone state to EMBRYONIC
      clone.nyes = EMBRYONIC

  Step 6 — Clone begins stepping
      EMBRYONIC: clone's SearchFir("x") calls resolve().
      Local search in clone's braneMemory: not found.
      Sends FulfillSearch("x") to newParent via Linkage 1.

  Step 7 — New parent resolves
      newParent processes FulfillSearch("x").
      Finds x = 42 in its braneMemory.
      Sends RespondToSearch("x", ValueFiroe(42)) to clone.

  Step 8 — Clone processes response
      SearchFir("x") receives result: ValueFiroe(42) at CONSTANT.
      SearchFir("x") becomes CONSTANT.
      All searches resolved. Clone transitions EMBRYONIC -> BRANING.

  Step 9 — BRANING completes
      No children to step (all CONSTANT).
      Clone transitions to CONSTANT.
```

The two-phase attachment (Linkage 1 in Step 2, Linkage 2 in Step 4) means there
is a brief window where the clone has a parent pointer but no memory chain
link. This window is safe because the clone does not step between construction
and the completion of attachment.

### WOCONSTANIC Clone Sequence

Source state: WOCONSTANIC (all searches succeeded, waiting on constanic dependencies).
Clone initial state: BRANING (do not re-search; wait for cloned children to resolve).

The clone does not need to re-search because the WOCONSTANIC FIR already found
every identifier it was looking for. Its SearchFirs are WOCONSTANIC, each
holding a reference to a constanic target. During cloning, those targets are
themselves cloned, and the clone waits for the cloned targets to make progress.

```
Timeline: WOCONSTANIC clone of brane G = {a = x; b = a;}
  Original state: SearchFir(x) is CONSTANIC. SearchFir(a) is WOCONSTANIC,
  target = SearchFir(x). Brane G is WOCONSTANIC.

  Step 1 — Create clone structure
      Same as CONSTANIC clone: all queues and listener tables are fresh and empty.

  Step 2 — Attach clone to new parent (Linkage 1)
      clone.setParentFir(newParent)

  Step 3 — Recursively clone children
      SearchFir(x) is CONSTANIC -> clone to EMBRYONIC.
        Cloned SearchFir(x) will re-search in new context.
      SearchFir(a) is WOCONSTANIC -> clone to BRANING.
        Cloned SearchFir(a) does not re-search.
        Its target reference must point to the CLONE of SearchFir(x),
        not the original. (See Open Design Questions for the remapping
        mechanism.)

  Step 4 — Attach clone to new context (Linkage 2)
      clone.braneMemory.parent = newParent.braneMemory

  Step 5 — Set clone state to BRANING

  Step 6 — Clone begins stepping
      BRANING: clone steps its children.
      Cloned SearchFir(x) is at EMBRYONIC. It calls resolve().
      Sends FulfillSearch("x") to newParent.

  Step 7 — Resolution cascades
      newParent resolves: x = 42. Sends RespondToSearch.
      Cloned SearchFir(x) becomes CONSTANT.
      Cloned SearchFir(x) sends StateChange to listeners.

  Step 8 — WOCONSTANIC child responds
      Cloned SearchFir(a) receives StateChange from cloned SearchFir(x).
      Re-calls resolve(). Finds SearchFir(x) is now CONSTANT.
      Dereferences to value 42. Becomes CONSTANT.

  Step 9 — Brane clone completes
      All children CONSTANT. Clone transitions to CONSTANT.
```

### CONSTANT and INDEPENDENT: No Clone

CONSTANT and INDEPENDENT FIRs are immutable. Their values never change
regardless of context. No cloning is needed; the reference is shared directly.

| Source State | Clone Action | Rationale |
|-------------|-------------|-----------|
| CONSTANT | Share reference | Immutable value. Safe to reference from any context. |
| INDEPENDENT | Share reference | Immutable and detached. Safe to reference from any context. |
| CONSTANIC | Recursive clone, start at EMBRYONIC | Re-search in new context. |
| WOCONSTANIC | Recursive clone, start at BRANING | Wait for cloned children to re-resolve. |
| nigh | Exception | Precondition: source must have achieved constanicity. |

### Message Fate During Cloning

When a constanic FIR is cloned, its message-related state is neither inherited
nor transferred. The clone starts clean. Here is the disposition of every
artifact:

| Artifact | Original at clone time | Clone's initial state | What happens to original's |
|----------|----------------------|----------------------|---------------------------|
| inboundQueue | May contain unprocessed RespondToSearch messages | Empty | Orphaned. Original is constanic and will never process them. |
| outboundQueue | May contain unsent FulfillSearch messages | Empty | Orphaned. Even if sent, responses go to the original, which ignores them. |
| searchIndex (LUID map) | Maps original FIR identities to positions | Rebuilt from cloned FIR identities | Original's map is irrelevant to clone. |
| waitForQueue | May contain pending SearchFirs | Empty | Original's waits are abandoned. Clone re-discovers waits through its own lifecycle. |
| stateChangeListeners | May have registrations from other FIRs | Empty | Original's registrations become dead subscriptions. See below. |
| braneMind | Empty (C5 invariant: constanic FIRs have empty braneMind) | Empty, populated by `prime()` during BRANING setup | No carryover. |
| braneMemory contents | All statement FIRs | Recursively cloned: CONSTANT shared, CONSTANIC/WOCONSTANIC deep-cloned | Original's memory is untouched. |
| braneMemory.parent | Points to original parent's memory | null at construction, then relinked to new context | Replaced entirely. |
| SearchFir.value (resolved reference) | Points to whatever the identifier resolved to | null (discarded) | Clone re-resolves from scratch. |

Why orphaned messages are safe:

1. The original is constanic. It will never call `step()` again. Messages in
   its queues will never be processed, and that is correct — the original's
   context is the old context.

2. The clone operates in a new context. The original's messages were addressing
   the original context. Inheriting them would be wrong — a RespondToSearch
   from the old parent carries results from the old context, which may differ
   from what the new context provides.

3. Dead listener registrations (original's SearchFirs registered on targets)
   are harmless. When the target sends StateChange, it delivers to the
   original's SearchFirs, which are constanic and do nothing. The overhead is
   one wasted message delivery per dead registration. Implementations may
   prune dead registrations lazily during StateChange dispatch (skip listeners
   that have achieved constanicity).

### Listener Re-Registration After Cloning

When a WOCONSTANIC SearchFir is cloned, it needs to listen for StateChange from
the clone of its target, not from the original target. This requires
re-registration.

For CONSTANIC clones (starting at EMBRYONIC), this happens naturally: the clone
re-does its entire lifecycle, re-discovers its targets through `resolve()`, and
registers as a listener through the normal EMBRYONIC search resolution process.

For WOCONSTANIC clones (starting at BRANING), the situation is more subtle. The
clone's SearchFir already has a target reference, but that reference needs to
point to the cloned target (not the original). Two aspects require resolution:

1. Target remapping: during recursive cloning, a remapping table maps each
   original FIR to its clone. After all children are cloned, the clone's
   WOCONSTANIC SearchFirs update their target references using this table.
   SearchFirs whose targets are outside the cloned subtree (in the parent
   context) retain their original target references.

2. Registration timing: the clone's first BRANING step registers WOCONSTANIC
   SearchFirs as listeners on their (possibly remapped) targets. This happens
   before any child stepping occurs, ensuring no StateChange is missed.

See [Open Design Questions](#open-design-questions) for unresolved aspects of
cross-brane target references.

### Worked Example: Constanic Clone in New Context

```foolish
{
    fn = {a = x;};    !! fn becomes CONSTANIC (x not found)
    x = 42;
    result = fn;      !! triggers constanic clone of fn
}
```

```
Outer brane (EMBRYONIC/BRANING)
  |
  |  statement 0: fn = {a = x;}
  |    fn is a brane. During fn's EMBRYONIC, SearchFir("x") sends
  |    FulfillSearch("x") to fn's parent (outer brane). Outer brane
  |    searches backward from fn's position: no "x" precedes fn.
  |    Sends RespondToSearch("x", NotFound) to fn.
  |    SearchFir("x") becomes CONSTANIC. fn becomes CONSTANIC.
  |
  |  statement 1: x = 42
  |    ValueFiroe(42). Becomes CONSTANT immediately (INDEPENDENT).
  |
  |  statement 2: result = fn
  |    SearchFir("fn") resolves: finds fn at statement 0.
  |    fn is CONSTANIC. SearchFir("fn") becomes WOCONSTANIC.
  |    Registers for StateChange on fn.
  |    Outer brane detects fn is constanic. Triggers constanic clone.
  |
  |  --- Constanic cloning ---
  |
  |  Clone fn' = clone of fn, attached to outer brane.
  |    fn'.parentFir = outer brane
  |    fn'.braneMemory.parent = outer brane's braneMemory
  |    fn'.nyes = EMBRYONIC
  |    fn' contains: SearchFir'("x"), value = null (reset)
  |
  |  fn' steps through EMBRYONIC:
  |    SearchFir'("x").resolve() -> local search fails.
  |    Sends FulfillSearch("x") to outer brane.
  |    Outer brane searches backward from fn''s position:
  |      finds x = 42 at statement 1 (precedes fn' in statement order).
  |    Sends RespondToSearch("x", ValueFiroe(42)).
  |    SearchFir'("x") becomes CONSTANT.
  |    fn' transitions: EMBRYONIC -> BRANING -> CONSTANT.
  |
  |  result = fn' (CONSTANT, value = {a = 42})
```

### Worked Example: WOCONSTANIC Clone with Cascading Resolution

```foolish
{
    fn = {a = x; b = a;};  !! a is CONSTANIC (x not found)
                            !! b is WOCONSTANIC (found a, but a is CONSTANIC)
                            !! fn is WOCONSTANIC
    x = 42;
    result = fn;            !! triggers constanic clone of fn
}
```

```
  --- Constanic cloning ---

  Clone fn' (WOCONSTANIC -> BRANING):
    SearchFir'(x) cloned from CONSTANIC -> state set to EMBRYONIC
    SearchFir'(a) cloned from WOCONSTANIC -> state set to BRANING
      SearchFir'(a).target remapped to SearchFir'(x)  (not original)
      SearchFir'(a) registers as StateChange listener on SearchFir'(x)

  fn' steps through BRANING:

  Step 1: fn' steps SearchFir'(x) (EMBRYONIC).
    SearchFir'(x).resolve() -> sends FulfillSearch("x") to outer brane.
    Outer brane finds x = 42. Sends RespondToSearch("x", ValueFiroe(42)).
    SearchFir'(x) becomes CONSTANT.
    SearchFir'(x) sends StateChange(WOCONSTANIC -> CONSTANT) to listeners.

  Step 2: fn' delivers StateChange to SearchFir'(a).
    SearchFir'(a) re-calls resolve().
    Target SearchFir'(x) is now CONSTANT.
    SearchFir'(a) dereferences to value 42. Becomes CONSTANT.

  Step 3: All children CONSTANT. fn' transitions to CONSTANT.

  result = fn' (CONSTANT, value = {a = 42; b = 42;})
```

---

## Message Routing and Address Model

### Address Structure

Each proto-brane or brane maintains:

- Parent address: the brane that contains this one. Root brane has no parent.
  FulfillSearch messages flow upward through this address.
- Child addresses: the sub-branes appended during PREMBRYONIC. RespondToSearch
  and StateChange messages flow downward through these addresses.
- LUID-based lookup table: maps LUID to FIR within this brane. Used to route
  RespondToSearch to the specific SearchFir that originated the request.

These addresses are established during the PREMBRYONIC to EMBRYONIC transition
and remain stable for the brane's lifetime. Constanic clones establish new
addresses during their own attachment sequence (the same two-linkage process
described above).

### Depth-Based Sanity Check

Every FIR in the brane tree has a depth — the number of brane boundaries
between it and the root. The message-passing system enforces:

> A FulfillSearch message must originate from a strictly deeper node than the
> receiver.

When a brane receives a FulfillSearch, it compares the sender's depth to its
own. If the sender's depth is not greater, the message is malformed — it
indicates a circular parent reference or broken tree structure. The brane raises
a PANIC alarm and drops the message.

This check costs one integer comparison per message and prevents infinite
forwarding loops.

### Message Bandwidth and Fairness

Each proto-brane processes a bounded number of messages per `step()`:

```java
void step() {
    int processed = 0;
    while (processed < MESSAGE_BANDWIDTH && !inboundQueue.isEmpty()) {
        Message msg = inboundQueue.dequeue();
        processMessage(msg);
        processed++;
    }
    // ... other step logic
}
```

MESSAGE_BANDWIDTH is a constant (typically 8-16 messages per step). This
ensures:
- No single brane monopolizes the message channel.
- Step cost is bounded and predictable.
- Fair round-robin progress across all branes in the tree.

---

## Order of Precedence: Search and Concatenation

This section establishes the binding order of the major UBC2 operations. When
multiple operations interact, precedence determines which operation's result is
"locked in" before the next operation can observe it.

### Precedence Table (Highest to Lowest)

| Precedence | Operation | Binding Time | Description |
|-----------|-----------|-------------|-------------|
| 1 (highest) | Literal evaluation | PREMBRYONIC | Literals become INDEPENDENT immediately. Their values are fixed before any search or concatenation occurs. |
| 2 | Anchored search (`e.member`, `e^`, `e$`, `e#N`, `e?pat`) | EMBRYONIC | The anchor expression `e` is evaluated first, then the search is performed within the resulting brane. The result is bound at evaluation time — it does not change retroactively. |
| 3 | Unanchored search (`x`, `?pattern`, `#-N`) | EMBRYONIC → CONSTANIC cloning | Initial search occurs during EMBRYONIC. If not found, the result is CONSTANIC, which can be re-evaluated in a new context via constanic cloning. |
| 4 (lowest) | Brane concatenation (`A B`) | BRANING | Concatenation merges two branes. It operates on the *values* of its operands — if an operand has already resolved to NK or CONSTANT, concatenation cannot change that resolution. |

### Why Precedence Matters

The precedence order means:

1. **Anchored search binds before concatenation.** When you write `x = e.value;
   y = {value=10} x;`, the expression `e.value` is fully resolved before `x` is
   available to participate in concatenation. Concatenation cannot retroactively
   change what `e.value` resolved to.

2. **Unanchored search can be deferred across concatenation.** When you write
   `f = {a = x;}; h = g f;`, the search for `x` in `f` produces CONSTANIC. When
   `f` is concatenated with `g`, constanic cloning re-evaluates `x` in the merged
   context. The CONSTANIC state is specifically designed to bridge this
   precedence gap.

3. **Concatenation never re-evaluates resolved searches.** Once a SearchFir
   reaches a terminal state (CONSTANT, NK, WOCONSTANIC), concatenation does not
   "undo" that resolution. It can only trigger constanic cloning for SearchFirs
   that are CONSTANIC (awaiting a new context).

### Interaction Summary

```
Anchored search:    e.value  →  binds immediately  →  NK or value (final)
                                                        ↓
                                                   concatenation cannot change this

Unanchored search:  x        →  binds or CONSTANIC  →  if CONSTANIC, re-evaluated
                                                        via constanic cloning when
                                                        concatenation provides new context
```

---

## Anchored vs Unanchored Search: NK and CONSTANIC Outcomes

When a search fails to find its target, the outcome depends on whether the search
is *anchored* or *unanchored*. This distinction is fundamental to the UBC2 design
and reflects the interaction between search binding and brane concatenation.

### The Core Rule

| Search Type | Not Found Outcome | Rationale |
|-------------|------------------|-----------|
| Unanchored (`x`, `?pattern`, `#-N`) | CONSTANIC (`🧠??`) | Concatenation could provide the binding later. |
| Anchored (`e.member`, `e~param`, `e^`, `e$`, `e#N`, `e?pattern`) | NK (`🧠???`) | The anchor expression was already bound before any concatenation could change the search context. |

### Why Unanchored Searches Produce CONSTANIC

An unanchored search like `x` in `{a = x;}` traverses the parent chain looking
for a binding. If no binding is found, the result is CONSTANIC — not NK — because
brane concatenation can extend a brane's contents after initial evaluation:

```foolish
{
    f = {a = x;};     !! x not found → f is CONSTANIC (🧠??)
    g = {x = 42;};
    h = g f;          !! concatenation prepends g's contents to f
                       !! clone of f re-searches in new context → x = 42
}
```

The CONSTANIC state is correct because `f` *might* gain a binding for `x` when
placed in a new context via concatenation or re-coordination. The search was
performed and nothing was found *in this context*, but another context could
provide the missing value.

### Why Anchored Searches Produce NK

An anchored search like `e.value` is different. The expression `e` is evaluated
first, producing a brane. Then `.value` searches *within that specific brane*.
The crucial insight is that **the anchor binding happens at evaluation time**, and
evaluation precedes any concatenation that might later affect the expression.

Consider:

```foolish
{
    e = {};
    x = e.value;        !! anchored search: e is {}, .value not found → 🧠???
    y = {value = 10} x; !! concatenation with x
}
```

Here, `e.value` is evaluated when `e` is bound to `{}`. The empty brane `{}`
is CONSTANT — it is fully evaluated and immutable. It will never gain new
members. The search for `value` within `{}` fails definitively, producing NK
(`🧠???`).

Even though `x` is later used in a concatenation (`{value = 10} x`), this does
not retroactively change the result of `e.value`. The anchored search `e.value`
was bound *before* `x` participated in any concatenation. The binding precedence
is:

1. `e` resolves to `{}` (CONSTANT)
2. `.value` searches within `{}` — not found → NK (`🧠???`)
3. `x` receives the value `🧠???`
4. `{value = 10} x` concatenates, but `x` is already NK — concatenation
   operates on `x`'s value, it does not re-evaluate the search that produced it

This is analogous to accessing an undefined member of an object in other
languages: `obj.nonexistent` evaluates to `undefined` (or throws) at the point
of access, regardless of what happens to `obj` later.

### Binding Precedence: Anchored Search Binds Before Concatenation

The key principle is that **anchored search has higher precedence than
concatenation**. When you write:

```foolish
{
    e = {alpha = 1;};
    x = e.beta;          !! e is CONSTANT, beta not found → 🧠???
    y = {beta = 2} x;    !! concatenation does not re-evaluate e.beta
}
```

The expression `e.beta` is fully resolved before `x` is available for
concatenation. The SearchFir for `e.beta` has already reached its terminal state
(NK) by the time any concatenation involving `x` could occur. There is no
mechanism to "go back" and re-evaluate the anchored search — it was a question
asked of a specific brane at a specific point in time, and the answer was
definitive.

This contrasts with unanchored searches, where the CONSTANIC state specifically
exists to allow re-evaluation in new contexts via constanic cloning.

### Anchored Search on CONSTANIC and WOCONSTANIC Branes

When the anchor brane itself is not yet fully resolved, the anchored search
cannot definitively fail. The outcome depends on the anchor's state:

| Anchor State | Member Found? | SearchFir Outcome | Rationale |
|-------------|--------------|-------------------|-----------|
| CONSTANT | Yes, at CONSTANT | CONSTANT | Fully resolved. |
| CONSTANT | Yes, at CONSTANIC | WOCONSTANIC | Member exists but its value is constanic. |
| CONSTANT | No | NK (`🧠???`) | Brane is immutable. Member will never exist. |
| CONSTANIC | Yes, at any state | WOCONSTANIC | Found member, but anchor may change in new context. |
| CONSTANIC | No | NK (`🧠???`) | Even though the anchor is constanic, the brane's *statement array* is fully accumulated — all identifiers are known. A missing member is definitively missing. |
| WOCONSTANIC | (not searched) | WOCONSTANIC | Cannot search inside WOCONSTANIC brane (still stepping). Wait for anchor to resolve. |
| NK (`🧠???`) | (not searched) | NK (`🧠???`) | Anchor is definitively unfindable. Searching on NK is NK. |

**Note on CONSTANIC anchors**: A CONSTANIC brane has completed its PREMBRYONIC
stage, so its statement array (the set of named identifiers) is fully known. The
CONSTANIC state means the brane's *values* may change in a new context, not that
new *identifiers* will appear. Therefore, if an anchored search on a CONSTANIC
brane fails to find a matching identifier, the result is NK — the member does
not exist in this brane's structure, regardless of what context the brane might
later be placed in.

### Message Protocol Implications

For the message protocol, the anchored vs unanchored distinction affects the
RespondToSearch result:

- **Unanchored search, not found at root**: RespondToSearch carries NotFound.
  The SearchFir becomes CONSTANIC (may resolve in future context).
- **Anchored search, member not found in CONSTANT/CONSTANIC anchor**: The
  SearchFir resolves locally (no FulfillSearch message needed — the anchor is
  already available). The result is NK, not CONSTANIC. No StateChange
  registration is needed because NK is a final, irreversible state.

This means anchored searches that produce NK do not participate in the
constanic cloning lifecycle. They are fully resolved at evaluation time and
their values are immutable.

---

## Search Isolation (ConcatenationBrane)

During EMBRYONIC, a ConcatenationBrane intercepts FulfillSearch messages from
its children and does not forward them to its own parent. This enforces search
isolation: children being concatenated cannot leak their searches into the
surrounding scope.

```
ConcatenationBrane (searchIsolation = true)
  |
  |  child A sends FulfillSearch("y")
  |    ConcatenationBrane searches locally among its concatenation children.
  |    If found: sends RespondToSearch("y", result) to child A.
  |    If not found: sends RespondToSearch("y", NotFound) to child A.
  |    Does NOT forward to parent. Child A's search fails if not found locally.
  |
  |  After all children reach constanicity:
  |    ConcatenationBrane merges children into a single brane.
  |    Merged brane performs its own PREMBRYONIC -> EMBRYONIC cycle.
  |    Merged brane CAN send FulfillSearch to ConcatenationBrane's parent.
  |    Search isolation ends.
```

The purpose of search isolation is to ensure that concatenation does not change
the meaning of the branes being concatenated. Each child brane resolves
identifiers using only what it and its co-concatenation siblings provide. After
merging, the combined brane searches the parent context as a unit.

---

## Exception Cases

### Cycle Detection

A cycle in the parent chain (A's parent is B, B's parent is C, C's parent is A)
would cause FulfillSearch messages to loop infinitely. The depth-based sanity
check (see Message Routing) detects this: a message arriving at a node whose
depth is not less than the sender's depth indicates a structural error.

### Stalled WOCONSTANIC

A WOCONSTANIC FIR waits for its constanic dependencies to make progress. If
those dependencies are never re-coordinated (no constanic cloning ever moves
them to a context where they can resolve), the WOCONSTANIC FIR is stalled
permanently.

This is not a deadlock — it is correct behavior. A WOCONSTANIC expression that
depends on a CONSTANIC expression that is never re-coordinated simply remains
WOCONSTANIC. The sequencer renders it as `🧠?`, accurately reflecting that the
expression found what it needed but the found thing is stuck.

If a timeout mechanism is desired (raising an alarm after N steps without
progress), the proto-brane can track the step count since last StateChange
received for each WOCONSTANIC SearchFir and raise an alarm when the count
exceeds a threshold. This is a diagnostic aid, not a correctness requirement.

---

## Open Design Questions

### WOCONSTANIC Clone Target Remapping

When a WOCONSTANIC FIR is recursively cloned, its SearchFirs hold references to
targets in the original tree. The clone's SearchFirs need references to the
cloned copies of those targets. The design calls for a remapping table built
during recursive cloning, but the exact mechanism is not specified:

- How is the table populated? During the recursive clone walk, each
  (original, clone) pair is recorded. After all cloning completes, a fixup pass
  updates target references.
- What about cross-brane targets? If a SearchFir's target is outside the cloned
  subtree (in the parent context, not in the brane being cloned), there is no
  clone to remap to. The SearchFir retains its reference to the original target.
  StateChange from that original must reach the clone — but the clone is in a
  different parent chain. The routing mechanism for this case is not defined.

### setParentFir Guard Interaction with Cloning

The depth-based sanity check and the "no reassignment after constanicity" guard
proposed for `setParentFir()` are sensible invariants, but they interact with
the cloning mechanism. The clone's nyes state at construction time determines
whether the constanicity guard fires. Implementations must ensure that
`setParentFir` is called before the clone's nyes is set to a constanic state,
or the guard must be explicitly bypassed during cloning (through a flag or
through constructor-based assignment that avoids the guard).

### Nyes State Predicate API

The `isAt(Nyes)`, `isPre(Nyes)`, `isPost(Nyes)`, `isNigh()`, and `getNyes()`
predicates referenced in the UBC2 design specification need to be formalized
across both the design documents and the implementation. The current codebase
uses `atConstanic()` and `isConstanic()` with subtly different semantics. A
consistent predicate API prevents bugs in state-conditional logic:

```java
boolean isAt(Nyes state);                // exactly at this state
boolean isPre(Nyes state);               // before and not including
boolean isPost(Nyes state);              // after and not including
boolean isNigh();                        // alias for isPre(CONSTANIC)
Nyes getNyes();                          // return current state (no setter)
// achievedConstanicity = isAt(CONSTANIC) || isPost(CONSTANIC)
```

---

## Appendix: State Name Mapping

The UBC2 design uses a four-stage lifecycle. The current UBC1 codebase uses a
seven-state Nyes enum. The mapping is approximate:

| UBC2 Stage | UBC1 Nyes Values | Notes |
|------------|-----------------|-------|
| PREMBRYONIC | UNINITIALIZED + `initialize()` | The atomic PREMBRYONIC step maps to the UNINITIALIZED to INITIALIZED transition. |
| EMBRYONIC | INITIALIZED + CHECKED | Identifier resolution occurs across these states. |
| BRANING | PRIMED + EVALUATING | PRIMED populates braneMind; EVALUATING steps children. |
| CONSTANIC | CONSTANIC | Same name, same meaning. |
| WOCONSTANIC | (not implemented) | UBC2 design-forward. Currently folded into CONSTANIC. |
| CONSTANT | CONSTANT | Same name, same meaning. |
| INDEPENDENT | (not implemented) | UBC2 design-forward. Currently all literals are CONSTANT. |

---

## Last Updated

**Date**: 2026-03-12
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Added reference to D0.6 Communication Medium document, clarifying the
relationship between message protocol (what messages are exchanged) and communication
adapter (how messages travel).
Previous (2026-03-03): Added "Order of Precedence: Search and Concatenation" section
establishing binding order (literal > anchored search > unanchored search >
concatenation). Added "Anchored vs Unanchored Search: NK and CONSTANIC Outcomes"
section explaining why unanchored searches produce CONSTANIC (concatenation can
provide bindings later) while anchored searches produce NK (binding happens at
evaluation time, before concatenation). Includes binding precedence argument,
worked examples, anchor state table, and message protocol implications.
Previous (2026-03-02): Complete rewrite with detailed protocol specification.
Previous (2026-02-27): Created message protocol design document for D0-D2
context. Finalized minimal 3-message protocol after analysis of UBC1 patterns.
