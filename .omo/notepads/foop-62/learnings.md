# FOOP-62 Learnings

## 2026-06-11 - Initial State
- UBCa crate exists at `foolish/foolish-ubca/` with delegation to UBC
- UBC (foolish-core) has ~70+ `.snap.new` regressions on alpha
- UBCa passes all tests because it delegates to UBC
- Goal: Build ProtoBrane implementation, pass all approved snapshots

## Key Architecture Decisions (from spec)
- `struct ProtoBrane` = shared field-holder with inherent methods
- `trait Fir` = dyn-dispatch surface (core/core_mut/fir_op_step/kind/leaf accessors)
- `FirRef = Rc<RefCell<dyn Fir>>` replaces enum + clone_into_fir
- `step_fir_ref(&FirRef, &Scope)` = FREE function, transient borrows
- Task queue = `VecDeque<FirRef>`, pop predicate = `is_settled()`
- `foolish_children` = immutable (no public mutator)
- `ubc_children` = mutable (push/clear only)
- Parent = `Weak<RefCell<dyn Fir>>`, immutable after construction
- Root parent = self-Weak, detected via `is_root()`
- Construction = nested `Rc::new_cyclic`
- Builders = `bon` crate, parent REQUIRED
- Scope = capability surface (search_ib/search_ab/index/how_ignorant/emit)
- Sequencer = thin FirQueryable adapter over ProtoBrane (first pass)
