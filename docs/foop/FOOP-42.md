---
foop: 24
title: Humanizing Sequencer formatting specification
author: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
status: Draft
type: Specification
created: 2026-06-03
phase: phase-2
supersedes: []
---

# FOOP-42: Humanizing Sequencer formatting specification

## Overview

The Humanizing Sequencer (HS) renders FIR (Foolish Internal Representation) into
human-readable text for snapshot approval tests. This specification defines the
output format for every FIR variant, the indentation model, state display rules,
and rendering conventions.

All FIR and their sub-FIR are recursively represented in the output.

## Indentation Model

Two orthogonal indentations compose the visual layout:

- **B_DENTS** — body indentationis a configurable, default 2, spaces applied at each structural
  nesting level (entering a `{`, a `Search(...) FIR`, etc.).
- **A_DENTS** — alignment indentation, computed dynamically as the count of
  characters from the start of the current indentation prefix to (and including)
  the opening delimiter (`{` or `(`). This ensures child content aligns under the
  opening delimiter.

When a node opens a new nesting scope, the child block's indent is:

```
child_indent = current_indent + A_DENTS + B_DENTS
```

where `A_DENTS` is the horizontal position of the opening delimiter relative to
the current indent. The closing delimiter aligns with the start of the opener's
line (i.e. at `current_indent`).

### Configuration

| Parameter | Value | Description |
|-----------|-------|-------------|
| `B_DENTS` | 2 | Fixed body indentation per nesting level |

`A_DENTS` is always computed dynamically and is never zero for a multi-line child.

### Example (annotated)

```
{
  a=1;
  b=2;
}
```

Annotated:
```
{
BB     ← root brane, B_DENTS=2 on top of A_DENTS=0
  a=1;
  b=2;
}           ← closing at root indent
```

Nested branes add A_DENTS:

```
{
BB        ← root B_DENTS=2 (A_DENTS=0 for root)
  a=1;
  b={
BBAAA   ← A_DENTS: count chars including `{` = 2+3=5
     BB    ← b's B_DENTS=2
       c={
BBAAABBAAA ← c's A_DENTS: count chars including `{` = 2+3+2+3=10
          BB  ← c's B_DENTS=2
            d=10;
            e=10;
       }
  }
}
```

The character-counting rule: when a statement on line L opens a brane, measure
how many characters on line L appear before the `{` (starting from the current
indent), inclusive. That is `A_DENTS` for the child brane's body. A rendered
version without annotations:

```
{
  a=1;
  b={
      c={
        d=10;
        e=10;
      }
  }
}
```

## FIR Variants

### 1. ConstantInt (Integer Literal)

Rendered as the integer's decimal representation. No state suffix.

```
10
```

Only for CONSTANT and INDEPENDENT states. If a ConstantInt has a non-terminal
state, see the Non-Terminal FIR rule.

### 2. NK (Not Known / ???)

Rendered as:

```
??? (reason)
```

If an alarm is present, append it:
```
??? (reason, ALARM_CODE: message)
```

NK always displays this way regardless of state.

### 3. Operator

Non-constant and non-independent operators render as:

```
name(args) STATE_TOKEN
```

Where STATE_TOKEN is one of `ECONSTANIC`, `WOCONSTANIC`, `PREMBRIONIC`,
`EMBRYONIC`, `BRANING`. For CONSTANT and INDEPENDENT operators, the value is
rendered (see below).

Operators that have reduced to a constant/independent value render their result
inline (as the value, not a labeled struct):

```
60
```

(no `Operator(+) → 60` wrapping — the FIR is transparent).

### 4. Search

Search renders one of two ways:

**Without a result** (still searching, or ended with NK/ECONSTANIC):
```
Search(pattern='^x$', dir=BACKWARD, UNANCHORED) STATE_TOKEN
```

**With a constant result:**
```
Search(pattern='^x$', dir=BACKWARD, UNANCHORED,
       result=RESULT
)
```
The `result=` line starts at `B_DENTS` from the opening line's indent. RESULT
is recursively rendered at its own indent.

**With a non-constant result (e.g. another Search):**
```
Search(pattern='^x$', dir=BACKWARD, UNANCHORED) STATE_TOKEN
       result=
         Search(pattern='^y$', ...)
```

When the result is another Search, it starts on its own indented line.

If result is raw value:
```
Search(pattern='^x$', dir=BACKWARD, UNANCHORED,
       result=10
)
```

Parameters: `pattern`, `dir` (BACKWARD/FORWARD), `UNANCHORED`/`ANCHORED`.

For ANCHORED searches, anchor is implicit (not displayed separately) — the
anchor resolved what the search targets on.

### 5. HeadTail

HeadTail format:
```
HeadTail(HEAD, WOCONSTANIC)
HeadTail(TAIL, ECONSTANIC,
         result=RESULT
)
```

HeadTail can only be ANCHORED (per language semantics). The `HEAD`/`TAIL` token
appears as the first argument. The state appears as the second positional token
(unbracketed), or omitted if CONSTANT/INDEPENDENT.

### 6. Index (Seek)

```
Index(offset=3, UNANCHORED) ECONSTANIC
Index(offset=-1, ANCHORED,
      result=RESULT
)
```

### 7. StayFoolish (SF)

```
StayFoolish(WOCONSTANIC)
StayFoolish(
  BODY
)
```

### 8. StayFullyFoolish (SFF)

```
StayFullyFoolish(WOCONSTANIC)
StayFullyFoolish(
  BODY
)
```

### 9. Concatenation

```
Concatenation(elements=3) WOCONSTANIC
Concatenation(
  ELEM1,
  ELEM2,
  ELEM3
)
```

When merged, the merged result follows the elements as an additional indented
block.

### 10. NormalBrane

Brane format:
```
{
  stmt1;
  stmt2;
  stmt3
}
```

Statements are separated by `;` appended to each line (except the last).
Statements with names are rendered as `name = body`. The body is inline if it
is an atomic value or an NK; multi-line otherwise.

If the brane is in a non-CONSTANT/non-INDEPENDENT state:
```
{WOCONSTANIC
  ...
}
```

The state token appears immediately after the opening `{`, no space.

Nested branes:
```
{
  outer = {
           inner = {
                   deep = 10;
                   deeper = 20;
           }
  }
}
```

Note the A_DENTS alignment: `outer = ` is 8 chars, `{` is the 9th, so the body
of outer's brane starts 9 chars in from outer's statement indent (A_DENTS=9),
then adds B_DENTS=2 for 11 total. `;` is appended to each statement line except
the last.

## Rendering Algorithm (Pseudo-Code)

The HS traverses the FIR tree recursively. Each node knows its current indent
(`indent` — a non-negative integer counting total spaces from column 0), and
whether it is being rendered inline (after `name = `) or at its own line.

```
function render(node: FirQueryable, indent: int, inline: bool, prefix: str) -> string

  state := node.state()
  show_state := state ∉ {CONSTANT, INDEPENDENT}

  // ── Literal ──
  if value := node.constant_int()
    if show_state:
      // Non-terminal int should not happen; fallback
      return format("{STATE_TOKEN} {value}")
    return format("{value}")

  // ── NK ──
  if (reason, alarm) := node.nk()
    let msg := "??? ({reason}"
    if alarm: msg += ", {alarm.code}: {alarm.message}"
    msg += ")"
    if not inline: msg = spaces(indent) + msg
    return msg

  // ── Operator ──
  if (op_name, operands) := node.operator()
    if state ∈ {CONSTANT, INDEPENDENT}:
      // Transparent — render the computed value (not the operator label)
      // In practice this means the operator has been reduced and we should
      // show the result. For now, the HS receives the reduced FIR, so the
      // operator will have been replaced by a ConstantInt in the brane body.
      // If still an Operator that happens to be CONSTANT, render as value:
      return render_reduced_value(node, indent, inline)
    else:
      return render_non_atomic("Operator", [op_name], [], indent, inline, state,
                               None, operands)

  // ── Search ──
  if (pattern, direction, anchored, anchor, target) := node.search()
    let params := "{pattern}, {direction}, {ANCHORED|UNANCHORED}"
    let header := format("Search({params})")
    if show_state: header += " " + state
    return render_with_optional_result("Search", params, indent, inline,
                                       state, show_state, target,
                                       {has_result: target.is_some()})

  // ── HeadTail ──
  if (is_head, anchored, anchor) := node.head_tail()
    let ht := is_head ? "HEAD" : "TAIL"
    let params := "{ht}"
    if show_state: params += ", " + state
    else: params += ", "  // placeholder
    return render_with_optional_result("HeadTail", params, indent, inline,
                                       state, show_state, anchor,
                                       {has_result: anchor.is_some()})

  // ── Index ──
  if (offset, anchored, anchor) := node.index()
    let anchor_kind := anchored ? "ANCHORED" : "UNANCHORED"
    let params := "offset={offset}, {anchor_kind}"
    if show_state: params += ", " + state
    return render_with_optional_result("Index", params, indent, inline,
                                       state, show_state, anchor,
                                       {has_result: anchor.is_some()})

  // ── StayFoolish / StayFullyFoolish ──
  if expr := node.stay_foolish()
    let name := "StayFoolish"
    return render_with_required_body(name, indent, inline, state, show_state, expr)

  if expr := node.stay_fully_foolish()
    let name := "StayFullyFoolish"
    return render_with_required_body(name, indent, inline, state, show_state, expr)

  // ── Concatenation ──
  if (elements, merged) := node.concatenation()
    let name := "Concatenation"
    let label := format("elements={len(elements)}")
    return render_non_atomic(name, [label], [], indent, inline, state,
                             merged, elements)

  // ── NormalBrane ──
  if (characterizations, statements) := node.brane()
    return render_brane(characterizations, statements, indent, inline, state, show_state)

  return "Unknown({node.variant()})"


function render_brane(chars, stmts, indent, inline, state, show_state) -> string
  let buf := ""
  if not inline: buf += spaces(indent)

  buf += "{"
  if show_state: buf += state

  if stmts.is_empty():
    buf += "}"
    return buf

  buf += "\n"

  let a_dents := compute_a_dents(indent, inline, starting_col)
  // a_dents counts chars from current indent start to and including the '{'
  // For a top-level brane, a_dents = 1 (the '{' character itself)
  let body_indent := indent + a_dents + B_DENTS
  // B_DENTS = 2

  let last := stmts.len() - 1
  for (i, stmt) in stmts:
    if stmt.name is Some(name):
      buf += spaces(body_indent) + name + " = "
      // The name assignment line establishes the alignment
      // The body A_DENTS is computed from the '=' to the '{'
      let name_line_cols := len(name) + 3  // " = "
      let a_dents_body := name_line_cols + 1  // include the '{'
      buf += render(stmt.body, body_indent + a_dents_body + B_DENTS, true, name)
      // Wait, actually for name = body, the body renders inline or multi-line
      // from the position after "name = "
      let body_prefix := spaces(body_indent) + name + " = "
      let rendered := render_to_string(stmt.body, body_indent, true)
      // For multi-line bodies, subsequent lines get body_indent + A_DENTS + B_DENTS
      if rendered is single_line:
        buf += rendered
      else:
        buf += rendered  // already has proper indentation
    else:
      let rendered := render(stmt.body, body_indent, false, "")
      buf += rendered

    buf += "\n"
    // Append ';' to all lines except the last
    if i < last:
      // Replace trailing newline with ';\n'
      buf := buf.strip_suffix('\n') + ";\n"

  buf += spaces(indent) + "}"
  return buf


function render_non_atomic(name, inline_args, body_args, indent, inline,
                            state, show_state, result, children) -> string
  // Construct the opening line
  let open_line := name + "(" + inline_args.join(", ")
  if show_state: open_line += ", " + state
  open_line += ")"

  let buf := spaces(indent) + open_line

  let has_body := children.is_not_empty() || result.is_some()
  if not has_body: return buf

  buf += "\n"

  let a_dents := len(name)  // chars from indent to '(' — but simpler:
  // a_dents = number of chars from indent start to and including '('
  // For name(args) the '(' is at position len(name)
  let a_dents := len(name) + 1  // include '('
  let body_indent := indent + a_dents + B_DENTS

  // Render children indented to body_indent
  for child in children:
    buf += render(child, body_indent, false, "") + "\n"

  if result is Some(res):
    buf += spaces(body_indent) + "result=" + render(res, body_indent, true, "result=")
    buf += "\n"

  // Closing ')'
  buf += spaces(indent) + ")"
  return buf


function render_with_optional_result(name, params, indent, inline,
                                      state, show_state, target,
                                      has_result) -> string
  let open_line := name + "(" + params + ")"

  if has_result and state == CONSTANT:
    // Result was found, display multi-line
    let buf := spaces(indent) + open_line + ",\n"
    let a_dents := len(name) + 1  // '('
    let res_indent := indent + a_dents + B_DENTS
    buf += spaces(res_indent) + "result="
    buf += render(target, res_indent + 7, true, "result=")  // 7 = len("result=")
    buf += "\n" + spaces(indent) + ")"
    return buf
  else:
    // No result or result still evaluating
    let buf := spaces(indent) + open_line
    if show_state: buf += " " + state
    if has_result and show_state:
      // Result exists but non-constant, show on next line
      buf += "\n"
      let a_dents := len(name) + 1
      let res_indent := indent + a_dents + B_DENTS
      buf += spaces(res_indent) + "result=\n"
      buf += render(target, res_indent + B_DENTS, false, "")
    return buf


function render_with_required_body(name, indent, inline, state, show_state, expr) -> string
  let open_line := name
  if show_state: open_line += "(" + state + ")"
  else: open_line += "("

  let buf := spaces(indent) + open_line + "\n"

  let a_dents := len(name) + 1  // '('
  let body_indent := indent + a_dents + B_DENTS

  buf += render(expr, body_indent, false, "")
  buf += "\n" + spaces(indent) + ")"
  return buf
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
| NK | Yes | `??? (reason)` format |

State tokens are displayed WITHOUT brackets. The `[STATE]` bracket syntax is
replaced by bare tokens (e.g., `ECONSTANIC`, not `[ECONSTANIC]`).

## Complete Examples

### Example 1: Simple branch with operations

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
NK{
  l1=NK{
      l2=NK{
           l3=NK{
                bad=??? (division by zero);
                good=42
           }
      }
  }
}
```

### Example 3: Searches

```
{
  r=Search(pattern='^x$', dir=BACKWARD, UNANCHORED) ECONSTANIC;
  r=Search(pattern='^x$', dir=BACKWARD, UNANCHORED) WOCONSTANIC,
     result=Search(pattern='^y$', ...);
  r=Search(pattern='^x$', dir=BACKWARD, UNANCHORED,
            result=1
  );
  r=Search(pattern='^x$', dir=BACKWARD, ANCHORED) ECONSTANIC;
  r=Search(pattern='^x$', dir=BACKWARD, ANCHORED) WOCONSTANIC,
     result=1;
  r=Search(pattern='^x$', dir=BACKWARD, ANCHORED) WOCONSTANIC,
     result=Search(pattern='^x$', ...);
  r=HeadTail(HEAD, WOCONSTANIC);
  r=HeadTail(HEAD,,
     result=1;
  r=HeadTail(TAIL, ECONSTANIC,
             result=Search(pattern='^x$', ...)
  )
}
```

### Example 4: NK with alarm

```
??? (division by zero)
??? (unknown identifier)
```

## Implementation Notes

1. **The HS should use character-based alignment**, not depth-based indentation.
   The current `format_fir_q` uses `"  ".repeat(depth)` which gives uniform 2-space
   nesting but does not align child content under the opening delimiter. The new
   format uses A_DENTS (dynamic, computed from opening delimiter position) and
   B_DENTS (fixed 2-space body indent).

2. **Operators are transparent when constant.** A `+` operator that has reduced to
   `60` should render as `60`, not `Operator(+) → 60`. The current implementation
   renders operators as `Operator(+, [EMBRYONIC])` with child indentation. The new
   format only renders non-constant/non-independent operators as labeled structs.

3. **Brane state is inlined after `{`**. Current format: `Brane [NK]{`. New format:
   `{` for constant branes, `NK{` for NK branes, `WOCONSTANIC{` for WOConstanic
   branes, etc.

4. **No `Brane` keyword.** The current `Brane{...}` prefix is removed. A bare `{`
   is the brane delimiter.

5. **Statements use `;` separators.** All statements except the last in a brane are
   terminated with `;`.

6. **Both UBC and UBCb must produce identical output.** The HS is shared code in
   `foolish-core/src/sequencer.rs`. Both evaluators use the same `Sequencer::format`
   function. Any format change affects both snapshot test suites simultaneously.

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

**Iteration rule**: HS implementation continues until this test file produces
output that exactly matches the FOOP-42 format specification. Run `cargo test -p
foolish-core --lib` after each change, inspect the `.snap.new`, fix discrepancies,
repeat.

## Verification

After implementation:
1. `cargo test -p foolish-core --lib` must pass (all unit tests + foop42 snapshot)
2. Run `cargo test -p foolish-ubcb --lib` to generate UBCb `.snap.new` files
3. Present ALL `.snap.new` files to human for review
4. AFTER human approval: accept snapshots

## Open Questions

1. Should the `Search` function-call parentheses close on the same line as the
   last argument, or on their own line? Current draft uses `)` on its own line
   aligned with `Search`.

2. Should `HeadTail` always include the anchor kind despite being always ANCHORED?
   Current draft omits it since there's no UNANCHORED HeadTail.

3. Should `Index` include its resolved position or only the offset? Current draft
   shows offset only.

4. Should the root brane in snapshot output be visually contained, or is the
   `[0] RESULT:` header sufficient? Current draft renders the root brane with braces.

## References

- Current HS implementation: `foolish-core/src/sequencer.rs`
- FIR type definitions: `foolish-core/src/fir.rs`
- UBC evaluator: `foolish-core/src/ubc.rs`
- UBCb evaluator: `foolish-ubcb/src/ubcb.rs`
- Snapshot test infrastructure: `foolish-core/src/snapshot_suite.rs`
- Prior HS spec: `UBC_humanizing_sequence_round_1.spec.md`
- **Acceptance test**: `foolish-core/snapshot_tests/input/foop42_humanizing_sequencer_formatting_exhaustive.foo`
- **Current baseline**: `foolish-core/snapshot_tests/approved/foop42_humanizing_sequencer_formatting_exhaustive.foo.snap.new`
- **Implementation plan**: `docs/foop/FOOP-42.plan.md`

## Last Updated

**Date**: 2026-06-03
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Fleshed out FOOP-42 draft with comprehensive requirements: indentation
model (A_DENTS + B_DENTS), per-FIR-variant formatting rules, recursive rendering
pseudo-code, full annotated examples, state display table, and implementation notes.
Added explicit coverage for all 10 FIR variants, search result display, non-atomic
function-call formatting, and transparent constant operators.
