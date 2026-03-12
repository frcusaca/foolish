# D0.6 — Communication Media

> AI agents: read [../../DOC_AGENTS.md](../../DOC_AGENTS.md) before editing this file.

This document specifies the Communication Media — the mechanisms by which branes exchange
messages. The UBC2 design specifies *what* messages branes exchange and *when*, while
providing flexibility for *how* they travel. Each brane implements a *communication adapter*
that abstracts the underlying transport. This document defines the adapter interface, message
types, bandwidth semantics, and discusses design alternatives for the media themselves.

The UBC2 MVP (Minimum Viable Product) uses **parent-to-ancestor chaining** as the baseline
communication medium. More advanced implementations may use delegated direct addressing or
other media as optimization layers.

See [D0.0 ProtoBrane Lifecycle](d0_0_protobrane.md), [D0.1 Brane with Boundary](d0_1_brane.md),
and [UBC2 Message Protocol](ubc2_message_protocol.md) for context on brane lifecycle and message
semantics.

---

## Design Principle: Media Independence

The UBC2 brane computer specifies *what* messages branes exchange and *when* they exchange
them, but intentionally defers *how* messages physically travel. This separation allows:

1. **Implementation flexibility** — Different deployments may use different communication
   strategies without changing brane logic.
2. **Performance tuning** — The media can be optimized for specific workloads (deep nesting,
   wide fanout, distributed execution) without redesigning the brane protocol.
3. **Correctness isolation** — Brane message handling logic stays simple; complexity moves to
   the adapter layer.

Each brane implements a `CommunicationAdapter` interface. The brane calls adapter methods
to send and receive messages. The adapter translates these calls into operations on the
concrete medium.

```java
interface CommunicationAdapter {
    void send(Message msg);
    List<Message> receive();
    boolean canSend();
    boolean canReceive();
}
```

---

## Message Types

The UBC2 protocol defines three message types (see
[UBC2 Message Protocol](ubc2_message_protocol.md) for full specification):

| Message | Direction | Purpose |
|---------|-----------|---------|
| `FulfillSearch` | Upward | Child requests parent to resolve an identifier search |
| `RespondToSearch` | Downward | Parent returns search result to child |
| `StateChange` | Registered | FIR notifies listeners of Nyes state transitions |

### Message Structure

All messages carry:
- **Source LUID** — Locally Unique Identifier of sender (for response routing)
- **Payload** — Type-specific data (search identifier, result FIR, new state, etc.)
- **Depth** — Sender's depth in brane tree (for sanity checks)

```java
record FulfillSearch(LUID source, String identifier, int depth) implements Message {}
record RespondToSearch(LUID target, FIR result, boolean isNye, int depth) implements Message {}
record StateChange(LUID source, Nyes newState, int depth) implements Message {}
```

---

## Communication Adapter

The `CommunicationAdapter` encapsulates all medium-specific logic. Each brane holds a
reference to its adapter, created during brane initialization.

### Adapter Responsibilities

1. **Message serialization** — Convert internal message representation to medium format
2. **Routing** — Determine destination address based on message type and brane topology
3. **Queue management** — Buffer outgoing messages, manage receive buffers
4. **Flow control** — Respect communication bandwidth limits
5. **Error handling** — Detect and report delivery failures

### Adapter Creation

Adapters are created during brane initialization with information about the brane's position
in the communication topology:

```java
CommunicationAdapter createAdapter(
    Brane brane,
    CommunicationAdapter parentAdapter,  // null for root brane
    Topology topology
)
```

The topology parameter describes the overall structure and allows the adapter to be aware
of global routing options.

---

## Bandwidth Semantics

Bandwidth limits control how much work a brane can do per step. There are two distinct
bandwidth dimensions:

### Compute Bandwidth

Compute bandwidth limits how many internal operations a brane performs per step:
- Number of searches resolved locally
- Number of child branes stepped
- Number of state transitions processed

This is the traditional bandwidth concept from UBC1.

### Communication Bandwidth

Communication bandwidth limits message exchange per step:
- Maximum outgoing messages sent
- Maximum incoming messages received
- Maximum messages forwarded (for relay branes)

Separating these dimensions allows independent tuning. A brane might have high compute
capacity but be communication-bound due to network latency or bandwidth constraints.

```java
record BandwidthLimits(
    int computeOperations,    // Internal operations per step
    int outgoingMessages,     // Messages sent per step
    int incomingMessages,     // Messages received per step
    int forwardedMessages     // Messages relayed per step (for intermediate branes)
)
```

### Bandwidth Enforcement

The adapter tracks outbound and inbound message counts. When limits are reached:
- `canSend()` returns false — brane must defer sending until next step
- `canReceive()` returns false — brane defers receive processing
- Pending messages queue for next step

This ensures fair step progression across all branes and prevents any single brane from
consuming all communication capacity.

---

## MVP: Parent-to-Ancestor Chaining

The UBC2 MVP uses **parent-to-ancestor chaining** as the baseline communication medium.
This is the reference implementation that establishes correct behavior before optimization.

### How It Works

Each brane knows only its immediate parent. Messages travel one hop at a time:

```
Child ─→ Parent ─→ Grandparent ─→ Great-Grandparent
    (FulfillSearch propagates upward)
Child ←── Parent ←── Grandparent
    (RespondToSearch returns downward)
```

### Adapter Structure

```java
class ParentAdapter implements CommunicationAdapter {
    CommunicationAdapter parent;  // null for root

    void send(FulfillSearch msg) {
        if (parent == null) {
            // Root brane — handle as final authority
            handleAsRoot(msg);
        } else {
            parent.send(msg);  // Forward to parent
        }
    }

    void send(RespondToSearch msg) {
        // RespondToSearch always goes to known target (via LUID lookup)
        deliverToTarget(msg.target);
    }
}
```

### Why MVP

1. **Correctness first** — Simple chain is easiest to reason about and debug
2. **Natural hierarchy** — Matches brane containment structure
3. **No global state** — No registry or routing table needed
4. **Foundation for optimization** — More complex media build on this baseline

Once the MVP is correct, optimizations can layer on top without changing brane logic.

---

## Advanced Media: Delegated Direct Addressing

After establishing the MVP, a more powerful communication medium enables branes to
communicate directly with ancestors beyond their immediate parent.

### The Problem with Pure Chaining

In deep brane trees, chaining becomes a bottleneck:

```
Depth 10: Child needs identifier X
  → Depth 9: Parent (forwards)
    → Depth 8: Grandparent (forwards)
      → Depth 7: Great-grandparent (FOUND!)
        → Depth 8: (returns)
          → Depth 9: (returns)
            → Depth 10: (child receives)

Total: 18 message hops for one search resolution
```

Each intermediate brane must process the message, consuming communication bandwidth.

### Delegated Direct Addressing Solution

Instead of always chaining, an ancestor brane can **delegate** a direct communication
reference to a distant brane:

```
1. Child sends FulfillSearch to Parent (chaining)
2. Parent doesn't have it, but knows Grandparent does
3. Parent responds with: "I don't have it, but TALK TO Grandparent directly"
   (includes Grandparent's communication reference)
4. Child now has direct path to Grandparent
5. Future searches from Child can go directly to Grandparent
```

### The Delegation Protocol

When a brane receives a `FulfillSearch` it cannot satisfy:

```java
handle(FulfillSearch msg) {
    FIR localResult = searchLocally(msg.identifier);

    if (localResult != null) {
        // Found it — respond normally
        sendRespondToSearch(msg.source, localResult);
    } else if (parent == null) {
        // Root — not found
        sendRespondToSearch(msg.source, NK);
    } else {
        // Forward with optional delegation
        if (shouldDelegate(msg.identifier)) {
            // I know who has this — give them the reference
            CommunicationRef ref = getAncestorRef(msg.identifier);
            sendRespondToSearch(msg.source, NotFoundWithDelegation(ref));
        } else {
            // Standard chaining
            parent.send(msg);
        }
    }
}
```

### Delegation Response

New response type for delegation:

```java
record RespondToSearchWithDelegation(
    LUID target,
    CommunicationRef delegatedRef,  // Reference to ancestor who may have the answer
    String identifier,              // What we're searching for
    String reason                   // Why this ancestor (for debugging)
) implements Message {}
```

### Building the Delegation Chain

Delegation can cascade:

```
1. Child (depth 10) asks Parent (depth 9) for "X"
2. Parent doesn't have X, but knows it's in Great-Grandparent (depth 7)
   Parent: "Talk to depth-7 directly" [sends ref to depth-7]
3. Child now communicates with depth-7
4. Depth-7 doesn't have X, knows Great-Great-Grandparent (depth 5) does
   Depth-7: "Talk to depth-5 directly" [sends ref to depth-5]
5. Child now has direct path to depth-5
6. Depth-5 has X, responds directly
```

The child builds a "delegation tree" of direct references to ancestors it has
been authorized to contact.

### Safety Constraints

1. **Ancestors only** — A brane can only receive references to its ancestors
   (depth < current depth). Prevents circular references.

2. **Purpose-bound** — A delegated reference is tied to a specific purpose
   (e.g., "search for identifier X"). The ancestor can reject off-purpose messages.

3. **Revocable** — An ancestor can invalidate a delegated reference. Subsequent
   messages fall back to chaining.

4. **Bounded** — A brane maintains a limited delegation cache (e.g., last 10
   references). Oldest entries evicted when full.

### Adapter with Delegation

```java
class DelegatingAdapter implements CommunicationAdapter {
    CommunicationAdapter parent;
    Map<String, CommunicationRef> delegationCache;  // identifier → direct ref

    void send(FulfillSearch msg) {
        CommunicationRef directRef = delegationCache.get(msg.identifier);

        if (directRef != null) {
            // Try direct path first
            directRef.send(msg);
        } else {
            // Fall back to chaining
            parent.send(msg);
        }
    }

    handle(RespondToSearchWithDelegation msg) {
        // Cache the delegation for future use
        delegationCache.put(msg.identifier, msg.delegatedRef);

        // Forward the search to the delegated ancestor
        msg.delegatedRef.send(
            new FulfillSearch(msg.source, msg.identifier)
        );
    }
}
```

### Comparison: MVP vs Delegated

| Aspect | MVP (Chaining) | Delegated Direct |
|--------|----------------|------------------|
| Complexity | Low | Medium |
| Deep tree latency | O(depth) | O(1) after delegation |
| Intermediate load | Every brane processes | Only delegation endpoints |
| State management | None | Delegation cache |
| Debuggability | Easy (single path) | Medium (cached paths) |

---

## Design Alternatives

This section discusses additional communication medium designs beyond the MVP and
delegated addressing discussed above.

### Topic-Based Pub/Sub

Branes publish and subscribe to message topics. The medium handles routing.

```
Child publishes:  "search.request.x@depth=3"
Parent subscribes: "search.request.*@depth=2"
```

**Advantages:**
- Decouples senders from receivers
- Supports broadcast patterns (notify all siblings)
- Medium can optimize routing
- Easy to add observers for debugging

**Disadvantages:**
- Topic naming convention must be designed carefully
- Potential for message storms
- Harder to trace message flow for debugging
- May overcomplicate simple parent-child exchange

### Global Registry

A central registry maps brane LUIDs to communication endpoints.

**Advantages:**
- True O(1) direct addressing
- Works across disconnected brane trees
- Enables cross-tree communication

**Disadvantages:**
- Single point of failure
- Requires synchronization
- May not scale to very large systems
- Violates "no shared state" principle

---

## Medium Requirements

Regardless of implementation, the communication medium must satisfy:

### Correctness Requirements

1. **Delivery** — Every sent message is eventually delivered (no silent drops)
2. **Ordering** — Messages from same sender arrive in send order
3. **Integrity** — Message contents are not corrupted in transit
4. **Addressing** — Messages reach intended recipients only

### Performance Requirements

1. **Bounded latency** — No message waits indefinitely
2. **Fair scheduling** — No brane starves others of communication capacity
3. **Scalability** — Performance degrades gracefully with brane count and depth

### Debuggability Requirements

1. **Traceability** — Message paths can be logged and traced
2. **Inspection** — Pending messages can be examined for debugging
3. **Metrics** — Bandwidth utilization is measurable

---

## Integration with Brane Lifecycle

The communication adapter integrates with the brane's step() method:

```java
void step() {
    switch (nyes) {
        case EMBRYONIC:
            // 1. Receive responses from previous sends
            while (adapter.canReceive() && adapter.hasIncoming()) {
                handle(adapter.receive());
            }

            // 2. Send new search requests (within bandwidth)
            while (adapter.canSend() && hasPendingSearches()) {
                adapter(sendFulfillSearch());
            }
            break;

        case BRANING:
            // Step children, handle state changes
            while (adapter.canReceive() && adapter.hasIncoming()) {
                handle(adapter.receive());
            }
            stepChildren();
            break;
    }
}
```

The adapter's `canSend()` and `canReceive()` methods enforce bandwidth limits.

---

## Implementation Guidance

When implementing a concrete communication adapter:

1. **Start simple** — Parent-to-ancestor chaining is the baseline; optimize later
2. **Log messages** — Include sender, receiver, type, and payload for debugging
3. **Respect bandwidth** — Enforce limits strictly; queue overflow is a protocol error
4. **Handle errors** — Undeliverable messages should produce recoverable errors, not crashes
5. **Test thoroughly** — Message ordering, delivery, and bandwidth limits all need unit tests

---

## Related Documents

- [D0.0 ProtoBrane Lifecycle](d0_0_protobrane.md) — Brane state machine
- [D0.1 Brane with Boundary](d0_1_brane.md) — Search escalation protocol
- [UBC2 Message Protocol](ubc2_message_protocol.md) — Message type definitions
- [UBC2 Design](ubc2_design.md) — Overall UBC2 architecture

---

## Last Updated

**Date**: 2026-03-12
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Changed title to "Communication Media" (plural). Added MVP section specifying parent-to-ancestor chaining as the baseline implementation. Added "Advanced Media: Delegated Direct Addressing" section with full protocol for ancestor-to-brane delegation, including RespondToSearchWithDelegation message type, safety constraints (ancestors-only, purpose-bound, revocable, bounded cache), and comparison table. Reorganized design alternatives section to remove redundant content and add global registry option.
