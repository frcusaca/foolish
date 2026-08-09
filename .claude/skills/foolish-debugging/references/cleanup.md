# Cleanup — Promote or Delete

Debug tests are temporary. Before you declare the task done, every `temporary_reproduce_to_debug_*` test must be either **promoted to a named regression test** or **deleted**. This is non-negotiable.

This is step 4 of the workflow.

---

## The two valid outcomes

### Outcome A — Promote to a regression test

Use this when the debug test captured a real bug and the fix is in place. The test becomes a permanent regression guard.

1. **Rename** the test from `temporary_reproduce_to_debug_*` to a descriptive name that documents what it guards:
   ```rust
   // Before:
   #[test]
   fn temporary_reproduce_to_debug_search_nk() { ... }

   // After:
   #[test]
   fn anchored_search_miss_resolves_to_nk_not_econstanic() { ... }
   ```
2. **Tighten assertions.** The debug test may have `eprintln!` and loose `assert!` calls. Replace with precise `assert_eq!` that pins the exact expected behavior. The `assert_progression` helper is ideal for NYES traces:
   ```rust
   let trace = step_to_settled(&search, &Scope::empty());
   assert_progression(&trace, Nyes::Nk, "anchored search miss");
   ```
3. **Remove debug `eprintln!`** unless they document non-obvious behavior for future readers. A regression test should be clean — no noise on `--nocapture`.
4. **Add a comment** if the test guards against a specific past bug:
   ```rust
   // Regression: FOOP-23 anchored search miss must settle NK, not ECONSTANIC.
   // Previously, the search engine returned ECONSTANIC for anchored misses.
   ```
5. **Run the test** to confirm it passes after the fix:
   ```bash
   cargo test -p foolish-ubca --lib -- anchored_search_miss_resolves_to_nk
   ```

### Outcome B — Delete the test

Use this when:
- The debug test was purely exploratory (you were reading state, not pinning a bug).
- The bug was a misunderstanding (the FVM behaved correctly; your expectation was wrong).
- The bug is covered by an existing test after the fix.

Delete the entire test function. Do not leave commented-out code or `#[ignore]`-d tests behind.

---

## When NOT to keep a debug test

- **It duplicates an existing `*_nyes_transitions` test.** The progression is already pinned. Delete yours.
- **It tests a hypothesis that turned out wrong.** A wrong-hypothesis test is noise. Delete it.
- **It has no assertions.** A test that only `eprintln!`s is not a test. Either add real assertions and promote, or delete.

---

## Output formatting for agent readability

When a debug test *is* kept (as a regression test or during debugging), its `eprintln!` output should be formatted so that an agent with context of the codebase can parse it. The output is **for the agent**, not for a human in a terminal.

### Principles

1. **Label every section.** An agent reading `cargo test --nocapture` output needs to know what each block means:
   ```rust
   eprintln!("=== debug: search for 'a' in {{a=1; b=a;}} ===");
   eprintln!("b_stmt: kind={:?} name={:?} line={:?}",
       b_stmt.borrow().kind(),
       b_stmt.borrow().as_stmt_name(),
       b_stmt.borrow().as_stmt_line_number());
   ```

2. **Use `{:?}` (Debug) for structured types.** `Nyes`, `FirKind`, `StepReport` all derive `Debug`. A trace like `[Prembrionic, Embryonic, Braning, Constant]` is immediately parseable.

3. **Dump the FIR tree with indentation.** Use the `dump_fir` walker from [fir-inspection.md](fir-inspection.md). Indented `kind={:?} nyes={:?}` lines let an agent reconstruct the tree structure from the output.

4. **Pair NYES traces with labels.** When printing a `Vec<Nyes>`, always prefix with what it is:
   ```rust
   eprintln!("root NYES trace: {trace:?}");
   eprintln!("search NYES trace: {search_trace:?}");
   ```
   Not just `{trace:?}` — the agent needs to know which FIR the trace belongs to.

5. **Include the Foolish source in the output.** When the test compiles a source string, echo it so the agent can correlate:
   ```rust
   let source = "{a = 1; b = a;}";
   eprintln!("=== source ===\n{source}");
   let root = Compiler::compile(source).unwrap().pop().unwrap();
   ```

6. **Print before AND after stepping.** The FIR tree before stepping shows the parsed structure; after stepping shows the resolved state. Both are needed to understand a transition:
   ```rust
   eprintln!("=== FIR tree (before stepping) ===");
   dump_fir(&root, 0);
   let trace = step_to_settled(&root, &Scope::empty());
   eprintln!("=== NYES trace ===\n{trace:?}");
   eprintln!("=== FIR tree (after stepping) ===");
   dump_fir(&root, 0);
   ```

### Example: a well-formatted debug test

```rust
#[test]
fn temporary_reproduce_to_debug_search_nk() {
    let source = "{a = 1; b = a;}";
    eprintln!("=== debug: search resolution in source ===\n{source}");

    let root = Compiler::compile(source).unwrap().pop().unwrap();
    let search = find_search(&root, "^a$").expect("search for a");
    let stmts = root.borrow().core().foolish_children().to_vec();
    let b_stmt = &stmts[1];

    eprintln!("=== FIR tree (before stepping) ===");
    dump_fir(&root, 0);

    eprintln!("=== ib_search from b_stmt ===");
    match b_stmt.borrow()._ib_search(b_stmt, "^a$") {
        Some((found, nyes)) => {
            let body = found.borrow().core().foolish_children()[0].clone();
            eprintln!("  found: kind={:?} nyes={:?} value={:?}",
                found.borrow().kind(), nyes, body.borrow().as_i64());
        }
        None => eprintln!("  NOT FOUND"),
    }

    let trace = step_to_settled(&root, &Scope::empty());
    eprintln!("=== root NYES trace ===\n{trace:?}");
    eprintln!("=== search final NYES: {:?} ===", search.borrow().core().get_nyes());

    eprintln!("=== FIR tree (after stepping) ===");
    dump_fir(&root, 0);

    // The bug assertion:
    assert_eq!(search.borrow().core().get_nyes(), Nyes::Constant);
}
```

An agent reading the `--nocapture` output of this test sees: the source, the parsed tree, the ib_search result (with value), the NYES progression, the final state, and the resolved tree. That is enough to diagnose almost any Foolish evaluation bug.

---

## Checklist before declaring done

- [ ] Every `temporary_reproduce_to_debug_*` test is either renamed to a descriptive regression test or deleted.
- [ ] Promoted tests have precise `assert_eq!` / `assert_progression` assertions (not just `eprintln!`).
- [ ] Promoted tests have no leftover debug-only `eprintln!` (unless they document non-obvious behavior).
- [ ] `cargo test -p foolish-ubca --lib` passes with no new failures.
- [ ] `cargo fmt` and `cargo clippy` are clean on changed files.
- [ ] No commented-out code or `#[ignore]`-d debug tests left behind.
