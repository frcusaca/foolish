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

- `core()` — `Fir` trait method, returns `&ProtoBrane`.
- `get_nyes()` — `ProtoBrane` method, reads from an interior `Cell<Nyes>`. `Nyes` is `Copy`.

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

Defined on the `FirRefExt` trait. Call it as `node.step(&scope)`.

**`StepReport`**:
```rust
pub enum StepReport {
    NoProgress,
    Progress(Nyes),  // the node's NYES after this step
}
```

One `step()` call performs **one unit of work**: it peeks the front of the task queue; if the front child is constanic it pops it, otherwise it recurses into the child; if the queue is empty it runs `fir_op_step` (the kind-specific combining work). See `step_inner`.

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

## ⭐ Breakpoints: `step_until*` — stop at a specific spot, then inspect

**Reach for these first when you have a "why does line N go wrong" question.**
They are the closest thing the FVM has to a debugger breakpoint: step the root
until a chosen statement reaches the **front of the job queue**, then pause and
inspect anything in the tree. Built during FOOP-13 and maintained as part of it.

Three public functions in `evaluator.rs` (import `use crate::evaluator::*;`):

```rust
// The general form: stop when `matcher` returns true for the front task.
// matcher receives Option<&FirRef> (None = queue empty → fir_op_step next).
pub fn step_until<F>(root: &FirRef, scope: &Scope, matcher: F)
    -> Result<usize, UbcError>
    where F: FnMut(Option<&FirRef>) -> bool;

// Convenience wrappers for the two common breakpoints:
pub fn step_until_line_number(root: &FirRef, scope: &Scope, line: usize)
    -> Result<usize, UbcError>;
pub fn step_until_statement_name(root: &FirRef, scope: &Scope, name: &str)
    -> Result<usize, UbcError>;
```

Each returns the **step count** at which it stopped, or an error if the root
settled first (`"FVM settled … before condition was met"`) or the `10_000`
step limit was hit. The underlying peek is `FirRef::debug_front_task()`
(`fir_trait.rs`), which reads the front of the task queue without popping.

**Breakpoint-and-inspect (the killer pattern):**

```rust
use crate::evaluator::step_until_statement_name;

let root = Compiler::compile("{a = 1; b = 2; extended = a + b;}")
    .unwrap().pop().unwrap();
let scope = Scope::empty();

// Run until `extended` is about to be worked on, then freeze and look.
let at = step_until_statement_name(&root, &scope, "extended")
    .expect("extended should reach the front before settling");
eprintln!("stopped at step {at}");

// Now inspect ANY node — nyes, children, ib_search, values — at this exact
// moment. (See fir-inspection.md for the full inspection toolkit.)
let stmts = root.borrow().core().foolish_children().to_vec();
let ext = stmts.iter()
    .find(|s| s.borrow().as_stmt_name() == Some("extended")).unwrap();
eprintln!("extended nyes at breakpoint: {:?}", ext.borrow().core().get_nyes());
```

Breakpoint by **source line** instead of name:

```rust
let at = step_until_line_number(&root, &scope, 3)?;  // 0-indexed line
```

Breakpoint on an **arbitrary condition** (e.g. "the first time any Operator
reaches the front"):

```rust
let at = step_until(&root, &scope, |front| {
    front.map(|f| f.borrow().kind() == FirKind::Operator).unwrap_or(false)
})?;
```

**Worked example in-tree:** `diag_concat_cb_shadow_uses_step_until` in
`evaluator.rs` (`mod step_until_tests`) is a real, kept diagnostic test using
exactly this breakpoint-and-inspect flow — read it for a complete pattern.

---

## Step-and-monitor: watch `_children` and NYES as you step

The technique used throughout FOOP-13 debugging: after breakpointing (or from
the start), step **one at a time** and dump the node's `foolish_children` /
`ubc_children` and their NYES each step. This is how you catch a store that
fills wrong, a child that never settles, or an nyes that regresses.

```rust
let scope = Scope::empty();
let target = /* the FIR you're watching, grabbed by name/line above */;

for i in 0..60 {
    // Snapshot both stores' NYES this step.
    let foolish: Vec<Nyes> = target.borrow().core().foolish_children()
        .iter().map(|c| c.borrow().core().get_nyes()).collect();
    let ubc: Vec<Nyes> = target.borrow().core().ubc_children()
        .iter().map(|c| c.borrow().core().get_nyes()).collect();
    eprintln!(
        "step {i:2}: self={:?}  foolish_children={foolish:?}  ubc_children={ubc:?}",
        target.borrow().core().get_nyes(),
    );

    if root.borrow().core().get_nyes().is_constanic() {
        break;
    }
    let _ = root.step(&scope).unwrap();
}
```

**Reading it:**
- `foolish_children` is the fixed parse-time store; its shape never changes,
  but its members' NYES advance as they settle.
- `ubc_children` is the compute-time store — search results, `_ConcatHelper`s,
  resolved SF/SFF values. Watching it appear/grow is how you see *when* a node
  produces its result.
- A member stuck pre-constanic while the parent spins = the culprit. A
  member's NYES going constanic → pre-constanic = a monotonicity bug.

To also see **what** is at the front each step (which node the driver is
actually working), add `debug_front_task`:

```rust
let front = root.borrow().debug_front_task();
eprintln!("  front: kind={:?} name={:?} nyes={:?}",
    front.as_ref().map(|f| f.borrow().kind()),
    front.as_ref().and_then(|f| f.borrow().as_stmt_name().map(str::to_owned)),
    front.as_ref().map(|f| f.borrow().core().get_nyes()));
```

---

## `step_to_settled` — record the full NYES trace

This helper steps up to 50 times, recording the NYES after each step, and returns the full sequence. **This is the standard debugging tool** — it shows you the exact progression from `PREMBRIONIC` to terminal.

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

This helper steps the **root** while recording the NYES of **both** the root and a specific child (the one you suspect) at every step. This is the killer snippet for "stop at the offending line" — you see exactly when the suspect enters each state.

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

This helper asserts the monotone-progression contract. Use it to turn your debug observation into a regression test.

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

This helper is for when you just need the root settled and want a panic if it doesn't (useful in setup, not for observing the progression):

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
