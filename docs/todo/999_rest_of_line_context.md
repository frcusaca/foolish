# Rest-of-Line Context Switching with Backtick

## Status: Proposal

## Summary

Introduce the backtick `` ` `` as a **syntactic characterization** operator that switches
the parser context for the rest of the line (or until a closing `` ` ``). The characterization
preceding the backtick determines which sub-parser handles the content.

## Motivation

Foolish's core operative verbs are:

1. **Naming** — assignments, ordinations, coordinations
2. **Concatenation** — proximity creates combination

Arithmetic operators (`+`, `-`, `*`, `/`), string literals, boolean logic, and other
notation are not core to the language — they are conveniences for working with specific
data types. Currently, these operators are baked into the main parser grammar, which
pollutes the core language with domain-specific syntax.

By moving these into **context-switched sub-parsers**, we:

- Reduce the core grammar to its essential verbs (naming + concatenation)
- Allow each data type to define its own natural notation
- Make the language extensible — new data types bring their own syntax
- Keep the familiar convenience of infix arithmetic and other notations

## Syntax

The general form is:

```
characterization` rest-of-line-content
```

The characterization before `` ` `` selects the parsing context. Everything from `` ` ``
to EOL is parsed by that context's sub-parser.

### Numeric Context: `#`

```foolish
{
    sum = #`1 + 2 + 3
    product = #`6 * 7
    expr = #`2 + 3 * 4
}
```

The `#` characterization activates the arithmetic operator parser. Inside this context,
`+`, `-`, `*`, `/`, `%`, parentheses, and precedence rules all apply. Outside this
context, those operators have no meaning in the core language.

### String Context: `$`

```foolish
{
    greeting = $`Hello, world!
    msg = $`This is a string without quotes
}
```

The `$` characterization activates a string literal parser. Everything from `` ` `` to EOL
is the string content — no quoting needed.

### Boolean Context: `?`

```foolish
{
    result = ?`a | b | c
    check = ?`x & y
}
```

The `?` characterization activates a boolean/logic parser with its own operators.

### Blocked Context (Inline)

The rest-of-line behavior is the default, but a closing backtick can terminate the
context mid-line:

```foolish
{
    t = #`1 + 2 + 3` ;
    name = $`some text` ;
}
```

This is not necessary in most cases — the EOL naturally closes the context — but it is
available when multiple expressions share a line.

## Relation to Foolish Characterization (`'`)

The backtick `` ` `` syntax superficially resembles the apostrophe `'` characterization:

| Syntax | Name | Purpose |
|--------|------|---------|
| `c'BLAH` | Foolish characterization | Semantic label — describes what a value *is* |
| `` c`BLAH `` | Syntactic characterization | Parser context — determines how content is *parsed* |

The apostrophe `'` is a **Foolish characterization**: it operates within the language's
semantic layer, labeling values with descriptions that programs can inspect.

The backtick `` ` `` is a **syntactic characterization**: it operates at the parser level,
selecting which grammar rules apply to the following content. It is resolved at parse time,
before any semantic evaluation.

Pin in this for now — the relationship between these two forms of characterization may
deepen as both features mature.

## Impact on Core Grammar

With this change, the core Foolish grammar reduces to:

- **Branes** `{ }`
- **Naming** (assignment/ordination) `x = expr`
- **Concatenation** (proximity) `brane1 brane2`
- **Searches** `.` `?` `~` `^` `$` `#`
- **Syntactic contexts** `` characterization`content ``

Arithmetic, string literals, boolean logic, and other notations move out of the core
grammar and into context-specific sub-parsers. Each data type owns its own syntax.

## Open Questions

- What is the full set of built-in syntactic characterizations?
- Can users define their own syntactic characterizations (custom sub-parsers)?
- How does this interact with liberation branes `[...]`?
- Should nested contexts be allowed (e.g., `` #`1 + $`len`` )?
- Error reporting: how do parse errors inside a context bubble up?
- How does this affect the ANTLR grammar architecture — lexer modes? Island grammars?

## Last Updated

**Date**: 2026-02-06
**Updated By**: Claude Code v1.0.0 / claude-opus-4-6
**Changes**: Initial proposal document.
