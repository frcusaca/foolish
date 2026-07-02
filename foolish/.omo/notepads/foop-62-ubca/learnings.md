# FOOP-62 UBCa Learnings

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
