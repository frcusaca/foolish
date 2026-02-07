# Foolish Style Guide

Universal instructions for writing and generating Foolish code examples, documentation,
and tests. These are preferences, not strictly enforced by the language.

## File Format

- Foolish source files use the `.foo` extension
- Text width target: **108 characters** for all textual documents (`.foo`, `.txt`, `.md`, `.html`, `.xml`)

## Indentation

Brane depth uses **tab characters** (not spaces) to reduce storage occupancy.

Multi-line statement alignment may use spaces after the leading tabs. The spaces align
continuation lines with the relevant part of the first line:

```foolish
{
	a = ( 1 +
	      2 + !! 6 spaces after tab to align with 1
	      3 +
	      4 +
	      5
	    ) !! closing paren at a's alignment depth
}
```

Alternatively, a simpler alignment is equally acceptable:

```foolish
{
	a = (
	 1 + !! 1 space to separate from 'a' visually
	 2 +
	 3 +
	 4 +
	 5
	) !! closing paren at a's depth
}
```

The rule: tabs mark brane depth only. Within a brane level, spaces handle sub-alignment.
Deep indentation is not necessary as long as it is consistent and aids reading.

## Terminology

Use these terms consistently in documentation and code:

| Term | Meaning |
|------|---------|
| **name** | Either an identifier or a characterized identifier |
| **identification** | An assignment (`x = expr`) identifies an expression with a name |
| **ordinate** | A name (axis/dimension) that an expression is ordinated to in a brane |
| **coordination** | An expression becoming coordinated with other ordinates during evaluation |
| **Foolisher** | A person who inhabits and develops Foolish |
| **nye** | (says "nigh") Any pre-constanic state; a FIR not yet at CONSTANIC |
| **constanic** | (says "cons-TAN-tic") Constant in context; may gain value in new context. Displays as `⎵⎵` |
| **no-no** | The `???` value (Not Known); definitely unknown, a final state |
| **retrospection** | Search upward/backward through scope |
| **prospection** | Search downward/forward |

### Development Stages

A feature progresses through these stages:

1. **Lexed** — Parses into AST, but downstream yields `???` when encountered
2. **Interpreted** — VM machinery correctly handles the feature
3. **Tested** — Good milestones are "lexed and tested" and "interpreted and tested"

## Naming Conventions

### Identifier Length Distribution

Variable names should follow a **power-law (Zipf) distribution**:

- **Most names**: 1-2 words (short, common)
- **Some names**: 3-4 words (occasional, descriptive)
- **Rare names**: 5+ words (expressive when needed)

Mean length targets:
- Short branes: ~3.5 characters
- Longer branes: ~5 characters

Like words in natural language — short ones are common, long ones are rare but expressive.

### Multi-Word Separators

Three separators are equivalent for intra-identifier word boundaries:

| Separator | Character | Unicode |
|-----------|-----------|---------|
| Underscore | `_` | U+005F |
| Modifier letter low macron | `ˍ` | U+02CD |
| Narrow no-break space | ` ` | U+202F |

The system normalizes `_` to `ˍ` (U+02CD) for display. Authors may write underscores;
the system converts them.

### Unicode and Multilingual Names

Foolish embraces Unicode. Use sensible names from all available scripts to improve
expressivity and disambiguate concepts:

| Script | Example | Note |
|--------|---------|------|
| Latin | `x`, `total_sum` | Most common |
| Greek | `π`, `δ_τιμή` | Mathematical, philosophical |
| Cyrillic | `Б`, `длинное_имя` | Slavic concepts |
| Hebrew | `א` | Right-to-left |
| Arabic | `سعر` | Right-to-left |
| Chinese | `值`, `入口点` | Logographic |
| Sanskrit | `शून्य` | Devanagari script |

## Voice and Register

When writing documentation that spans several sentences or paragraphs, explicitly signal
to the reader whether you are speaking **about the Foolish language** (in layman's terms)
or speaking **in Foolish** (using Foolish terminology).

Use the language's name as the transition marker:

- "In Foolish, ..." signals that what follows uses Foolish-specific terminology
  (branes, ordinates, coordination, constanic, nye, etc.). The reader should expect
  technical vocabulary.

- "In plain terms, ..." or an unmarked paragraph signals a general explanation
  accessible to someone who has never seen Foolish before.

Do not italicize or bold the transition phrases themselves — the English linguistic
feature of capitalization ("In Foolish" vs "in foolish terms") is sufficient to mark
the register shift. However, do use italic emphasis when introducing a Foolish term
for the first time (e.g., "In Foolish, this is called *retrospective search*") — this
helps the reader distinguish a technical term from the surrounding prose.

This prevents the reader from confusing a metaphor with a technical term. For example:

> A container holds values — think of it like a box. In Foolish, this container is
> called a brane, and the values it holds become ordinated to the brane, meaning
> they gain named positions like coordinates on a graph.

The goal is clarity across audiences. A philosopher reading `docs/why/` should not need
to know what "ordinate" means to follow the argument. A Foolisher reading `docs/howto/`
should see the precise terminology when it matters. Signal the register shift explicitly
so both readers know where they stand.

This convention applies to all documentation directories, but is especially important in
`docs/why/` (where prose is often philosophical) and `docs/howto/` (where tutorials
alternate between explanation and code).

## Testing

### Unit Tests

Unit tests specifically test each unit of software — the primary check of correctness.

### Approval Tests

Approval tests illustrate behavior to users. They should:

- Correspond to unit tests where possible
- Illustrate the most **important** and most **easily confused** aspects of behavior
- Be comprehensive to show compatibility between different VM implementations
- Establish mutual understanding between human users and the FVM in a human-readable way

Approval tests use full-width space (`＿`) instead of tabs to show indentation depth
more precisely.

## Last Updated

**Date**: 2026-02-06
**Updated By**: Claude Code v1.0.0 / claude-opus-4-6
**Changes**: Initial creation from docs/vintage_legacy/STYLES.md content, restructured as universal style guide.
