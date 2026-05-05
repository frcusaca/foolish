# Phase 3 — Concatenation

> Goal: Implement the concatenation operator `A B C ...` per FOOP=3
> (revised). Concatenation produces a new merged brane of
> `constanic_clone`'d copies of each input, in order, and delegates
> further `step()` calls to that merged brane.

Read [FOOP=3](../../../../foop/FOOP=3.md) for the full design rationale.

---

## Phase 3 Deliverable

A `Concatenation` FIR variant plus a step rule implementing the FOOP=3 algorithm:

1. Each element FIR has reached a constanic terminal state.
2. The step constructs a new `NormalBrane` whose statements are
   `constanic_clone`'d copies of each input's statements, in order.
3. Subsequent `step()` calls delegate to the merged brane.

The Phase 1 compiler is updated:
- `Concatenation { elements }` now compiles to `Concatenation { elements, .. }`
- Removed from Phase 1's compile-time rejection list.

---

## Step Algorithm

```rust
fn step_concatenation(fir: &FirRef) -> Result<(), UbcError> {
    // Precondition: each element has stepped to constanic terminal
    // (Phase 2's depth-first ordering guarantees this)

    // NK propagation
    if let Fir::Concatenation { elements, .. } = &*fir.borrow() {
        if elements.iter().any(|e| e.borrow().state == Nyes::Nk) {
            fir.borrow_mut().state = Nyes::Nk;
            return Ok(());
        }
    }

    // Construct merged statements
    let merged_statements: Vec<StatementFir> = {
        let this = fir.borrow();
        if let Fir::Concatenation { elements, .. } = &*this {
            elements.iter().flat_map(|elem| {
                let brane = deref_to_brane(elem);
                if let Fir::NormalBrane { statements, .. } = &*brane.borrow() {
                    statements.iter().map(|stmt| {
                        let cloned = constanic_clone(&stmt.body);
                        StatementFir {
                            name: stmt.name.clone(),
                            body: cloned,
                            state: Nyes::Embryonic,
                        }
                    }).collect::<Vec<_>>()
                } else {
                    vec![]
                }
            }).collect()
        } else {
            vec![]
        }
    };

    let merged_brane = Rc::new(RefCell::new(Fir::NormalBrane {
        characterizations: vec![],
        statements: merged_statements,
        state: Nyes::Embryonic,
    }));

    // Delegate to merged brane
    if let Fir::Concatenation { merged, state, .. } = &mut *fir.borrow_mut() {
        *merged = Some(Rc::clone(&merged_brane));
        *state = Nyes::Embryonic;
    }

    Ok(())
}
```

After this step, ordinary brane stepping takes over. Constanic clones inside the
merged brane re-step in the merged context.

---

## FIR Additions

```rust
// Added to the Fir enum:
Concatenation {
    elements: Vec<FirRef>,
    merged: Option<FirRef>,
    state: Nyes,
}
```

Roundtrip test required: construct a `Concatenation`, encode to JSON, decode,
compare structurally.

---

## Tests

Existing `.foo` files:
- `concatenationBasics.foo`
- `concatenationResolution.foo`
- `concatenationSearch.foo`
- `concatenationResolutionAdv.foo`

New dedicated tests:
- `p3_concat_simple.foo` — basic merge
- `p3_concat_resolution.foo` — recoordination resolving searches
- `p3_concat_order_matters.foo` — left-to-right order sensitivity
- `p3_concat_chain.foo` — `A B C` produces single merged brane

---

## Phase 3 Exit Criteria

- All 4 concatenation `.foo` tests pass.
- All 4 new dedicated tests pass.
- `Concatenation` roundtrips through serde.
- A test demonstrates recoordination producing different results than original.
- A test demonstrates left-to-right order sensitivity.

---

## Last Updated

**Date**: 2026-05-05
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — Rust Phase 3 concatenation plan. Adapted from Scala
version with Rc<RefCell<Fir>> patterns and Rust match syntax.
