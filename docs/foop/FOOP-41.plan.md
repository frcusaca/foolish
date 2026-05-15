# FOOP-41: UBCb — Message-passing brane computer; SPA1 parity plan — Implementation Plan

## CP-0: Parser and FIR (shared with UBC)

- [ ] Share UBC's parser, compiler, and FIR algebra
- [ ] Add message-passing fields to FIR types (LUID, message queue)
- [ ] Verify FIR roundtrip tests pass (shared with UBC)
- [ ] Governing FOOPs: FOOP-2, FOOP-4, FOOP-5, FOOP-9, FOOP-21

## CP-1: Basic evaluation (new UBCb FOOPs needed)

- [ ] Write UBCb Message Protocol FOOP (new FOOP to be created)
- [ ] Implement brane stepping via messages (no search, no constanic cloning)
- [ ] Implement literal value propagation
- [ ] Implement identification resolution within single brane
- [ ] Implement arithmetic reduction (all operands constant)
- [ ] UBCb produces identical output to UBC on literal-only branes

## CP-2: Search and constanic coordination (new UBCb FOOPs needed)

- [ ] Write UBCb Constanic Coordination FOOP (new FOOP to be created)
- [ ] Implement search resolution via messages
- [ ] Implement wake-up message queue and dependency tracking
- [ ] Implement constanic cloning asynchronously
- [ ] UBCb passes all 60+ Phase 2 approval tests

## CP-3: Concatenation (new UBCb FOOP needed)

- [ ] Write UBCb Concatenation Protocol FOOP (new FOOP to be created)
- [ ] Implement concatenation merge via message-passing
- [ ] UBCb passes all Phase 3 concatenation tests

## CP-4: Full SPA1 parity

- [ ] UBCb passes complete SPA1 test suite
- [ ] Cross-validation: byte-for-byte comparison with UBC approved baselines
- [ ] Decide: shared CLI binary with VM flag, or separate binary?

## New FOOPs to Create (deferred)

- [ ] Create "UBCb Message Protocol" FOOP — message types, channels, scheduling
- [ ] Create "UBCb Constanic Coordination" FOOP — wake-up queue, dependency tracking
- [ ] Create "UBCb Concatenation Protocol" FOOP — message-driven merge

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

- [ ] Create `foolish-ubcb-cli/` directory with `Cargo.toml` (workspace member)
- [ ] Extend `EvaluationResult` to carry `FirRef` per statement
- [ ] Update `UbcbEngine::evaluate()` / `evaluate_brane()` / `evaluate_single()` to populate FIR
  values in `EvaluationResult`
- [ ] Implement `run` subcommand (file read, compile, evaluate, print)
- [ ] Implement `repl` subcommand (brace-aware loop, compile, evaluate, print)
- [ ] Add `--states` flag to `run`, wire to output formatting
- [ ] Run `cargo test --workspace` — all tests must pass
- [ ] Sanity check: `foolish-ubcb-cli run` produces same output as `foolish-cli run` for CP-1 inputs

## Worktree

- [ ] Create worktree at `${HOME}/tmp/foolish-worktrees/5394-foop-14` with branch `foop/14-ubcb-spa1`
- [ ] Verify all work is complete in `${HOME}/tmp/foolish-worktrees/5394-foop-14` and committed to `foop/14-ubcb-spa1`
- [ ] Merge `foop/14-ubcb-spa1` to alpha
