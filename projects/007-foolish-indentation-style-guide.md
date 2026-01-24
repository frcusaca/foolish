# Foolish Indentation Style Guide

## Overview

This document describes the preferred indentation style for `.foo` files and all Foolish code.

## Core Rules

### 1. Multi-line Branes Always Use Separate Lines for Curly Brackets

When a brane spans more than one line, the opening and closing curly brackets must be on their own lines:

```foolish
// Good - multi-line brane
{
	a = 1;
	b = 2;
}

// Also acceptable - single-line brane
{a = 1}
```

### 2. Each Brane Layer Adds One Tab Character

Each level of brane nesting adds exactly **one tab character** (`\t`) to subsequent lines:

```foolish
{
	outer = 1;
	{
		inner = 2;
		{
			deepest = 3;
		};
	};
}
```

### 3. User-Selected Space Indentation is Preserved

Users may choose to add **space characters** for alignment within expressions. The formatter should preserve these spaces as best as possible:

```foolish
{
	a = (
	   !! This line: 1 tab + 3 spaces (user-selected)
	   1 + 2 + 3 + {
		   !! This line: 1 tab + 3 spaces + 1 tab
		   nested = 100;
	   }
	);
}
```

In this example:
- The line after `a = (` is indented with **1 tab + 3 spaces**
  - 1 tab from the outer brane `{`
  - 3 spaces are user-selected for alignment with the opening `(`
- The line inside the nested brane is indented with **1 tab + 3 spaces + 1 tab**
  - 1 tab from outer brane
  - 3 spaces preserved from parent expression
  - 1 tab from the nested brane `{`

## Test Output Rendering

In test output (`.approved.foo` and `.received.foo` files), the rendering uses special characters:

### Wide Underscore for Tabs

Tab characters are replaced with the **full-width low line** character `＿` (U+FF3F):

```
FINAL RESULT:
{
＿a = 1;
＿{
＿＿b = 2;
＿};
}
```

Each `＿` represents one level of brane nesting.

### Actual Space Characters are Preserved

Regular space characters (U+0020) used for user-selected alignment are preserved as-is in the output:

```
{
＿a = (
＿   !! Three actual spaces after the tab
＿   value
＿);
}
```

## Summary Table

| Context | Indentation Type | Character | Purpose |
|---------|-----------------|-----------|---------|
| Brane nesting | Tab | `\t` | Each brane level adds one tab |
| User alignment | Space | ` ` (U+0020) | User-selected, preserved by formatter |
| Test output (brane) | Wide underscore | `＿` (U+FF3F) | Renders tabs in test files |
| Test output (alignment) | Space | ` ` (U+0020) | Preserved as-is |

## Rationale

### Why Tabs for Branes?

- **Semantic meaning**: Each tab represents one level of scope/brane nesting
- **Consistency**: Easy to verify correct nesting depth
- **Flexibility**: Users can configure tab width in their editor

### Why Spaces for Alignment?

- **Visual alignment**: Allows precise alignment of multi-line expressions
- **User control**: Developers choose alignment that makes sense for their code
- **Formatter preservation**: Formatter respects user's alignment choices

## Examples

### Simple Nested Branes

```foolish
{
	outerVar = 1;
	{
		innerVar = 2;
	};
}
```

### Mixed Tabs and Spaces

```foolish
{
	result = someFunction(
	         argument1,
	         argument2,
	         {
		         nested = value;
	         }
	);
}
```

Breakdown:
- Line 2: 1 tab (outer brane)
- Line 3: 1 tab + 9 spaces (aligned with opening paren)
- Line 4: 1 tab + 9 spaces
- Line 5: 1 tab + 9 spaces
- Line 6: 1 tab + 9 spaces + 1 tab (nested brane)
- Line 7: 1 tab + 9 spaces

### Complex Nesting with Comments

```foolish
{
	a = (
	   !! Comment: 1 tab + 3 spaces
	   computeValue() + {
		   !! Comment: 1 tab + 3 spaces + 1 tab
		   localVar = 42;
	   }
	);
}
```

## Current Implementation Status

### What Works

- Brane nesting with tabs is correctly implemented in `Sequencer4Human.java` and `Sequencer4Human.scala`
- Test output correctly renders tabs as `＿` characters
- Space preservation generally works

### Known Issues

- [ ] Formatter may not perfectly preserve user-selected space indentation in all cases
- [ ] ASTFormatter needs review to ensure it follows this convention
- [ ] Need to verify all test files follow this convention

## Related Files

- `foolish-core-java/src/main/java/org/foolish/fvm/ubc/Sequencer4Human.java` - Java test output formatter
- `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/Sequencer4Human.scala` - Scala test output formatter
- `foolish-parser-java/src/main/java/org/foolish/ast/ASTFormatter.java` - AST formatter for PARSED AST section

## TODO

- [ ] Audit all `.foo` test files to ensure they follow this convention
- [ ] Update ASTFormatter to ensure proper tab/space handling
- [ ] Add formatter tests to verify indentation preservation
- [ ] Document how formatters should handle edge cases (empty lines, comments, etc.)

---

## Last Updated

**Date**: 2026-01-25
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-5-20250929
**Changes**: Initial documentation of Foolish indentation style guide based on user specification.
