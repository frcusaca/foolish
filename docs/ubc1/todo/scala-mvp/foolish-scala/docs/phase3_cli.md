# Phase 3 — CLI

> Goal: A command-line tool that wires Phase 1 + Phase 2 into a daily-driver
> binary. Users can compile, evaluate, and inspect Foolish source from the shell.

---

## Phase 3 Deliverable

A `foolish` executable with subcommands:

| Subcommand | Behavior |
|-----------|----------|
| `foolish compile <file.foo>` | Phase 1 only — emit FIR JSON to stdout |
| `foolish run <file.foo>` | Phase 1 + Phase 2 — emit final evaluation result |
| `foolish step <file.foo>` | Phase 1 + Phase 2 — emit intermediate steps for debugging |
| `foolish repl` | Interactive REPL: each line extends a persistent top-level brane |

---

## REPL Session Model

Each REPL line is appended as a new statement to one persistent top-level brane.
Later lines see earlier names via unanchored search. Constanic results may resolve
when later input adds the missing identifier.

```
> x = 42          ← statement 1, Constant 42
> y = x + 1       ← statement 2, sees x, Constant 43
=> y = 43
> z = missing     ← statement 3, Constanic (missing not found)
=> z = 🧠??
> missing = 7     ← statement 4, Constant
=> missing = 7
=> z = ???        ← REPL re-steps z, now Constant 7
=> z = 7
```

The REPL runs `Ubc.runToCompletion` after each line and re-displays any
previously-Constanic statements that have changed.

---

## Implementation Notes

- Use `scopt` or build a small hand-rolled arg parser — no need for a heavy CLI
  framework.
- Use `jline3` for line editing in the REPL (history, multi-line input via
  unmatched `{`).
- Multiline input: prompt `..` while `{` is unbalanced.
- Error recovery: parse errors print a friendly message and don't kill the REPL
  session.

---

## Phase 3 Exit Criteria

- `foolish run` matches Phase 2 approval test output for every `.foo` file.
- REPL handles multiline input, parse errors, and constanic re-resolution.
- Tab completion is *not* in scope.
- `foolish --help` is informative; subcommands have their own `--help`.

---

## Last Updated

**Date**: 2026-05-01
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Initial Phase 3 outline — CLI with compile/run/step/repl subcommands.
