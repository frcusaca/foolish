# Plan: Remove einmo evaluate subcommand, update docs

## Context
`einmo evaluate` is redundant — it's the same `evaluate_all` infrastructure as `run_einmo_tests`, just at Output validation level. The test framework already supports escalating validation levels (Output, Checked, Verified). Users/agents should run tests at the appropriate level instead.

## Todos

### 1. Remove `einmo evaluate` from CLI
**File:** `einmo/src/cli.rs`
- Remove `Evaluate(EvaluateArgs)` variant from `Command` enum
- Remove `EvaluateArgs` struct
- Remove `CommandEvaluator` struct and `impl Evaluator for CommandEvaluator`
- Remove `fn cmd_evaluate(args: EvaluateArgs)` function
- Remove `Command::Evaluate(a) => cmd_evaluate(a)` from dispatch
- Remove unused `use std::io::Write` if only used by CommandEvaluator
**Verify:** `cargo check -p einmo` passes

### 2. Remove `tempfile` from dev-dependencies if unused
**File:** `einmo/Cargo.toml`
- Check if tempfile is used anywhere else in einmo (tests)
- If only used by the removed CommandEvaluator, remove it
**Verify:** `cargo check -p einmo` passes

### 3. Update README.md
**File:** `README.md`
- Replace `einmo evaluate` examples with `cargo test` examples showing each validation level:
  - Output level (evaluate only, no correspondence check)
  - Checked level (evaluate + correspondence with checked/)
  - Verified level (evaluate + correspondence with verified/)
- Keep `einmo compare`, `einmo promote`, `poor_einmo.sh` examples

### 4. Update AGENTS.md
**File:** `AGENTS.md`
- Replace `einmo evaluate` examples with `cargo test` examples
- Show how to run at each validation level
- Keep the einmo review workflow (compare, promote, poor_einmo.sh)

### 5. Update einmo.README.md
**File:** `einmo.README.md`
- Remove any references to `einmo evaluate` subcommand
- Update CLI table to remove evaluate entry

### 6. Verify tests pass
- `cargo test -p foolish-ubca --lib -- run_einmo_tests` passes
- `git diff output/` shows 0 changes (skip-write optimization still works)

### 7. Commit
```
feat(einmo): remove evaluate subcommand, update docs for validation levels

einmo evaluate was redundant — same evaluate_all infrastructure as
run_einmo_tests, just at Output level. Users should run tests at the
appropriate validation level instead.

opencode, mimo-v2.5-pro
```
