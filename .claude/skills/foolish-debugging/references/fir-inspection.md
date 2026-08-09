# FIR Inspection — Querying FVM State

How to inspect FIR state at the offending point: read values and kinds, walk the parent chain, query name resolution via `ib_search` / `ab_search`, and dump the FIR tree. This is step 3 of the workflow.

---

## The inspection API at a glance

All of these are available from inside the `#[cfg(test)] mod tests` block in `foolish-ubca/src/fir_kinds.rs`:

| What | Call | Returns |
|------|------|---------|
| NYES state | `node.borrow().core().get_nyes()` | `Nyes` |
| FIR kind | `node.borrow().kind()` | `FirKind` |
| Integer value | `node.borrow().as_i64()` | `Option<i64>` |
| Search pattern | `node.borrow().as_search_pattern()` | `Option<&str>` |
| Statement name | `node.borrow().as_stmt_name()` | `Option<&str>` |
| Statement line | `node.borrow().as_stmt_line_number()` | `Option<usize>` |
| NK reason | `node.borrow().as_nk_reason()` | `Option<&str>` |
| Operator name | `node.borrow().as_op_name()` | `Option<&str>` |
| Deepest value | `node.value()` | `FirRef` |
| Parent (one up) | `node.borrow().core().parent()` | `Option<FirRef>` |
| Home brane (parent-walk) | `node.borrow()._get_my_brane(&node)` | `Option<FirRef>` |
| Foolish children | `node.borrow().core().foolish_children()` | `&[FirRef]` |
| UBC children | `node.borrow().core().ubc_children()` | `Vec<FirRef>` (owned clone) |
| IB search | `node.borrow()._ib_search(&node, "^pat$")` | `Option<(FirRef, Nyes)>` |
| AB search | `node.borrow()._ab_search(&node, "^pat$")` | `Option<(FirRef, Nyes)>` |

*(These are `Fir`-trait methods on `foolish-ubca`; the child stores are on
`ProtoBrane` via `core()`. Locate any by name with `grep -n`.)*

---

## Reading values and kinds

```rust
// After stepping to settlement:
let nyes = node.borrow().core().get_nyes();
let kind = node.borrow().kind();
let value = node.value();           // walks ubc_children to deepest resolved
let int_val = value.borrow().as_i64();

eprintln!("nyes={:?} kind={:?} value_int={:?}", nyes, kind, int_val);
```

**`value()`** walks `ubc_children[0]` recursively until it reaches a terminal value (one with no `ubc_children`, like `IndepInt`, `Nk`, or `BraneFir`). For pre-constanic FIRs, returns a clone of `self`.

**`as_i64()`** defaults to reading `ubc_children().first()` and recursing. `IndepIntFir` overrides it to return `Some(value)` directly.

### The two child stores

Every `ProtoBrane` has **two** child vectors:
- `foolish_children()` — the *syntactic* children (statements in a brane, operands of an operator, etc.). This is the tree as parsed.
- `ubc_children()` — the *computed* children (results pushed during stepping). A settled search has its found result in `ubc_children[0]`.

When debugging, inspect both:
```rust
eprintln!("foolish_children: {}",
    node.borrow().core().foolish_children()
        .iter().map(|c| format!("{:?}", c.borrow().kind()))
        .collect::<Vec<_>>());
eprintln!("ubc_children: {}",
    node.borrow().core().ubc_children()
        .iter().map(|c| format!("{:?}={:?}", c.borrow().kind(), c.borrow().as_i64()))
        .collect::<Vec<_>>());
```

---

## Walking the parent chain

**There is no `get_my_parents`.** The parent is a single `Weak<RefCell<dyn Fir>>` stored in `ProtoBrane`. Walk it with a loop:

```rust
fn dump_parent_chain(node: &FirRef) {
    eprintln!("--- parent chain ---");
    let mut current = Some(Rc::clone(node));
    let mut depth = 0;
    while let Some(n) = current {
        let b = n.borrow();
        eprintln!("  [{depth}] kind={:?} name={:?} nyes={:?} line={:?}",
            b.kind(),
            b.as_stmt_name(),
            b.core().get_nyes(),
            b.as_stmt_line_number(),
        );
        current = b.core().parent();
        depth += 1;
    }
}
```

The chain terminates when `parent()` returns `None` (the root, which is self-parenting — detected via `Rc::ptr_eq`).

### Getting the home brane directly

If you only need the containing brane (not the full chain), use `_get_my_brane`:

```rust
let brane = node.borrow()._get_my_brane(&node);
// brane: Option<FirRef> — the first brane-like kind reached by walking .parent()
```

Returns `None` if `node` is itself the root brane.

---

## `ib_search` — search the Immediate Brane (IB)

`_ib_search` searches **backward** from the current statement's position within its home brane. It finds names declared *before* the current line in the same brane.

**Signature:**
```rust
fn _ib_search(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)>
```

- `name` is a **regex pattern**. Anchor with `^...$` for exact match: `"^a$"`, `"^point$"`.
- Returns `Some((found_statement, its_nyes))` on hit, `None` on miss.
- The returned `FirRef` is the **found statement** (a `StatementFir`), not its body. Read the body via `stmt.borrow().core().foolish_children()[0]`.

**Usage:**
```rust
let root = Compiler::compile("{a = 1; b = a;}").unwrap().pop().unwrap();
let stmts = root.borrow().core().foolish_children().to_vec();
let b_stmt = &stmts[1]; // "b = a"

let result = b_stmt.borrow()._ib_search(b_stmt, "^a$");
match result {
    Some((found, nyes)) => {
        let body = found.borrow().core().foolish_children()[0].clone();
        eprintln!("ib_search found 'a': nyes={:?} value={:?}", nyes, body.borrow().as_i64());
    }
    None => eprintln!("ib_search: 'a' not found in immediate brane"),
}
```

**Semantics:** `_ib_search` scans backward from the line before the current statement down to index 0. A statement at **line 0** (the first in its brane) has no preceding range, so it returns `None` — it never finds itself. (See `StatementFir::_ib_search`.)

---

## `ab_search` — search the Ancestral Brane (AB) chain

`_ab_search` tries `_ib_search` on the current brane; on miss, it **recurses into the parent brane**. This finds names declared in ancestor branes that are not in the immediate brane.

**Signature:**
```rust
fn _ab_search(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)>
```

**Usage:**
```rust
let root = Compiler::compile("{a = 1; b = {c = a;};}").unwrap().pop().unwrap();
let outer_stmts = root.borrow().core().foolish_children().to_vec();
let b_stmt = &outer_stmts[1];
let inner_brane = b_stmt.borrow().core().foolish_children().first().unwrap().clone();
let inner_stmts = inner_brane.borrow().core().foolish_children().to_vec();
let c_stmt = &inner_stmts[0]; // "c = a"

// IB search: 'a' is NOT in the inner brane.
assert!(c_stmt.borrow()._ib_search(c_stmt, "^a$").is_none());

// AB search: 'a' IS found in the ancestral (outer) brane.
let result = inner_brane.borrow()._ab_search(&inner_brane, "^a$");
assert!(result.is_some());
let (found, nyes) = result.unwrap();
eprintln!("ab_search found 'a': nyes={:?} value={:?}",
    nyes,
    found.borrow().core().foolish_children()[0].borrow().as_i64());
```

**Semantics:** `BraneFir::_ab_search` first tries `_ib_search` on the brane's own statement; on miss, it gets the parent brane and recurses. The chain terminates at the root.

---

## Shadowing: IB wins over AB

When a name exists in both the immediate brane and an ancestor, `_ib_search` finds the immediate (shadowing) one:

```rust
let root = Compiler::compile("{a = 1; b = {a = 2; c = a;};}").unwrap().pop().unwrap();
// 'c = a' should find inner a=2, not outer a=1

let inner_brane = /* ... navigate to inner brane ... */;
let c_stmt = /* ... navigate to 'c = a' ... */;

let (found, _) = c_stmt.borrow()._ib_search(c_stmt, "^a$").unwrap();
let body = found.borrow().core().foolish_children()[0].clone();
assert_eq!(body.borrow().as_i64(), Some(2), "must find inner a=2, not outer a=1");
```

---

## The `_search_brane` primitive

Both `_ib_search` and `_ab_search` delegate to `_search_brane` on `BraneFir`:

```rust
fn _search_brane(&self, expression: &str, starting_index: usize, ending_index: usize)
    -> Option<(usize, FirRef, Nyes)>
```

- If `starting_index >= ending_index`, scans **backward** (rear-to-front).
- Otherwise scans **forward**.
- Returns `(index, statement, nyes)` — the position, the found `StatementFir`, and its NYES.
- Matches via `SearchFir::matches_pattern(stmt_name, expression)` — regex.

You can call `_search_brane` directly on a brane for custom scan ranges:
```rust
let (idx, stmt, nyes) = brane.borrow()._search_brane("^x$", 3, 0).unwrap();
// Scanned backward from index 3 to index 0.
```

---

## Dumping a FIR subtree (for agent-readable output)

When debugging, dump the FIR tree structure so an agent with code context can read it. This walker mirrors `find_search` but prints everything:

```rust
fn dump_fir(node: &FirRef, indent: usize) {
    let b = node.borrow();
    let pad = "  ".repeat(indent);
    let name = b.as_stmt_name().map(|n| format!(" name={n}")).unwrap_or_default();
    let line = b.as_stmt_line_number().map(|l| format!(" line={l}")).unwrap_or_default();
    let val = b.as_i64().map(|v| format!(" i64={v}")).unwrap_or_default();
    let pat = b.as_search_pattern().map(|p| format!(" pat={p}")).unwrap_or_default();
    let op = b.as_op_name().map(|o| format!(" op={o}")).unwrap_or_default();
    let nk = b.as_nk_reason().map(|r| format!(" nk_reason={r}")).unwrap_or_default();
    eprintln!("{pad}{:?} nyes={:?}{name}{line}{val}{pat}{op}{nk}",
        b.kind(), b.core().get_nyes());
    for c in b.core().foolish_children() {
        dump_fir(c, indent + 1);
    }
    if !b.core().ubc_children().is_empty() {
        eprintln!("{pad}  ubc_children:");
        for c in b.core().ubc_children() {
            dump_fir(c, indent + 2);
        }
    }
}
```

Usage:
```rust
let root = Compiler::compile("{a = 1; b = a;}").unwrap().pop().unwrap();
eprintln!("=== FIR tree (before stepping) ===");
dump_fir(&root, 0);

let trace = step_to_settled(&root, &Scope::empty());
eprintln!("=== NYES trace ===\n{trace:?}");

eprintln!("=== FIR tree (after stepping) ===");
dump_fir(&root, 0);
```

The output is structured so an agent reading it can see: kind, NYES state, name, line number, integer value, search pattern, operator name, NK reason — for every node, with indentation showing containment.

---

## Borrow discipline (avoid panics)

`FirRef` is `Rc<RefCell<dyn Fir>>`. The cardinal rule: **never hold a `borrow()` across another `borrow()` or `borrow_mut()`**. Always:

1. Borrow, extract what you need (clone `FirRef`s out), drop the borrow.
2. Then borrow the next node.

**Wrong** (panics):
```rust
let stmt = root.borrow().core().foolish_children()[0].clone();
let kind = root.borrow().kind();          // ← root still borrowed via stmt? No, but...
let body = stmt.borrow().core().foolish_children()[0].clone();
let val = stmt.borrow().as_i64();         // ← if stmt is still borrowed above, panic
```

**Right:**
```rust
let stmt = {
    let b = root.borrow();
    b.core().foolish_children()[0].clone()
}; // borrow dropped
let kind = root.borrow().kind();          // fresh borrow, fine
let body = {
    let b = stmt.borrow();
    b.core().foolish_children().first().map(Rc::clone)
};
let val = stmt.borrow().as_i64();         // fresh borrow, fine
```

When in doubt, wrap each access in a block and clone the `FirRef` out before the block ends.

---

## Programmatic search engine (advanced — `pub(crate)` only)

For debugging the search engine itself, the `contextful_search` module exposes the one-engine model. **These types are `pub(crate)`** — only reachable from inside `foolish-ubca` (i.e. from the `mod tests` block in `fir_kinds.rs`).

```rust
use contextful_search::{
    BraneNavigator, SearchPredicate, CursorSource,
    contextful_search_scan, contextful_search_scan_no_body_check, ScanOutcome,
};
```

**`SearchPredicate`**:
```rust
pub(crate) enum SearchPredicate {
    Name { pattern: String },              // ?name / ~name / .name
    Value { pattern: FirRef },             // ?=v / ~=v
    NameValue { name: String, value: FirRef }, // ?name=v (atomic conjunctive)
    Index(i32),                            // #N (negative = from end)
    Head,                                  // ^
    Tail,                                  // $
}
```

**Running a search programmatically** (mirrors `SearchFir::ib_search_with_engine`):
```rust
use contextful_search::{BraneNavigator, SearchPredicate, contextful_search_scan_no_body_check, ScanOutcome};

let stmt = /* the statement to search from */;
let brane = stmt.borrow()._get_my_brane(&stmt).unwrap();
let idx = brane.borrow().find_stmt_index(&stmt).unwrap();
let search_end = idx.saturating_sub(1);

let mut nav = BraneNavigator::new(&brane, false); // false = backward (IB semantics)
nav.set_range(0, search_end);
let predicate = SearchPredicate::Name { pattern: "a".to_string() };
match contextful_search_scan_no_body_check(&mut nav, &predicate) {
    ScanOutcome::Found(found) => {
        eprintln!("engine found: kind={:?} nyes={:?}",
            found.borrow().kind(), found.borrow().core().get_nyes());
    }
    ScanOutcome::NkStop => eprintln!("engine: NK stop"),
    ScanOutcome::Miss => eprintln!("engine: miss"),
}
```

Use this only when you need to debug the search engine's traversal logic itself. For normal "does this name resolve?" debugging, `_ib_search` and `_ab_search` are sufficient.

---

## Next steps

- Promote or delete the debug test → [cleanup.md](cleanup.md)
