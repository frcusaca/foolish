# Phase 4 — CLI

> Goal: A command-line tool that wires Phase 1 + Phase 2 + Phase 3 into a
> daily-driver binary. Users can compile, evaluate, and inspect Foolish
> source from the shell, and use the REPL to compose Foolish snippets
> interactively.

> Phase 4 follows Phase 3 (Concatenation) so the CLI ships with
> meaningful composition capability. Without concatenation, the REPL
> would be limited to single-statement extensions; with it, the REPL can
> evaluate composed expressions interactively.

---

## Phase 4 Deliverable

A `foolish` executable with subcommands:

| Subcommand | Behavior |
|-----------|----------|
| `foolish compile <file.foo>` | Phase 1 only — emit FIR JSON to stdout |
| `foolish run <file.foo>` | Phase 1 + Phase 2 + Phase 3 — emit final evaluation result |
| `foolish step <file.foo>` | Phase 1 + Phase 2 — emit intermediate steps for debugging |
| `foolish repl` | Interactive REPL: each line extends a persistent top-level brane |

---

## REPL Session Model

Each REPL line is appended as a new statement to one persistent
top-level brane. Later lines see earlier names via unanchored backward
search. Per Foolish's writing-order semantics (FOOP=2 area), **earlier
statements do NOT retroactively see names defined by later lines** —
backward search only walks backward.

```
> x = 42                ← statement 1, CONSTANT 42
> y = x + 1             ← statement 2, sees x; CONSTANT 43
=> y = 43
> z = missing           ← statement 3, ECONSTANIC (missing not found in IB or AB)
=> z = 🧠??
> missing = 7           ← statement 4, CONSTANT — but z stays ECONSTANIC.
=> missing = 7          ← (z is unchanged; backward search means z can't see this.)
> z2 = missing          ← statement 5, NEW search; finds missing = 7; CONSTANT 7
=> z2 = 7
```

If the user wants to resolve a previously-ECONSTANIC reference, they
must write a new statement that re-does the search. The REPL does NOT
re-step previously-evaluated statements when new lines arrive.

This matches Foolish's core writing-order semantic: meanings (names)
become available sequentially, and a name's visibility is determined by
where it sits relative to its references.

---

## Implementation Notes

- Use `scopt` or a small hand-rolled arg parser — no heavy CLI
  framework needed.
- Use `jline3` for line editing in the REPL (history, multi-line
  input via unmatched `{`).
- Multiline input: prompt `..` while `{` is unbalanced.
- Error recovery: parse errors print a friendly message and don't kill
  the REPL session.
- Concatenation in the REPL: each line CAN be a concatenation
  expression. The REPL appends the concatenation result to the session
  brane just like any other statement.

---

## Phase 4 Exit Criteria

- `foolish run` matches Phase 2 + Phase 3 approval test output for
  every `.foo` file.
- REPL handles multiline input, parse errors, concatenation, and
  ECONSTANIC display correctly.
- A REPL session test demonstrates that previously-ECONSTANIC
  statements are NOT retroactively resolved by later definitions
  (writing-order semantics preserved).
- Tab completion is *not* in scope.
- `foolish --help` is informative; subcommands have their own `--help`.

---

## Last Updated

**Date**: 2026-05-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Renumbered Phase 3 → Phase 4 (after promoting Concatenation
to Phase 3). Corrected REPL session model: per Foolish's writing-order
semantics, previously-ECONSTANIC statements are NOT re-resolved when
later lines define the missing name. Added Phase 3 (concatenation) to
the CLI deliverable list.
