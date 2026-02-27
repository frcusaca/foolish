# agents.status.md — AI Action and Progress Log

> AI agents: read [../DOC_AGENTS.md](../DOC_AGENTS.md) before editing this file.

Append-only. Add new entries at the bottom with `---` header and `[YYYY-MM-DD] agent / model`.
Reference by line number from other documents (lines never shift upward).

The content below through line ~992 is the original UBC2 Design Specification — Status and
Continuation Guide, preserved as historical record and context. New log entries follow after
the original Last Updated section.

---

*Original document header:* UBC2 Design Specification — Status and Continuation Guide.
This document captures the complete state of the UBC2 design work as of 2026-02-20, so that
another Claude session or a human reader can resume work without loss of context.

---

## Table of Contents

- [Session Summary](#session-summary)
- [What Was Produced](#what-was-produced)
- [UBC2 Design — Compressed Context](#ubc2-design--compressed-context)
  - [Core Concept](#core-concept)
  - [Message-Passing Architecture](#message-passing-architecture)
  - [Every Expression Is a Brane](#every-expression-is-a-brane)
  - [FIR Lifecycle: PREMBRYONIC → EMBRYONIC → BRANING → constanic](#fir-lifecycle-prembryonic--embryonic--braning--constanic)
  - [Terminal States](#terminal-states)
  - [SF and SFF Markers](#sf-and-sff-markers)
  - [Constanic Cloning (Replaces CMFir)](#constanic-cloning-replaces-cmfir)
  - [NK vs CONSTANIC Clarification](#nk-vs-constanic-clarification)
  - [Sequencing Output Symbols](#sequencing-output-symbols)
  - [FIR Type Hierarchy: IndependentFir + ProtoBrane](#fir-type-hierarchy-independentfir--protobrane)
  - [Lessons Learned from UBC1 (9 items)](#lessons-learned-from-ubc1-9-items)
- [Design Decisions Made During This Session](#design-decisions-made-during-this-session)
- [Design TODO — Unspecified Features (Priority Order)](#design-todo--unspecified-features-priority-order)
- [Open Issues (Items 8–22 from UBC1 Review)](#open-issues-items-822-from-ubc1-review)
  - [High Priority — Affects UBC2 Design](#high-priority--affects-ubc2-design)
  - [Medium Priority — Implementation Quality](#medium-priority--implementation-quality)
  - [Lower Priority — Cleanup and Future](#lower-priority--cleanup-and-future)
- [UBC1 Code Review Findings](#ubc1-code-review-findings)
  - [State Machine Defects](#state-machine-defects)
  - [Search System Fragility](#search-system-fragility)
  - [Parent Chain and Memory Integrity](#parent-chain-and-memory-integrity)
  - [Concatenation Implementation Gaps](#concatenation-implementation-gaps)
  - [Output Rendering Inconsistencies](#output-rendering-inconsistencies)
  - [Global State and Error Handling](#global-state-and-error-handling)
- [Git History: Design Iteration Patterns](#git-history-design-iteration-patterns)
- [Documentation Gaps from Vintage Legacy Review](#documentation-gaps-from-vintage-legacy-review)
- [Does UBC2 Address These Problems?](#does-ubc2-address-these-problems)
- [HUMAN ACTION ITEMS — Outstanding Design Questions](#human-action-items--outstanding-design-questions)
- [Uncommitted Files](#uncommitted-files)
- [What To Do Next](#what-to-do-next)

---

## Session Summary

The session (2026-02-16 through 2026-02-20) produced the UBC2 design specification through 9
rounds of iterative refinement between the user and Claude Code (Opus 4.6). The work proceeded
as follows:

1. **Gathered existing UBC documentation** from `docs/vintage_legacy/` — read ECOSYSTEM.md,
   UBC_FEATURES.md, NAMES_SEARCHES_N_BOUNDS.md, search-semantics.md,
   004-nyes-state-simplification.md, 008-cmfir-nyes-state-review.md, 003-Detachment_Project.md,
   009-Concatenation_Project.md, and others.

2. **Created `docs/how/ubc_engineering.md`** — a consolidated reference for the existing UBC1
   implementation (FIR hierarchy, Nyes state machine, BraneMind/BraneMemory, search/query system,
   CMFir context manipulation, constraints, parallel Java/Scala implementations). This is the
   "what exists today" document.

3. **Created `docs/how/ubc2_design.md`** — the UBC2 design specification (1137 lines), iteratively
   refined across 6 user feedback cycles. This is the "what we're building next" document.

4. **Ran a comprehensive review** of all UBC concerns from source code
   (`foolish-core-java/src/main/java/org/foolish/fvm/ubc/`), git history (12 weeks, Nov 2025 –
   Feb 2026), and documentation. Three parallel research agents produced 22 prioritized issues.

5. **Addressed items 1–7** directly in the design document (NK vs CONSTANIC clarified, IfFiroe
   removed, concatenation/detachment added to Design TODO, CMFir superseded by constanic cloning,
   state machine redesigned, search churn acknowledged in Lessons Learned).

6. **Restated items 8–22** as Open Issues in the design document, reprioritized based on all
   design decisions made during the session.

7. **Updated `docs/README.md`** with links to the two new engineering documents.

8. **Updated `docs/todo/0_documentation.md`** Phase 2 progress (3 items marked complete).

**All changes are currently uncommitted** on the `docs` branch.

---

## What Was Produced

| File | Status | Description |
|------|--------|-------------|
| `docs/how/ubc2_design.md` | **NEW (untracked) | UBC2 design specification, 1137 lines. The primary deliverable. |
| `docs/how/ubc_engineering.md` | **NEW (untracked) | Consolidated UBC1 engineering reference, ~325 lines. |
| `docs/README.md` | **MODIFIED** | Added Engineering Reference links to new docs. |
| `docs/todo/0_documentation.md` | **MODIFIED** | Phase 2: 3 items checked off. |
| `docs/todo/1_ubc2_design_status.md` | **NEW (this file) | This continuation guide. |

---

## UBC2 Design — Compressed Context

This section compresses the full 1137-line design document into the essential concepts needed to
resume work. For full details, read `docs/how/ubc2_design.md`.

### Core Concept

The UBC2 (Unicellular Brane Computer Mark 2) is the **reference implementation** for establishing
expected behavior of a brane computer. Analogous to a stack frame on a Xeon processor: when a
brane is loaded, we know exactly what to do. It takes Foolish source → AST → FIRs → steps FIRs
through a lifecycle → terminal constanic state.

Five design principles: (1) every expression is a brane, (2) conservative staging, (3) message
passing, (4) writing-order precedence, (5) explicit lifecycle.

### Message-Passing Architecture

Branes communicate via two message types:
- **FulfillSearch** (child → parent): "I have an unresolved search, can you find this?"
- **RespondToSearch** (parent → child): "Here is your search result."

Unresolvable searches cascade upward to the root brane. Each brane maintains a **braneMind** —
the set of active FIRs and pending communications.

**Circular sanity check**: Every FIR has a depth (brane boundary count from root). A FulfillSearch
sender must be strictly deeper than the receiver. Violation = PANIC alarm + drop message. One
integer comparison per message prevents infinite forwarding loops.

### Every Expression Is a Brane

All non-literal expressions are branes with one unified lifecycle. This includes:
- **Curly-brace branes** `{...}`: define a search scope boundary. `.value()` returns the brane.
- **Boundary-less branes (proto-branes): binary ops (`a + b`), unary ops (`-x`), searches.
  No search scope boundary (searches start in parent). `.value()` returns a literal.
- **Literal values**: Always INDEPENDENT. No lifecycle. They simply exist.

**If-then-else is removed.** UBC1's `IfFiroe` had infinite-loop bugs and fragile recursive
`step()`. UBC2 uses search-based path selection instead (not yet designed — see Design TODO).

### FIR Lifecycle: PREMBRYONIC → EMBRYONIC → BRANING → constanic

**PREMBRYONIC** — Seven explicit steps on the Foolish code AST:
0. Establish LUID (statement_number : expression_disambiguator)
1. Count lines in brane
2. Create statement array from AST
3. Gather characterized identifiers; cache fully characterized names for search. Named statements
   form a linked list into the statement array. Skip unnamed statements.
4. Instantiate RHS FIRs in PREMBRYONIC
5. Find all searches in RHS expressions, stopping at brane boundaries. Maintain writing order
   (first-to-write-first-to-find precedence).
6. Resolve intra-brane searches. Bind locally resolvable ones. Aggregate unresolved into braneMind
   for parent communication.
7. Append child branes from RHS to braneMind in writing order.

**EMBRYONIC** — Active communication for search resolution:
- Sends FulfillSearch to parent for all unresolved searches.
- Receives FulfillSearch from children; resolves locally or forwards to parent.
- Receives RespondToSearch; applies results, forwards to children.
- **Wait-for mechanism**: If a search result is non-constanic (target still evaluating), the
  dependent statement is enqueued to wait. Initial implementation: busy-checking (periodic poll).
  Callback-based is a future refinement.
- Transitions to BRANING when all non-brane expressions are constanic.

**BRANING** — Steps child branes. Continues handling inbound/outbound communications. Persists
until all children and expressions reach constanic states.

**constanic** — Terminal. See next section.

### Terminal States

| State | Meaning |
|-------|---------|
| **CONSTANIC** | Search performed, **nothing found**. May gain value via recoordination. |
| **WOCONSTANIC** | Waiting On CONSTANICs. All searches found, but dependencies are themselves CONSTANIC/WOCONSTANIC. Structurally complete, waiting on deps. |
| **CONSTANT** | Fully evaluated. Immutable. |
| **INDEPENDENT** | Promoted from CONSTANT, detached from parent. Future-facing; not exercised in initial UBC2. |

`achievedConstanic` = `at_constanic || at_woconstanic || at_constant || at_independent`.

The CONSTANIC/WOCONSTANIC split matters for: SF/SFF markers, constanic cloning behavior, and
sequencing output symbols.

### SF and SFF Markers

**SFF `<<expr>>`**: Stay-Fully-Foolish. PREMBRYONIC → CONSTANIC directly. No search at all.
All identifiers frozen unresolved.

**SF `<expr>`**: Stay-Foolish. Participates in parent's EMBRYONIC. Resolves its own symbols
only. (1) Does not forward children's messages. (2) Does not resolve found results further.
Typical terminal state: WOCONSTANIC (found everything, didn't evaluate).

### Constanic Cloning (Replaces CMFir)

UBC1 wrapped constanic FIRs in a CMFir with two-phase evaluation. UBC2 replaces this with
**constanic cloning**: recursively clone the expression tree, replace the slot directly, no
wrapper.

Cloning rules by source state:
- **CONSTANT / INDEPENDENT**: Reference to immutable (no copy). CONSTANT never changes.
- **CONSTANIC**: Recursive clone. Clone transitions back to **EMBRYONIC** for fresh search
  resolution in new context.
- **WOCONSTANIC**: Recursive clone. Clone stays **WOCONSTANIC** — searches already found,
  just waiting on cloned dependencies to settle.
- **nigh**: Not cloned. Wait-for ensures cloning only after constanic.

CONSTANT and INDEPENDENT sub-expressions within clones are references to immutable originals.
Only CONSTANIC/WOCONSTANIC sub-expressions are recursively cloned.

### NK vs CONSTANIC Clarification

**The rule**: NK (`🧠???`) only when the UBC can **prove** the search will fail in ALL possible
future executions — including when CONSTANIC expressions are discovered by search and
coordinated elsewhere. Otherwise: CONSTANIC.

**Why this matters**: Brane concatenation can extend a brane's contents. A search that fails
locally could succeed after concatenation. Therefore most intra-brane search failures produce
CONSTANIC, not NK.

**Reverses UBC1**: `{#-1}` (seek previous line when none exists) was NK in UBC1. In UBC2
it is CONSTANIC, because concatenation could prepend content.

**NK is reserved for**: anchored search on CONSTANT brane that doesn't contain the target,
arithmetic errors, depth limit exceeded.

### Sequencing Output Symbols

| Symbol | State | Meaning |
|--------|-------|---------|
| `🧠???` | NK | Definitively unfindable |
| `🧠??` | CONSTANIC | Not found yet, might be found later |
| `🧠?` | WOCONSTANIC | Found everything, waiting on dependencies |
| *(value)* | CONSTANT / INDEPENDENT | Fully evaluated |

CONSTANIC and WOCONSTANIC branes are **expanded** in output to show internal state.
Indentation uses full-width underscore `＿` (U+FF3F). The sequencing format includes
`!!INPUT!!`, `PARSED AST:`, `UBC EVALUATION:` steps, `FINAL RESULT:`, `COMPLETION STATUS:`.

### FIR Type Hierarchy: IndependentFir + ProtoBrane

UBC2 proposes a **shallow hierarchy** replacing UBC1's 15+ FIR subclasses:

```
FIR (abstract base — state machine, depth, LUID, parent reference)
 ├── IndependentFir     — Literal values. No lifecycle. Always INDEPENDENT.
 └── ProtoBrane         — Everything else. One lifecycle.
      Behavior varies by traits (not subclasses):
      - hasBoundary: true = curly-brace brane, false = proto-brane
      - isDetachment: true = liberation brane (future)
      - sfMode: NONE / SF / SFF
```

One `step()` implementation consults traits. No `instanceof` chains. Additive/subtractive:
default ProtoBrane does full lifecycle; traits subtract steps (SFF skips EMBRYONIC) or add
behavior (detachment adds search filtering).

### Lessons Learned from UBC1 (9 items)

1. Don't use ordinal comparisons for state transitions (use named-state checks)
2. Don't let FIR types implement their own `step()` (one step() with trait-based selection)
3. Don't swallow exceptions as NK values (only catch domain-specific: ArithmeticException)
4. Don't use reflection to access private fields (expose through public interface)
5. Don't rename core concepts during implementation (finalize terminology first)
6. Don't wrap when you can clone-and-replace (CMFir problems → constanic cloning)
7. Don't force-approve tests (always manual diff review)
8. Stabilize in one language first (Java before Scala)
9. Validate parent chain integrity (depth-based sanity check)

---

## Design Decisions Made During This Session

These decisions were made through user-Claude dialogue and are now baked into the design document.
A resuming session should treat these as settled unless the user explicitly reopens them.

| Decision | Rationale |
|----------|-----------|
| Every expression is a brane | Unifies lifecycle. Binary/unary ops are boundary-less proto-branes. One `step()` for everything. |
| If-then-else removed from UBC2 | UBC1's `IfFiroe` had infinite-loop bugs and fragile recursive step(). Search-based path selection to be designed. |
| CONSTANIC split into CONSTANIC + WOCONSTANIC | CONSTANIC = nothing found. WOCONSTANIC = all found but deps stuck. Different cloning behavior; different output symbols. |
| INDEPENDENT terminal state (future) | Detached from parent. Reserved for liberation/detachment. Not exercised in initial UBC2. |
| Constanic cloning replaces CMFir | Clone-and-replace-slot, no wrapper. CONSTANIC clones → EMBRYONIC. WOCONSTANIC clones stay WOCONSTANIC. CONSTANT = reference (no copy). |
| NK only for provably unfindable | Reverses UBC1's `{#-1}` = `🧠???`. Concatenation means most intra-brane failures are CONSTANIC. |
| SFF: PREMBRYONIC → CONSTANIC directly | No search resolution at all. |
| SF: own symbols only, no child forwarding, no further resolution | Typically WOCONSTANIC. |
| Shallow FIR hierarchy (IndependentFir + ProtoBrane) | Traits (hasBoundary, isDetachment, sfMode) instead of 15+ subclasses. |
| Sequencing symbols: `🧠??` = CONSTANIC, `🧠?` = WOCONSTANIC, `🧠???` = NK | CONSTANIC/WOCONSTANIC branes expanded in output. |
| Circular message-passing sanity check | Depth-based invariant: sender must be strictly deeper than receiver. |
| Writing-order precedence | First-to-write-first-to-find. Programmer controls resolution order by code order. |
| Wait-for mechanism (busy-checking initially) | Callback-based is a future refinement. |

---

## Design TODO — Unspecified Features (Priority Order)

These features are required for UBC2 but **not yet designed**. They are captured in
`docs/how/ubc2_design.md` under "Design TODO".

### 1. Brane Concatenation (Highest Priority)

Core Foolish language operation. Affects search semantics fundamentally (drives the NK vs
CONSTANIC distinction). Design must specify:

- How concatenation interacts with PREMBRYONIC (re-do steps 1–7?)
- How concatenation interacts with EMBRYONIC (search messages after brane extension)
- Concatenation of CONSTANT + CONSTANIC branes
- Writing-order precedence in concatenated branes
- Three-level precedence rules from NAMES_SEARCHES_N_BOUNDS.md: left-associate liberations,
  liberation right-associates with branes, brane free association

**Source material**: `docs/vintage_legacy/009-Concatenation_Project.md` (3-stage design,
incomplete), `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` (precedence rules, ~50K chars).

### 2. Detachment / Liberation Branes (Highest, after concatenation)

Depends on concatenation design. Detachment branes `[...]` left-associate with each other and
right-associate with branes before concatenation. Design must specify:

- Detachment as `isDetachment` trait on ProtoBrane (not a separate class)
- Search filter chain: `[a][b][+a]` left-associates, result right-associates with following brane
- Free variable semantics (NOT permanent blocking — UBC1's `DetachmentBraneFiroe` got this wrong)
- Forward search liberation (`[~pattern]`, `[#N]`)
- P-branes (`[+...]`) for selective undetachment
- Detachment removal before identification/ordination/coordination during concatenation

**Source material**: `docs/vintage_legacy/003-Detachment_Project.md`,
`docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` sections on liberation.

### 3. Search-Based Path Selection (High, replaces if-then-else)

The mechanism by which UBC2 does conditional/branching behavior without `IfFiroe`. Specifics
not yet designed. UBC1's IfFiroe had infinite-loop bugs.

---

## Open Issues (Items 8–22 from UBC1 Review)

A comprehensive review of UBC1 code, git history, and documentation produced 22 issues. Items 1–7
were addressed directly in the design document. Items 8–22 remain open and are documented in
`docs/how/ubc2_design.md` under "Open Issues". Here they are with full detail for resumption.

### High Priority — Affects UBC2 Design

**Issue 8: Forward Anchored Search (`B/pattern`) Not Implemented**

UBC1 never completed forward anchored search. `RegexpSearchFiroe` handles backward local, forward
local, and global, but grammar support for `B/pattern` (forward from start of brane B) is
incomplete. UBC2 needs this for search-based path selection. Also: `Query.java` auto-anchors
patterns with `^...$`, which breaks patterns that already contain anchors.

*Action needed*: Design forward anchored search operator semantics. Fix the auto-anchoring logic.
Required before search-based path selection (Design TODO item 3).

**Issue 9: SF and SFF Markers Not Implemented in Code**

`FIR.java` stubs SFF with `NKFiroe("SFF marker (<<==>> syntax) not yet implemented")`. The UBC2
design now fully specifies SF/SFF lifecycle behavior. This is a fresh implementation — no UBC1
code to salvage. SF/SFF are central to constanic cloning and coordination.

*Action needed*: Implement SF/SFF as ProtoBrane `sfMode` trait. SFF shortcut: skip EMBRYONIC.
SF: resolve own symbols only, no child forwarding, no further resolution of found results.

**Issue 10: Identifier Resolution via Message Passing**

UBC1's `IdentifierFiroe.isAbstract()` returns `true` and `getValue()` throws
`UnsupportedOperationException`. In UBC2, identifier resolution goes through FulfillSearch /
RespondToSearch messages. The ProtoBrane's EMBRYONIC stage handles this uniformly.

*Action needed*: Implement identifier resolution as message passing. This is the core of the
EMBRYONIC stage and must work before anything else in the lifecycle can be tested.

**Issue 11: Recursive Search / `↑` Operator Semantics Incomplete**

The `↑` operator materializes one step of recursion by substituting the AST of the expression that
created the current brane. UBC1 has `SearchUpFiroe` but no cycle detection and incomplete
interaction with concatenation (`UnanchoredSeekFiroe` has a TODO). UBC2's depth-based sanity check
helps, but `↑` semantics in the presence of concatenation and detachment need explicit design.

*Action needed*: Specify `↑` semantics for UBC2. Define interaction with concatenation, depth
limit, and cycle detection.

### Medium Priority — Implementation Quality

**Issue 12: Exception Swallowing in Arithmetic**

UBC1's `BinaryFiroe` catches all `Exception` and wraps in NKFiroe, hiding bugs
(NullPointerException, ClassCastException become `🧠???`). UBC2 should only catch
`ArithmeticException` and produce NK with descriptive comment. Others propagate as alarms.

*Action needed*: When implementing ProtoBrane arithmetic, narrow the catch clause.

**Issue 13: Parent Chain Integrity**

No cycle detection on parent FIR chains in UBC1. The depth-based message sanity check catches
circular messages, but `setParent()` should also assert the new parent is shallower.

*Action needed*: Add depth comparison to `setParent()` on ProtoBrane.

**Issue 14: Depth Limiting Should Be Communicated**

UBC1's `EXPRMNT_MAX_BRANE_DEPTH = 96_485` silently makes a brane CONSTANT with `🧠???`. UBC2
should produce an alarm through the message channel. The limit should be configurable.

*Action needed*: Design alarm message type for depth exhaustion. Make limit configurable.

**Issue 15: Approval Tests Were Force-Approved**

Git commit `0032e47`: "Temporarily approve some of these." Baselines may be incorrect. The new
`🧠?` / `🧠??` / `🧠???` output symbols also mean ALL existing approval tests need regeneration.

*Action needed*: Audit existing `.approved.foo` baselines before using them as UBC2 references.
Regenerate all approval tests once UBC2 Sequencer4Human updates are complete.

### Lower Priority — Cleanup and Future

**Issue 16: Old Detachment Tests Test Wrong Semantics**

Five UBC1 test files test permanent-blocking semantics. Correct semantics: free-variable
liberation. Tests: `detachmentBlockingIsApproved.foo`, `detachmentChainingIsApproved.foo`,
`detachmentFilterChainIsApproved.foo`, `detachmentNonDistributionIsApproved.foo`,
`pbraneSelectiveBindingIsApproved.foo`.

*Action needed*: Remove or rewrite when detachment is designed (blocked by Design TODO item 2).

**Issue 17: Missing Test — Partial Parameter Specification**

Need a test where `b?x` returns CONSTANT 10 even though `b` is WOCONSTANIC — validating that
anchored search succeeds on partially-evaluated branes.

*Action needed*: Write test after CONSTANIC/WOCONSTANIC is implemented.

**Issue 18: Scala Cross-Validation Lagging**

UBC2 should be fully stable in Java before Scala. The cross-validation module enforces
byte-identical output. Any output symbol changes cascade to Scala.

*Action needed*: Defer Scala until Java UBC2 passes all approval tests.

**Issue 19: Sequencer Rendering Updates**

Sequencer4Human must be updated: `🧠??` for CONSTANIC (was `⎵⎵`), `🧠?` for WOCONSTANIC (new),
`🧠???` for NK (now 🧠-prefixed), expanded rendering of CONSTANIC/WOCONSTANIC branes, remove/update
`(CONSTANIC)` state suffix.

*Action needed*: Implement after design is stable. Source:
`foolish-core-java/src/main/java/org/foolish/fvm/ubc/Sequencer4Human.java`.

**Issue 20: Depth Search Not Designed**

`NAMES_SEARCHES_N_BOUNDS.md` mentions extending search "into the computation graph" but provides
no design. Future language feature.

*Action needed*: None for UBC2 initial implementation.

**Issue 21: Corecursion / Coordinated State Updates**

`ADVANCED_FEATURES.md` asks about centrally organized coordinated state updates. No design.
UBC2's message-passing may provide a foundation.

*Action needed*: Research topic. Not blocking.

**Issue 22: Future Language Features**

`000-TODO_FEATURES.md` lists: mutable branes, characterization system, loops/iterators,
traceability, program generation/differentiation, native AI facilities, async, library
ecosystem. None designed. ProtoBrane architecture should not preclude them.

*Action needed*: Architecture must remain extensible. Not blocking.

---

## UBC1 Code Review Findings

A deep review of all 34 Java files in `foolish-core-java/src/main/java/org/foolish/fvm/ubc/`
uncovered the following design-level concerns. These are not code quality nits — they represent
fundamental questions about whether the UBC2 design handles these cases correctly.

### State Machine Defects

**CONSTANIC value stability is protocol-enforced, not type-enforced.** Constraint C3 states "A
CONSTANIC FIR's value can only change due to coordination (re-evaluation in new context via CMFir
or ConcatenationFiroe)." But nothing prevents manual re-evaluation paths at the type level.
`CMFir.startPhaseB()` clones a CONSTANIC FIR and resets it to INITIALIZED — this is "legitimate"
but depends on discipline, not the compiler. *UBC2's constanic cloning addresses this by making
the clone-and-replace the ONLY path, but the question remains: should CONSTANIC FIRs be
sealed/immutable at the type level?*

**Unary/Binary null-result state mismatch.** Both `UnaryFiroe` and `BinaryFiroe` override
`achievedConstanic()` to return `true` when `result == null && state == CONSTANT`. The state says
CONSTANT (done) but the result is null (no value). This breaks `achievedConstanic()` semantics. Output
rendering works around this by checking `atConstanic()` for the special null-result case.
*UBC2's ProtoBrane should never have this mismatch — WOCONSTANIC handles the "structurally
complete but not resolved" case explicitly.*

**UnanchoredSeekFiroe overloads CONSTANIC for two meanings.** `achievedConstanic()` returns `true` when
`value == null` — but this covers both "not yet resolved" (nigh) and "out of bounds" (genuinely
unresolved). These should be distinguishable. The code has a TODO: "When implementing brane
concatenation, ensure unanchored seeks re-evaluate after concatenation." *UBC2 needs to decide:
should there be an explicit "awaiting concatenation" state, or is CONSTANIC sufficient?*

**`Query.isAnchored()` field is computed but never read.** The field exists to track whether the
user provided anchors, but grep confirms it's never used in any behavior-altering code. Either
activate it or remove it. *UBC2's search design should decide upfront whether anchoring is
tracked.*

### Search System Fragility

**Anchor unwrapping is a massive `instanceof` chain.** `AbstractSearchFiroe` lines 107–234
contain a 15-case chain: IdentifierFiroe unwrap, AssignmentFiroe unwrap, recursive
AbstractSearchFiroe unwrap, UnanchoredSeekFiroe unwrap, BraneFiroe unwrap (with two separate
search paths), NKFiroe handling. Each new FIR type requires adding to this chain. No explicit
ordering guarantees. *UBC2's ProtoBrane with one type eliminates this entirely — the unwrapping
chain goes away because there's nothing to unwrap.*

**`Query.java` auto-anchors patterns with `^...$`.** A pattern `"abc"` silently becomes
`"^abc$"`. This breaks patterns that already contain anchors. The transformation is hidden and
there's no way to distinguish user-provided anchoring from auto-anchoring in matching logic.
*UBC2 should either: never auto-anchor (explicit anchors only), track original intent separately,
or make anchoring explicit in a type hierarchy.*

**Search is one-shot: `searchPerformed` flag prevents re-search.** After a search executes, the
result is cached and subsequent steps return the cached value. But if the anchor changes (e.g.,
through constanic cloning/coordination), should the search re-execute? Current behavior is
one-shot; UBC2 should make this semantics explicit. *With constanic cloning, the CONSTANIC clone
re-enters EMBRYONIC and its searches execute fresh — so this is likely handled, but should be
documented.*

**Detachment filtering is hidden inside BraneMemory.** `BraneMemory.get(Query, fromLine)` calls
`shouldFilterMatch(query)` which walks up the parent chain looking for `DetachmentBraneFiroe`.
Search silently skips results — no indication why a match was filtered. The `filterActive` flag
becomes `false` after CONSTANIC but is never logged. If the filter deactivates, search re-runs
find different results (non-idempotent). *UBC2 should move detachment logic out of BraneMemory
into an explicit search filter — or into the ProtoBrane's `isDetachment` trait.*

### Parent Chain and Memory Integrity

**Parent chain re-assigned in three different places.** Constraint C4 says "parentFir must not be
reassigned after initial setup ... except during context manipulation." But three unrelated
locations do this: `ConcatenationFiroe.performJoin()` (re-parents source FIRs),
`CMFir.startPhaseB()` (re-links memory parent), `FiroeWithBraneMind.linkMemoryParent()`.
No type marker indicates "this FIR has been re-coordinated." *UBC2's constanic cloning creates
fresh clones with correct parents from the start, but `setParent()` should still enforce that the
new parent is shallower (depth check).*

**No cycle detection on parent chains.** UBC1 documented the constraint but never enforced it.
CMFir and ConcatenationFiroe both modify parent references without checking for cycles. *UBC2's
depth-based message sanity check catches the symptom (circular messages) but `setParent()` should
assert directly.*

### Concatenation Implementation Gaps

**ConcatenationFiroe uses reflection to bypass ordination.** `addWithoutOrdination()` uses
`java.lang.reflect.Field.setAccessible(true)` to access `braneMemory` and `indexLookup` from
`FiroeWithBraneMind`. This is a workaround for missing abstraction — there's no "add to memory
without ordination callback" method. *UBC2's ProtoBrane must expose what concatenation needs
through its public interface.*

**Three-stage design has fragile transitions.** Stage A uses `ExecutionFir` (itself a
`FiroeWithBraneMind`!) adding unnecessary indirection. Stage B has the reflection hack. Stuck
detection during Stage A triggers CONSTANIC but never reports what caused the stuck state.
*UBC2 should design concatenation as a lifecycle event on ProtoBrane, not a separate FIR type.*

**`ConcatenationFiroe.getValue()` throws `UnsupportedOperationException`.** Concatenations
containing branes can't be evaluated to a value. *UBC2 needs to decide: should concatenation
results be expressible as values, or should output sequencing handle this transparently?*

**Search locking during Stage A not implemented.** `009-Concatenation_Project.md` line 79 says
"CRITICAL: LHS searches into this ConcatenationFiroe are blocked during Stage A." The
`searchLocked = true` field exists but the blocking mechanism is incomplete.

### Output Rendering Inconsistencies

**UnaryFiroe renders constanic as single `"⎵"` while everything else uses double `"⎵⎵"`.** This
is a bug in `Sequencer4Human.java` — inconsistent rendering across FIR types.

**Mixed symbols for not-found vs not-known.** `"???"` (NK_STR) is used for actually computed NK
values. `"⎵⎵"` (CC_STR) is used for constanic unresolved values. But search not-found uses
CC_STR instead of NK_STR. The semantic distinction between "not found" and "not known" is
unclear in the rendering. *UBC2's three-symbol system (`🧠?` / `🧠??` / `🧠???`) clarifies this.*

### Global State and Error Handling

**`ExecutionContext.getCurrent()` is global/thread-local.** Error formatting uses a global
ExecutionContext, which means: no parallel execution contexts, thread-unsafe shared state,
nested evaluations can't have different filenames. *UBC2 should pass context through the call
chain, not use thread-local storage.*

**`BinaryFiroe` catches all `Exception` and wraps in NKFiroe.** NullPointerException,
ClassCastException, and other programming bugs become `🧠???` — hidden behind "errors as values."
*UBC2 should only catch `ArithmeticException`.*

**`FiroeWithBraneMind` re-enqueues on error then rethrows.** If an exception persists, the FIR
gets re-enqueued infinitely before the RuntimeException propagates. No maximum retry limit.

---

## Git History: Design Iteration Patterns

Analysis of 12 weeks of git history (Nov 2025 – Feb 2026) reveals which concepts were hardest
to get right and where the design was iterated most heavily.

### Constanic Semantics: 20 Days of Iteration (Jan 19 – Feb 8)

The CONSTANIC concept went through constant refinement:
- Jan 19: Initial CMFir implementation with `CONSTANTIC` state (note original spelling)
- Jan 21: Renamed to "Constanic" across 46 files (~2,000 insertions/deletions)
- Jan 24: Major refactoring — "removed several internal states" (101 files)
- Jan 26–29: Five "Updated CMFir" commits in quick succession — state machine unstable
- Feb 3: Debugging prints added — "Print out constanic", "Output Nyes for debugging when
  results are not constanic" — validation still failing
- Feb 8: Final clarification commit "Clarify ??? versus constanic" — needed explicit
  documentation to resolve confusion between NK (`???`) and constanic (`?C?`) rendering

The NK vs CONSTANIC distinction was NOT obvious and required 20 days and a documentation
clarification pass to stabilize.

### CMFir: 4 Days of State Machine Churn (Jan 26–29)

CMFir went through multiple redesigns in rapid succession:
- Jan 26: Three "Updated CMFir" commits on one day
- Jan 27: "Refactor CMFir to be without brane mind"
- Jan 28: Same refactoring again (duplicate commit)
- Jan 29: "WIP: Implement PRIMED state and cloneConstanic for copy-on-write FIRs" — incomplete

The "with/without braneMind" oscillation and the WIP commit indicate the initial CMFir design
was fundamentally wrong and required multiple attempts to stabilize.

### Search Features: 5 Failed Attempts in 2 Days (Dec 19–20)

Search operators had the most implementation churn:
- Dec 19: Three separate search operator commits (attempts/retries)
- Dec 20: Two more retries
- Jan 7: "support regex operators" committed twice (duplicate)
- Jan 25: Four identical fixes for `regexSearchShadowy.foo` on the same day
- Jan 30: "Add brane search functionality and fix detachment grammar" — still fixing

The `regexSearchShadowy.foo` test became a recurring failure point, requiring 4 identical fix
commits on Jan 25 alone.

### Concatenation: Duplicated Implementations (Feb 1–3)

- Feb 1: "Fix parser to support brane concatenation" committed TWICE (two separate commits)
- Feb 1: "Enable sequence of branes in parsing" — third attempt
- Feb 2: Tests added AFTER implementation (backwards from best practice)
- Feb 3: "Enable the final concatenationt test" — typo in commit message indicates rushed work

### The "Shat All Over It" Incident (Feb 6)

Commit `3e1b676`: **"dump on oc to fix after claude and antigravity shat all over it"**

7 files changed, 95 insertions/74 deletions. Reverted changes to BraneMemory.java, CMFir.java,
ConcatenationFiroe.java, FIR.java, FiroeWithBraneMind.java. This was a recovery from a
multi-agent conflict where multiple Claude instances and another agent ("antigravity") made
conflicting changes. The branch `oc/post_claude_antigrav_human` was created as a recovery point.

### Force-Approved Tests

- `0032e47` (Feb 1): **"Temporarily approve some of these"** — explicitly acknowledges baselines
  are not final
- Multiple selective approvals throughout Jan 20–Feb 3
- `regexSearchShadowy` test approved, then broke, then re-approved across 4 days

### Brain → Brane Rename (Feb 4–5)

The old "Brain" terminology was completely replaced with "Brane" across 13 files. Combined with
the "Constantic" → "Constanic" rename (Jan 21), this represents two major terminology changes
during active implementation — a source of confusion that UBC2 must avoid by finalizing
terminology before coding begins.

### High-Churn Files (Most Frequently Modified)

1. `BraneMemory.java` — state/memory semantics (modified in dump commit)
2. `CMFir.java` — state machine (5+ updates in 4 days)
3. `Nyes.java` — state enum (~8 commits)
4. `Sequencer4Human.java` — rendering logic (major refactorings)
5. `AbstractSearchFiroe.java` — search operators (~6 commits)
6. `FiroeWithBraneMind.java` — brane context (10+ commits)

---

## Documentation Gaps from Vintage Legacy Review

Review of all vintage_legacy documentation reveals design questions that were raised but never
resolved. These are gaps that UBC2 must address or explicitly defer.

### Search Semantics Left Unspecified

- `ECOSYSTEM.md` line 169: "UBC search evaluation: 1. TBD:" — complete search evaluation
  semantics never written
- `ECOSYSTEM.md`: "The Multicellular Brane Computer - [ ] TBD" and "The Next Step in the
  Evolution Chain - [ ] TBD" — future architectures undefined
- `search-semantics.md`: "Recursive searches (searches that reference themselves) may behave
  differently and need special handling. This is not yet fully specified or tested."
- `search-semantics.md`: "What happens if an intermediate parent is CONSTANIC? Should the search
  continue to higher parents or stop?"
- `NAMES_SEARCHES_N_BOUNDS.md`: Context search ("insert brane in middle without shifting line
  numbers") semantics incomplete
- `NAMES_SEARCHES_N_BOUNDS.md` line 1409: "Depth Search — [ ] TODO:" — completely unspecified

### Detachment Design Not Settled

- `003-Detachment_Project.md` lines 654–664: **"There's a mistake in the specifications. The
  detachment should not occur permanently."** The original permanent-blocking semantics were
  wrong. Correct semantics: free-variable liberation, effective only during evaluation-time
  binding.
- Forward search liberation (`[~pattern]`, `[#N]`) syntax exists in documentation but NOT in
  grammar
- P-Branes (`[+...]`) can only apply to SF-marked expressions — why this coupling? Not explained.
- Re-detachment semantics unclear: can you re-detach something already detached? Line 423 asks
  this but doesn't answer.
- Default parameter values in detachment (`[r; pi=3.14]{...}`) — binding time unclear (parse
  time vs evaluation time?)

### Concatenation Precedence Not Fully Worked Out

- `009-Concatenation_Project.md`: Search locking during Stage A marked "CRITICAL" but not
  implemented
- `009-Concatenation_Project.md` lines 240–244: Interactions completely unspecified:
  - P-branes with concatenation
  - Detachment and concatenation
  - Nested concatenation (`{A B} {C D}`)
- `NAMES_SEARCHES_N_BOUNDS.md`: Three-level precedence rules (liberation left-associates,
  liberation right-associates with branes, brane free association) are described but no tests
  validate them

### State Machine Gaps

- `004-nyes-state-simplification.md`: CHECKED state is a "pass-through" — reserved for future
  type/reference checking that was never implemented. What if type checking needs to happen at
  a different point in the lifecycle?
- `008-cmfir-nyes-state-review.md`: What happens if Phase B's clone reaches CONSTANIC? Does it
  stay CONSTANIC, or try Phase B again? Not specified.

### Language Feature Gaps

- `000-TODO_FEATURES.md` line 10: "Foolish is call by value, EXCEPT for when branes are
  partially detached" — then it behaves "partially by reference." This dual semantics never
  clearly explained.
- `ADVANCED_FEATURES.md`: Corecursion and mutual recursion have no Foolish-native solution.
  "How do we centrally organize several coordinated state updates in a single line of code?"
- `ARITHMETIC_ERRORS.md`: Floating-point NaN and infinity handling deferred — "Should
  `1.0 / 0.0` be NK or a special value?"

---

## Does UBC2 Address These Problems?

Cross-referencing the code review, git history, and documentation gaps against the UBC2 design:

| UBC1 Problem | UBC2 Status | Notes |
|-------------|-------------|-------|
| CONSTANIC semantic confusion (20 days of iteration) | **ADDRESSED.** CONSTANIC/WOCONSTANIC split makes the distinction explicit. `🧠?`/`🧠??`/`🧠???` symbols eliminate rendering ambiguity. | Terminology must be finalized before implementation. |
| CMFir two-phase instability (4+ redesigns) | **ADDRESSED.** Constanic cloning replaces CMFir entirely. Clone-and-replace-slot, no wrapper, no phase confusion. | |
| `instanceof` chains in search unwrapping (15 cases) | **ADDRESSED.** ProtoBrane with traits eliminates type dispatch. One `step()`, no unwrapping needed. | |
| Reflection abuse in concatenation | **ADDRESSED in principle.** ProtoBrane exposes public interface. But concatenation design is not yet specified. | **NEEDS DESIGN** |
| Search operator fragility (5 failed attempts) | **PARTIALLY ADDRESSED.** Message-passing architecture replaces implicit parent-chain traversal. But search operators themselves not yet redesigned. | |
| `Query.java` auto-anchoring breaks patterns | **NOT ADDRESSED.** UBC2 design doesn't specify Query/pattern handling. | **NEEDS DECISION** |
| Parent chain re-assignment in 3 places | **ADDRESSED.** Constanic cloning creates fresh clones with correct parents. Depth-based sanity check catches circular chains. | |
| Global ExecutionContext | **NOT ADDRESSED.** UBC2 design doesn't mention execution context threading. | **NEEDS DECISION** |
| BinaryFiroe catches all Exception | **ADDRESSED in Lessons Learned.** Item 3: only catch ArithmeticException. | |
| Force-approved test baselines | **ACKNOWLEDGED.** Issue 15 requires audit. | **NEEDS HUMAN REVIEW** |
| Brain→Brane, Constantic→Constanic renames | **ADDRESSED in Lessons Learned.** Item 5: finalize terminology before implementation. | |
| IfFiroe infinite loops | **ADDRESSED.** IfFiroe removed from UBC2. Search-based path selection to replace it. | **NEEDS DESIGN** |
| Detachment permanent blocking was wrong | **ACKNOWLEDGED in Design TODO.** Free-variable semantics specified as requirement. | **NEEDS DESIGN** |
| NK vs CONSTANIC: anchored vs unanchored inconsistency | **ADDRESSED.** Clear rule: NK only for provably unfindable. Concatenation means most failures are CONSTANIC. | |
| Forward search liberation not in grammar | **NOT ADDRESSED.** Design TODO item 2 lists it as requirement but no grammar changes specified. | **NEEDS DESIGN** |
| Concatenation search locking not implemented | **NOT ADDRESSED.** Concatenation design is TODO item 1. | **NEEDS DESIGN** |
| CHECKED state does nothing | **ADDRESSED.** PREMBRYONIC steps 0–7 replace CHECKED with explicit substeps. | |
| Unanchored seek overloads CONSTANIC | **PARTIALLY ADDRESSED.** CONSTANIC/WOCONSTANIC split helps but "awaiting concatenation" not distinguished. | **NEEDS DECISION** |
| Sequencer rendering inconsistencies | **ADDRESSED.** New symbol table (`🧠?`/`🧠??`/`🧠???`) with explicit rendering rules. | |
| Corecursion/mutual recursion | **NOT ADDRESSED.** Acknowledged as future research in Issue 21. | |
| Floating-point NaN/infinity | **NOT ADDRESSED.** Out of scope for initial UBC2. | |
| `↑` operator semantics incomplete | **NOT ADDRESSED.** Issue 11 notes this. | **NEEDS DESIGN** |
| Call-by-value vs call-by-reference duality | **NOT ADDRESSED.** Not mentioned in UBC2 design. | **NEEDS DECISION** |
| P-Brane coupling to SF marker | **NOT ADDRESSED.** Detachment design is TODO item 2. | **NEEDS DESIGN** |

---

## HUMAN ACTION ITEMS — Outstanding Design Questions

This section lists every design question that requires a **human decision** before UBC2
implementation can proceed. Each item is tagged with priority and points to the section of this
document or `docs/how/ubc2_design.md` where the full context lives.

### BLOCKING — Must Decide Before Implementation Begins

**H1. Brane Concatenation Semantics**
*Priority: BLOCKING. Referenced: [Design TODO item 1](#1-brane-concatenation-highest-priority),
[Does UBC2 Address These Problems?](#does-ubc2-address-these-problems)*

The single most impactful undesigned feature. Concatenation affects NK vs CONSTANIC semantics,
search resolution, detachment, and writing-order precedence. Questions requiring human input:

- When brane A is concatenated with brane B, does A re-execute PREMBRYONIC steps 1–7?
- Does concatenation happen during EMBRYONIC, BRANING, or as a separate lifecycle event?
- What is the search locking mechanism during concatenation? (UBC1 said "CRITICAL" but never
  implemented it.)
- The three-level precedence (liberation left-associates, liberation right-associates with branes,
  brane free association) — are these correct and complete?
- Nested concatenation: `{A B} {C D}` — what's the result?

*Source material*: `docs/vintage_legacy/009-Concatenation_Project.md`,
`docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` lines 1082–1244.

**H2. Detachment / Liberation Semantics**
*Priority: BLOCKING (after H1). Referenced: [Design TODO item 2](#2-detachment--liberation-branes-highest-after-concatenation),
[Documentation Gaps](#detachment-design-not-settled)*

Depends on concatenation design. Critical sub-questions:

- The original permanent-blocking semantics were **wrong** (per `003-Detachment_Project.md`
  line 654). The correct semantics are free-variable liberation — is this fully understood?
- Forward search liberation (`[~pattern]`, `[#N]`) — what exactly are these operators? The
  syntax appears in docs but not in the ANTLR grammar.
- P-Branes `[+...]` can only apply to SF-marked `<expr>` — why? Is this coupling intentional
  or an artifact?
- Re-detachment: can you re-detach something already partially bound? The spec asks this
  question (line 423) but doesn't answer.
- Default parameters in detachment: `[r; pi=3.14]{...}` — are defaults resolved at parse time
  or evaluation time?

*Source material*: `docs/vintage_legacy/003-Detachment_Project.md`,
`docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` lines 739–1244.

**H3. Search-Based Path Selection (Replacing If-Then-Else)
*Priority: BLOCKING. Referenced: [Design TODO item 3](#3-search-based-path-selection-high-replaces-if-then-else),
[Issue 8](#high-priority--affects-ubc2-design)*

UBC2 removes `IfFiroe` but has no replacement yet. Questions:

- What is the mechanism? Pattern matching on search results? Value-conditional search?
- Does this require forward anchored search (`B/pattern`) to work first?
- How does this interact with the existing search operators?
- Are there Foolish code examples of how conditional logic would be expressed?

### HIGH — Should Decide Before Core Implementation

**H4. `↑` Operator Semantics in Presence of Concatenation**
*Priority: HIGH. Referenced: [Issue 11](#high-priority--affects-ubc2-design)*

The `↑` operator materializes one step of recursion. UBC1's `SearchUpFiroe` has no cycle
detection. Questions:

- What happens to `↑` when the brane is concatenated? Does it still reference the original
  expression?
- How does `↑` interact with detachment?
- What is the cycle detection mechanism? Is the depth-based message sanity check sufficient?
- `NAMES_SEARCHES_N_BOUNDS.md` mentions downward cursor `→` cannot work — why not? Can UBC2's
  message passing solve mutual recursion differently?

**H5. Query Auto-Anchoring: Keep, Fix, or Remove?**
*Priority: HIGH. Referenced: [Search System Fragility](#search-system-fragility)*

`Query.java` silently adds `^...$` to patterns. This breaks patterns with existing anchors.
Three options:

- **Never auto-anchor** — user must always provide explicit anchors
- **Track original intent** — separate "user pattern" from "compiled pattern"
- **Make anchoring part of the type** — `AnchoredQuery` vs `UnanchoredQuery`

Which approach fits UBC2's message-passing search model?

**H6. Should CONSTANIC FIRs Be Immutable at the Type Level?**
*Priority: HIGH. Referenced: [State Machine Defects](#state-machine-defects)*

UBC1 relied on protocol discipline to prevent illegal mutation of CONSTANIC FIRs. Constanic
cloning improves this but doesn't fully seal the type. Should `ProtoBrane` enforce immutability
when in a constanic state (e.g., throw on `step()` calls, make fields unmodifiable)?

**H7. "Awaiting Concatenation" — Distinct State or Just CONSTANIC?**
*Priority: HIGH. Referenced: [State Machine Defects](#state-machine-defects)*

`UnanchoredSeekFiroe` uses CONSTANIC for both "not yet resolved" and "out of bounds awaiting
concatenation." UBC2's CONSTANIC means "search not found, might be found later" — is this
sufficient, or does "awaiting concatenation" need its own state for diagnostic clarity?

### MEDIUM — Should Decide During Implementation

**H8. ExecutionContext: Global/Thread-Local or Passed?**
*Priority: MEDIUM. Referenced: [Global State and Error Handling](#global-state-and-error-handling)*

UBC1's `ExecutionContext.getCurrent()` prevents parallel execution and is thread-unsafe. UBC2's
message-passing architecture suggests passing context through the message chain. Is this the
right approach, or is a global context acceptable for the reference implementation?

**H9. Call-by-Value vs Call-by-Reference Duality**
*Priority: MEDIUM. Referenced: [Documentation Gaps](#language-feature-gaps)*

`000-TODO_FEATURES.md` says: "Foolish is call by value, EXCEPT for when branes are partially
detached" — then it behaves "partially by reference." This dual semantics was never fully
explained. Does UBC2's constanic cloning mechanism (which creates copies) change this? What are
the reference semantics for CONSTANT branes that are shared (not copied)?

**H10. Approval Test Baselines: Audit Scope**
*Priority: MEDIUM. Referenced: [Issue 15](#medium-priority--implementation-quality),
[Git History](#force-approved-tests)*

Some `.approved.foo` baselines were force-approved (`0032e47`: "Temporarily approve some of
these"). The new `🧠?`/`🧠??`/`🧠???` symbols mean ALL tests need regeneration anyway. Questions:

- Should old baselines be audited for correctness before UBC2, or just regenerated from scratch?
- Which tests exercised semantics that UBC2 changes (NK vs CONSTANIC)?
- The five detachment tests use wrong semantics (permanent blocking) — remove or rewrite?

**H11. Depth Limit: What Value and How Communicated?**
*Priority: MEDIUM. Referenced: [Issue 14](#medium-priority--implementation-quality)*

UBC1 uses `EXPRMNT_MAX_BRANE_DEPTH = 96_485` — an arbitrary, undocumented magic number. Should
UBC2:
- Lower this significantly (100? 1000?)?
- Make it configurable?
- Communicate via alarm message through the message channel?
- The label "EXPRMNT" suggests it was experimental — should UBC2 finalize this?

### LOWER — Can Defer to Later Design Iterations

**H12. Floating-Point: NK or Special Value for NaN/Infinity?**
*Priority: LOWER. Referenced: [Documentation Gaps](#language-feature-gaps)*

When floating-point arithmetic is added: should `1.0 / 0.0` be NK, or a special infinity value?
Should NaN propagate or become NK?

**H13. Corecursion and Coordinated State Updates**
*Priority: LOWER. Referenced: [Issue 21](#lower-priority--cleanup-and-future)*

"How do we centrally organize several coordinated state updates in a single line of code?"
UBC2's message-passing may provide a foundation, but no design exists.

**H14. Localized `?` vs Globalized `??` Search in Recursive Contexts**
*Priority: LOWER. Referenced: [Documentation Gaps](#search-semantics-left-unspecified)*

`NAMES_SEARCHES_N_BOUNDS.md` defines `?` as not searching parents and `??` as searching parents.
But in recursive contexts with `↑`:
- Does `↑` create a new context that isolates `?` searches?
- Can `??` find variables bound at previous recursion levels?
- How does detachment affect this boundary?

---

## Uncommitted Files

As of 2026-02-20, all session work is **uncommitted** on the `docs` branch:

```
Untracked:
  docs/how/ubc2_design.md
  docs/how/ubc_engineering.md
  docs/todo/1_ubc2_design_status.md   (this file)

Modified:
  docs/README.md
  docs/todo/0_documentation.md
```

The `docs` branch is up to date with `origin/docs`. No conflicts.

---

## What To Do Next

The user has not explicitly stated the next step. Likely continuations (in rough priority order):

1. **Commit the current work.** All files are uncommitted. The user previously said "auto-commit"
   but the session ran out of context before committing.

2. **User reviews the UBC2 design document**, particularly:
   - The Sequencing section (added last)
   - The Open Issues 8–22 (restated and reprioritized last)
   - The Design TODO items (concatenation, detachment, search-based path selection)

3. **Design brane concatenation** (Design TODO item 1, highest priority). This is the most
   impactful unspecified feature. Source material: `docs/vintage_legacy/009-Concatenation_Project.md`
   and `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md`.

4. **Design detachment/liberation** (Design TODO item 2). Blocked by concatenation design.

5. **Design search-based path selection** (Design TODO item 3). Blocked by forward anchored
   search (Issue 8).

6. **Begin UBC2 implementation** in Java. The design is stable enough to start on the core:
   ProtoBrane class, IndependentFir, PREMBRYONIC steps, EMBRYONIC message passing, BRANING
   child stepping, constanic terminal states.

---

## Last Updated

Date: 2026-02-24
Updated By: Claude Code v1.0.0 / claude-sonnet-4-6
Changes: Applied 🧠 prefix to all sequencer output symbols throughout: `🧠?` (WOCONSTANIC),
`🧠??` (CONSTANIC), `🧠???` (NK). Removed erroneous claim that `???` is a valid Foolish input
token (it is not — all three symbols are sequencer output only). Updated Sequencing Output
Symbols table, NK vs CONSTANIC section, issues 12/14/15/18/19, comparison table rows, and
human action item H10.
Previous session (2026-02-20):** Initial creation with comprehensive review. Contains:
session summary, compressed UBC2 design context, 13 settled design decisions, 3 Design TODOs,
15 open issues (items 8–22), deep UBC1 code review, git history iteration analysis, vintage
legacy documentation gaps, UBC2 coverage assessment (22-row matrix), and 14 prioritized human
action items (H1–H14).

---
[2026-02-24] Claude Code v1.0.0 / claude-sonnet-4-6

Structural documentation session. Changes committed across two commits (f5cdfd2, 8a8424d,
and this commit):

- Created `docs/DOC_AGENTS.md` — authoritative AI agent instructions for the docs/ tree.
  Covers project purpose (UBC→UBC2 transition via BDD), directory map, key files to read,
  mandatory per-commit actions, append-only log format, and style guide (English/Java/Foolish).
- Renamed this file from `1_ubc2_design_status.md` to `agents.status.md`. All cross-references
  updated in human_todo_index.md and 0_documentation.md.
- Created `agents.scratch.log.md` — append-only AI scratch pad for non-trivial reasoning.
  Seeded with three entries: WOCONSTANIC dereferencing rationale, output symbol ordering
  rationale, and achievedConstanic() terminology anchoring.
- Added DOC_AGENTS.md notice header (`> AI agents: read ...`) to all *.md files in docs/
  (excluding DOC_AGENTS.md itself and vintage_legacy/).
- Added link to `0.a.concatenation.woes.md` in human_todo_index.md D2 entry and in
  DOC_AGENTS.md key-files table.
- Updated human_todo_index.md Last Updated section; this append entry serves as the
  agents.status.md update for this commit.

Outstanding items unchanged — see human_todo_index.md for current list.

---
[2026-02-24] Claude Code v1.0.0 / claude-sonnet-4-6

Added "Literals Are Branes" subsection to ubc2_design.md, placed between
"Literal Values Are Always INDEPENDENT" and "Proto-Branes: Boundary-Less Expressions".

Key content: every literal `1`, `2`, `"hello"` is internally a single-element system brane
(`🧠1`, `🧠2`, etc.). The three access forms `🧠1.value()`, `🧠1$.value()`, `🧠1^.value()` are
all equivalent and evaluate to the literal. This uniformity is what makes the postfix
concatenation `{r = 🧠1 🧠2 🧠+}` → `{r = {🧠1, 🧠2, 🧠+}}` coherent without special cases.

The 🧠 prefix on literal branes is an internal representation detail; programmers write plain
literals in source, the parser converts them during AST-to-FIR conversion.

Updated: docs/how/ubc2_design.md (new subsection + ToC entry + Last Updated).

---
[2026-02-24] Claude Code v1.0.0 / claude-sonnet-4-6

Clarified the meaning of the 🧠 prefix throughout docs/:

The 🧠 prefix is the **denotational marker** for Foolish semantics. It signals "we are talking
about the meaning of this expression, not its syntax" — analogous to ⟦⟧ in formal denotational
semantics. This unifies all three uses: `🧠1` (literal brane), `🧠+` (operator), `🧠??` (state).

Updated files:
- ubc2_design.md: "Literals Are Branes" section rewritten around denotational framing; Output
  Symbols paragraph rewritten; System Operators opening rewritten.
- systems.md: opening paragraph rewritten.
- DOC_AGENTS.md: style guide bullet on 🧠 replaced with denotational explanation; Foolish section
  updated to "denotational symbols."
- 50_ubc2_baseline_migration.md: two notes about 🧠 prefix rewritten.
- agents.scratch.log.md: new entry explaining the denotational marker insight and its relation
  to standard ⟦⟧ notation.
