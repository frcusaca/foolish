# DOC_AGENTS.md — AI Agent Instructions for the `docs/` Directory

Read this file before working on any document in `docs/` or its subdirectories.

---

## Style Authority

The comprehensive English style guide for Foolish documentation is
[`styleguide.md`](styleguide.md). Read it before writing or editing any document in this
directory tree. The style guide is strict for human-consumption documentation (`why/`,
`howto/`) and strongly preferred for agent-facing documentation (`how/`, `todo/`).

When this file and `styleguide.md` conflict, this file takes precedence.

---

## Purpose of This Documentation

The `docs/` tree covers the design and implementation of the Foolish programming language and
its brane computer (UBC). The current phase is transitioning from UBC (the working but
imperfect reference implementation) to UBC2. UBC2 takes the lessons learned from UBC
implementation — documented thoroughly in the engineering and status files — and produces a
better design before planning and implementation begins.

The development discipline is strict BDD. Tests come first:

1. Migrate UBC1 approval tests to form the UBC2 test suite (with suitable updates for the
   new output symbols and NK-vs-CONSTANIC semantics — see `todo/50_ubc2_baseline_migration.md`).
2. Write unit tests for UBC2 internal state behavior before or alongside implementation.
3. No feature is considered done until its tests pass.

---

## Directory Map

| Directory | Purpose | Numeric prefix means |
|-----------|---------|----------------------|
| `docs/how/` | Engineering reference and design specs | Lower = higher priority |
| `docs/howto/` | Literate `.foo` tutorials (Foolish syntax) | Chapter order |
| `docs/why/` | Philosophy and design rationale | — |
| `docs/todo/` | Project tracking, outstanding items, agent logs | Lower = higher priority |
| `docs/vintage_legacy/` | Pre-reorganization files; read for context, do not edit | — |

Numeric file prefixes encode priority — a file numbered `0` is more important and more
foundational than one numbered `50`. Within a group, `0.a` precedes `0.b`, etc. When adding new
tracking documents, choose a number that reflects where the work fits in the overall sequence.

### Readability Priorities by Directory

The `how/` and `todo/` directories are working documents that agents read and update frequently.
These directories prioritize agent readability: unambiguous symbol references, consistent
terminology, and structural clarity that an LLM can parse reliably. When editing files in these
directories, maintain this standard on an ongoing basis — each edit should leave the file at
least as readable to an agent as it was before.

The `why/` and `howto/` directories are primarily for human readers. They explain philosophy,
motivation, and tutorial material in a style that a person reads end-to-end. Prioritize natural
prose flow and readability for humans in these directories.

---
## Interactions Through @Agent Feedback Comments

!!!@AGENT,
   this doc comment addressed to you within this section of the DOC_AGENTS.md is to be read but not altered.
   DO NOT REMOVE !!
!!!

Humans will usually leave feedback to agents during collaboration by inserting Foolish comments with '@AGENT' tag in side a comment near(just before or after) where an information source can be found, OR it may be near where a change is being requested by the human. The comment is inserted, so the surrounding context should be read ignoring the comment when understanding it for the information or making the update.
```foolish
{ uhoh=1/(1-1); !!!@AGENT correct this please!!! }
```
```java
Clas....
   //!!@Agent In side other programming language blocks, the Foolish comment tend to reside within the language's native comment.
```
```python
def fn...
   ##!!@Agent In side other programming language blocks, the Foolish comment tend to reside within the language's native comment.
def raise_error: !!!@ Agent, I might tell you that this method needs to be called "generate_error" in a feedback comment block not native to this language!!!
    return """ This block of language describes the problem
          !!@Agent Sometimes, it's just a Foolish comment inside part of program of another language.
    """
```


When agent sees this comment, it knows to regard it as a human input regarding text in proximity of the comment and that it is expected to react to address the comment to an extent that the input @AGENT comment itself can be entirely removed. Agent should tollerate line comments

!! @Agent: This is a line comment you should keep in this document section for illustration purposes. DO NOT REMOVE.

This requires that, outside this current markdown "## Interactions" section, we never generate this type of comment AND that agent is to leave these explanation alone. 
## Key Files to Read When Starting Work

| File | When to read |
|------|-------------|
| `todo/human_todo_index.md` | Always — authoritative list of outstanding design decisions |
| `todo/agents.status.md` | Always — append-only progress log; scan recent entries |
| `how/ubc2_design.md` | For any UBC2 design or implementation work |
| `how/ubc_engineering.md` | For UBC1 reference (lessons learned, what not to repeat) |
| `todo/0.a.concatenation.woes.md` | Before working on concatenation, search dereferencing, or WOCONSTANIC — captures the resolved debate that shaped the current design |
| `todo/agents.scratch.log.md` | When a topic seems complex — prior sessions may have left reasoning here |
| `how/SYMBOL_TABLE.md` | When adding or referencing any symbol |

---

## Mandatory Actions on Every Commit

1. Update `todo/human_todo_index.md` — mark resolved items, add new ones if the work
   revealed open questions.
2. Append to `todo/agents.status.md` — one timestamped entry per commit summarising what
   changed and why. See format below.
3. Append to `todo/agents.scratch.log.md` if the session involved non-trivial reasoning
   that may be useful in future sessions (architecture debates, rejected alternatives, etc.).
4. Update the `## Last Updated` section of any `*.md` file you modify. Use plain labels
   (`Date:`, `Updated By:`, `Changes:`, `Previous:`) — the colon already marks them as labels.

---

## Append-Only Log Format

Both `agents.status.md` and `agents.scratch.log.md` are append-only. Never edit earlier
entries. Add new entries at the bottom.

Each entry header:

```
---
[$(date --iso-8601=ns)] <agent-identity> / <model-version>
```

For example:

```
---
[2026-02-24T11:22:27,026055815-08:00] Claude Code v1.0.0 / claude-sonnet-4-6
```

These files can be referenced by line number from other documents because they never shrink. These files are written specifically for agents to read, so they should differ from git commit which are meant for human consumption and more formal documentation.

---

## Style and Formatting

For English prose conventions — emphasis, structure, voice, punctuation, and formatting — read
[`styleguide.md`](styleguide.md). That file is the comprehensive reference. The subsections
here cover domain-specific conventions that supplement it.

Nyes state names (PREMBRYONIC, EMBRYONIC, BRANING, CONSTANIC, etc.) need no additional emphasis
when capitalized. The capitals already distinguish them from ordinary English.

### Terminology Conventions

State names are written in capitals when used as technical terms: PREMBRYONIC, EMBRYONIC,
BRANING, CONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT. Collectively the progression is
called the *Nyes states* (proper noun; the enum is `Nyes`).

For predicates, use `atCONSTANIC()` / `at_constanic()` for the exact-state check and
`achievedConstanic()` / `achieved_constanic()` for "at or beyond CONSTANIC." The adjective
*nigh* means pre-constanic (before and not including CONSTANIC). `isNigh()` returns true when
the FIR has not yet achieved constanic.

"Nigh X" in prose means "before and not including X." There is no `X-` notation. `X+` in
prose means "X and all states beyond it."

### The 🧠 Prefix

The 🧠 prefix is the Foolish semantic bracket prefix. `🧠e` is the single-token form of
`🧠⟦e⟧` — the meaning of expression `e`. Writing `🧠1 🧠2 🧠+` token by token is equivalent
to writing `🧠⟦1 + 2⟧` as a whole. It appears on literal branes (`🧠1`), system operators
(`🧠+`), and state symbols (`🧠?`, `🧠??`, `🧠???`). None are source syntax; all are
denotations. See `how/SYMBOL_TABLE.md` for unicode codepoints.

The sequencer output state symbols are `🧠?` (WOCONSTANIC), `🧠??` (CONSTANIC), and `🧠???`
(NK). NK (Not Known) is produced only when the UBC can prove a search fails in all possible
future contexts. Ordinary search failure is CONSTANIC, not NK.

### Java

- State enum: `Nyes` (the enum type); `Nyes.PREMBRYONIC`, `Nyes.CONSTANIC`, etc.
- Exact-state predicate: `fir.atConstanic()` — returns true only when `nyes == Nyes.CONSTANIC`.
- Achieved-constanic predicate: `fir.achievedConstanic()` — equivalent to
  `at_constanic || at_woconstanic || at_constant || at_independent`.
- Nigh predicate: `fir.isNigh()` — returns true while the FIR has not yet achieved constanic.
- Denotational symbols live in `BraneSymbol` (planned enum) — do not hard-code
  `"🧠?"` etc. as string literals in rendering code.
- Prefer the shallow FIR hierarchy: one `ProtoBrane` class with traits (`hasBoundary`,
  `isDetachment`, `sfMode`) over subclasses per expression type. See
  `how/ubc2_design.md § FIR Type Hierarchy`.
- Exception handling: catch only `ArithmeticException` in arithmetic FIRs. Let all other
  exceptions propagate (no silent NK wrapping of NPE / ClassCastException).

### Foolish

- Brane literals: `{...}` for full branes, `[...]` for detachment/liberation branes.
- Denotational symbols (`🧠+`, `🧠-`, `🧠*`, `🧠/`, `🧠−`, `🧠!`, `🧠1`, `🧠?`, etc.) are
  not source syntax; they are the meanings that source syntax maps to. A programmer writes
  `1 + 2`; the denotation is `{🧠1, 🧠2, 🧠+}`.
- Indentation: tabs for brane depth, optional spaces for continuation alignment.
  See `how/foolish_style_guide.md` for full rules.
- Comments: `!!` starts an inline comment.
- Sequencer output state symbols (`🧠?`, `🧠??`, `🧠???`) may appear in `.approved.foo` files
  but never in `.foo` source input files.
- Full style reference: `how/foolish_style_guide.md`.

---

## Concatenation Design Note

The file `todo/0.a.concatenation.woes.md` documents the resolved design debate over search
dereferencing and WOCONSTANIC race conditions. Its conclusions are now canonical parts of
`how/ubc2_design.md`. Read it before working on concatenation, the ConcatenationBrane
lifecycle, or anything touching CONSTANIC/WOCONSTANIC state transitions. The document records
*why* certain decisions were made, not just *what* was decided — this context prevents
re-opening closed debates.

---

## Last Updated

Date: 2026-02-27
Updated By: Claude Code v1.0.0 / claude-opus-4-6
Changes: Moved English prose style content (emphasis rules, structural elements table, code
block guidance, rendering target, LLM over-emphasis note) into `styleguide.md`. Replaced
verbose "Style Guide for New UBC2 Documents" section with compact "Style and Formatting"
linking to `styleguide.md`. Added "Style Authority" section establishing `styleguide.md` as
the comprehensive reference — strict for human docs, strongly preferred for agent docs, with
`DOC_AGENTS.md` having final say on conflicts. Resolved `bold_emphasis` DEFERRED item in
`styleguide.md`. Retained domain-specific subsections (Terminology, 🧠 Prefix, Java, Foolish).
Previous (2026-02-26): Added Rendering Target subsection — markdown files target browser rendering, HTML
entities acceptable when needed. Added Readability Priorities by Directory under Directory Map:
`how/` and `todo/` prioritize agent readability (ongoing), `why/` and `howto/` prioritize human
readability. Re-reviewed full document for agent readability; fixed heading level on directory
readability subsection.
Previous (2026-02-26): Renamed "Structure Before Emphasis" to "Structure for Emphasis." Added
references to Unix man pages, IEEE standards, Chicago Manual of Style, and Knuth as exemplars.
Added complete markdown structural elements table. Added "Code Blocks" subsection requiring
typed fenced blocks for any code longer than five tokens. Removed bold from Last Updated labels
throughout docs directory.
Previous (2026-02-26): Added "Writing Normal English" subsection framing conventions as normal
technical English practice. Addressed LLM over-emphasis pattern explicitly.
Previous (2026-02-25): Rewrote English Prose style guide with subsections for structure,
emphasis, and prose style. Removed @Agent feedback comments.
Previous (2026-02-24): Initial creation by claude-sonnet-4-6.
