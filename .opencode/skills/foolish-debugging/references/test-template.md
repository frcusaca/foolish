# Test Template — The Minimal Setup

How to write a temporary unit test that compiles Foolish code, grabs the FIR at the line you suspect, and is ready to step. This is step 1 of the workflow.

---

## Where to put the test

Add your test inside the `#[cfg(test)] mod tests` block at the bottom of:

```
foolish-ubca/src/fir_kinds.rs
```

This is where all the helpers live (`step_to_settled`, `find_search`, `assert_progression`, `step_watching`, `make_*`). Find the module with `grep -n '^mod tests' fir_kinds.rs`, then scroll to the end of the existing tests and add yours.

**Why not a `tests/` integration file?** The helpers and the `pub(crate)` engine types (`SearchPredicate`, `BraneNavigator`, `contextful_search_scan`) are only reachable from inside the crate. A `tests/debug_foo.rs` file would force you to copy helpers and would lose access to the engine internals.

---

## Imports already in scope

The test module header brings everything in:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fir_trait::StepReport;
    // ...later in the module:
    use crate::compiler::Compiler;
```

You do not need additional `use` statements for `FirRef`, `Nyes`, `FirKind`, `Scope`, `StepReport`, `Rc`, `RefCell`. They are all in scope via `use super::*`.

---

## The minimal template (3 variants)

### Variant A — Compile real Foolish source (most common)

Use this when the bug is in how Foolish source evaluates. This is the starting point for nearly all debugging.

```rust
#[test]
fn temporary_reproduce_to_debug_search_returns_nk() {
    // 1. Compile the offending Foolish source.
    let root = Compiler::compile("{a = 1; b = a;}").unwrap().pop().unwrap();
    let scope = Scope::empty();

    // 2. Grab the FIR at the line you suspect.
    //    Option (a): positional — the Nth statement is the Nth child.
    let stmts = root.borrow().core().foolish_children().to_vec();
    let b_stmt = &stmts[1]; // "b = a" is the second statement

    //    Option (b): find a Search FIR by its regex pattern.
    let search = find_search(&root, "^a$").expect("search for a");

    eprintln!("--- debug: search for 'a' in {{a=1; b=a;}} ---");
    eprintln!("b_stmt kind:  {:?}", b_stmt.borrow().kind());
    eprintln!("b_stmt name:  {:?}", b_stmt.borrow().as_stmt_name());
    eprintln!("search kind:  {:?}", search.borrow().kind());

    // 3. Step to settlement and inspect. (See nyes-tracing.md and fir-inspection.md.)
    let trace = step_to_settled(&root, &scope);
    eprintln!("root NYES trace: {trace:?}");

    // 4. Assert what you observe. This is the bug — it returns NK but should be Constant.
    assert_eq!(
        search.borrow().core().get_nyes(),
        Nyes::Constant,
        "BUG: search for 'a' should resolve to Constant"
    );
}
```

### Variant B — Build FIRs by hand (when you need precise structural control)

Use this when the bug is in a specific FIR kind's internal logic and you want to control the exact tree shape without parser noise. The `make_*` builders are near the top of the `mod tests` block.

```rust
#[test]
fn temporary_reproduce_to_debug_operator_nk() {
    let a = make_constant_int(10);
    let b = make_constant_int(0);
    let op = make_operator("/", vec![Rc::clone(&a), Rc::clone(&b)]);
    let scope = Scope::empty();

    eprintln!("--- debug: division by zero ---");
    let trace = step_to_settled(&op, &scope);
    eprintln!("Operator(/) NYES trace: {trace:?}");
    eprintln!("final NYES: {:?}", op.borrow().core().get_nyes());

    // The bug: should be NK but is something else.
    assert_eq!(op.borrow().core().get_nyes(), Nyes::Nk);
}
```

Available `make_*` builders (all return `FirRef`):
| Builder | Signature |
|---------|-----------|
| `make_constant_int` | `(value: i64)` |
| `make_nk` | `(reason: &str)` |
| `make_operator` | `(op: &str, operands: Vec<FirRef>)` |
| `make_statement` | `(name: &str, line: usize, body: FirRef)` |
| `make_brane` | `(children: Vec<FirRef>)` |
| `make_search` | `(pattern: &str, anchored: bool, anchors: Vec<FirRef>)` |
| `make_index` | `(offset: i32, anchored: bool, children: Vec<FirRef>)` |
| `make_headtail` | `(is_head: bool, anchored: bool, children: Vec<FirRef>)` |
| `make_stay_foolish` | `(expr: FirRef)` |
| `make_stay_fully_foolish` | `(expr: FirRef)` |
| `make_concatenation` | `(elements: Vec<FirRef>)` |

### Variant C — Grab a FIR by line number

There is no `find_at_line` helper. Compose it inline using `as_stmt_line_number()`:

```rust
#[test]
fn temporary_reproduce_to_debug_line_3() {
    let root = Compiler::compile("{a = 1; b = 2; c = a + b;}").unwrap().pop().unwrap();
    let stmts = root.borrow().core().foolish_children().to_vec();

    let target_line = 2; // 0-indexed internally; "c = a + b" is the third statement
    let suspect = stmts.iter()
        .find(|s| s.borrow().as_stmt_line_number() == Some(target_line))
        .expect("statement at target line")
        .clone();

    eprintln!("--- debug: line {} ---", target_line);
    eprintln!("name: {:?}", suspect.borrow().as_stmt_name());
    // ... step and inspect ...
}
```

---

## Grabbing nested FIRs

For a brane nested inside a statement, chain `foolish_children()`:

```rust
// Source: {a = 1; b = {c = a;};}
let root = Compiler::compile("{a = 1; b = {c = a;};}").unwrap().pop().unwrap();
let outer_stmts = root.borrow().core().foolish_children().to_vec();
let b_stmt = &outer_stmts[1];                       // "b = {c = a;}"
let inner_brane = b_stmt
    .borrow()
    .core()
    .foolish_children()
    .first()
    .unwrap()
    .clone();
let inner_stmts = inner_brane.borrow().core().foolish_children().to_vec();
let c_stmt = &inner_stmts[0];                       // "c = a"
```

Always `.clone()` the `FirRef` out of the borrow before you use it, to avoid holding a `Ref` across another `borrow()` (which would panic).

---

## Finding a Search FIR anywhere in the tree

The `find_search` walker recursively searches `foolish_children()` for a `SearchFir` whose pattern matches:

```rust
fn find_search(node: &FirRef, pattern: &str) -> Option<FirRef>
```

Usage:
```rust
let search = find_search(&root, "^a$").expect("search for a");
```

The `pattern` argument is compared via `as_search_pattern() == Some(pattern)`, so it must match **exactly** including the `^` and `$` anchors the parser emits.

---

## Running the test

```bash
cargo test -p foolish-ubca --lib -- temporary_reproduce_to_debug
```

The `--` substring match runs only tests whose names contain `temporary_reproduce_to_debug`. To see `eprintln!` output, add `--nocapture`:

```bash
cargo test -p foolish-ubca --lib -- temporary_reproduce_to_debug --nocapture
```

---

## Next steps

- Step the FVM and record the NYES sequence → [nyes-tracing.md](nyes-tracing.md)
- Query FIR state with `ib_search` / `ab_search` → [fir-inspection.md](fir-inspection.md)
- Promote or delete the test when done → [cleanup.md](cleanup.md)
