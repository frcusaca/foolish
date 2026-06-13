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
