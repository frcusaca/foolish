---
name: foolish-debugging
description: "MUST USE for debugging Foolish FVM/FIR behavior — wrong brane evaluation, unexpected NK/ECONSTANIC, search resolution failures, NYES state machine bugs, name-lookup errors. The Foolish VM has no interactive debugger; debugging is unit-test-driven: write a temporary test that reproduces the issue, step the FVM to the offending line, inspect FIR NYES state and query the FVM via ib_search/ab_search, then promote the test to a regression test or delete it. This skill gives the exact Rust API and copy-pasteable templates. Triggers: 'debug Foolish', 'FVM bug', 'NYES wrong', 'brane not resolving', 'search returns NK', 'ib_search', 'ab_search', 'why is my brane NK', 'FIR state', 'step the FVM', 'foolish unit test debug', 'temporary_reproduce_to_debug'."
---

# Foolish Debugging

The Foolish VM has **no interactive debugger**. The `step` and `run` CLI commands both fully evaluate to settlement and print the final tree — they do not pause or single-step. Real Foolish debugging is **unit-test-driven**: you write a temporary Rust test that reproduces the problem, step the FVM programmatically, inspect FIR state (NYES, children, values), and query the resolution engine (`ib_search` / `ab_search`) at any point.

> **The knowledge is in `references/`.** This file is a map. Open the reference for what you are about to do.

---

## When to use this skill

- A Foolish program evaluates to the wrong value, to `???` (NK), or does not settle.
- A search resolves to NK when you expected a hit (or vice versa).
- A brane's NYES progression looks wrong (e.g. regresses from constanic to pre-constanic).
- You are touching FIR kinds, the search engine, or the stepper in `foolish-ubca`.
- You need to understand *why* a specific line of Foolish evaluates the way it does.

**Do NOT use this skill for:** Rust-level crashes/panics in the compiler machinery (use the general `/debugging` skill with the `rust.md` runtime reference), or snapshot/approval test output changes (that is the snapshot review workflow in `AGENTS.md`, not debugging).

---

## The workflow in four steps

| # | Step | Reference |
|---|------|-----------|
| 1 | **Write a temporary test** that compiles the offending Foolish code and grabs the FIR at the line you suspect. | [references/test-template.md](references/test-template.md) |
| 2 | **Step the FVM** to the offending line (or to settlement) while recording the NYES sequence. | [references/nyes-tracing.md](references/nyes-tracing.md) |
| 3 | **Inspect FIR state** — read NYES, walk the parent chain, query `ib_search` / `ab_search`, read values and children. | [references/fir-inspection.md](references/fir-inspection.md) |
| 4 | **Clean up** — promote the debug test to a named regression test, or delete it. Never leave it to rot. | [references/cleanup.md](references/cleanup.md) |

**Read the reference for the step you are entering.** Each is self-contained and short.

---

## Critical orientation (read before you start)

### Target crate

The active FIR implementation is **`foolish-ubca`**. `FirRef` there is `Rc<RefCell<dyn Fir>>` (defined at `foolish-ubca/src/fir_trait.rs:17`). The `ib_search` / `ab_search` API lives on the `Fir` trait in that crate.

`foolish-core` has a *different* `FirRef` (`Rc<RefCell<dyn Steppable>>`) and a simpler `search_in_brane` free function. **Do not mix the two crates' APIs.** If you are debugging brane evaluation, you are in `foolish-ubca`.

### Where to put the debug test

The test scaffolding helpers (`make_*`, `step_to_settled`, `find_search`, `assert_progression`, `step_watching`) live inside the `#[cfg(test)] mod tests` block at the bottom of **`foolish-ubca/src/fir_kinds.rs`** (starts ~line 2339). They are **not** accessible from `tests/` integration test files.

**Two options:**
1. **Add your test to the `mod tests` block in `fir_kinds.rs`** — recommended; all helpers are in scope, `pub(crate)` engine types are reachable.
2. **Write a separate `tests/` file** — you must copy the helpers you need (`step_to_settled`, `find_search`) into your file, and you cannot use `pub(crate)` engine types (`SearchPredicate`, `BraneNavigator`, etc.). Only the public `Fir` trait methods (`_ib_search`, `_ab_search`, `core()`, `kind()`, `value()`) and `Compiler::compile` are available.

Option 1 is almost always correct. See [references/test-template.md](references/test-template.md) for the exact placement.

### The NYES state machine

Every FIR progresses through NYES (Not Yet Evaluated State) states. This is the primary observability surface:

```
PREMBRIONIC → EMBRYONIC → BRANING → (ECONSTANIC | WOCONSTANIC) → (CONSTANT | INDEPENDENT | NK)
```

- **Pre-constanic** ("nigh"): `PREMBRIONIC`, `EMBRYONIC`, `BRANING` — more stepping is appropriate.
- **Constanic** (terminal): `ECONSTANIC`, `WOCONSTANIC`, `CONSTANT`, `INDEPENDENT`, `NK`.
- `is_constanic()` returns true for all five terminal states.

Authoritative definitions: `foolish-core/src/fir.rs:116` (the enum), `docs/foop/FOOP-62.md` (semantics). See [references/nyes-tracing.md](references/nyes-tracing.md) for what each terminal state means and how to read the progression.

### There is no `get_my_parents`

The user-facing concept of "walking the parent chain" is served by two APIs:
- `node.borrow().core().parent() -> Option<FirRef>` — one step up the `Weak` parent link.
- `node.borrow().get_my_brane(&node) -> Option<FirRef>` — walks the parent chain to the first `FirKind::Brane` (the "home brane").

There is no plural `get_my_parents`. To walk the full chain, loop on `core().parent()`. See [references/fir-inspection.md](references/fir-inspection.md).

---

## Non-negotiable safety invariants

1. **Debug tests are temporary.** A `temporary_reproduce_to_debug_*` test must be promoted to a named regression test or deleted before the task is done. Never leave it behind. ([references/cleanup.md](references/cleanup.md))
2. **Never auto-accept snapshots.** Do not run `cargo insta accept` or `INSTA_UPDATE=always`. Do not alter `.snap`, `.approved.foo`, or `.snap.new.approved` files. (See `AGENTS.md`.)
3. **Never start Phase+ work when tests are broken.** Fix or disable (with human permission) the broken test first.
4. **Output is for the agent to read.** Format debug `eprintln!` output so an agent with code context can parse it — labeled sections, `{:?}` debug formatting of NYES traces, structured FIR-tree dumps. See [references/cleanup.md](references/cleanup.md#output-formatting-for-agent-readability).
5. **Never commit from inside this skill.** Commits belong to the user's explicit request.
6. **Runtime state is the only source of truth.** A hypothesis about why a brane evaluates to NK without an observed NYES trace and an `ib_search` result is a guess. Do not fix guesses.

---

## What to do right now

1. Read [references/test-template.md](references/test-template.md) and write the temporary test.
2. Read [references/nyes-tracing.md](references/nyes-tracing.md) and step the FVM while recording NYES.
3. Read [references/fir-inspection.md](references/fir-inspection.md) and query the FIR/brane state at the offending point.
4. Read [references/cleanup.md](references/cleanup.md) and promote or delete the test.
