# NYES Tracing — Stepping the FVM and Reading State

How to step the FVM to the offending line (or to settlement) while recording the NYES sequence. This is step 2 of the workflow. NYES (Not Yet Evaluated State) is the primary observability surface — it tells you exactly what every FIR is doing at every step.

---

## The NYES states

Defined at `foolish-core/src/fir.rs:116-133`:

| State | Class | Meaning |
|-------|-------|---------|
| `PREMBRIONIC` | Pre-constanic (nigh) | Initial state. No stepping has happened yet. |
| `EMBRYONIC` | Pre-constanic | Stepping begun. The FIR is building its task queue (e.g. a search is resolving its anchor). |
| `BRANING` | Pre-constanic | Stepping in progress. Draining child tasks / running searches. |
| `ECONSTANIC` | Constanic (terminal) | Search performed, nothing found. *May gain a value via recoordination in a new context.* Unanchored search miss. |
| `WOCONSTANIC` | Constanic (terminal) | Search found a result, but the found dependency is itself ECONSTANIC/WOCONSTANIC. "Waiting on constanic." |
| `CONSTANT` | Constanic (terminal) | Fully evaluated to a genuine value. |
| `INDEPENDENT` | Constanic (terminal) | Self-contained constant — no context dependencies (e.g. a literal integer). |
| `NK` | Constanic (terminal) | `???` — provably unfindable. Anchored search miss, division by zero, out-of-bounds index. Terminal. |

**Predicates** (`foolish-core/src/fir.rs:135-177`):
- `is_constanic()` — true for `ECONSTANIC | WOCONSTANIC | CONSTANT | INDEPENDENT | NK`.
- `is_nye()` — true for `PREMBRIONIC | EMBRYONIC | BRANING` (pre-constanic).

**The monotone-progression contract:** once a FIR becomes constanic, it must never regress to pre-constanic. This is enforced by `assert_progression` (see below). If you see a regression, that is a bug.

---

## Reading a FIR's current NYES

```rust
let nyes: Nyes = node.borrow().core().get_nyes();
```

- `core()` — `Fir` trait method (`fir_trait.rs:101`), returns `&ProtoBrane`.
- `get_nyes()` — `ProtoBrane` method (`proto_brane.rs:155`), reads from an interior `Cell<Nyes>`. `Nyes` is `Copy`.

Common assertions:
```rust
assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);  // exact variant
assert!(node.borrow().core().get_nyes().is_constanic());         // any terminal state
assert_eq!(node.borrow().core().get_nyes(), Nyes::Nk);            // specific terminal
```

---

## The single-step primitive

```rust
fn step(&self, scope: &Scope) -> Result<StepReport, UbcError>
```

Defined on the `FirRefExt` trait (`fir_trait.rs:306`), implemented at `:326`. Call it as `node.step(&scope)`.

**`StepReport`** (`fir_trait.rs:22-27`):
```rust
pub enum StepReport {
    NoProgress,
    Progress(Nyes),  // the node's NYES after this step
}
```

One `step()` call performs **one unit of work**: it peeks the front of the task queue; if the front child is constanic it pops it, otherwise it recurses into the child; if the queue is empty it runs `fir_op_step` (the kind-specific combining work). See `step_inner` (`fir_trait.rs:335`).

### One-step-at-a-time (manual control)

Use this when you need to inspect state between specific steps — e.g. to see a FIR enter `EMBRYONIC` before it reaches `BRANING`:

```rust
let scope = Scope::empty();
assert_eq!(search.borrow().core().get_nyes(), Nyes::Prembrionic);

search.step(&scope).unwrap();  // PREMBRIONIC → ?
eprintln!("after step 1: {:?}", search.borrow().core().get_nyes());

search.step(&scope).unwrap();  // → ?
eprintln!("after step 2: {:?}", search.borrow().core().get_nyes());

let report = search.step(&scope).unwrap();
eprintln!("after step 3: {:?}", search.borrow().core().get_nyes());
assert!(matches!(report, StepReport::Progress(Nyes::Constant)));
```

### Run to settlement with a simple loop

```rust
let scope = Scope::empty();
for _ in 0..20 {
    let report = node.step(&scope).unwrap();
    if let StepReport::Progress(nyes) = report
        && nyes.is_constanic()
    {
        break;
    }
}
assert!(node.borrow().core().get_nyes().is_constanic());
```

---

## `step_to_settled` — record the full NYES trace

This helper (`fir_kinds.rs:2561`) steps up to 50 times, recording the NYES after each step, and returns the full sequence. **This is the standard debugging tool** — it shows you the exact progression from `PREMBRIONIC` to terminal.

```rust
fn step_to_settled(node: &FirRef, scope: &Scope) -> Vec<Nyes> {
    let mut transitions = vec![node.borrow().core().get_nyes()];
    for _ in 0..50 {
        let report = node.step(scope).unwrap();
        match report {
            StepReport::Progress(nyes) => {
                transitions.push(nyes);
                if nyes.is_constanic() { break; }
            }
            StepReport::NoProgress => break,
        }
    }
    transitions
}
```

Usage:
```rust
let trace = step_to_settled(&root, &Scope::empty());
eprintln!("root NYES trace: {trace:?}");
// Example output: [Prembrionic, Embryonic, Braning, Braning, Constant]
```

The trace always starts with `PREMBRIONIC` (the seed) and ends with the terminal state (or the last state before `NoProgress`). If it ends on a pre-constanic state, the FVM got stuck — that is a bug.

---

## `step_watching` — watch the offending line as the root steps

This helper (`fir_kinds.rs:3605`) steps the **root** while recording the NYES of **both** the root and a specific child (the one you suspect) at every step. This is the killer snippet for "stop at the offending line" — you see exactly when the suspect enters each state.

```rust
fn step_watching(root: &FirRef, watched: &FirRef, scope: &Scope) -> Vec<(Nyes, Nyes)> {
    let mut trace = vec![(
        root.borrow().core().get_nyes(),
        watched.borrow().core().get_nyes(),
    )];
    for _ in 0..100 {
        if root.borrow().core().get_nyes().is_constanic() {
            break;
        }
        let _ = root.step(scope).unwrap();
        trace.push((
            root.borrow().core().get_nyes(),
            watched.borrow().core().get_nyes(),
        ));
    }
    trace
}
```

Usage:
```rust
let root = Compiler::compile("{a = 1; b = a;}").unwrap().pop().unwrap();
let search = find_search(&root, "^a$").expect("search for a");
let trace = step_watching(&root, &search, &Scope::empty());
eprintln!("(root, search) NYES per step:");
for (i, (r, s)) in trace.iter().enumerate() {
    eprintln!("  step {:2}: root={:?}  search={:?}", i, r, s);
}
```

---

## `assert_progression` — pin the expected progression

This helper (`fir_kinds.rs:3694`) asserts the monotone-progression contract. Use it to turn your debug observation into a regression test.

```rust
fn assert_progression(trace: &[Nyes], expected_terminal: Nyes, label: &str) {
    eprintln!("{label} nyes transitions: {trace:?}");
    assert!(!trace.is_empty(), "{label}: empty trace");
    assert_eq!(*trace.first().unwrap(), Nyes::Prembrionic,
        "{label}: must start PREMBRIONIC");
    let last = *trace.last().unwrap();
    assert!(last.is_constanic(), "{label}: must end constanic (got {last:?})");
    assert_eq!(last, expected_terminal, "{label}: wrong terminal state");
    let mut seen_constanic = false;
    for n in trace {
        if seen_constanic {
            assert!(n.is_constanic(), "{label}: regressed from constanic to {n:?}");
        }
        seen_constanic = n.is_constanic();
    }
}
```

It checks: (a) non-empty, (b) starts `PREMBRIONIC`, (c) ends constanic, (d) ends at the expected terminal variant, (e) **monotone** — no constanic-to-pre-constanic regression.

Usage:
```rust
let trace = step_to_settled(&brane, &Scope::empty());
assert_progression(&trace, Nyes::Constant, "Brane(a=1,b=2)");
// If the brane has an NK child, use Nyes::Nk instead.
```

---

## `settle_root` — step a compiled root to settlement (panics if stuck)

This helper (`fir_kinds.rs:4321`) is for when you just need the root settled and want a panic if it doesn't (useful in setup, not for observing the progression):

```rust
fn settle_root(root: &FirRef) {
    let scope = Scope::empty();
    for _ in 0..200 {
        let report = root.step(&scope).unwrap();
        if let StepReport::Progress(nyes) = report
            && nyes.is_constanic()
        {
            return;
        }
    }
    panic!("root did not settle within 200 steps");
}
```

---

## Interpreting the trace — common bug patterns

| Observation | Likely cause |
|-------------|-------------|
| Trace ends on `BRANING` (stuck, not constanic) | A child never settles. Check for infinite loops or missing `fir_op_step` transitions. |
| Trace contains a constanic → pre-constanic regression | State machine bug. The monotone contract is violated. |
| Search ends `NK` but you expected a hit | Anchored search (`a?name`) found nothing. Use `_ab_search` (see [fir-inspection.md](fir-inspection.md)) to check whether the name exists in the ancestral brane. |
| Search ends `ECONSTANIC` but you expected NK | Unanchored search (`?name`) found nothing — correct behavior, it may resolve in a new context. If it should be NK, the search must be anchored. |
| Brane ends `NK` because one child is `NK` | `_decide_nyes_due_to_children` propagates NK (worst wins). Find the NK child and debug it. |
| Operator ends `NK` | Division by zero, or an operand is itself NK. Check `ubc_children()[0]`. |
| `WOCONSTANIC` | A search found its target, but the target is ECONSTANIC/WOCONSTANIC — it is waiting on a deeper dependency. |

---

## Next steps

- Query the FIR/brane state with `ib_search` / `ab_search` → [fir-inspection.md](fir-inspection.md)
- Promote or delete the test when done → [cleanup.md](cleanup.md)
