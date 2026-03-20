# Foolish Style Guide

> AI agents: read [../DOC_AGENTS.md](../DOC_AGENTS.md) before editing this file.

Universal instructions for writing and generating Foolish code examples, documentation,
and tests. These are preferences, not strictly enforced by the language.

## File Format

- Foolish source files use the `.foo` extension
- Text width target: 108 characters for all textual documents (`.foo`, `.txt`, `.md`, `.html`, `.xml`)

## Indentation

Brane depth uses tab characters (not spaces) to reduce storage occupancy.

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
| **nye** / **nigh** | (pronounced "nigh", interchangeable) Any pre-constanic state; a FIR not yet at CONSTANIC |
| **constanic** | (pronounced "con-TAN-tic") Adjective: constant in context; may gain value in new context. Use lowercase mid-sentence; "Constanic" at sentence start or in titles. |
| **CONSTANIC** | (pronounced "con-TAN-tic") Noun: the exact Nyes state name. Always all-caps. |
| **constanicity** | (pronounced "con-stan-ISS-ity") Noun: the property or fact of being constanic. |
| **no-no** | The `???` value (Not Known); definitely unknown, a final state |
| **retrospection** | Search upward/backward through scope |
| **prospection** | Search downward/forward through scope |

### Nyes State Names (Always All-Caps)

| State | Meaning |
|-------|---------|
| **PREMBRYONIC** | Holding AST; transitioning to structured representation |
| **EMBRYONIC** | Actively resolving searches through local work and message passing |
| **BRANING** | Child execution and message forwarding; monitoring dependencies |
| **ECONSTANIC** | Exactly CONSTANIC: search found nothing, needs new context |
| **WOCONSTANIC** | Waiting On CONSTANICs: found all identifiers but some are still constanic |
| **CONSTANT** | Fully evaluated; immutable |
| **INDEPENDENT** | Detached from parent; reserved for future development |

### Usage Examples

| Correct | Incorrect | Reason |
|---------|-----------|--------|
| "The FIR is constanic." | "The FIR is Constanic." | Adjective form is lowercase mid-sentence |
| "Constanic states are terminal." | "constanic states are terminal." | Adjective capitalizes at sentence start |
| "The FIR is in CONSTANIC state." | "The FIR is in Constanic state." | State name is always all-caps |
| "The FIR is still nye." | "The FIR is still CONSTANIC." | "nye" = not yet constanic |
| "The brane's constanicity ensures stability." | "The brane's constanic ensures stability." | Use noun form "constanicity" for the property |

### Development Stages

A feature progresses through these stages:

1. **Lexed** — Parses into AST, but downstream yields `???` when encountered
2. **Interpreted** — VM machinery correctly handles the feature
3. **Tested** — Good milestones are "lexed and tested" and "interpreted and tested"

## Naming Conventions

### Identifier Length Distribution

Variable names should follow a power-law (Zipf) distribution:

- Most names: 1-2 words (short, common)
- Some names: 3-4 words (occasional, descriptive)
- Rare names: 5+ words (expressive when needed)

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
to the reader whether you are speaking about the Foolish language (in layman's terms)
or speaking in Foolish (using Foolish terminology).

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
- Illustrate the most important and most easily confused aspects of behavior
- Be comprehensive to show compatibility between different VM implementations
- Establish mutual understanding between human users and the FVM in a human-readable way

Approval tests use full-width space (`＿`) instead of tabs to show indentation depth
more precisely.

## Last Updated

**Date**: 2026-03-20
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Added comprehensive terminology table for constanic/CONSTANIC/constanicity distinction.
Clarified nye/nigh as interchangeable. Added Nyes State Names table and Usage Examples table.
Previous (2026-02-26): Reduced emphatic markings — removed bold from running prose.
