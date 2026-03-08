# Human TODO Index

> AI agents: read [../DOC_AGENTS.md](../DOC_AGENTS.md) before editing this file.

Outstanding design decisions and implementation tasks requiring human judgment.
Most items cannot be resolved by an AI session alone — they need design choices,
Foolish language intuition, or review of legacy material.

Each item links back to its full context in the engineering doc and/or the AI scratchpad.

---

## DESIGN DECISIONS — Pre-Implementation (in priority order)

### D0. FIR Subtype Contracts: What Changes After Inheriting ProtoBrane

*Status: Needs decision and write-up*
*Full context: [ubc2_design.md § FIR Type Hierarchy](../how/ubc2_design.md#fir-type-hierarchy)*
*Related: [d0_fir_subtype_contracts.md](../how/d0_fir_subtype_contracts.md)*
*Scratchpad cross-ref: [agents.status.md § FIR Type Hierarchy](agents.status.md#fir-type-hierarchy-independentfir--protobrane)*

Four concrete FIR roles need their delta from ProtoBrane specified:

- **A. Normal brane `{...}`** — Curly-brace brane holding expressions. `hasBoundary = true`, defines search scope.
- **B. System operator brane `🧠+`, `🧠-`, etc.** — Operation brane from desugaring. `hasBoundary = false`.
- **C. ConcatenationBrane** — See D2 for detailed open sub-questions.
- **D. Detachment brane `[...]`** — See D5 for detailed design.

---

### D1. ~~Grouping and Operator Precedence~~

*Status: Moved to top-level todo/ as P2 item*

This has been moved to `../../todo/01_grouping_and_precedence.md` as a P2 task for the
shared parser library. Grouping and precedence operators are effects of brane concatenation,
not a primal concern for UBC1 design.

---

### D2. Brane Concatenation (open sub-questions for D0.C)

*Status: Structure specified; sub-questions remain*
*Full context: [ubc2_design.md § Brane Concatenation](../how/ubc2_design.md#brane-concatenation)*
*Resolved debate: [0.a.concatenation.woes.md](0.a.concatenation.woes.md)*
*Legacy: [vintage_legacy/009-Concatenation_Project.md](../vintage_legacy/009-Concatenation_Project.md)*

Open sub-questions for ConcatenationBrane specification:
- Three-level precedence rules from NAMES_SEARCHES_N_BOUNDS.md
- Concatenating proto-branes (scalars) vs full branes — probably not allowed
- Write-order precedence across concatenated branes
- Eager vs lazy merging

---

### D3. Nyes Microstates for Value Searches (precursor to D4)

*Status: Needs deep investigation*
*Full context: [ubc2_design.md § SearchFir Lifecycle](../how/ubc2_design.md#searchfir-lifecycle-integration)*
*Scratchpad cross-ref: [agents.status.md § FIR Lifecycle](agents.status.md#fir-lifecycle-prembryonic--embryonic--braning--constanic)*

Before designing search-based path selection, we need to understand how searches
whose *result is used as a value* (not just a binding) propagate through the Nyes
state machine. Walk through each microstate (PREMBRYONIC, EMBRYONIC, BRANING,
CONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT) and specify exactly:
- What does a "value search" do in each state?
- When does it fire a message vs. block vs. short-circuit?
- How does dereferencing interact when the search target is a system operator brane
  (whose `value()` is a scalar, not a brane)?

This is a precursor to D4.

---

### D4. Search-Based Path Selection (replaces if-then-else)

*Status: Needs design; no mechanism proposed yet*
*Full context: [ubc2_design.md § Design TODO 4](../how/ubc2_design.md#4-search-based-path-selection-replacing-if-then-else)*

UBC2 has no `if-then-else`. Conditional/branching behavior is expressed via search.
Mechanism is undesigned. Requires D3 (value search microstates) first. Also likely
requires forward anchored search `B/pattern` (see H5/Issue 8).

---

### D5. Detachment / Liberation Branes (D0.D specification)

*Status: Needs design; depends on D2*
*Full context: [ubc2_design.md § Design TODO 3](../how/ubc2_design.md#3-detachment-liberation-branes)*
*Legacy: [vintage_legacy/003-Detachment_Project.md](../vintage_legacy/003-Detachment_Project.md)*

Free variable semantics (NOT permanent blocking). `isDetachment` trait on ProtoBrane.
Filter chain `[a][b][+a]` left-associates, right-associates with following brane.
P-branes, forward search liberation, detachment removal during concatenation.

---

## OPEN ISSUES — High Priority (decide before or during core implementation)

### H1. `↑` Semantics Before Search Implementation

*Full context: [ubc2_design.md § Open Issues 11](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § H1](agents.status.md#h1-recursion-operator-semantics-before-search-implementation)*

Unanchored seek semantics not fully specified. Must decide before search implementation.

---

### H2. Concatenation Search Locking

*Full context: [ubc2_design.md § Open Issues](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § H2](agents.status.md#h2-concatenation-search-locking)*

How an EMBRYONIC brane handles search messages when a concatenation is actively in progress.

---

### H3. If-Then-Else Replacement (BLOCKING)

*Full context: [ubc2_design.md § Design TODO 4](../how/ubc2_design.md#4-search-based-path-selection-replacing-if-then-else)*
*Scratchpad cross-ref: [agents.status.md § H3](agents.status.md#h3-if-then-else-replacement-blocking)*

No mechanism yet. Blocked by D3 (value search microstates) and possibly forward anchored search.

---

### H4. `↑` Operator With Concatenation

*Full context: [ubc2_design.md § Open Issues 11](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § H4](agents.status.md#h4-recursion-operator-semantics-in-presence-of-concatenation)*

Cycle detection. Interaction with detachment. Downward cursor `→` unusable — why?

---

### H5. Query Auto-Anchoring: Keep, Fix, or Remove?

*Full context: [ubc2_design.md § Open Issues 8](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § H5](agents.status.md#h5-query-auto-anchoring-keep-fix-or-remove)*

`Query.java` silently adds `^...$`. Three options: never auto-anchor / track original intent /
make anchoring part of the type.

---

### H6. CONSTANIC FIR Immutability at the Type Level

*Full context: [ubc2_design.md § Open Issues](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § H6](agents.status.md#h6-should-constanic-firs-be-immutable-at-the-type-level)*

Should `ProtoBrane` throw on `step()` when in a constanic state?

---

### H7. "Awaiting Concatenation" — Distinct State or Just CONSTANIC?

*Full context: [ubc2_design.md § Open Issues](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § H7](agents.status.md#h7-awaiting-concatenation--distinct-state-or-just-constanic)*

`UnanchoredSeekFiroe` uses CONSTANIC for two different conditions. Does UBC2 need to
distinguish "out of bounds awaiting concatenation" from "not found"?

---

### Issue 8. Forward Anchored Search (`B/pattern`) Not Implemented

*Full context: [ubc2_design.md § Open Issues 8](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § Issue 8](agents.status.md#issue-8-forward-anchored-search-bpattern-not-implemented)*

Grammar support incomplete. Needed for search-based path selection.

---

### Issue 9. `↑` Has No Cycle Detection in UBC1

*Full context: [ubc2_design.md § Open Issues 9](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § Issue 9](agents.status.md#issue-9-recursion-operator-cycle-detection)*

`SearchUpFiroe` will loop forever on mutual recursion.

---

### Issue 10. Write-Order Precedence Across Concatenated Branes

*Full context: [ubc2_design.md § Open Issues 10](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § Issue 10](agents.status.md#issue-10-write-order-precedence-in-concatenated-branes)*

Within a single brane: first-to-write-first-to-find. Across concatenated branes: unclear.

---

### Issue 11. `↑` Semantics Incomplete

*Full context: [ubc2_design.md § Open Issues 11](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § Issue 11](agents.status.md#issue-11-recursion-operator-semantics-incomplete)*

Materialization of recursion undefined when combined with detachment.

---

## OPEN ISSUES — Medium Priority (decide during implementation)

### H8. ExecutionContext: Global/Thread-Local or Passed?

*Full context: [ubc2_design.md § Open Issues](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § H8](agents.status.md#h8-executioncontext-globalthread-local-or-passed)*

UBC1's global prevents parallelism. Pass through message chain?

---

### H9. Call-by-Value vs Call-by-Reference Duality

*Full context: [ubc2_design.md § Open Issues](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § H9](agents.status.md#h9-call-by-value-vs-call-by-reference-duality)*

Fully constant branes shared (not copied). Partially detached branes "partially by reference."
Never fully specified in UBC1.

---

### H10. Approval Test Baseline Audit Scope

*Full context: [ubc2_design.md § Open Issues 15](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § H10](agents.status.md#h10-approval-test-baselines-audit-scope)*
*Migration doc: [50_ubc2_baseline_migration.md](50_ubc2_baseline_migration.md)*

Audit force-approved baselines before UBC2, or regenerate from scratch?
Five detachment tests use wrong semantics — remove or rewrite?

---

### H11. Depth Limit: Value and Communication Mechanism

*Full context: [ubc2_design.md § Open Issues 14](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § H11](agents.status.md#h11-depth-limit-what-value-and-how-communicated)*

`EXPRMNT_MAX_BRANE_DEPTH = 96_485` is unexplained magic. Lower it? Make configurable?
Communicate exhaustion via alarm message?

---

### Issue 12. Exception Swallowing in Arithmetic

*Full context: [ubc2_design.md § Open Issues 12](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § Issue 12](agents.status.md#issue-12-exception-swallowing-in-arithmetic)*

`BinaryFiroe` catches all `Exception`. UBC2 should only catch `ArithmeticException`.

---

### Issue 13. Parent Chain Integrity

*Full context: [ubc2_design.md § Open Issues 13](../how/ubc2_design.md#open-issues)*
*Scratchpad cross-ref: [agents.status.md § Issue 13](agents.status.md#issue-13-parent-chain-integrity)*

`setParent()` should assert new parent is strictly shallower than child.

---

## OPEN ISSUES — Lower Priority

### H12. Floating-Point: NK or Special Value for NaN/Infinity?

*Scratchpad cross-ref: [agents.status.md § H12](agents.status.md#h12-floating-point-nk-or-special-value-for-naninfinity)*

### H13. Corecursion and Coordinated State Updates

*Scratchpad cross-ref: [agents.status.md § H13](agents.status.md#h13-corecursion-and-coordinated-state-updates)*

### H14. Localized `?` vs Globalized `??` Search in Recursive Contexts

*Scratchpad cross-ref: [agents.status.md § H14](agents.status.md#h14-localized--vs-globalized--search-in-recursive-contexts)*

`NAMES_SEARCHES_N_BOUNDS.md` defines `?` as local-only, `??` as parent-searching. Behavior
across recursion levels via `↑` unspecified.

---

## APPROVAL TEST MIGRATION

*Full tracking doc: [50_ubc2_baseline_migration.md](50_ubc2_baseline_migration.md)*

All `.approved.foo` files need regeneration. No `.foo` input file should use `???` (not valid
Foolish source syntax). Five detachment tests encode wrong semantics.

---

## Last Updated

Date: 2026-03-07
Updated By: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
Changes: Restored D0 as FIR Subtype Contracts overview (four roles: Normal brane, System operator,
ConcatenationBrane, Detachment brane). Created D2 as separate item for Concatenation sub-questions.
Shifted numbering: Value Searches → D3, Search-Based Path Selection → D4, Detachment → D5.
Added reference to d0_fir_subtype_contracts.md. Previous: Rolled D2 into D0.C (reverted).
Previous (2026-02-26): Reviewed for emphasis — file already uses bold appropriately
(definition-list leading labels only). No substantive changes needed.
Previous (2026-02-24): Added DOC_AGENTS.md notice header. Updated cross-refs from
1_ubc2_design_status.md → agents.status.md. Added link to 0.a.concatenation.woes.md in D2.
Previous (2026-02-24): Initial creation.
