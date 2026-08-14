# Foolish Programming Language

*Where proximity meets computation, and containment creates clarity*

[![License](https://img.shields.io/badge/license-Open%20Source-blue.svg)](#)
[![Status](https://img.shields.io/badge/status-Active%20Development-green.svg)](#)
[![Einmo Output](https://github.com/frcusaca/foolish/actions/workflows/einmo-gate-output.yml/badge.svg)](https://github.com/frcusaca/foolish/actions/workflows/einmo-gate-output.yml)
[![Einmo Checked](https://github.com/frcusaca/foolish/actions/workflows/einmo-gate-checked.yml/badge.svg)](https://github.com/frcusaca/foolish/actions/workflows/einmo-gate-checked.yml)
[![Einmo Verified](https://github.com/frcusaca/foolish/actions/workflows/einmo-gate-verified.yml/badge.svg)](https://github.com/frcusaca/foolish/actions/workflows/einmo-gate-verified.yml)

## Welcome to the Future of Programming

Foolish is a revolutionary programming language that reimagines how humans interface with computers.
Built from the ground up for the 21st century, Foolish combines functional programming elegance with
intuitive natural metaphors to create a uniquely expressive and powerful development experience.

### Why Foolish?

- **Nature- and Life-Inspired Design**: Programming concepts mirror natural processes like cellular
  organization and proximity-based interactions found throughout living systems
- **High Semantic Throughput**: Express complex ideas with minimal cognitive overhead and remarkably
  low temporal latency
- **Advanced Search Capabilities**: Revolutionary brane search system for navigating and querying
  code structures with unprecedented flexibility
- **Compositional Architecture**: Functions, objects, and data structures unify seamlessly through
  elegant concatenation patterns
- **Concise and Sweet Syntax**: Extensive syntactical sugaring focuses your attention on logic
  rather than ceremony

### Our Vision and Goals

Foolish aspires to become the natural way anyone thinks about interacting with computers. We are
building toward a future where programming is:

- **Maximally Human**: Accessible, ergonomic, and adaptable design that is precise and
  high-functioning for human use
- **Safe and Precise**: Side-effect-free abstract functional programming foundation ensures
  reliability and predictability
- **Completely Transparent**: Full code transparency enables unprecedented understanding and
  debugging capabilities
- **Automatically Enhanced**: Built-in hooks support automatic programming, formal proofs,
  verification, and computing with uncertainty
- **Bidirectionally Communicative**: The system maximizes both human-to-computer and
  computer-to-human communication efficiency
- **Grounded in Reality**: Built-in methods seamlessly connect abstract thought to real-world
  implementations

A Foolisher might say the variable is *nye* (says 'nigh', any pre-constanic state) when they
encounter a FIR that has not reached constanic, or that it's *constanic* (says 'cons-TAN-nic',
constant in context) when it has reached a terminal evaluation state — one of WOCONSTANIC,
ECONSTANIC, CONSTANT, INDEPENDENT, or NK — and no further stepping is needed in the current
context. We say "that's a no-no" when we see `???` (NK). Fully evaluated expressions are values
that have achieved CONSTANT.

---

## Running the Rust Implementation

Requires Rust (cargo) installed.

```bash
cargo build --package foolish-cli --release
```

**Run a `.foo` file:**

```bash
cargo run --package foolish-cli -- run path/to/program.foo
```

**Debug with step-by-step evaluation:**

```bash
cargo run --package foolish-cli -- step path/to/program.foo
```

**Interactive REPL:**

```bash
cargo run --package foolish-cli -- repl
```

Type `{` to start a brane — the REPL accumulates lines until braces balance, then compiles, evaluates, and prints the result. Press `^D` to exit.

**Run all tests:**

```bash
cargo test --workspace                                   # all unit tests, all crates
cargo test -p foolish-ubca --lib -- einmo_gate_checked   # einmo approval gate (output == checked)
```

**Approval tests (einmo) — the three gates:**

```bash
cargo test -p foolish-ubca --lib -- einmo_gate_output    # every input evaluates + self-verifies in output/
cargo test -p foolish-ubca --lib -- einmo_gate_checked   # output matches the signed checked/ baseline
cargo test -p foolish-ubca --lib -- einmo_gate_verified  # checked matches the human-signed verified/

# Review and promote:
einmo compare output checked foolish-ubca/einmo_suite    # see what changed
einmo promote output to checked foolish-ubca/einmo_suite # promote (ONLY after the Promotion Review Gate — see foop.md)
poor_einmo.sh foolish-ubca/einmo_suite                   # interactive review
```

**Snapshot workflow**: Run a test → if output differs, einmo reports the divergent sections →
review with `einmo compare` → promote with `einmo promote output to checked`.
Use `poor_einmo.sh` for the interactive review loop (vim-based).

Foolish programs use the `.foo` extension and embrace a philosophy where **proximity creates
combination** and **containment enables organization**. The language provides rigorous abstraction
capabilities while maintaining interfaces that ground your computations to the physical and
biological realities you want to model.

## Running specific tests

**The central reference for running ONE test case or a SUBSET of cases** — the fast-iteration
loop while developing a feature. FOOP plan checkboxes link here and name their cases; the
command forms live ONLY in this section, so when the tooling changes (the einmo CLI is still
evolving), this one section is what gets updated.

A subset run is a development-loop ANALYSIS tool — it never replaces the full suite. At every
sub-section and phase boundary, the canonical judgment is:

```bash
cargo test --workspace                                   # all unit tests
cargo test -p foolish-ubca --lib -- einmo_gate_checked   # einmo approval gate
```

### Unit tests — select by name filter

`cargo test` selects tests by name filter after `--` (substring match on the full test path):

```bash
# One test group (every test whose full path contains "step_until"):
cargo test -p foolish-ubca --lib -- step_until

# Batch — several filters in ONE invocation; a test matching ANY filter runs:
cargo test -p foolish-ubca --lib -- step_until creation_display value_search

# Exactly one test, by full path (no substring matching):
cargo test -p foolish-ubca --lib -- --exact \
    evaluator::step_until_tests::step_until_line_number_finds_line

# Discover test names to filter on:
cargo test -p foolish-ubca --lib -- --list value_search
```

### Einmo cases — select with `--filter` and file arguments

The einmo GATE tests (`einmo_gate_output|checked|verified`) always evaluate the WHOLE suite —
they have no case selection. Case selection lives in the einmo CLI and works on the throwaway
`output/` stage only: a subset run never touches `checked/` or `verified/`.

The evaluator command below pipes each case's source through the Foolish CLI via `/dev/stdin`;
`head -c -1` strips the trailing newline so the result matches the gate byte-for-byte. Run from
the repository root and build the binaries first: `cargo build -p foolish-cli -p einmo` (then
use `einmo` from your PATH, or `./target/debug/einmo`).

```bash
# Re-evaluate ONE case, then compare it against the signed checked/ baseline:
einmo evaluate foolish-ubca/einmo_suite \
    --command "sh -c './target/debug/foolish-cli run /dev/stdin | head -c -1'" \
    --filter "foop/23/name_value_atomic"
einmo compare output checked foolish-ubca/einmo_suite \
    foop/23/name_value_atomic.foo.einmo

# Batch — the filter is a substring of the case path, so a shared prefix selects
# many cases at once (here: every foop/23 case). Cases sharing no substring need
# one invocation per group:
einmo evaluate foolish-ubca/einmo_suite \
    --command "sh -c './target/debug/foolish-cli run /dev/stdin | head -c -1'" \
    --filter "foop/23"

# Compare several SPECIFIC cases in one invocation (mirror-relative paths):
einmo compare output checked foolish-ubca/einmo_suite \
    foop/23/name_value_atomic.foo.einmo \
    foop/23/comprehensive.foo.einmo \
    misc/simple_addition.foo.einmo

# Which cases exist / currently differ:
einmo list foolish-ubca/einmo_suite --filter "foop/23" --differing
```

Note: einmo skips re-writing an output file whose evaluated body is unchanged; a file it DOES
rewrite gets einmo's default `①` envelope separator rather than the suite's `!!` Foolish
separator. That is framing only — the gate compares section bodies — but do not commit the
churn: `git checkout -- foolish-ubca/einmo_suite/output/` restores the gate-written framing.

If a subset run reveals a divergence, that is broken code, not a stale baseline — fix the code;
never `einmo promote` to make the diff go away (see `rust_instructions.md` §"Phase-by-phase
testing discipline").

---

## Documentation Layout

- **ubc1** — `docs/ubc1/` — current development version based on message-passing infrastructure
- **ubc0_1** — `docs/ubc0_1/` — reimplementation of ubc0 semantics using clarified microstates from ubc1 design
- **ubc0 (legacy)** — `docs/vintage_legacy/` — original UBC implementation, legacy reference

### Shared Documentation

The following documentation applies across all versions and remains at `docs/`:

- **why/** - Design philosophy and motivations (version-agnostic)
- **howto/** - Literate programming tutorials
- **styleguide.md** - Code style and formatting conventions
- **DOC_AGENTS.md** - Guidance for AI agents working on the codebase

## For AI Agents and Contributors

**AI coding assistants** (Claude Code, GitHub Copilot, Cursor, and others) should consult
**[AGENTS.md](AGENTS.md)** for comprehensive development guidance including:

- Environment detection
- Build requirements (Rust)
- Build commands
- Project structure (the Rust workspace)
- The Unicellular Brane Computer (UBC) implementation details
- Testing workflows (unit tests, approval tests)
- Git workflow and branch naming conventions for AI agents
- Common development tasks with complete examples

`AGENTS.md` is specifically written to enable AI agents to effectively contribute to the Foolish
project with minimal friction.

### Working with FOOPs (Foolish Optimization Process)

Language design proposals are tracked as FOOPs in `docs/foop/`. FOOP
filenames use a little-endian numbering convention — the digits in
`FOOP-XY.md` are written front-to-back, so `FOOP-31.md` is the FOOP
*after* `FOOP-21.md` (chronological order is recovered by reversing the
digits). The filename digits are the identifier; do not use the
frontmatter `foop:` sort key as the identifier in prose.

When creating a new FOOP or auditing the directory, use the helper
script — it handles the encoding for you:

```bash
python3 docs/foop/scripts/foop_check.py check      # verify no numbering gaps
python3 docs/foop/scripts/foop_check.py gen_next   # filename for the next FOOP
python3 docs/foop/scripts/foop_check.py list       # all FOOPs in chronological order
python3 docs/foop/scripts/foop_check.py get_last   # most-recently-created FOOP
```

See [foop.md](foop.md) for the full FOOP specification — the numbering
convention, the two-file (`FOOP-#.md` spec / `FOOP-#.plan.md` plan) layout,
plan construction, and the checkbox lifecycle. `AGENTS.md` carries only a
short summary and likewise points to `foop.md`.

## Key Features That Set Foolish Apart

### 🧬 Bio-Inspired Programming Model

Unlike traditional languages that force you to think in terms of machines, Foolish lets you think in
terms of natural systems. Branes mirror cellular organization, proximity drives interaction, and
containment enables natural hierarchies.

### 🔍 Revolutionary Search as First-Class Citizen

Stop scrolling through endless files. Foolish's integrated search system lets you query your code
from within the language itself. Find code by variable name, by value, or by association.

### 🎯 Functional Purity with Real-World Grounding

Enjoy the safety and predictability of pure functional programming while maintaining seamless
interfaces to the messy, stateful real world. The best of both paradigms, unified.

### 🔗 Natural Composition Through Proximity

Functions, data, and objects combine simply by being placed near each other. No complex import
systems, no verbose inheritance hierarchies—just natural, intuitive composition that mirrors how
ideas naturally combine in human thought.

### 🍯 Sweetened Syntax, Powerful Semantics

Extensive syntactical sugar means you write what you mean, not what the compiler wants. Focus on
your logic while Foolish handles the ceremony.

### 🤖 Built for the AI Age

Native support for uncertainty, automatic program generation, formal verification, and human-AI
collaborative programming. Foolish doesn't just run on computers—it bridges human and computational
intelligence.

### 👼 Built for Good

As we stand at the precipice of evolution, we must imbue our creations with all the best that we
have come to know. Foolish aims to be built for good, not evil.

---

## Table of Contents

- [Branes: The Foundation](#branes-the-foundation)
- [Sizes](#sizes)
- [Comments](#comments)
  - [Line Comments](#line-comments)
  - [Block Comments](#block-comments)
- [Expressions and Values](#expressions-and-values)
- [Names and Scope](#names-and-scope)
- [The Unknown](#the-unknown)
- [Renaming](#renaming)
- [Names, Searches, and Bounds](docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md) - Comprehensive guide to naming, search system, and detachment
  - [Names and Ordinates](docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md#names-and-ordinates)
  - [Scope and Name Resolution](docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md#scope-and-name-resolution)
  - [The Search System](docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md#the-search-system)
  - [Detachment Branes](docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md#detachment-branes-controlling-scope-boundaries)
  - [Search Paths](docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md#search-paths)
- [Advanced Features](docs/vintage_legacy/ADVANCED_FEATURES.md)
  - [Brane Operations](docs/vintage_legacy/ADVANCED_FEATURES.md#brane-operations)
  - [Control Flow](docs/vintage_legacy/ADVANCED_FEATURES.md#control-flow)
  - [Recursion](docs/vintage_legacy/ADVANCED_FEATURES.md#recursion)
- [Ecosystem](docs/vintage_legacy/ECOSYSTEM.md)
  - [Computer Reading Branes](docs/vintage_legacy/ECOSYSTEM.md#computer-reading-branes)
  - [The Unicellular Brane Computer (UBC)](docs/vintage_legacy/ECOSYSTEM.md#the-unicellular-brane-computer-ubc)
  - [The Multicellular Brane Computer](docs/vintage_legacy/ECOSYSTEM.md#the-multicellular-brane-computer)
  - [Typing](docs/vintage_legacy/ECOSYSTEM.md#typing)
  - [Relational Coordinates](docs/vintage_legacy/RELATIONAL_COORDINATES.md)
- [Development Notes](docs/vintage_legacy/DEVELOPMENT_NOTES.md)
- [TODO Items](docs/vintage_legacy/000-TODO_FEATURES.md)
- [Appendix](docs/vintage_legacy/APPENDIX.md)
  - [Styles](docs/vintage_legacy/STYLES.md)
  - [Keyboard Aid](docs/vintage_legacy/APPENDIX.md#keyboard-aid)
  - [Documentation Contributors](docs/vintage_legacy/APPENDIX.md#documentation-contributors)

---

## Branes: The Foundation

The Foolish language is in its most primitive form a containment and organization of values. We
contain values in something called a *brane*. The brane brings to mind concepts such as cell or
nucleus mem*brane*. The Foolish brane resembles traditional mathematical and programmatic concepts
such as sets, lists, maps or associative arrays, structs, enums or records. Foolish uses curly
braces to enclose values `{}`:

```foolish
{}                    !! Empty brane
{{}}                  !! Brane containing an empty brane
{{};{{}};{{{}}};}     !! Complex nested structure with multiple branes
{🌌={};}              !! Supports customizable alphabet
{愚=↑;}
{👶=👍;}
```

This ability to create containment will ultimately help us organize ideas. Branes can be nested
arbitrarily deep, allowing for sophisticated hierarchical data structures that mirror how we
naturally think about complex systems. Just as biological cells contain organelles, which contain
molecules, which contain atoms, Foolish branes can contain other branes in a natural, intuitive
manner.

## Sizes

In the physical world, containment membranes and units of organization tend to have observably
limited size. Cells can only be so big before they split or die. Atomic and subatomic objects have
to be so close, or otherwise the forces that keep them together stop working. Therefore, the Foolish
brane is also limited in size. The Foolish brane certainly should have finite size, and depending on
the computer, it may have a specified limit on the number of entries.

Inside the brane, it is a one-dimensional object. Its entries line up one after another. The true
dimension of entries are computational dependencies. So depending on the code inside, the actual
dependency axis could have smaller dimensions best visualized as a dependency DAG. The dependency
DAG and its subdimensions branch out and progress forward in their own time dimensions. Sometimes
more than one timeline merges together by being involved in an expression that refers to all of
them. Overall, the brane's fracturing timelines can be linearized, by stringing them into the
sequence of code as they originally appear inside the Foolish code, into a single time dimension.

## Comments

Comments in Foolish are expressions that contain unparsed Foolish. They are generally escaped using
multiple exclamation marks. Comments are part of the program that have no expressive effect on the
evaluation of the program.

### Line Comments

Line comments are like those from standard languages. The marking that begins the line comment is a
double exclamation mark `!!`.

```foolish
{
	!! This is a comment inside the brane
	!! This is another comment inside the brane.
	!! We can even exclaim inside a comment !!!
	!! Or simulate a banner !!
	!! - [ ] TODO: Check that this is possible ----^^^
}
```

### Block Comments

Block comments are enclosed by a pair of consecutive triple exclamation marks:

```foolish
{
	!!! Move along nothing to see here.

	     ##
	    ###
	     ##
	                #
	    ###################
	    #####################
	                #    ## #
	                    #####

	        ####
	      #########
	     ##       ##
	    #           #
	    #          ##
	     ##       ##
	      #########
	        ####
	        ####
	      #########
	     ##       ##
	    #           #
	    #          ##
	     ##       ##
	      #########
	        ####
	!!!
}
```

## Expressions and Values

Aside from comments, branes may contain expressions. Expressions are symbols that follow Foolish
rules and should be evaluable to a fixed value. Here are a few integer expressions inside a brane:

```foolish
{
	1;    !! This is the number 1
	2;    !! This is the number 2
	3.14; !! Floating point numbers work too
	"hello"; !! Strings are also expressions
}
```

A brane written in Foolish is itself an expression.

## Names and Scope

One very powerful concept we have for abstracting thoughts and thinking of complex matter with
complex properties and interactions is the substitution of the statement or object of consideration
with a name. Names such as `x`, `y`, `foolish`, `programming language`. This is an important
concept in Foolish—we are able to **identify** value expressions with names using the identification
operator `=`:

```foolish
{
	a = 1;                                    !! Identify 1 as 'a'
	point = {x=10; y=20;};                   !! Nested brane with ordinates
	x_coord = point.x;                        !! Accessing an ordinate: x_coord = 10
	calculation = a + point.x;                !! Using names in expressions
}
```

When these assignments are evaluated in a brane, each identified expression becomes **ordinated** to
that brane—the brane gains **ordinates** (named axes/dimensions). Names (also called *ordinates* or
*coordinates*) serve as navigational reference points within branes. Basic access uses `.` for
dereferencing: `brane.name` retrieves the value. Names are scoped to "before the current
expression"—references look backwards in the current brane, then search upward through parent branes
if needed.

For comprehensive documentation on names, the search system (including contextless
`.` `?` `~` `#` `^` `$`, value search `~=` `?=`, contexted `&`-searches `&?` `&~` `&#` `&^` `&$`
`&~=` `&?=`, and cursor movements), and how detachment branes `[...]` control scope boundaries, see
[Names, Searches, and Bounds](docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md) and
[FOOP-23](docs/foop/FOOP-23.md) for the authoritative operator specification.

Also checkout the syntactic sugar for accessing computing results: 'A =$ B' means 'A = B$', and
sequencer verses it. The '=$' call the reader/coder's attention to the fact that we're extracting
computation result and not recording an entire Foolish subtree.

## The Unknown

The ***NK*** (*Not Knowable*, pronounced "no-no") is of paramount importance to us, therefore we
dedicate a symbol to express the *NK* state in Foolish `???`. Here we declare we do not know the
answer:

```foolish
{
	answer=???;
}
```

In fact every unnamed expression is an assignment to an *NK* name:

```foolish
{1;2;3;}
```

is shorthand to

```foolish
{
	???=1;
	???=2;
	???=3;
}
```

### NK from a search

A **search** settling NK is the narrow, exceptional outcome — not the default. NK from a
search means "provably not there," and is reserved for specific cases:

- an **anchored** contextless search (`a?name`) that finds nothing settles **NK**
- an **unanchored** search (`?name`) that finds nothing settles **ECONSTANIC** instead — it
  may still gain a value later via recoordination
- a value search (`=`) whose pattern is itself `???`/NK, or whose anchor is NK, settles NK

Most searches are constanic, not NK — treat an unexplained `NK` on a search as something to
investigate, not the expected shape of a miss. See
[FOOP-23](docs/foop/FOOP-23.md#specification) for the authoritative, in-force specification
of search outcomes (the "NK vs ECONSTANIC miss outcomes" rule and the full operator family).

## Renaming

Foolish permits reusing names with static single assignment semantics. Each reference captures the
value at the time of use:

```foolish
{a=1; b=a; a=2; c=a;}  !! b=1, c=2 (each 'a' reference uses the current value)
```

For details on scope resolution and name reuse, see [Names, Searches, and Bounds](docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md).

### Named creations cannot be renamed

A `⬤`/`{*}` [creation](docs/why/creation_postulate.md) that is the entire right-hand side of a
**null-characterized** statement (`'name = ⬤`) is a **named creation**; the null-characterized
name is that creation's **original name** — a name that uniquely and durably identifies the one
creation it names, e.g. `'True`/`'False` in `system.foo`.

Giving an already-named creation a *second*, *different* null-characterized name is forbidden:

```foolish
{
	'a = ⬤;
	'a = 'a;   !! permitted -- re-states 'a's OWN existing name, not a rename
	'b = 'a;   !! FORBIDDEN -- 'a already has an original name; 'b='a would rename it
}
```

`'b = 'a` settles NF ("Named creations cannot be renamed") — the same not-foolish mechanism as
the null-characterized name constant rule. This applies transitively: reaching the same creation
through an intermediate plain (non-null-characterized) reference still counts as a rename attempt,
because a creation's identity — and its original name — survives being passed around. A creation
reached only through a plain name, or one that is merely an operand of a larger expression, has no
original name and may be given a null-characterized name for the first time freely.

---

## Documentation

See [Documentation Layout](#documentation-layout) above for an overview of ubc0, ubc0_1, and ubc1.

### Documentation Organization

- **`docs/why/`** - "Philosophy of Foolish" - origins, inspirations, and design philosophy
- **`docs/howto/`** - "How to Express it in Foolish" - literate programming tutorials as .foo files
- **`docs/ubc1/how/`** - Engineering documentation for ubc1 (message-passing infrastructure)
- **`docs/ubc1/todo/`** - Active project tracking for ubc1 development
- **`docs/ubc0_1/how/`** - Engineering documentation for ubc0_1 (ubc0 semantics with ubc1 microstates)
- **`docs/ubc0_1/todo/`** - Project tracking for ubc0_1 development
- **`docs/vintage_legacy/`** - Legacy ubc0 documentation (historical reference)

### Legacy References (ubc0)

The following documents in `docs/vintage_legacy/` document the original ubc0 implementation:

- **[Names, Searches, and Bounds](docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md)** - Comprehensive guide to naming
  systems, the search operators (`.`, `?`, `??`, `?*`, cursor movements), and how detachment
  branes `[...]` control scope boundaries for globalized searches
  *(Note: value search notation `?=`, `?=*`, `doc:4` in this vintage doc is superseded by
  FOOP-23's `~=`/`?=` family and contexted `&`-searches.)*
- **[Advanced Features](docs/vintage_legacy/ADVANCED_FEATURES.md)** - Brane operations (concatenation, proximity
  is combination), control flow, and recursion
- **[Ecosystem](docs/vintage_legacy/ECOSYSTEM.md)** - Implementation details including the original UBC, typing systems, and relational coordinates
- **[Symbol Table](docs/ubc1/how/SYMBOL_TABLE.md)** - Reference table of Foolish symbols and Unicode mappings

---

## Last Updated

**Date**: 2026-08-12
**Updated By**: Sisyphus / oqwen/qwen/qwen3.8-max
**Changes**: Added **§"Running specific tests"** — the CENTRAL reference for running one test
case or a subset of cases (what FOOP plan checkboxes link to): unit-test selection by name
filter (single filter, multi-filter batch with OR semantics, `--exact`, `--list`), and einmo
case selection via the einmo CLI (`evaluate --filter` with the verified
`foolish-cli run /dev/stdin | head -c -1` evaluator command — byte-identical to the gate's
output; `compare` with specific case files; `list --filter --differing`). Repaired the stale
test block above it: `run_einmo_tests` no longer exists (the three `einmo_gate_*` tests
replaced it) and `einmo evaluate --command "cat"` was broken (it echoed INPUT as OUTPUT; the
CLI also has no stdin mode — the `/dev/stdin` form is the working command).

This log keeps only the single newest entry — see `git log README.md` for full history.
