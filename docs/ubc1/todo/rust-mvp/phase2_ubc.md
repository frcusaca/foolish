# Phase 2 — UBC: Depth-First Sequential Step Evaluation

> Goal: Read FIRs (in memory or via serde deserialization), step the FIR tree
> until every node reaches a constanic state (CONSTANT, INDEPENDENT, ECONSTANIC,
> WOCONSTANIC) or NK. The hard part is **search short-circuiting** through
> WOCONSTANIC chains and **constanic cloning** when search results need to be
> placed in their searcher's context.

> Phase 2 is **depth-first, sequential, no message passing**. We adopt UBC2's
> Nyes lifecycle and FIR taxonomy but evaluate by direct function-call stepping.
> Breadth-first parallel evaluation is deferred to Phase 5. See FOOP=6.

Read [00_accumulated_specs](../../scala-mvp/00_accumulated_specs.md) for the Nyes state
definitions before reading this document.

---

## Phase 2 Deliverable

A `Ubc::step(fir: &mut Rc<RefCell<Fir>>) -> Result<(), UbcError>` function that
performs one evaluation step on a FIR tree, returning the (possibly partly)
advanced FIR. Plus a `Ubc::run_to_completion(fir: &Rc<RefCell<Fir>>) -> Result<(), UbcError>`
that loops `step` until no FIR can make progress.

The 60 `.foo` approval tests move into Phase 2 as the validation suite.

---

## Evaluation Order: Depth-First, Left-to-Right, Sequential

Phase 2 evaluates a brane as follows:

1. Walk the brane's statements **left-to-right** (the order they appear in the source).
2. For each statement, **step its body to a constanic terminal state** before
   moving to the next statement.
3. Stepping a body that is itself a brane recurses into it, walking *its*
   statements depth-first left-to-right.
4. There is no concurrency, no parallelism, no message passing.

This guarantees that **when statement N is being evaluated, all statements 0..N-1
in the same brane have already reached constanic terminal states**.

---

## FIR Interior Mutability in Rust

FIR objects carry mutable state. In Rust, we use `Rc<RefCell<Fir>>` for shared
ownership with interior mutability:

```rust
use std::cell::RefCell;
use std::rc::Rc;

type FirRef = Rc<RefCell<Fir>>;

fn step_to_completion(fir: &FirRef) -> Result<(), UbcError> {
    while !fir.borrow().state.is_constanic() && fir.borrow().state != Nyes::Nk {
        step(fir)?;
    }
    Ok(())
}

fn step(fir: &FirRef) -> Result<(), UbcError> {
    let fir_mut = fir.borrow_mut();
    match &mut *fir_mut {
        Fir::NormalBrane { statements, state, .. } => {
            // Step children, update state
            // Cannot access children here because we hold a borrow_mut on the parent.
            // See below for the correct pattern.
        }
        _ => {}
    }
    Ok(())
}
```

**Borrow checker pattern for stepping branes:**

Because we need to mutate children while the parent's state depends on children's
final states, the step function works by borrowing each child independently:

```rust
fn step_brane(fir: &FirRef) -> Result<(), UbcError> {
    let mut this = fir.borrow_mut();
    if let Fir::NormalBrane { statements, state, .. } = &mut *this {
        drop(this);  // Release borrow before stepping children
        for child in statements {
            step_to_completion(child)?;
        }
        // Re-borrow to update state based on children's final states
        let mut this = fir.borrow_mut();
        if let Fir::NormalBrane { state, statements, .. } = &mut *this {
            *state = compute_brane_state(statements);
        }
    }
    Ok(())
}

fn compute_brane_state(statements: &[StatementFir]) -> Nyes {
    if statements.iter().all(|s| matches!(s.state, Nyes::Constant | Nyes::Independent)) {
        Nyes::Constant
    } else if statements.iter().any(|s| matches!(s.state, Nyes::Econstanic | Nyes::Woconstanic)) {
        Nyes::Woconstanic
    } else {
        Nyes::Braning
    }
}
```

---

## The Hard Part — Search Short-Circuiting and Constanic Coordination

### The Worked Example

```
{
  y = z,
  x = y,
  w = x,
  v = w + x,
  u = v + w
}
```

Walk through this statement by statement, applying the rules below.

### Statement 1: `y = z`

`z` is an unanchored backward search. The search exhausts the scope chain finding
nothing.

After stepping:
- `y`'s body (a `Search`) — state = `ECONSTANIC`
- `y` (the `Statement`) — state = `WOCONSTANIC`

### Statement 2: `x = y`

Search for `y` succeeds — finds `y = z` from line 1.

`constanic_clone` is invoked on the ECONSTANIC search result, producing a clone
reset to EMBRYONIC, parented to `x`'s brane.

After stepping:
- `x`'s body — `target` points at the clone of `y`'s ECONSTANIC search
- `x` — state = `WOCONSTANIC`

### Statement 3: `w = x` — Search Short-Circuiting

Search for `x` finds line 2's `x = y`. The search result is `x`'s body — a
`Search` whose `target` points at the clone of `y`'s ECONSTANIC.

`x`'s body is WOCONSTANIC. Search short-circuiting kicks in:

> When a search resolves to a target whose state is WOCONSTANIC, follow the
> target's `target` chain through subsequent WOCONSTANICs until you arrive at
> the underlying ECONSTANIC (or CONSTANT). Rewrite the new search's `target`
> field to point directly at the chain's end.

After stepping:
- `w`'s body — `target` points DIRECTLY at the cloned ECONSTANIC (not at `x`'s body)

### Search Short-Circuit Algorithm

```rust
fn short_circuit(search: &FirRef) {
    let mut current = search.borrow().target.clone();
    while let Some(ref target) = current {
        let state = target.borrow().state;
        if state != Nyes::Woconstanic {
            break;
        }
        // Follow the chain
        current = target.borrow().target.clone();
    }
    search.borrow_mut().target = current;
}
```

---

## Constanic Cloning — `constanic_clone`

**Calling contract** (FOOP=7):

> Every search result is `constanic_clone`'d before being assigned to the
> Search FIR's result field. UBC stepping, applied iteratively, takes care
> of all subsequent state transitions.

```rust
fn constanic_clone(source: &FirRef) -> FirRef {
    match source.borrow().state {
        Nyes::Constant | Nyes::Independent | Nyes::Nk => {
            // Share reference — terminal, immutable
            Rc::clone(source)
        }
        Nyes::Econstanic => {
            // Clone, reset to EMBRYONIC
            let clone = deep_clone(source);
            clone.borrow_mut().state = Nyes::Embryonic;
            clone
        }
        Nyes::Woconstanic => {
            // Clone with recursively-cloned children, reset to BRANING
            let clone = deep_clone(source);
            clone.borrow_mut().state = Nyes::Braning;
            clone
        }
        _ => panic!("constanic_clone called on nigh FIR — caller bug"),
    }
}
```

The `deep_clone` function recursively clones the FIR tree, replacing child
references with cloned copies. It must handle cycles (impossible by Foolish's
writing-order rule, but a visited set prevents surprises).

---

## Per-FIR Step Rules

### `ConstantInt`

Already INDEPENDENT. `step()` is a no-op.

### `NKFir`

Already NK. `step()` is a no-op.

### `NormalBrane`

```rust
fn step_brane(fir: &FirRef) -> Result<(), UbcError> {
    let state = fir.borrow().state;
    match state {
        Nyes::Prembrionic => { fir.borrow_mut().state = Nyes::Embryonic; }
        Nyes::Embryonic => { fir.borrow_mut().state = Nyes::Braning; }
        Nyes::Braning => {
            drop(state);
            // Step all statements depth-first
            if let Fir::NormalBrane { statements, .. } = &*fir.borrow() {
                for stmt in statements {
                    step_to_completion(&stmt.body)?;
                    // Update statement's state to mirror body
                    stmt.state = stmt.body.borrow().state.clone();
                }
                // Compute brane state from children
                let brane_state = compute_brane_state(statements);
                fir.borrow_mut().state = brane_state;
            }
        }
        _ => {}  // Already terminal
    }
    Ok(())
}
```

### `Statement`

```rust
fn step_statement(stmt: &StatementFir) -> Result<(), UbcError> {
    step_to_completion(&stmt.body)?;
    stmt.state = stmt.body.borrow().state;
    Ok(())
}
```

### `BinaryOp`

```rust
fn step_binary_op(fir: &FirRef) -> Result<(), UbcError> {
    // Step operands
    if let Fir::BinaryOp { op, left, right, state } = &*fir.borrow() {
        step_to_completion(left)?;
        step_to_completion(right)?;

        let ls = left.borrow().state;
        let rs = right.borrow().state;

        match (ls, rs) {
            (Nyes::Nk, _) | (_, Nyes::Nk) => {
                *state = Nyes::Nk;
            }
            (cs, _) | (_, cs) if cs.is_constanic() => {
                *state = Nyes::Woconstanic;
            }
            (Nyes::Constant | Nyes::Independent, Nyes::Constant | Nyes::Independent) => {
                // Perform computation
                let result = compute_binary(op, left, right)?;
                // Replace this FIR with the result
                *fir = Rc::new(RefCell::new(result));
            }
            _ => { *state = Nyes::Braning; }
        }
    }
    Ok(())
}
```

### `Search` (unanchored)

```rust
fn step_search_unanchored(fir: &FirRef) -> Result<(), UbcError> {
    let state = fir.borrow().state;
    match state {
        Nyes::Embryonic => {
            // Walk IB backward, then AB chain
            let result = scope_walk(fir);
            match result {
                None => { fir.borrow_mut().state = Nyes::Econstanic; }
                Some(found) => {
                    let cloned = constanic_clone(&found);
                    fir.borrow_mut().target = Some(cloned);
                    // Update state based on clone's state
                    let clone_state = cloned.borrow().state;
                    if clone_state.is_constanic() {
                        if clone_state == Nyes::Constant || clone_state == Nyes::Independent {
                            fir.borrow_mut().state = Nyes::Constant;
                        } else {
                            short_circuit(fir);
                            fir.borrow_mut().state = Nyes::Woconstanic;
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
```

### `Search` (anchored)

```rust
fn step_search_anchored(fir: &FirRef) -> Result<(), UbcError> {
    if let Fir::Search { anchor: Some(ref anchor), .. } = *fir.borrow() {
        step_to_completion(anchor)?;
        let anchor_state = anchor.borrow().state;
        match anchor_state {
            Nyes::Nk => { fir.borrow_mut().state = Nyes::Nk; }
            Nyes::Constant | Nyes::Independent => {
                // Search only within anchor's brane
                let result = search_local(fir, anchor);
                match result {
                    None => { fir.borrow_mut().state = Nyes::Nk; }
                    Some(found) => {
                        let cloned = constanic_clone(&found);
                        // ... same as unanchored
                    }
                }
            }
            _ => { fir.borrow_mut().state = Nyes::Nk; }
        }
    }
    Ok(())
}
```

---

## What Phase 2 Does NOT Do

- **No concatenation** — Phase 3 (done separately)
- **No detachment** — Phase 7 (SF/SFF were pulled forward and implemented in P2)
- **No breadth-first / parallel** — Phase 5
- **No wake-up queues, no message passing**

## SF/SFF/Seek — Pulled Forward from Phase 7

SF (`<expr>`), SFF (`<<expr>>`), and unanchored/anchored seeks (`#-N`) were
originally planned for Phase 7 but proved implementable as part of Phase 2.
See [phase2_sf_sff_seek_insights.md](phase2_sf_sff_seek_insights.md) for
implementation details and lessons learned.

These features required:
- Two new FIR variants: `StayFoolish` and `StayFullyFoolish`
- `Scope` extension with `current_brane`, `current_stmt_idx`, and `block_brane_searches`
- `constanic_clone(permit_nye: bool)` flag for SF/SFF coordination
- `step_except_brane_searches()` for SF semantics
- `strip_sf_wrapper()` for arithmetic operand unwrapping
- Unanchored seek resolution via `Scope.current_brane` + `current_stmt_idx`

---

## Tests

The 60 active `.foo` files become Phase 2's validation suite.

Test harness: a Rust module implementing approval test infrastructure that:
1. Reads `.foo` files from `test-resources/`
2. Pipes source through `Compiler::compile()` then `Ubc::run_to_completion()`
3. Formats output using `Sequencer`
4. Compares against `.approved.foo` files

Output lives in `foolish/foolish-core/src/test/resources/org/foolish/fvm/rubc/`.

---

## Phase 2 Exit Criteria

- All 60 active `.foo` approval tests pass.
- The worked example brane has a unit test asserting the expected final-state table.
- `step` is idempotent on terminal states.
- `constanic_clone` has unit tests for each of the 5 input states.

---

## Last Updated

**Date**: 2026-05-05
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — Rust Phase 2 UBC plan. Adapted from Scala version:
added Rust-specific borrow checker patterns for Rc<RefCell<Fir>>, showed drop()
pattern for releasing borrows before stepping children, used Rust match syntax
for step rules.
