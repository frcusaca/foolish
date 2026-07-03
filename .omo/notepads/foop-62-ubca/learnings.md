# Learnings — FOOP-62 UBCa Foundation

## Key Patterns

### `Rc::new_cyclic` for parent wiring
- Must call on **concrete types** (`Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| ...)`)
- `Rc::new_cyclic` requires `T: Sized` — cannot use with `dyn Fir` directly
- Unsized coercion from `Rc<RefCell<ConcreteType>>` to `Rc<RefCell<dyn Fir>>` works at `let` binding with type annotation
- Same for `Weak` coercion: `Weak<RefCell<BraneFir>>` → `Weak<RefCell<dyn Fir>>`

### `Weak::new()` does NOT work with `dyn Trait`
- `Weak::new()` requires `T: Sized` — `dyn Fir` is `!Sized`
- Cannot create an "empty" `Weak<RefCell<dyn Fir>>`
- Always use `Rc::new_cyclic` to get a valid `Weak<dyn Fir>`

### `step_fir_ref` debug_assert invariant
- The invariant `front_task.is_some() || nyes.is_settled()` only holds when `fir_op_step` produces a settled state or pushes more tasks
- For test stubs that advance one step at a time, this invariant breaks
- Fix: either make `fir_op_step` produce terminal state in one call, or remove the assert

### NYES-driven stepping with task queue
- Brane: Prembrionic → Braning (push tasks at the same time)
- Leaf: Prembrionic → Woconstanic (immediate, no children to drain)
- Task queue pop condition: `is_settled()` (= `is_constanic() || == Nk`)
- `step_fir_ref` is a FREE FUNCTION (not trait method) for borrow discipline
- Transient borrows: peek front under short borrow, drop, then recurse

### Clippy considerations
- `pub(crate)` methods used only in `#[cfg(test)]` still trigger `dead_code` in non-test compilation
- Use `#[allow(dead_code)]` for methods intended for future FIR implementations
- `!x.is_none()` → `x.is_some()` (clippy `nonminimal_bool`)

## File Structure

```
foolish-ubca/
├── Cargo.toml          (edition 2024, deps: foolish-core, serde, thiserror)
└── src/
    ├── lib.rs           (module declarations + re-exports)
    ├── nyes_ext.rs      (NyesExt trait: is_settled())
    ├── proto_brane.rs   (ProtoBrane struct + inherent methods)
    └── fir_trait.rs     (Fir trait + FirRef + Scope stub + UbcError + StepReport + step_fir_ref)
```

## RefCell Borrow Discipline Fix (2026-06-13)

**Problem**: `step_fir_ref_inner` held `borrow_mut()` on `this` for the entire `fir_op_step` call. When `fir_op_step` (SearchFir, IndexFir, HeadTailFir) walked the parent chain and borrowed a sibling that IS `this`, it panicked: "already mutably borrowed".

**Root Cause**: `Rc::clone(this)` does NOT clone the RefCell — it's the same RefCell. So `self_clone.borrow_mut()` ≡ `this.borrow_mut()`.

**Fix**: Interior mutability in ProtoBrane + shared borrows in stepping:
- `nyes: Nyes` → `Cell<Nyes>` (Nyes is Copy)
- `tasks: VecDeque<FirRef>` → `RefCell<VecDeque<FirRef>>`
- `ubc_children: Vec<FirRef>` → `RefCell<Vec<FirRef>>`
- `fir_op_step(&mut self)` → `fir_op_step(&self)`
- `step_fir_ref_inner` uses `borrow()` (shared) instead of `borrow_mut()`
- Removed `core_mut()` from Fir trait (no longer needed)

**Key Insight**: Multiple shared borrows (`Ref`) on the same RefCell are permitted. When `fir_op_step` walks the parent chain and borrows a sibling that IS the node being stepped, it gets a second shared borrow — which is fine. The panic only occurs with mixed mutable + shared borrows.

**SearchFir::found_body**: Changed from `Option<FirRef>` to `RefCell<Option<FirRef>>` for interior mutability.

**ubc_children() API Change**: Returns `Vec<FirRef>` (cloned) instead of `&[FirRef]` because the underlying data is behind RefCell. Callers that index into the result need a local variable.

## Search Scoping Fix (2026-06-14)

**Root cause**: `matches_pattern()` used unanchored regex as fallback. `extract_simple_name("^a$")` stripped anchors to `"a"`, then `Regex::new("a")` matched `"ac"` (containing `a`).

**Fix**: Use `&self.pattern` (full anchored pattern like `"^a$"`) instead of `extract_simple_name(&self.pattern)`. The regex `^a$` correctly rejects `"ac"`.

**Key insight**: `matches_pattern()` has two paths - exact match and regex fallback. For simple names like `"a"`, the exact match fails against statement names, and the unanchored regex `/a/` falsely matches any name containing `a`. Anchoring the regex (`^a$`) fixes this.

**Debug approach**: Added eprintln! tracing to the search loop to see which brane children were searched and which matched. The output `[1] name='ac' ... -> FOUND!` revealed the false match immediately.
