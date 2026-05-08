---
foop: 12
title: Alarms — diagnostic levels emitted by compiler and evaluator
author: hc <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-05-08
phase: phase-1
supersedes: []
---

# FOOP-12: Alarms — diagnostic levels emitted by compiler and evaluator

## Abstract

Defines an alarm system for the Foolish compiler and evaluator. Alarms are
structured diagnostic messages with severity levels that flow through the
compilation and evaluation pipeline. Unlike errors (which halt execution),
alarms are informational — they provide visibility into what the system is
doing without stopping it.

## Motivation

Currently, the compiler and evaluator produce minimal diagnostics: parse errors
as `Result::Err`, and nothing else. When a program has subtle issues (forward
references that take many steps to resolve, searches that become ECONSTANIC,
division by zero), the user sees no warning — just a potentially unexpected
result.

Alarms give the user a window into the evaluation process:

- **INFO**: trace-level details (e.g., "search resolved x = 5")
- **WARN**: potential issues (e.g., "search became ECONSTANIC — unbound name")
- **MILD**: notable but non-fatal (e.g., "division by zero produced NK")
- **PANIC**: internal errors that should never happen (e.g., "invariant violated")

This helps Foolish users debug their programs and helps developers debug the
compiler/evaluator itself.

## Specification

### Alarm Levels

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlarmLevel {
    Info,    // Trace-level: useful for debugging
    Warn,    // Potential issue in user code
    Mild,    // Notable event (division by zero, etc.)
    Panic,   // Internal error — should never happen
}
```

### Alarm Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub level: AlarmLevel,
    pub code: String,        // Machine-readable code: "DIV-BY-ZERO", "UNBOUND-NAME"
    pub message: String,     // Human-readable description
    pub source: AlarmSource, // Where it came from
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlarmSource {
    Compiler,
    Evaluator,
}
```

### AlarmSink Trait

```rust
pub trait AlarmSink: Send + Sync {
    fn record(&self, alarm: Alarm);
}
```

The `AlarmSink` is injected into the compiler and evaluator. By default, a
`VecAlarmSink` collects all alarms into a `Vec<Alarm>`. A `FilteringAlarmSink`
can suppress INFO-level messages. A `NullAlarmSink` discards everything.

### NKFir Alarm Integration

Per the spec, `NKFir` gains an optional alarm field:

```rust
pub struct NkFir {
    pub(crate) reason: String,
    pub(crate) state: Nyes,
    pub(crate) alarm: Option<Alarm>, // New: why did we become NK?
}
```

When a `NKFir` is created due to a specific alarm-worthy event (division by zero,
search failure), the FIR carries the alarm. When the program is sequenced, the
alarm is emitted.

### Compiler Alarms

| Code | Level | When |
|------|-------|------|
| `PARSE-ERROR` | Warn | Syntax error in source |
| `DEPRECATED` | Warn | Feature is deprecated |
| `DEFERRED` | Warn | Feature deferred to future phase |

### Evaluator Alarms

| Code | Level | When |
|------|-------|------|
| `DIV-BY-ZERO` | Mild | Division by zero produces NK |
| `UNBOUND-NAME` | Info | Search becomes ECONSTANIC (unbound name) |
| `DEPTH-EXCEEDED` | Panic | Evaluation step limit reached |
| `INVARIANT-VIOLATED` | Panic | Internal invariant broken |

### Alarm Threading

The alarm sink is threaded through the evaluation scope:

```rust
pub struct Scope {
    // ... existing fields ...
    pub alarms: Option<Rc<RefCell<dyn AlarmSink>>>,
}
```

When an alarm-worthy event occurs, the code emits:

```rust
if let Some(ref sink) = scope.alarms {
    sink.borrow().record(Alarm {
        level: AlarmLevel::Mild,
        code: "DIV-BY-ZERO".to_string(),
        message: "Division by zero".to_string(),
        source: AlarmSource::Evaluator,
    });
}
```

## FIR Impact

- `NKFir` gains `alarm: Option<Alarm>` field
- JSON serialization includes alarm when present
- Roundtrip tests must preserve alarm data

## UBC Step Impact

- Division by zero emits `DIV-BY-ZERO` alarm instead of silently producing NK
- Search becoming ECONSTANIC emits `UNBOUND-NAME` alarm
- Step limit exceeded emits `DEPTH-EXCEEDED` alarm
- Scope gains optional `AlarmSink`

## Test Plan

- Unit tests for alarm level ordering and serialization
- Unit tests for `VecAlarmSink` collecting alarms
- Unit tests for `NKFir` with alarm roundtripping through JSON
- Approval test: division by zero produces NK with alarm
- Approval test: unbound name produces ECONSTANIC with alarm
- Approval test: normal program produces no alarms

## Rejected Alternatives

### A. Use Rust logging (log/tracing)

**Rejected**: logging is external to the Foolish VM. Alarms are part of the
language semantics — they're emitted by the program's execution, not by
the runtime infrastructure. Logging is for the host; alarms are for the
Foolisher.

### B. Errors instead of alarms

**Rejected**: errors halt execution. Alarms are informational — they
provide visibility without stopping the program. A division by zero
producing NK is notable but not fatal.

### C. No alarm system

**Rejected**: without alarms, the user has no visibility into why their
program produced unexpected results. Debugging becomes guesswork.

## Open Questions

- Should alarms be emitted synchronously or collected and returned?
  (Current: collected in sink, returned after evaluation)
- Should the CLI show alarms by default, or only with a flag?
  (Current: show WARN and above by default)
- Should alarms have source locations (line/column)?
  (Defer: add when parser provides spans)

## References

- `foolish-core/src/fir.rs`: NKFir definition
- `foolish-core/src/ubc.rs`: Scope and evaluator
- `foolish-core/src/compiler.rs`: Compiler
