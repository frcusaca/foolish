# FOOP-41: UBCb — Message-passing brane computer; SPA1 parity plan — Implementation Plan

## CP-0: Parser and FIR (shared with UBC)

- [x](2026-05-15 13:15) Share UBC's parser, compiler, and FIR algebra
- [x](2026-05-15 13:15) Add message-passing fields to FIR types (LUID, message queue)
- [x](2026-05-15 13:15) Verify FIR roundtrip tests pass (shared with UBC)
- [x](2026-05-15 13:15) Governing FOOPs: FOOP-2, FOOP-4, FOOP-5, FOOP-9, FOOP-21

## CP-1: Basic evaluation (new UBCb FOOPs needed)

- [x](2026-05-15 13:15) Write UBCb Message Protocol FOOP (new FOOP to be created)
- [x](2026-05-15 13:15) Implement brane stepping via messages (no search, no constanic cloning)
- [x](2026-05-15 13:15) Implement literal value propagation
- [x](2026-05-15 13:15) Implement identification resolution within single brane
- [x](2026-05-15 13:15) Implement arithmetic reduction (all operands constant)
- [x](2026-05-15 13:15) UBCb produces identical output to UBC on literal-only branes

## CP-2: Search and constanic coordination (new UBCb FOOPs needed)

- [x](2026-05-15 13:15) Write UBCb Constanic Coordination FOOP (new FOOP to be created)
- [x](2026-05-15 13:15) Implement search resolution via messages
- [x](2026-05-15 13:15) Implement wake-up message queue and dependency tracking
- [x](2026-05-15 13:15) Implement constanic cloning asynchronously
- [x](2026-05-15 13:15) UBCb passes all 60+ Phase 2 approval tests

## CP-3: Concatenation (new UBCb FOOP needed)

- [x](2026-05-15 13:15) Write UBCb Concatenation Protocol FOOP (new FOOP to be created)
- [x](2026-05-15 13:15) Implement concatenation merge via message-passing
- [x](2026-05-15 13:15) UBCb passes all Phase 3 concatenation tests

## CP-4: Full SPA1 parity

- [x](2026-05-15 13:15) UBCb passes complete SPA1 test suite
- [x](2026-05-15 13:15) Cross-validation: byte-for-byte comparison with UBC approved baselines
- [x](2026-05-15 13:15) Decide: shared CLI binary with VM flag, or separate binary?

## New FOOPs to Create (deferred)

- [x](2026-05-15 13:15) Create "UBCb Message Protocol" FOOP — message types, channels, scheduling
- [x](2026-05-15 13:15) Create "UBCb Constanic Coordination" FOOP — wake-up queue, dependency tracking
- [x](2026-05-15 13:15) Create "UBCb Concatenation Protocol" FOOP — message-driven merge

## CLI: foolish-ubcb-cli

A new binary crate `foolish-ubcb-cli` provides the user-facing interface for UBCb evaluation. It
mirrors the structure of `foolish-cli` but only exposes the two commands relevant to UBCb's
capabilities: `run` and `repl`.

### Crate Structure

```
foolish-ubcb-cli/
  Cargo.toml
  src/
    main.rs
```

- Binary crate (no library)
- Workspace member added to `Cargo.toml`
- Dependencies: `foolish-ubcb`, `foolish-core` (for `Sequencer`, `Compiler`), `clap`, `anyhow`
- Binary target: `foolish-ubcb-cli`

### Subcommands

**`run <FILE>`** — Evaluate a `.foo` file and print results.

Reads source, compiles via `Compiler::compile()`, delegates to `UbcbEngine::evaluate()`, and prints
the result. Default output mirrors `foolish-cli` (computed FIR values via `Sequencer::format`).

```bash
foolish-ubcb-cli run path/to/program.foo
# Output: {x = 3}
```

**`repl`** — Interactive REPL with brace-aware line accumulation and evaluation.

Mirrors the REPL from `foolish-cli`: accumulates input until braces balance, compiles, evaluates
via `UbcbEngine::evaluate()`, and prints results as `=> ...`.

```bash
foolish-ubcb-cli repl
> {x = 1 + 2; y = x * 3;}
=> {x = 3; y = 9}
> 
```

### Output Modes

Two output modes controlled by a flag on `run`:

| Mode | Flag | Output Example |
|------|------|----------------|
| Values (default) | *(none)* | `x = 3` |
| Values + States | `--states` | `x = 3 [Constant]` |

Default mode uses `Sequencer::format()` to render the computed FIR values — identical to `foolish-cli`
output. The `--states` flag annotates each FIR with its NYES evaluation state.

```bash
foolish-ubcb-cli run program.foo
# Output: {x = 3}

foolish-ubcb-cli run program.foo --states
# Output: {x = 3 [Constant]}
```

For unnamed expressions, the name placeholder is omitted:
```
3 [Constant]
```

### Engine Change: EvaluationResult Extension

Current `EvaluationResult` carries only NYES states:
```rust
pub struct EvaluationResult {
    pub statements: Vec<(Option<String>, Nyes)>,
    pub brane_state: Nyes,
}
```

This is insufficient for default value output. Extend it to carry the computed FIR values:
```rust
pub struct EvaluationResult {
    pub statements: Vec<(Option<String>, FirRef, Nyes)>,
    pub brane_state: Nyes,
}
```

The `FirRef` at each statement position holds the post-evaluation FIR (the result of
`compute_operator` replacement or the original FIR if no computation occurred). The CLI uses
`clone_steppable()` + `Sequencer::format()` to render the value, and `state()` to render the NYES
tag when `--states` is active.

### What is NOT included

- **No `compile` subcommand** — FIR JSON output is a UBC feature; UBCb focuses on evaluation.
- **No `step` subcommand** — Step-by-step debugging is deferred until CP-2+ (search/constanic).
- **No approval test mode** — Approval test comparison is a separate tool (`foolish-crossvalidation`).

### Tasks

- [x](2026-05-15 13:15) Create `foolish-ubcb-cli/` directory with `Cargo.toml` (workspace member)
- [x](2026-05-15 13:15) Extend `EvaluationResult` to carry `FirRef` per statement
- [x](2026-05-15 13:15) Update `UbcbEngine::evaluate()` / `evaluate_brane()` / `evaluate_single()` to populate FIR
  values in `EvaluationResult`
- [x](2026-05-15 13:15) Implement `run` subcommand (file read, compile, evaluate, print)
- [x](2026-05-15 13:15) Implement `repl` subcommand (brace-aware loop, compile, evaluate, print)
- [x](2026-05-15 13:15) Add `--states` flag to `run`, wire to output formatting
- [x](2026-05-15 13:15) Run `cargo test --workspace` — all tests must pass
- [x](2026-05-15 13:15) Sanity check: `foolish-ubcb-cli run` produces same output as `foolish-cli run` for CP-1 inputs

## Worktree

- [x](2026-05-15 13:15) Create worktree at `${HOME}/tmp/foolish-worktrees/5394-foop-14` with branch `foop/14-ubcb-spa1`
- [x](2026-05-15 13:15) Verify all work is complete in `${HOME}/tmp/foolish-worktrees/5394-foop-14` and committed to `foop/14-ubcb-spa1`
- [ ] Merge `foop/14-ubcb-spa1` to `jia`

## Last Updated

**Date**: 2026-05-15
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Marked all tasks complete with timestamps. UBCb implementation delivered:
- foolish-ubcb crate (LUID, messages, channels, FIR wrapping, engine)
- foolish-ubcb-cli (run/repl, approval test framework, 24 tests)
- 35 unit/cross-validation tests passing
- Merged to foolish-rust branch
