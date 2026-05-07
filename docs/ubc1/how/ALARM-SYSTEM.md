# Foolish Alarm System: Rust Tracing Instrumentation Specification

## Overview

This specification defines the instrumentation of the `tracing` crate throughout the Foolish Rust
codebase. In Foolish parlance, structured logging and diagnostic observation is called the **Alarm
System** — because the language itself has no print statement, all internal observation is an
*alarm* raised by the machine.

## Current State

- **foolish-core**: Zero logging. Runtime is entirely silent.
- **foolish-parser**: Zero logging. Parser errors surface via `Result` only.
- **foolish-cli**: `eprintln!` at three error sites (lines 88, 130, 134 of `main.rs`).
- **foolish-web**: Zero logging. HTTP errors surface via axum response only.
- **No tracing/logging crate** is present in any `Cargo.toml`.

## Design Principles

1. **`tracing` crate** — industry standard for structured, hierarchical, annotated logging in Rust.
   Over `log`/`env_logger`: event vs. span distinction, layered subscribers, zero-cost when
   disabled, structured fields, async-safe.
2. **Library modules emit events only** — `foolish-core` and `foolish-parser` are libraries. They
   call `trace!`, `debug!`, `info!`, `warn!`, `error!`. They do NOT configure subscribers.
3. **Applications configure subscribers** — `foolish-cli` and `foolish-web` decide format, output
   destination, and level. The library is subscriber-agnostic.
4. **Bounded cost** — every macro call is gated by a level check. When tracing is disabled, cost
   is a single static comparison. No format string evaluation unless the event is emitted.
5. **No secrets in traces** — FIR content, source code, and parameter values are safe. No
   credentials, keys, or sensitive material is logged.

## Dependencies

Add to workspace `Cargo.toml`:

```toml
[workspace.dependencies]
tracing = "0.1"
```

Per-module:

| Module          | Dependency              | Purpose                        |
|-----------------|-------------------------|--------------------------------|
| foolish-parser  | `tracing`               | Parse event tracing            |
| foolish-core    | `tracing`               | UBC execution tracing          |
| foolish-cli     | `tracing`, `tracing-subscriber` | Subscriber config (fmt, env filter) |
| foolish-web     | `tracing`, `tracing-subscriber` | Subscriber config (fmt, env filter) |

## Span Hierarchy: Branes as Spans

The fundamental principle: **each brane is a span**.

Branes are nested containment structures in Foolish. Tracing spans are nested containment
structures in Rust. The two structures mirror each other naturally — not as a forced analogy, but
because both describe the same thing: *scoped units of computation with parent-child relationships*.

### Brane-Span Mapping

| Brane Concept              | Tracing Equivalent                            |
|---------------------------|-----------------------------------------------|
| Brane opening             | Span creation (`info_span!`)                  |
| Brane state transition    | Event within the span (`debug!`)              |
| Statement evaluation      | Sub-event or child span within the brane span |
| Nested brane evaluation   | Child span (entered from parent brane span)   |
| Search across brane boundary | Event that references the parent span       |
| Constanic clone (detachment) | New span for the recoordinated clone        |
| Brane reaches Constant/Nk | Span closure                                  |
| Brane depth in source     | Span depth in trace output                    |

### Span Tree Example

For the Foolish program `{a=1, b=<a>; c=a+b;}`:

```
[info]  compile "source"                                    source_len=24
  [debug] compile_astn "Brane"
    [info]  brane "main"                                    state=Embryonic, stmts=3
      |--- (top-level brane span — lives for entire evaluation)
      |
      | [debug] state_transition                           from=Embryonic, to=Braning
      |
      | [debug] statement                                  idx=0, name="a"
      |   [debug] state                                    Int(1), Independent
      |
      | [debug] statement                                  idx=1, name="b"
      |   [debug] search_resolved                          pattern="^a$", target_state=Constant
      |   [debug] state                                    Int(1), Constant
      |
      | [debug] statement                                  idx=2, name="c"
      |   [debug] binary_op_evaluated                      op="+", left=1, right=1, result=2
      |   [debug] state                                    Int(2), Constant
      |
      [info]  state_transition                              from=Braning, to=Constant
    [/info]  brane "main" — closed
```

### Why Brane=Span Works

1. **Visual correspondence**: The trace output's indentation mirrors the brane structure. A reader
   can look at the trace and see the brane's containment hierarchy.
2. **Lifecycle alignment**: A span opens when a brane begins evaluation, closes when it reaches a
   terminal state. No artificial span boundaries.
3. **Search across boundaries**: When a search in brane A resolves to a value in brane B's parent
   span, the trace shows the event in A's span referencing B's parent — exactly like the
   semantic relationship.
4. **Detachment and recoordination**: When a brane is cloned and recoordinated (constanic_clone),
   the new evaluation opens a new span — detached from the original, just like the FIR clone
   is detached from its original AB/IB context.

### Non-Brane Spans

Not everything is a brane. Compiler and parser phases use procedural spans:

```
compile (source)                                        [info]  — one span per source invocation
  compile_astn (ast_node)                               [debug] — one span per AST node
  brane (evaluated FIR)                                 [info]  — one span per brane
    statement (indexed evaluation)                      [debug] — one event per statement
      search_resolved / binary_evaluated / ...          [debug] — result events
    constanic_clone (detached recoordination)           [debug] — new child span
```

### Span Fields

Each span carries structured fields:

| Span                     | Fields                                              |
|--------------------------|-----------------------------------------------------|
| `compile`               | `source_len` (usize)                                |
| `compile_astn`          | `variant` (&str)                                    |
| `brane`                 | `characterizations` (&[str]), `stmt_count` (usize), `initial_state` (Nyes) |
| `statement`             | `idx` (usize), `name` (Option<&str>)                |
| `constanic_clone`       | `source_state` (Nyes), `permit_nye` (bool), `new_state` (Nyes) |

## Event Taxonomy

Events represent discrete, observable occurrences. Every alarm site in the codebase maps to a
tracing event.

### Level Assignment Policy

| Level    | When                                                        | Examples                                            |
|----------|-------------------------------------------------------------|-----------------------------------------------------|
| `trace!` | Per-step/loop iteration, high frequency                    | Each step loop iteration, child stepping            |
| `debug!` | State transitions, search resolution, algorithm decisions  | NYES state change, search resolved, short-circuit   |
| `info!`  | Top-level milestones                                        | Compilation complete, evaluation complete           |
| `warn!`  | Recoverable anomalies                                      | State stuck (no progress), max steps approaching    |
| `error!` | Failures that return `Err` or produce Nk FIR               | Division by zero, unknown op, infinite loop         |

### Complete Event Map

#### Parser Phase (`foolish-parser`)

| Location                              | Level  | Event                                    |
|---------------------------------------|--------|------------------------------------------|
| `parse()` entry                       | `info!`  | "Parsing source" `source_len`            |
| Each AST node produced                | `debug!` | "Parsed node" `variant`                  |
| Parser error returned                 | `error!` | "Parse failed" `error`                   |

#### Compiler Phase (`foolish-core::compiler`)

| Location                              | Level  | Event                                    |
|---------------------------------------|--------|------------------------------------------|
| `compile()` entry                     | `info!`  | "Compiling source" `source_len`          |
| Unsupported construct (if-then-else)  | `error!` | "Unsupported: if-then-else (FOOP=2)"    |
| Unsupported construct (upward search) | `error!` | "Unsupported: upward search (Phase 7)"  |
| Unsupported construct (detachment)    | `error!` | "Unsupported: detachment brane (Phase 7)"|
| `NotImplemented` AST                  | `error!` | "Not yet implemented" `reason`           |

#### UBC Runtime (`foolish-core::ubc`)

| Location                              | Level  | Event (within brane span)                |
|---------------------------------------|--------|------------------------------------------|
| Brane evaluation begins               | `info!`  | "Brane evaluation started" `stmt_count` `characterizations` |
| Brane state transition                | `debug!` | "Brane state" `from` `to`                |
| Statement stepping                    | `debug!` | "Statement" `idx` `name` `state_after`   |
| Step iteration (hot loop)             | `trace!` | "Step" `step_n` `state_before` `state_after` |
| Brane reaches terminal state          | `info!`  | "Brane complete" `final_state` `steps`   |
| Infinite loop detected                | `error!` | "Brane infinite loop" `final_state` `steps` |
| `constanic_clone` — Econstanic reset  | `debug!` | "Clone detached" `from` `to`             |
| `constanic_clone` — Woconstanic reset | `debug!` | "Clone detached" `from` `to`             |
| `constanic_clone` — NYE permitted     | `debug!` | "Clone detached" `state`                 |
| `constanic_clone` — NYE rejected      | `warn!`  | "Clone failed, producing Nk" `state`     |
| `compute_binary` — division by zero   | `error!` | "Division by zero" `left` `right`        |
| `compute_binary` — unknown op         | `error!` | "Unknown binary op" `op`                 |
| `compute_unary` — unknown op          | `error!` | "Unknown unary op" `op`                  |
| `short_circuit` — chain followed      | `debug!` | "Short circuit" `links_followed` `end_state` |
| `has_unresolved_forward_refs` — true  | `debug!` | "Forward refs unresolved"                |

#### FIR Stepping (`foolish-core::fir`)

| Location                              | Level  | Event                                    |
|---------------------------------------|--------|------------------------------------------|
| Search resolved (unanchored)          | `debug!` | "Search resolved" `pattern` `target_state` |
| Search resolved (anchored)            | `debug!` | "Anchored search resolved" `pattern` `target_state` |
| Search -> Econstanic (not found)      | `debug!` | "Search not found" `pattern`             |
| Search -> Nk (bad anchor)             | `debug!` | "Search anchor failed" `pattern` `anchor_state` |
| Search short-circuit                  | `debug!` | "Search short-circuited" `chain_length`  |
| BinaryOp -> constant (evaluated)      | `debug!` | "BinaryOp evaluated" `op` `left` `right` `result` |
| BinaryOp -> Nk (propagated)           | `debug!` | "BinaryOp NK propagated" `op`            |
| UnaryOp -> constant (evaluated)       | `debug!` | "UnaryOp evaluated" `op` `val` `result`  |
| UnaryOp -> Nk (propagated)            | `debug!` | "UnaryOp NK propagated" `op`             |
| Index resolved                        | `debug!` | "Index resolved" `offset` `anchored`     |
| Index -> Nk                           | `debug!` | "Index failed" `offset`                  |
| HeadTail resolved                     | `debug!` | "HeadTail resolved" `is_head`            |
| Concatenation merged                  | `debug!` | "Concatenation merged" `element_count` `stmt_count` |
| Concatenation -> Nk                   | `debug!` | "Concatenation NK" `nk_index`            |
| Brane state transition                | `debug!` | "Brane state" `from` `to`                |

#### CLI Boundary (`foolish-cli`)

| Location                              | Level  | Event                                    |
|---------------------------------------|--------|------------------------------------------|
| `cmd_run` — evaluation error          | `error!` | "Evaluation failed" `error`              |
| `cmd_step` — step error               | `error!` | "Step error" `error`                     |
| `cmd_repl` — compile error            | `error!` | "REPL compile error" `error`             |
| `cmd_repl` — eval error               | `error!` | "REPL eval error" `error`                |

## Subscriber Configuration

### CLI (`foolish-cli`)

Default: human-readable, stdout, `INFO` level. Controlled by `RUST_LOG` env var.

```rust
// In main() before any command execution
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("foolish=info".parse()?)
    )
    .with_target(true)
    .init();
```

| Mode      | `RUST_LOG`          | Description                              |
|-----------|---------------------|------------------------------------------|
| Default   | (unset)             | `INFO` and above                         |
| Debug     | `RUST_LOG=debug`    | All debug events                         |
| Verbose   | `RUST_LOG=trace`    | Per-step tracing, high volume            |
| Quiet     | `RUST_LOG=off`      | No tracing output                        |
| Core only | `RUST_LOG=foolish_core=debug` | Only core, not parser or CLI     |

A `--verbose` / `-v` CLI flag sets `RUST_LOG=debug` before subscriber init.

### Web (`foolish-web`)

Same subscriber config, but default level is `WARN` (less noise in server logs).
Each request can carry an optional span for correlation.

## Implementation Phases

### Phase 1: Dependency and Subscriber Setup

1. Add `tracing` to workspace dependencies
2. Add `tracing` to `foolish-parser` and `foolish-core` dependencies
3. Add `tracing` + `tracing-subscriber` to `foolish-cli` and `foolish-web`
4. Initialize subscriber in `foolish-cli::main()` and `foolish-web` entry point
5. Verify: `cargo build --workspace` succeeds

### Phase 2: Parser and Compiler Spans

1. Wrap `Compiler::compile()` and `compile_astn()` in spans
2. Add `info!` for compilation start/end
3. Add `error!` for unsupported constructs
4. Verify: all 16 approval tests pass

### Phase 3: Brane Spans and UBC Runtime Events

1. Add brane span in `NormalBraneFir::step_one()` — each brane gets its own span
2. Add statement-level `debug!` events within the brane span
3. Add `trace!` for step loop iterations (fires inside the brane span)
4. Instrument NYES state transitions as `debug!` events within the brane span
5. Instrument `constanic_clone` as child span — represents brane detachment/recoordination
6. Instrument error paths with `error!` and `warn!`
7. Verify: all 16 approval tests pass, `RUST_LOG=debug` shows brane tree structure

### Phase 4: FIR Step Events

1. Add `step_one` span in `Steppable` trait default or per-implementation
2. Instrument search resolution events
3. Instrument arithmetic evaluation events
4. Instrument brane state transitions
5. Verify: all 16 approval tests pass

### Phase 5: Validation and Cleanup

1. Run `cargo test` — all tests pass at all log levels
2. Run `RUST_LOG=trace cargo run --package foolish-cli -- step <file>` — output is readable
3. Run `RUST_LOG=off` — performance regression is negligible (<5% at `trace!` level off)
4. Review: no secrets, no user source code in error events

## Nk FIR and Tracing

Nk FIR is the **in-language** error representation. Tracing is the **out-of-band** diagnostic.
They are complementary:

- **Nk FIR**: What the program sees. A value that propagates through computation.
- **Tracing event**: What the developer sees. A diagnostic that explains why Nk was produced.

When Nk FIR is produced, **both** should fire:

```rust
// Example: division by zero in compute_binary()
error!(
    target: "foolish_core::ubc::arithmetic",
    operation = "div",
    left = left,
    right = right,
    "Division by zero"
);
Ok(Fir::Nk(Box::new(NkFir {
    reason: "division by zero".to_string(),
    state: Nyes::Nk,
})))
```

## Target Namespaces

Events use target namespacing for subscriber filtering:

| Target                                  | Module                     |
|-----------------------------------------|----------------------------|
| `foolish_parser::parse`                 | Parser                     |
| `foolish_core::compiler`                | AST-to-FIR compilation     |
| `foolish_core::ubc`                     | UBC runtime evaluation     |
| `foolish_core::ubc::arithmetic`         | Binary/unary computation   |
| `foolish_core::ubc::search`             | Search resolution          |
| `foolish_core::ubc::constanic`          | Constanic clone/recoord    |
| `foolish_core::ubc::brane`              | Brane stepping             |
| `foolish_core::fir`                     | FIR step implementations   |
| `foolish_core::sequencer`               | Formatting/output          |
| `foolish_cli`                           | CLI command execution      |
| `foolish_web`                           | Web server request handling|

## Cost and Performance

- **No subscriber initialized**: tracing macros are no-ops. Cost = zero.
- **Subscriber initialized, event off**: level check only. Cost = one static comparison.
- **Subscriber initialized, event on**: field serialization + format. Cost = microseconds.
- **`trace!` level**: high frequency (per step). Total volume = steps x events_per_step. Can be
  millions of events for long-running programs. Use `RUST_LOG=trace` only for debugging.

## Anti-Patterns to Avoid

1. **Do NOT** use `println!` or `eprintln!` in `foolish-core` or `foolish-parser`.
2. **Do NOT** configure subscribers in library modules.
3. **Do NOT** log FIR trees inline (use `Sequencer::format` in debug only, never at `info!`).
4. **Do NOT** log user source code in production events (compiler phase only, and only length).
5. **Do NOT** use `error!` for expected language behavior (NK is not an error condition per se).
6. **Do NOT** add tracing to hot loops without measuring impact. The step loop is hot.

## Example: Instrumented Brane Evaluation

```rust
/// Step a brane, recording events within a span that mirrors the brane itself.
impl Steppable for NormalBraneFir {
    fn step_one(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        let span = tracing::debug_span!(
            "foolish_core::ubc::brane",
            stmts = self.statements.len(),
            chars = ?self.characterizations,
            current_state = %self.state,
        );

        _ = span.enter(); // events fire within this span

        match self.state {
            Nyes::Prembrionic => {
                self.state = Nyes::Embryonic;
                debug!(from = "Prembrionic", to = "Embryonic", "state_transition");
                Ok(None)
            }
            Nyes::Embryonic => {
                self.state = Nyes::Braning;
                debug!(from = "Embryonic", to = "Braning", "state_transition");
                Ok(None)
            }
            Nyes::Braning | Nyes::Woconstanic => {
                // Each statement step is an event within the brane span
                crate::ubc::re_step_brane_bodies(self, scope)?;
                debug!(to = %self.state, "re_step_complete");
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
```

And the step loop — where `trace!` fires inside the brane span:

```rust
pub fn run_to_completion_with_scope(fir: &mut FirRef, scope: &Scope)
    -> Result<(), UbcError>
{
    // If root FIR is a brane, open the brane span here.
    // Otherwise open a generic evaluation span.
    let initial_state = fir.borrow().state();
    let variant = fir.borrow().fir_variant();

    let span = tracing::info_span!(
        "foolish_core::ubc",
        fir_variant = variant,
        initial_state = %initial_state,
        "evaluation"
    );
    let _guard = span.enter();

    let mut steps = 0u64;
    let mut max_steps = 100000;

    loop {
        if max_steps == 0 {
            error!(steps = steps, "Infinite loop detected");
            return Err(UbcError::Eval("infinite loop detected".to_string()));
        }
        max_steps -= 1;
        steps += 1;

        let prev_state = fir.borrow().state();
        if prev_state.is_terminal() { break; }

        let replacement = step_with_scope(fir, scope)?;
        if let Some(repl) = replacement {
            *fir = fir_to_ref(repl);
        }

        let new_state = fir.borrow().state();
        trace!(step = steps, from = %prev_state, to = %new_state, "step");

        if prev_state == new_state { break; }
        // ... rest of loop
    }

    info!(steps = steps, final_state = %fir.borrow().state(), "complete");
    Ok(())
}
```

## Last Updated

**Date**: 2026-05-07
**Updated By**: Claude Code / cyankiwi/Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Brane-as-span paradigm. Spans now mirror brane structure: each brane is a span, state transitions are events within the span, nested brane evaluation creates child spans, constanic_clone (detachment) opens new spans. Added visual trace example showing brane hierarchy, updated UBC event map and implementation phases.
