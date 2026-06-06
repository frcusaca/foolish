---
foop: d24
title: Humanizing FIR Sequencer formatting specification
author: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
credits: Harold Cooper (hcbusy) — indentation model, proto-brane abstraction, line_hint design
status: Draft
type: Specification
created: 2026-06-03
phase: phase-2
supersedes: []
---

# FOOP-42: Humanizing FIR Sequencer formatting specification

## Nomenclature

As part of this FOOP, the Humanizing Sequencer is renamed to better reflect its
domain:

| Old Name | New Name | Short Form |
|----------|----------|------------|
| Humanizing Sequencer | Humanizing FIR Sequencer | HFS |
| `HumanizingSequencerRef` | `HumanizingFirSequencerRef` | `hfs` |
| `Sequencer` | `FirSequencer` | — |

The short form **HFS** (or lowercase **hfs** for Rust identifiers) replaces **HS**
in documentation, log output, snapshot footers, and code comments.  Both UBC and
UBCb codebases are updated to use the new names.

The snapshot output header changes from `[0] RESULT:\n\`\`\`hssnap` to
`[0] RESULT:\n\`\`\`hfssnap`.

## Overview

The Humanizing FIR Sequencer (HFS) renders FIR (Foolish Internal Representation)
into human-readable text for snapshot approval tests. This specification defines the
output format for every FIR variant, the indentation model, state display rules,
and rendering conventions.

All FIR and their sub-FIR are recursively represented in the output.

### Why a Custom Formatter

The HFS is the debugging and testing lens into the Foolish VM. Its output is the
authoritative record of correct evaluation behavior in approval (snapshot) tests.
We cannot rely on third-party formatters — they change between versions, make
arbitrary layout choices, and cannot understand FIR semantics. The sequencer
must produce reliable, informative FIR information in a repeatable manner, byte
for byte identical across runs, implementations (UBC and UBCb), and machines.

Every whitespace decision is deliberate. Indentation conveys containment
structure. State tokens reveal evaluation progress. The format is designed to
be readable by humans comparing snapshot diffs and by tools verifying
cross-implementation parity.

## Indentation Model

Each formatter returns a list of `(prefix, text)` pairs.  It does **not**
materialize space characters or place delimiters.  The parent adds its own
`body_indent` to every child prefix.  Only the outermost level converts
prefixes to actual spaces.

### Parameters

| Term | Meaning |
|------|---------|
| `open_indent` | Hint: columns from parent body start to the child's opening delimiter |
| `close_indent` | Hint: columns from parent body start to the child's closing delimiter |
| `B_DENT` | Fixed body indent — **constant, value 2** |
| `body_indent` | Prefix for body lines: `min(open_indent − close_indent + B_DENT, 2 × B_DENT)` |

`open_indent` and `close_indent` are **hints**, not instructions.  The formatter
does not place the opening or closing delimiter — the parent handles that inline.
The hints tell the formatter where the delimiters sit so it can compute how much
space is available for body content.

### Rules

1. **Body lines** get prefix `body_indent`.
2. **Closer** gets prefix `0` — the parent positions it at the parent's
   `body_indent`, which is where the `result=` label or statement name starts.
3. The parent adds its own `body_indent` to every child pair.
4. The formula `min(open_indent − close_indent + B_DENT, 2 × B_DENT)` caps the
   body prefix at `2 × B_DENT = 4`.  Combined with the parent's own `body_indent`,
   the total indentation from the grandparent can reach ~`3 × B_DENT`.
5. For root branes and unnamed nested branes: `open_indent = 0`, `close_indent = 0`.
6. For named statements (`name = {`): `open_indent = len(name) + 1` (for `=`),
   `close_indent = 0`.

### Worked Example

Source: `{a=1; b={c={d=10; e=10;}}}`

**Rendered output with column positions:**

| 0    | 1   | 2   | 3   | 4   | 5   | 6   | 7   | 8   | 9   | 10+      | Annotation                     |
| ---  |---  |---  |---  |---  |---  |---  |---  |---  |---  |          | ------------------------------ |
|  `{` |     |     |     |     |     |     |     |     |     |          | root open on col 0             |
|      |     | `a` | `=` | `1` | `;` |     |     |     |     |          | stmt at col 2                  |
|      |     | `b` | `=` | `{` |     |     |     |     |     |          | stmt at col 2, `{` at col 4    |
|      |     |     |     |     |     | `c` | `=` | `{` |     |          | stmt at col 6, `{` at col 8    |
|      |     |     |     |     |     |     |     |     |     | `d=10;`  | stmt at col 10                 |
|      |     |     |     |     |     |     |     |     |     | `e=10;`  | stmt at col 10                 |
|      |     |     |     |     |     | `}` |     |     |     |          | c close at col 6               |
|      |     | `}` |     |     |     |     |     |     |     |          | b close at col 2               |
|  `}` |     |     |     |     |     |     |     |     |     |          | root close at col 0            |

**Indentation chain:**

```hfs
Root:   open=0, close=0  → body_indent = min(0−0+2, 4) = 2   → stmts at prefix 2

b={}:   open=2, close=0  → body_indent = min(2−0+2, 4) = 4   → stmts at prefix 2+4 = 6
        (open=2 because "b=" is 2 cols from stmt start)

c={}:   open=2, close=0  → body_indent = min(2−0+2, 4) = 4   → stmts at prefix 6+4 = 10
        (open=2 because "c=" is 2 cols from stmt start)
```

Each level's `body_indent` is a prefix that the parent adds to.  The closer gets
prefix `0`, so it appears at the parent's `body_indent` (the statement start).

### Summary

- `body_indent = min(open_indent − close_indent + B_DENT, 2 × B_DENT)` — prefix, capped at 4
- Closer prefix is always `0` — parent positions it
- Parent adds its `body_indent` to all child prefixes
- Formatter does NOT place delimiters — parent handles inline merging
- Named statements: `open_indent = len(name) + 1`, `close_indent = 0`
- Root / unnamed: `open_indent = 0`, `close_indent = 0`

## Proto-Brane Formatter

Branes, searches, operators, HeadTail, Index, StayFoolish, StayFullyFoolish, and
Concatenation all share the same structural pattern: an opening line, indented body
content, and a closing line. A single **proto-brane formatter** handles all of them.

The formatter does NOT place delimiters — the parent handles that inline.
The formatter receives `open_indent` and `close_indent` as **hints** about where
the delimiters sit, computes `body_indent`, and returns body lines with prefixes.

### Parameters

| Parameter | Meaning |
|-----------|---------|
| `pbid` | Proto-brane identifier (source trigger) |
| `opener` | Opening delimiter string (`"{"`, `"("`, `"<<"`) |
| `closer` | Closing delimiter string (`"}"`, `")"`, `">>"`) |
| `open_indent` | Hint: cols from parent body to child's opener |
| `close_indent` | Hint: cols from parent body to child's closer |

### Indent Computation

```hfs
body_indent = min(open_indent − close_indent + B_DENT, 2 × B_DENT)
```

- **Body lines**: prefix = `body_indent`
- **Closer line**: prefix = `0` (parent positions it)
- Parent adds its own `body_indent` to all child prefixes

### Configuration Table

| FIR Variant | `pbid` | `opener` | `closer` | `internal` |
|-------------|--------|----------|----------|------------|
| NormalBrane | `""` | `"{"` | `"}"` | 1 |
| Search (←) | `"?"` | `"("` | `")"` | 2 |
| Search (→) | `"/"` | `"("` | `")"` | 2 |
| HeadTail (head) | `"^"` | `"("` | `")"` | 2 |
| HeadTail (tail) | `"$"` | `"("` | `")"` | 2 |
| Index | `"#"` | `"("` | `")"` | 2 |
| Operator | `"Op" + op string` | `"("` | `")"` | len("Op" + op) + 1 |
| StayFoolish | `""` | `"<"` | `">"` | 1 |
| StayFullyFoolish | `""` | `"<<"` | `">>"` | 2 |
| Concatenation | `"⨃"` | `"("` | `")"` | 4 |

### Example: Search (backward, ECONSTANIC — failed)

```hfs
open_indent = 4, close_indent = 0  (inside "a = ?(...)")
body_indent = min(4 − 0 + 2, 4) = min(6, 4) = 4

Returns:
  (4, "pattern='^x$',")       ← trailing comma
  (4, "UNANCHORED,")          ← trailing comma
  (4, "UNANCHORED, ECONSTANIC")  ← last item, no comma... wait, state is separate

Actually, the items are comma-separated by the formatter:
  (4, "pattern='^x$',")
  (4, "UNANCHORED,")
  (4, "ECONSTANIC")           ← last item, no comma

Parent places opener "?(" and closer ")" inline.
```

### Example: Search (backward, WOCONSTANIC — multi-line result)

When the result is a brane, the result line starts at `body_indent` and the
brane's closer aligns at `close_indent` (= `body_indent` from the caller).

```hfs
{
  a = ?(result={
              aha=1;
              oohoo=2;
              yoohooooooooooooooooooooooooooooooo=20
          },
          pattern='^x$',
          UNANCHORED,
          WOCONSTANIC)
}
```

**How the indentation works:**

The search formatter generates non-result items first, then the result last.
The brane receives `open_indent = 7` (len("result=")) and `close_indent = 0`.

```hfs
search body_indent = min(4 − 0 + 2, 4) = 4   (open=4 from "a = " prefix)
brane body_indent  = min(7 − 0 + 2, 4) = 4   (open=7 from "result=" prefix)
```

The brane body prefix (4) is added by the search to its own body_indent.
The brane closer prefix (0) places `}` at the search's body_indent.

### Example: Search (backward, CONSTANT — found a value)

```hfs
Returns (single line):
  (body_indent, "result=42,")
  (body_indent, "pattern='^x$',")
  (body_indent, "UNANCHORED")
```

### Example: Operator (EMBRYONIC)

```hfs
Returns:
  (body_indent, "10,")
  (body_indent, "20,")
  (body_indent, "EMBRYONIC")
```

### Example: StayFoolish (WOCONSTANIC)

State on opener line, body is inner FIR:
```hfs
open_indent = 0, close_indent = 0
body_indent = min(0−0+2, 4) = 2

Parent places "<WOCONSTANIC" inline, then body at prefix 2, then ">" at prefix 0.
```

## FIR Variants

Every variant whose FIR body contains children uses the proto-brane formatter
(see above). Only variant-specific details — which args appear, result handling,
state placement, transparency rules — are listed here.

All body lines produced by the proto-brane formatter carry a trailing comma
**except the last**.  Commas are appended by the current formatter itself,
not by the parent.  This invariant means a parent can freely adjust prefix
spacing without breaking the separator syntax.

For branes, the separator is `;` instead of `,` — same rule: trailing `;` on
all body lines except the last.

### 1. ConstantInt (Integer Literal)

Rendered as the integer's decimal representation. No state suffix. For CONSTANT
and INDEPENDENT states only — non-terminal ints should not occur.

```hfs
10
```

### 2. NK (Not Known / ???)

```hfs
??? (reason)
```

With an alarm:
```hfs
??? (reason, ALARM_CODE: message)
```

NK always displays this way regardless of state. No proto-brane formatter needed.

### 3. Operator

Proto-brane: `pbid = "Op" + op_string` (e.g. `"Op+"`, `"Op-"`, `"Op*"`, `"Op/"`), `opener = "("`,
`closer = ")"`.

**CONSTANT or INDEPENDENT**: transparent — renders the computed value inline
(e.g. `60`). The FIR has been replaced by its result.

**EMBRYONIC** (operands ready, not yet stepped): operands comma-separated,
state at end:

```hfs
Op+(10, 20, EMBRYONIC)
```

**WOCONSTANIC** (some operands constanic): inline children, state at end:

```hfs
Op+(?(result=<x>, pattern='^x$', UNANCHORED, WOCONSTANIC), 20, WOCONSTANIC)
```

When expanded to multi-line:

```hfs
Op+(
  ?(result=<x>, pattern='^x$', UNANCHORED, WOCONSTANIC),
  20,
  WOCONSTANIC
)
```

### 4. Search

Proto-brane: `opener = "("`, `closer = ")"`.  The `pbid` is the **source trigger**:
`"?"` for backward search, `"/"` for forward search (matching Foolish source
`?(expr)` and `/(expr)`).  Direction is implicit — not an arg.

**ECONSTANIC** (not found): args then state at end:

```hfs
?(pattern='^x$', UNANCHORED, ECONSTANIC)
```

**CONSTANT** (found, resolved): result first, skip state:

```hfs
?(result=42, pattern='^x$', UNANCHORED)
```

**WOCONSTANIC** (found but target is constanic): result first, keep state:

```hfs
?(result=<target>, pattern='^x$', UNANCHORED, WOCONSTANIC)
```

Multi-line with nested result:

```hfs
?(
  result=?(
    pattern='^y$',
    UNANCHORED,
    ECONSTANIC
  ),
  pattern='^x$',
  UNANCHORED,
  WOCONSTANIC
)
```

### 5. HeadTail

Proto-brane: `opener = "("`, `closer = ")"`.  `pbid = "^"` for head, `pbid = "$"`
for tail (matching Foolish source `^` and `$`).  Always ANCHORED — not listed.

**NK** (empty brane): state at end:

```hfs
^(NK)
```
no result needed here. But if `^` found a search that resulted in NK, it would be
```hfs
^(result=Search(..., NK), NK)
```

**ECONSTANIC** (not found): state at end:

```hfs
^(ECONSTANIC)
```

**CONSTANT** (found, resolved): transparent — renders the extracted value.

**WOCONSTANIC** (found, target constanic): result first, keep state:

```hfs
$(result=<target>, WOCONSTANIC)
```

### 6. Index (Seek)

Proto-brane: `opener = "("`, `closer = ")"`.  `pbid = "#"` (matching Foolish
source `#N`).

**NK** (out of bounds): `offset`, anchor kind, state:

```hfs
#(offset=99, ANCHORED, NK)
```

**CONSTANT** (found): result first, inline args:

```hfs
#(result=30, offset=0, ANCHORED)
```

**ECONSTANIC** (not found):

```hfs
#(offset=3, UNANCHORED, ECONSTANIC)
```

### 7. StayFoolish (SF)

Proto-brane: `pbid = ""`, `opener = "<"`, `closer = ">"`.

Like a brane with angle-bracket delimiters.  State (if non-constant) follows `<`
with no space; body is the inner FIR.  CONSTANT/INDEPENDENT SF renders the inner
value transparently.

```hfs
<WOCONSTANIC
  ?(pattern='^sf_target$', UNANCHORED, ECONSTANIC)
>
```
```hfs
<
  { ... }
>
```

### 8. StayFullyFoolish (SFF)

Proto-brane: `pbid = ""`, `opener = "<<"`, `closer = ">>"`.

2-character delimiters, otherwise identical to StayFoolish.

```hfs
<<EMBRYONIC
  ?(pattern='^sff_target$', UNANCHORED)
>>
```
```hfs
<<
  { ... }
>>
```

### 9. Concatenation

Proto-brane: `pbid = "⨃"` (U+2A03, n-ary union), `opener = "("`, `closer = ")"`.

Elements comma-separated, state at end if non-constant:

```hfs
⨃(elements=3, EMBRYONIC)
```

Multi-line:

```hfs
⨃(
  {a=1; b=2},
  {c=3; d=4},
  {e=5; f=6},
  EMBRYONIC
)
```

CONSTANT/INDEPENDENT: renders as a brane with `⨃` prefix:

```hfs
⨃{a=1; b=2; c=3; d=4; e=5; f=6}
```

### 10. NormalBrane

Proto-brane: `pbid = ""`, `opener = "{"`, `closer = "}"`.

Body: statement lines separated by `;` (trailing on all non-last lines, appended
by the brane formatter itself). If the brane has characterizations, they appear
before `{`: `name'{`.

State token (if non-constant) appears immediately after `{` with no space:

```hfs
{WOCONSTANIC
  stmt1;
  stmt2
}
```

Empty branes render on one line:
```hfs
{}
```

Named statements in the enclosing brane body pass `opener_indent = len(name) + 1`
to the child brane's proto-brane formatter, with the `{` merged inline:

```hfs
{
  outer = {
           inner = {
                   deep = 10;
                   deeper = 20
           }
  }
}
```

Note: `;` on all but last stmt line.  `deep = 10;` gets it, `deeper = 20` does not.

## Rendering Algorithm

The HFS dispatches on FIR variant, returning `[(prefix, text), ...]` pairs.
Atomic types (ConstantInt, NK) produce a single pair.  All non-atomic types
delegate to `proto_brane_formatter`.

### Core Dispatch

```
function render(node, open_indent, close_indent)
  state := node.state()
  show_state := state ∉ {CONSTANT, INDEPENDENT}

  // ── Atomic ──
  if value := node.constant_int()
    return [(open_indent, format("{value}"))]

  if (reason, alarm) := node.nk()
    let msg := "??? ({reason}"
    if alarm: msg += ", {alarm.code}: {alarm.message}"
    msg += ")"
    return [(open_indent, msg)]

  // ── Operator (transparent when constant) ──
  if (op_name, operands) := node.operator()
    if state ∈ {CONSTANT, INDEPENDENT}
      return [(open_indent, format("{reduced_value}"))]
    let body_items := []
    for op in operands:
      body_items += [render_to_string(op)]
    if show_state:
      body_items += [format("{state}")]
    return proto_brane_formatter(
      pbid:          op_name,
      opener:        "(",
      closer:        ")",
      open_indent:   open_indent,
      close_indent:  close_indent,
      items:         body_items,
    )

  // ── Search ──
  if (pattern, direction, anchored, _, target) := node.search()
    let trigger := direction == BACKWARD ? "?" : "/"
    let anchor_str := anchored ? "ANCHORED" : "UNANCHORED"
    // Deferred result: generate non-result items first, result last
    let non_result_items := ["pattern='{pattern}'", anchor_str]
    if state ∈ {WOCONSTANIC, ECONSTANIC}:
      non_result_items += [format("{state}")]
    else if state ∉ {CONSTANT, INDEPENDENT}:
      non_result_items += [format("{state}")]
    return proto_brane_formatter_with_result(
      pbid:          trigger,
      opener:        "(",
      closer:        ")",
      open_indent:   open_indent,
      close_indent:  close_indent,
      non_result:    non_result_items,
      result:        target,             // rendered last; its close_indent = parent body_indent
    )

  // ── HeadTail ──
  if (is_head, _, anchor) := node.head_tail()
    let trigger := is_head ? "^" : "$"
    let non_result_items := []
    if state ∈ {WOCONSTANIC, ECONSTANIC}:
      non_result_items += [format("{state}")]
    else if state ∉ {CONSTANT, INDEPENDENT}:
      non_result_items += [format("{state}")]
    return proto_brane_formatter_with_result(
      pbid:          trigger,
      opener:        "(",
      closer:        ")",
      open_indent:   open_indent,
      close_indent:  close_indent,
      non_result:    non_result_items,
      result:        anchor,
    )

  // ── Index ──
  if (offset, anchored, anchor) := node.index()
    let ak := anchored ? "ANCHORED" : "UNANCHORED"
    let non_result_items := ["offset={offset}", ak]
    if state ∈ {WOCONSTANIC, ECONSTANIC}:
      non_result_items += [format("{state}")]
    else if state ∉ {CONSTANT, INDEPENDENT}:
      non_result_items += [format("{state}")]
    return proto_brane_formatter_with_result(
      pbid:          "#",
      opener:        "(",
      closer:        ")",
      open_indent:   open_indent,
      close_indent:  close_indent,
      non_result:    non_result_items,
      result:        anchor,
    )

  // ── StayFoolish ──
  if inner := node.stay_foolish()
    return proto_brane_formatter(
      pbid:          "",
      opener:        "<",
      closer:        ">",
      open_indent:   open_indent,
      close_indent:  close_indent,
      opener_state:  show_state ? format("{state}") : None,
      body:          render(inner, ...),
    )

  // ── StayFullyFoolish ──
  if inner := node.stay_fully_foolish()
    return proto_brane_formatter(
      pbid:          "",
      opener:        "<<",
      closer:        ">>",
      open_indent:   open_indent,
      close_indent:  close_indent,
      opener_state:  show_state ? format("{state}") : None,
      body:          render(inner, ...),
    )

  // ── Concatenation ──
  if (elements, merged) := node.concatenation()
    let body_items := ["elements={len(elements)}"]
    for elem in elements:
      body_items += [render_to_string(elem)]
    if merged is Some(m):
      body_items += ["merged=" + render_to_string(m)]
    if show_state:
      body_items += [format("{state}")]
    return proto_brane_formatter(
      pbid:          "⨃",
      opener:        "(",
      closer:        ")",
      open_indent:   open_indent,
      close_indent:  close_indent,
      items:         body_items,
    )

  // ── NormalBrane ──
  if (charact, statements) := node.brane()
    return proto_brane_formatter(
      pbid:          charact,             // "" or "name'"
      opener:        "{",
      closer:        "}",
      open_indent:   open_indent,         // 0 for root/unnamed
      close_indent:  close_indent,
      opener_state:  show_state ? format("{state}") : None,
      body:          render_statements(statements, ...),
    )

  return [(open_indent, "Unknown({node.variant()})")]
```

### Proto-Brane Formatter

Two variants: `items`-based (comma-separated args list) and `body` + `opener_state`-based
(branes, SF, SFF where body is multi-line child content).  A third variant
`proto_brane_formatter_with_result` handles Search/HeadTail/Index with deferred
result generation.

```
function proto_brane_formatter(pbid, opener, closer, open_indent, close_indent,
                               items, opener_state, body)

  // body_indent is a PREFIX for body lines, relative to the formatter's origin.
  // Parent adds its own body_indent to all returned prefixes.
  let body_indent := min(open_indent − close_indent + B_DENT, 2 × B_DENT)

  let lines := []

  if items is not None:
    // Comma-separated items.  Single-line if fits; multi-line with trailing
    // commas on all but last.  Comma appended by this formatter, not parent.
    let last := items.len() - 1
    if can_single_line(items, line_hint):
      let joined := items.join(", ")
      lines += [(body_indent, joined)]
    else:
      for (i, item) in items:
        let suffix := if i < last then "," else ""
        lines += [(body_indent, item + suffix)]

  if body is not None:
    // Multi-line body for branes/SF/SFF
    for line in body:
      lines += [(body_indent + line.prefix, line.text)]

    // `;` on all non-last stmt lines (branes); appended by this formatter

  // Closer at prefix 0 — parent positions it at parent's body_indent
  lines += [(0, closer)]

  return lines
```

**Deferred result variant** for Search/HeadTail/Index:

```
function proto_brane_formatter_with_result(pbid, opener, closer,
                                            open_indent, close_indent,
                                            non_result, result)

  let body_indent := min(open_indent − close_indent + B_DENT, 2 × B_DENT)

  let lines := []

  // Pass 1: generate non-result items (pattern, anchor, state)
  let non_result_last := non_result.len() - 1
  for (i, item) in non_result:
    let suffix := if i < non_result_last then "," else ""
    lines += [(body_indent, item + suffix)]

  // Pass 2: generate result last — now we know the line budget
  if result is Some(t):
    let result_label := "result="
    // Result child: open_indent = len("result=") + len(pbid + opener)
    //               close_indent = 0 (closer aligns with this formatter's body_indent)
    let inner_open  := len(result_label) + len(pbid + opener)
    let inner_close := 0
    let result_lines := render(t, inner_open, inner_close)
    // Prepend "result=" label to first result line
    result_lines[0].text = result_label + result_lines[0].text
    // Add comma after result if there were non-result items
    if non_result.len() > 0:
      result_lines[last].text += ","
    lines += result_lines

  // Closer at prefix 0
  lines += [(0, closer)]

  return lines
```

### Statement Rendering in Branes

```
function render_statements(statements, body_indent)
  let lines := []
  let last := statements.len() - 1
  for (i, stmt) in statements:
    if stmt.name is Some(name):
      lines += [(body_indent, name + "=")]    // { merges inline from child
      let child := render(stmt.body, body_indent + len(name) + 1)
      merge_first_child_line_inline(lines, child, "=")
    else:
      lines += render(stmt.body, body_indent)
  // Trailing ; on all non-last; appended by THIS formatter
  for i in 0..last:
    lines[i].text += ";"
  return lines
```

## State Display Summary

| State | Display? | Format |
|-------|----------|--------|
| CONSTANT | No | Omitted |
| INDEPENDENT | No | Omitted |
| ECONSTANIC | Yes | Token immediately after opening delimiter or on same line |
| WOCONSTANIC | Yes | Same |
| PREMBRIONIC | Yes | Same |
| EMBRYONIC | Yes | Same |
| BRANING | Yes | Same |
| NK | Yes | `??? (reason)` format — applies to both standalone NK FIRs and NK state on branes |

State tokens are displayed WITHOUT brackets. The `[STATE]` bracket syntax is
replaced by bare tokens (e.g., `ECONSTANIC`, not `[ECONSTANIC]`).

## Complete Examples

### Example 1: Simple brane with operations

Input:
```foolish
{x=10; y=20; z=30; sum = x + y + z; avg = sum / 3;}
```

Output:
```
{
  x=10;
  y=20;
  z=30;
  sum=60;
  avg=20
}
```

### Example 2: Division by zero in nested brane

Input:
```foolish
{
  l1={
    l2={
      l3={
        bad=1/0;
        good=42
      }
    }
  }
}
```

Output:
```
??? (division by zero){
  l1=??? (division by zero){
      l2=??? (division by zero){
           l3=??? (division by zero){
                bad=??? (division by zero);
                good=42
           }
      }
  }
}
```

### Example 3: Forward reference (ECONSTANIC)

Input:
```foolish
{fwd=x; x=42;}
```

Output:
```
WOCONSTANIC{
  fwd=?(pattern='^x$', UNANCHORED, ECONSTANIC);
  x=42
}
```

### Example 4: Search with result, HeadTail, Index

```
{
  found=?(result=42, pattern='^x$', UNANCHORED);
  not_found=?(pattern='^γ$', UNANCHORED, NK);
  empty_head=^(NK);
  head_with_result=^(result=10);
  seek_oob=#(offset=99, ANCHORED, NK);
  seek_found=#(result=30, offset=0, ANCHORED);
  concat=⨃(elements=2, WOCONSTANIC)
}
```

### Example 5: StayFoolish and StayFullyFoolish

```
{
  sf=<WOCONSTANIC
      ?(pattern='^target$', UNANCHORED, ECONSTANIC)
  >;
  sff=<<EMBRYONIC
        ?(pattern='^target$', UNANCHORED)
  >>;
  sf_resolved=<{a=1; b=2}>
}
```

### Example 6: Multi-line operator with constanic operands

```
sum=Op+(
  ?(result=<x>, pattern='^x$', UNANCHORED, WOCONSTANIC),
  ?(result=<y>, pattern='^y$', UNANCHORED, WOCONSTANIC),
  WOCONSTANIC
)
```

## Implementation Notes

1. **The HFS uses character-based alignment.** Predecessor code used depth-count
   (`"  ".repeat(depth)`). FOOP-42 replaces this with `open_indent`/`close_indent`
   hints: `body_indent = min(open_indent − close_indent + B_DENT, 2 × B_DENT)`.

2. **Operators are transparent when CONSTANT/INDEPENDENT.** A `+` that reduced
   to `60` renders as `60`.  Only non-terminal operators show the proto-brane
   wrapper with comma-separated operands and trailing state.

3. **State placement**: inside proto-brane parentheses, state is a trailing
   comma-separated item (ALL CAPS).  For branes/SF/SFF, state follows the
   opener inline (e.g. `??? (division by zero){`, `<WOCONSTANIC`).

4. **Commas on body lines**: appended by the formatter itself on all body
   lines except the last.  `;` for branes.

5. **Two indent parameters**: `open_indent` (where opener goes) and
   `close_indent` (where closer goes).  Usually identical; differ for
   `result=` children where closer aligns at parent's body_indent.

6. **Deferred result generation**: Search/HeadTail/Index generate non-result
   items first, then result last.  The result child receives `close_indent`
   = parent body_indent, so its `)` aligns with the `result=` label.

7. **Both UBC and UBCb** produce identical output — they share the same
   `FirSequencer::format()` code.

## Acceptance Test

The canonical acceptance test is `foop42_humanizing_sequencer_formatting_exhaustive.foo`
under `foolish-core/snapshot_tests/input/`. This single large test file exercises:

- All 10 FIR variants (ConstantInt, NK, Operator, Search, HeadTail, Index,
  StayFoolish, StayFullyFoolish, Concatenation, NormalBrane)
- All visible states (ECONSTANIC, WOCONSTANIC, NK, PREMBRIONIC, EMBRYONIC, BRANING)
- Flat branes (1 level deep) with short and long variable names
- Nested branes 2, 3, 4, and 5 levels deep
- Unicode identifiers (π, Δ, Σ)
- All syntactic forms: arithmetic, division by zero, forward references,
  anchored/unanchored searches, HeadTail, Index/seeks, StayFoolish,
  StayFullyFoolish, concatenation, named branes, unary operators, empty branes

**Iteration rule**: HFS implementation continues until this test file produces
output that exactly matches the FOOP-42 format specification. Run `cargo test -p
foolish-core --lib` after each change, inspect the `.snap.new`, fix discrepancies,
repeat.

## Verification

After implementation:
1. `cargo test -p foolish-core --lib` must pass (all unit tests + foop42 snapshot)
2. Run `cargo test -p foolish-ubcb --lib` to generate UBCb `.snap.new` files
3. Present ALL `.snap.new` files to human for review
4. AFTER human approval: accept snapshots

## References

- Current HFS implementation: `foolish-core/src/sequencer.rs`
- FIR type definitions: `foolish-core/src/fir.rs`
- UBC evaluator: `foolish-core/src/ubc.rs`
- UBCb evaluator: `foolish-ubcb/src/ubcb.rs`
- Snapshot test infrastructure: `foolish-core/src/snapshot_suite.rs`
- Prior HFS spec: `UBC_humanizing_sequence_round_1.spec.md`

## Last Updated

**Date**: 2026-06-04
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Rewrote Indentation Model with hints-based approach: `open_indent` and
`close_indent` are hints to the formatter about delimiter positions, not instructions.
Formula: `body_indent = min(open_indent − close_indent + B_DENT, 2 × B_DENT)` as a
prefix. Closer prefix always 0. Formatter does NOT place delimiters — parent handles
inline merging. Updated proto_brane_formatter pseudo-code to match. Cleaned up examples
section — removed confused analysis, added three clean demonstrations matching user's
idealized output. Deferred result variant updated with correct inner_open/inner_close
computation.
