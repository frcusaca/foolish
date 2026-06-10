# FOOP-62 Deepseek Feedback — UBCa Two-Store ProtoBrane Design Review

**Date**: 2026-06-09
**Reviewer**: Deepseek V4 Pro (via opencode)
**Spec**: `docs/foop/FOOP-62.md`
**Codebase**: `foolish/foolish-core/src/{fir.rs, ubc.rs, sequencer.rs, search.rs}`

---

## Summary Verdict

The two-store ProtoBrane design is **architecturally excellent** — the `foolish_children`/`ubc_children` split, the task-list stepping model, the immutable parent, and the `bon` builder enforcement all make the type system express the existing semantics directly. However, the design has **several mechanical incompatibilities with the stated acceptance constraints** that must be resolved before implementation can succeed. These are fixable but require specification changes, not just implementation tweaks.

---

## Concern 1 (BLOCKING): Fixpoint Loop Termination Is Incompatible with One-Transition-Per-Step

### The problem

FOOP-62 §3 says each `step()` call does **one** NYES transition and returns the **node's own** NYES. FOOP-62 §7 says the outer fixpoint loop from `run_to_completion_with_scope` is **"retained unchanged."**

The current fixpoint loop checks:
```rust
if prev_state == new_state { break; }
```

In the current UBC, each call to `step_one()` aggressively advances state — e.g., `NormalBraneFir::step_one()` transitions from `Braning` to `Woconstanic` or `Constant` in **one call** (because `re_step_brane_bodies` does a full re-evaluation of all statements). The root's NYES changes every few calls, and the fixpoint progresses naturally.

In the ProtoBrane model, while a node's children are stepping (the task queue is non-empty), the node's **own NYES does not change** — it stays `Braning` until `fir_op_step` runs, which happens only when the task queue **completely empties**. So for a brane with N statements:

- Call 1: step front task (stmt_0.body, one transition) → return NYES = Braning → fixpoint: prev=Embryonic(?), new=Braning → continue
- Call 2: step stmt_0.body again (one more transition) → return NYES = Braning → fixpoint: prev=Braning, new=Braning → **BREAK!**

The fixpoint stops after **two** calls even though work remains. It never reaches `fir_op_step`.

### Why the "retained unchanged" claim is wrong

The one-transition-per-step model fundamentally changes the granularity of NYES changes. In the current UBC, the root FIR's NYES changes on almost every `step_one` call. In ProtoBrane, it changes only when the task queue empties (which may take dozens of calls per child). The `prev == new` check is a **stuck detection** that assumes each call produces observable state change — an assumption that the ProtoBrane model violates.

### Resolution options

Three viable paths, listed in order of preference:

**Option A (recommended): Slightly adapt the fixpoint loop.** Replace the single-call `prev == new` check with a **progress-aware** variant: track `prev_state` over multiple calls and stop only when the state hasn't changed for N consecutive calls (N = 2 or 3). The loop body becomes:

```rust
let mut stall_count = 0u32;
loop {
    // ... terminal checks ...
    let prev_state = fir.borrow().state();
    let new_state = fir.borrow_mut().step(scope)?;
    if new_state != prev_state { stall_count = 0; }
    else {
        stall_count += 1;
        if stall_count >= 2 { break; }
    }
    // ... Woconstanic + forward_refs check ...
}
```

This preserves the stuck-detection semantics while tolerating the granularity change. The FOOP would need to replace "retained unchanged" with "retained with this minor adaptation."

**Option B: Have `step()` return a progress indicator alongside NYES.** Change the signature to `step(&mut self, scope: &Scope) -> Result<(Nyes, bool), UbcError>` where the boolean indicates whether any internal progress was made. The fixpoint stops only when `!made_progress && prev_nyes == new_nyes`. This is clean but changes the trait signature.

**Option C: Keep fixpoint unchanged, change when NYES advances.** Have `fir_op_step` or a fast-path classify the node's NYES based on current child states on **every call**, not just when the queue empties. This defeats the separation of concerns between task-drain and own-work, and produces an NYES that oscillates (Braning → Woconstanic → Braning → Woconstanic...) without the `prev == new` check to catch it.

**Recommendation**: Option A. It's the smallest change, preserves the semantic intent, and makes the "retained" claim true modulo the stall-counter addition.

---

## Concern 2 (BLOCKING): Step Count Mismatch with Snapshot Oracle

FOOP-62 requires **byte-for-byte** reproduction of 131 approved snapshots, including **per-input step counts**. The current UBC's step count = number of `run_to_completion_with_scope` fixpoint loop iterations. The ProtoBrane model makes each iteration do **one** child transition rather than many — so even if Concern 1 is fixed, the step count for each `.foo` input will be **dramatically different**.

### Concrete example

Take `a=1; b=2; c=Op+(a, b)` (3 statements). Current UBC step count (approximate):
- Call 1: Prembrionic → Embryonic (root)
- Call 2: Embryonic → Braning (root)
- Call 3: re_step_brane_bodies: steps all 3 statements to completion (each step_boxed internally loops), then sets Woconstanic or Constant
- Call 4: Constant → done

Step count: ~4 (+ internal sub-loops that aren't counted because they're inside step_boxed).

ProtoBrane with fixed fixpoint (Option A):
- Prembrionic → Embryonic (1 call)
- Embryonic → build tasks, Braning (1 call)
- Stepping stmt_0 body (a search for `1`): Prembrionic → Embryonic → Econstanic? → ... maybe 5-10 calls
- Stepping stmt_1 body (search for `2`): similar
- Stepping stmt_2 body (Op+): Prembrionic → Embryonic → Braning → op or constanic → ... maybe 8-15 calls
- fir_op_step: compute_brane_state → Constant

Step count: ~25-40. Significantly higher. The snapshot says "STEPS: 4" — UBCa produces "STEPS: 37."

### This is not a "match UBC transitions" problem

The FooP says "mirror UBC transitions." But the step count divergence is from the **granularity of work per call**, not from different transition rules. Even if every NYES transition is byte-identical, the outer loop iterates many more times.

### Resolution

Either:
1. **Change the snapshot format** to track a different metric (e.g., "evaluation cost" instead of step count). This would require human re-approval of all 131 snapshots.
2. **Add a step-count normalization** in the cross-check harness that only compares output, not step counts. The spec would need to drop the "step counts must match" constraint.
3. **Bundle transitions** — in `protobrane_step`, when the front task is not constanic, step it **recursively until it is constanic** (effectively a mini-fixpoint within the node). This gives step counts closer to current UBC but breaks "one transition per step." It makes each `step()` call advance one child to completion, which is still bounded and deterministic.

**Recommendation**: Option 3 (bundle to child-completion) is the best path to byte-identical step counts. The `check-then-act` behaviour still works: `step()` either pops a finished front or advances an unfinished front to its next NYES plateau. The front task's own `step()` is called recursively within its own task-list drain until it reaches a stable state. This is exactly what `step_boxed` + `run_to_completion_with_scope` does today — it's just expressed differently.

---

## Concern 3 (SIGNIFICANT): NormalBrane Re-Stepping Differs from Current Model

### The current model

`re_step_brane_bodies` (ubc.rs:262-309):
1. Clones the current statements into a temp `brane_ref`
2. Walks each statement, calling `step_boxed` on the body (full run-to-completion)
3. Creates **new** `StatementFir` instances with fresh states
4. Updates `brane_ref` statements as it goes (so Index/HeadTail see evaluated bodies)
5. Computes `compute_brane_state` on the stepped statements
6. Replaces `brane.statements` wholesale

Key point: statement bodies are **rebuilt from scratch** on each re-step. Their state is reset by `step_boxed`'s internal `run_to_completion_with_scope` which starts fresh.

### The ProtoBrane model

The brane's task queue contains the statement bodies **directly**. They are stepped **in-place** — each `step()` mutates the body's internal NYES, children, etc. When all are constanic, `fir_op_step` (the brane's own work) runs `compute_brane_state` on the **mutated** statements.

### The difference matters

1. **In-place mutation means the bodies accumulate state across calls.** If a statement body goes Prembrionic→Embryonic→Braning→Econstanic (unbound name), and later the name becomes bound, the search needs to **re-step**. The re-step mechanism (clear ubc_children) clears computed results but the body's NYES has already advanced past Embryonic. Does the re-step reset the body's NYES?

2. **Index/HeadTail visibility.** The current model updates `brane_ref` statements as it goes — early statements see already-evaluated early bodies. The task-list model drains tasks sequentially: stmt_0 body is stepped to constanic, then stmt_1 body, etc. When stmt_1's body (say, `#(1)`) tries to index the brane, it needs to see stmt_0's **evaluated** body. In the task-list model, stmt_0's body IS constanic by the time stmt_1 starts. But the Index's search goes through `scope.current_brane` → `scope.current_stmt_idx`. The scope uses the original brane statements, which are updated as we go. This should work IF the scope correctly reflects the current state.

3. **The scope mechanism must be preserved or adapted.** The current model creates a `brane_ref` temporary to thread through `with_brane()`. The ProtoBrane model would need to either use the same self-reference pattern or give searches direct access to the brane's `foolish_children` (which contain the statements). FOOP-62 §2 says "`search_ib`/`search_ab` iterate `foolish_children`" — this suggests direct iteration rather than scope-based lookup. If so, the Index/HeadTail logic changes from scope-mediated to direct child-iteration, which is a **semantic change** that would produce different output.

### Resolution

The NormalBrane stepping needs a detailed design section, not just "mirror UBC." Specifically:
- Where does the scope come from when an Index inside a statement body needs to look at sibling statements?
- How are statement bodies reset for re-stepping?
- Does `fir_op_step` for a brane rebuild statements (like current) or just classify their states?

---

## Concern 4 (SIGNIFICANT): Task-List Granularity for Statement Bodies

### The issue

A brane's `foolish_children` contains statements. But statements are **not** `Fir` nodes — they're `StatementFir` wrappers with `name`, `body: FirRef`, and metadata. The body IS a `Fir`. The task-list model says tasks are `FirRef`s. So the brane's task queue would contain statement bodies, NOT the `StatementFir` wrappers.

But this means the statement's name/line_number metadata is not directly accessible to the task-drain mechanism. When a statement body resolves (becomes constanic), the brane needs to know which statement it belonged to, to update scope. Currently `re_step_brane_bodies` walks statements in order and pushes their names into scope after each evaluation.

### Resolution

Either:
- The brane's `fir_op_step` handles statement iteration (NOT through the task queue), mimicking the current `re_step_brane_bodies` but with one-transition-per-body calls. The task queue is only used for non-brane FIR kinds.
- Or `StatementFir` itself implements `Fir` (wrapping the body), with its own `fir_op_step` that pushes its name into scope after its body is constanic. This seems cleaner and makes statements first-class ProtoBrane nodes.

FOOP-62 §2 says "Statement: `[body]` (len 1)" — implying Statement itself is a ProtoBrane with one `foolish_child`. But the spec doesn't elaborate on how the name and line metadata travel. This needs explicit coverage in the spec.

---

## Concern 5 (MODERATE): Re-Stepping Mechanics Under-Specified

FOOP-62 §4 says: "To recompute: **clear `ubc_children` and re-derive.**"

But what about the task list? The tasks were built during Embryonic and tasks that are already constanic were popped. If we clear `ubc_children` for a re-step, we also need to rebuild the task list. Do all children get re-enqueued? Including children that were already drained?

### The problem

Consider an Operator `+(a, b)` where `a` resolved but `b` was Econstanic. The task list would have drained `a` (now completed) and left `b` (Econstanic, task hangs because it can't advance without a name resolution). When the name for `b` later becomes available (via scope updates), the brane is Woconstanic. The fixpoint loop would call `step()` on the brane.

But wait — in the current model, when a brane is Woconstanic, `re_step_brane_bodies` runs again, which **fully re-evaluates** all statements from scratch. In the ProtoBrane model with in-place mutation, the already-constanic `a` stays constanic and the Econstanic `b` needs to advance. But `b` was already consumed from the task queue (it was the front task, stepped to Econstanic, task not popped because Econstanic is constanic... wait, Econstanic IS constanic per `is_constanic()`).

Actually, re-reading: `is_constanic()` returns true for Econstanic. So in `protobrane_step`, when a child becomes Econstanic, the next call observes it constanic and **pops** it from the queue. The task is gone! When a forward reference later resolves, how does the node know to re-evaluate that child?

The spec needs to address this explicitly. Likely answer: when re-stepping is triggered, the task list is **rebuilt** from `foolish_children`, re-enqueuing all children regardless of current state. Those already constanic would immediately pop; those still needing work would step. Combined with "clear ubc_children and re-derive," this forms a coherent model. But it must be specified.

---

## Concern 6 (MODERATE): SFF/SF Scope Propagation

### The current model

`StayFoolishFir::step_one` calls `step_except_brane_searches` which iterates internally until its child is constanic or Econstanic (blocking brane searches). `StayFullyFoolishFir::step_one` calls `step_boxed` with an SFF-context scope, which runs a full fixpoint.

### In the ProtoBrane model

SF/SFF wrappers have one `foolish_child` (the expr). The task queue drains that child. But the child's stepping needs the **modified scope** (blocking brane searches for SF, SFF context for SFF). Where is this scope modification threaded?

The `step()` method takes `scope: &Scope`. For SF/SFF's children, the scope must carry the modified context. Options:
1. The SF/SFF wrapper creates a modified scope before each child `step()` call. This requires `protobrane_step` to accept a scope modifier, or the wrapper to override `step()`.
2. The child's `step()` checks the scope's `eval_context` directly (already supported) — but for SF, the "block brane searches" flag is separate from eval_context.

The SF wrapper's `step_except_brane_searches` does MORE than just block brane searches — it has its own fixpoint loop with custom termination and per-variant special handling (the `Variant` enum with `UnanchoredSearch`, `AnchoredSearch`, `Operator`, `StayFullyFoolish`, etc.). This is NOT simply a scope flag — it's a fundamentally different evaluation algorithm.

### Resolution

Either:
- The SF/SFF wrappers override `step()` to implement their custom algorithm (defeating the "written once" goal)
- Or the `step_except_brane_searches` logic is redesigned to work through scope flags + standard step (possible but requires analysis of whether the current special handling is reducible to scope flags)

This is the one area where the "one algorithm fits all" claim of the ProtoBrane model is most strained. The SF/SFF wrappers genuinely do different things. The spec should acknowledge this and specify how they fit.

---

## Concern 7 (MODERATE): Parent Weak Self-Reference Construction

FOOP-62 §5 says: "the root node's parent is a Weak pointing at itself, constructed via `Rc::new_cyclic`."

The current design uses `Rc<RefCell<dyn Steppable>>` — a trait-object Rc. `Rc::new_cyclic` requires a closure that receives a `&Weak<T>` and returns `T`. For trait objects, the closure would need to return `Box<dyn Steppable>` or `Fir` enum. This is doable:

```rust
let root: FirRef = Rc::new_cyclic(|weak_self| {
    RefCell::new(Fir::NormalBrane(Box::new(NormalBraneFir {
        parent: ParentPtr(weak_self.clone()), // wait, ParentPtr takes FirRef, not Weak
        ...
    })))
});
```

The issue is the type mismatch: `Rc::new_cyclic` gives a `Weak<T>` where `T = RefCell<dyn Steppable>`. The parent needs to be embedded in the inner struct. The struct is accessed via `RefCell<dyn Steppable>` — getting the `Weak` requires the outer Rc, which `new_cyclic` provides. But storing the `Weak<RefCell<dyn Steppable>>` INSIDE the struct requires the struct to know about `Weak<RefCell<dyn Steppable>>` — which is possible if the struct field is `Weak<RefCell<Fir>>` (not `dyn Steppable`).

Wait — FOOP-62 proposes `parent: Weak<RefCell<Fir>>` where `Fir` is the **enum** (not the trait object). This is a concrete type, so `Weak` works. The `FirRef` type alias changes from `Rc<RefCell<dyn Steppable>>` to `Rc<RefCell<Fir>>`. This is significant because:
- It eliminates the need for `Steppable` trait objects for node access
- It means `Fir` is the concrete type, not a trait-object wrapper
- But it also means `Fir` must implement the `step()`/`fir_op_step()` methods directly (dispatch via enum matching, not vtable)

The spec is ambiguous about whether `FirRef` remains trait-object-based or becomes enum-based. Given the `bon` builder discussion, it seems like the plan is for `Fir` to be an enum. This is a design point that needs explicit clarification — it has deep implications for how method dispatch works.

---

## Concern 8 (MINOR): Sequencer Re-Expression Risk

FOOP-62 §8 says the sequencer must be re-expressed over `kind()` + child iterators + leaf accessors, replacing `FirQueryable`. Output must be **byte-exact** against 131 approved snapshots. The spec acknowledges this risk:

> "If the FirQueryable retirement proves to make byte-exact output materially harder than keeping a thin query shim, the fallback is to keep a minimal query adapter rather than risk the output"

This is good hedging. The specific risks are:

1. **`result=` positioning**: The sequencer's `proto_brane_formatter_with_result` renders `result=` items FIRST, then non-result items. In the two-store model, `ubc_children` IS the result — the mapping is direct. But the exact formatting (commas, trailing commas, indentation) must match byte-for-byte.

2. **Search display**: Current output for Search includes `pattern='xxx'`, `ANCHORED`/`UNANCHORED`, NYES state, and `result=`. The leaf accessors on the Search provide pattern/direction/anchored; the `ubc_children` provides the result. This should map cleanly.

3. **Transparent rendering**: For Constant/Independent nodes, the sequencer renders the child transparently (e.g., an operator reduces to its result). This logic depends on `hs_state()`. The replacement uses the node's `get_nyes()` via the uniform accessor — should be equivalent.

4. **Concatenation rendering**: Current output has special `⨃` prefix for constanic concats, and `elements=N`/`merged=...` for non-constanic. The two-store model maps elements→foolish_children and merged→ubc_children. The sequencer would use `kind() == Concatenation` + `get_nyes()` + child iterators to reproduce this.

**Recommendation**: Keep `FirQueryable` as a thin adapter in UBCa initially, prove byte-exactness, THEN retire it. The spec already allows this fallback — make it the default plan rather than a fallback.

---

## Concern 9 (MINOR): `Woconstanic` Short-Circuit in Search

The current `SearchFir::wo_short_circuit_self` (fir.rs:1770-1789) follows a chain of Woconstanic targets and shortcuts to the end. This is a performance optimization that also affects what the sequencer sees (the short-circuited target is different from the intermediate chain).

In the ProtoBrane model, a search's result is in `ubc_children`. The Woconstanic chain is implicit (the result points to another search which points to another...). The short-circuit needs to be preserved or the sequencer output changes. If the sequencer recursively renders `ubc_children`, the chain would still be visible. If the search copies the short-circuited value into its own `ubc_children`, the chain collapses. Either way produces different output unless explicitly matched to current behavior.

The spec should mention whether Woconstanic short-circuiting is preserved and how.

---

## Concern 10 (MINOR): `bon` Value-to-Builder Bridge

The spec says `bon` does "NOT auto-generate the value→builder bridge," requiring each payload to have a hand-written `updater(self) -> XFirBuilder`. This works but:

1. Each of the ~10 FIR payloads needs its own `updater`. This is ~30 lines of boilerplate per type, defeating some of the value of `bon`. Tooling (a macro) could generate it, but adds complexity.

2. The updater consumes `self` — the source is destroyed. For constanic clone, the source is a value extracted from the FIR enum (which was cloned). This works because constanic clone creates a clone via `clone_into_fir()` first, then updates that clone. But `clone_into_fir()` already allocates a new value — so the updater consumes that allocation and creates another. It's an extra allocation per clone.

3. If `FirRef` is `Rc<RefCell<Fir>>` (the enum), then constanic clone extracts the enum value (via `clone()`), updates it, and wraps it in a new `Rc<RefCell<>>`. This is fine but worth noting.

---

## Positive Observations

Despite the concerns above, the design gets many things right:

1. **Two-store split is semantically perfect.** `foolish_children` as parse-time structure, `ubc_children` as computation results — this directly encodes the AGENTS.md "Foolish Semantic Immutability" section into the type system.

2. **Immutable parent** paired with `clone_with_parent` for re-parenting correctly expresses the detach-and-recoordinate semantics.

3. **Builder-only construction** via privacy + `#[non_exhaustive]` is a clean way to enforce invariants at compile time.

4. **Clone-and-gut with UBC oracle** is the right approach for a representation change of this magnitude.

5. **No mutable child iterator** (Rejected Alternative C) is the correct decision — the task-list model avoids the `RefMut` aliasing hazard elegantly.

6. **The narrow `Fir` trait** (just `step`, `fir_op_step`, `kind`, + leaf accessors) is a massive improvement over the 50-method `Steppable` god-trait.

7. **Separate workspace crate `foolish-ubca`** (mirroring `foolish-ubcb`) keeps the oracle untouched and gives a clean comparison point.

8. **Test-first development** with the cross-check harness running each `.foo` through both UBC and UBCa is the right acceptance strategy.

---

## Consolidated Recommendations

1. **Fix the fixpoint loop** (Concern 1) — add a stall counter or progress indicator. Update §7 to reflect the change.

2. **Reconcile step counts** (Concern 2) — either bundle child-transitions per call, or drop the "step counts must match" constraint and update the snapshots.

3. **Specify NormalBrane re-stepping** (Concern 3) — detail how stmt bodies are reset, how scope is built incrementally, and how Index/HeadTail see evaluated siblings.

4. **Clarify Statement-as-FIR** (Concern 4) — decide whether `StatementFir` becomes a `Fir` variant and specify how it contributes name/line metadata to scope.

5. **Specify re-step task-list rebuild** (Concern 5) — explicitly state that re-stepping rebuilds the task list from `foolish_children`.

6. **Address SFF/SF special-casing** (Concern 6) — acknowledge that SF/SFF wrappers have genuinely different evaluation and specify whether they override `step()` or work through scope flags.

7. **Clarify `FirRef` type** (Concern 7) — specify whether `FirRef` is `Rc<RefCell<Fir>>` (enum-based) or stays `Rc<RefCell<dyn Steppable>>` (trait-object-based), with the implications for parent `Weak` and method dispatch.

8. **Keep `FirQueryable` initially** (Concern 8) — prove byte-exactness before retiring the trait.

9. **Document `Woconstanic` short-circuit** (Concern 9) — specify whether and how it is preserved.

10. **Consider a macro for `updater`** (Concern 10) — reduce hand-written boilerplate per payload type.

---

## Overall Assessment

**Design quality**: 8/10 — the architecture is elegant and well-motivated.
**Implementation feasibility**: 6/10 — fixable issues, but the step-count/state-change granularity mismatch (Concerns 1-2) will cause significant friction during the "iterate until byte-identical" phase.
**Estimated effort**: Phase 2-4 (implementation + iteration) will take 2-3× the time the spec suggests, primarily due to Concern 2 (step count matching).

The specification needs its "retained unchanged" claims reviewed against the mechanical reality of the one-transition model, and the NormalBrane re-stepping needs a dedicated section. Once those are resolved, the implementation is straightforward and the architecture is genuinely better than the status quo.

---

## Last Updated

**Date**: 2026-06-09
**Updated By**: Deepseek V4 Pro (via opencode)
**Changes**: Initial design review of FOOP-62. Identified 10 concerns spanning fixpoint compatibility, step count matching, NormalBrane re-stepping, scope propagation, and sequencer byte-exactness risk. All are fixable but require specification updates before implementation.