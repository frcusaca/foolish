# UBC (Unicelular Brane Computer) New Features

## Summary of Changes

This document describes the new features added to the UBC system.

## 1. NK (Not-Known) Values with Comments

### Overview
NK values now include optional comments explaining why a value is not-known.

### Implementation
- **NKFiroe** (src/main/java/org/foolish/fvm/ubc/NKFiroe.java)
  - Added `nkComment` field (String, defaults to null)
  - Constructor: `NKFiroe(AST ast, String comment)`
  - Method: `getNkComment()` returns the comment explaining the NK value

### Error Handling with Comments
All arithmetic operators now provide detailed error messages:

#### Division and Modulo by Zero
```java
10 / 0  → ??? with comment: "Division by zero"
10 % 0  → ??? with comment: "Modulo by zero"
```

#### Propagated NK Values
```java
??? + 5  → ??? with comment: "Operand is not-known"
```

#### Runtime Exceptions
All binary and unary operations are wrapped in try-catch blocks:
```java
try {
    // perform operation
} catch (Exception e) {
    result = new NKFiroe(ast, e.getMessage());
}
```

## 2. IdentifierFiroe - Identifiers with Characterizations

### Overview
Identifiers can now have characterizations forming a chain.

### AST Structure
The `Identifier` record in AST.java:
```java
record Identifier(Identifier characterization, String id)
```

- Simple: `x`
- Characterized: `type'x`
- Chained: `outer'inner'x`

The characterization chain terminates when characterization is null.

### Implementation
- **IdentifierFiroe** (src/main/java/org/foolish/fvm/ubc/IdentifierFiroe.java)
  - Stores identifier name and characterization chain
  - Methods:
    - `getId()` - returns identifier name
    - `getCharacterization()` - returns characterization Identifier (or null)
    - `getCharacterizationChain()` - returns full chain as string
  - `isAbstract()` returns `true` (identifier lookup not yet implemented)
  - `getValue()` throws UnsupportedOperationException

### Future Work
Full identifier lookup from environment is not yet implemented in UBC. When implemented, IdentifierFiroe will:
1. Search for the identifier in the current scope
2. Use characterizations to disambiguate between multiple bindings
3. Return the bound value or NK if not found

## 3. The `fi` Marker

### Overview
The `fi` keyword marks the end of an if-expression block.

### Grammar
```
ifExpr: ifExprHelperIf (ifExprHelperElif)* (ifExprHelperElse)? endIf?
endIf: FI ;
```

The `fi` marker is **optional** and purely syntactic - it doesn't affect evaluation.

### Purpose
1. **Readability**: Makes nested if-expressions easier to read
2. **Association**: Visually shows which `fi` closes which `if`
3. **Style**: Similar to `fi` in shell scripts or `end` in other languages

### Examples

Without `fi`:
```
if 1 then if 0 then 10 else 20 else 30
```

With `fi`:
```
if 1 then
    if 0 then 10 else 20 fi
else 30 fi
```

### Implementation Note
The parser recognizes `fi` but it doesn't create any AST nodes or affect the FIR evaluation. It's purely a parsing/syntactic feature.

## 4. Implicit `else ???` in If-Expressions

### Overview
If-expressions without an explicit `else` branch now automatically get an implicit `else ???` branch.

### Behavior

**Before:**
```
if 0 then 42        // What happens if condition is false?
```

**After:**
```
if 0 then 42        // Implicitly: if 0 then 42 else ???
// Result: ???
```

### Implementation
In `IfFiroe.initialize()` (src/main/java/org/foolish/fvm/ubc/IfFiroe.java):

```java
if (ifExpr.elseExpr() == AST.UnknownExpr.INSTANCE || ifExpr.elseExpr() == null) {
    // No explicit else branch - add implicit else ???
    enqueueFirs(new NKFiroe("No matching condition in if-elif chain"));
} else {
    enqueueFirs(createFiroeFromExpr(ifExpr.elseExpr()));
}
```

### Rationale
1. **Safety**: Every if-expression has a defined result, even when no branch matches
2. **Explicitness**: Makes it clear that unhandled cases result in NK
3. **Consistency**: Aligns with the "errors as values" philosophy

### Examples

```foolish
{
	// Condition false, no else → ???
	if 0 then 42;

	// Same as above, explicit
	if 0 then 42 else ???;

	// Elif chain, none match → ???
	if 0 then 10
	elif 0 then 20
	elif 0 then 30;
	// Result: ???

	// Explicit else prevents NK
	if 0 then 10 else 99;
	// Result: 99
}
```

## 5. If-Then-Else Tests

### Test Coverage
Added 10 comprehensive tests in `UbcApprovalTest.java`:

1. **simpleIfThenElseIsApproved** - Basic if-then-else with true condition
2. **simpleIfThenElseFalseIsApproved** - Basic if-then-else with false condition
3. **ifThenNoElseImplicitNKIsApproved** - Tests implicit else ???
4. **ifElifElseChainIsApproved** - Multiple elif branches
5. **ifWithComplexConditionIsApproved** - Arithmetic in condition
6. **ifWithComplexThenValueIsApproved** - Arithmetic in then branch
7. **nestedIfThenElseIsApproved** - If inside if
8. **ifWithFiMarkerIsApproved** - Tests fi marker
9. **deeplyNestedIfWithFiIsApproved** - Multiple levels with fi
10. **multipleNestedIfCheckingFiAssociationIsApproved** - Verifies fi associates correctly

### Known Issues
The IfFiroe implementation has pre-existing bugs causing infinite loops in evaluation. These tests are added but currently fail. The IfFiroe logic needs to be debugged separately.

## Architecture Notes

### Error Handling Philosophy
**Errors as Values**: Rather than exceptions that halt computation:
- Arithmetic errors produce NK values
- NK values propagate through computations
- Allows partial results even when some operations fail
- Better for interactive/exploratory environments

### Type System Future
When types are added:
- Characterizations will be used for type annotations
- Type mismatches could produce NK with appropriate comments
- Type inference could use characterization chains

### Environment Future
When environment/scope is implemented:
- IdentifierFiroe will lookup values from environment
- Characterizations can disambiguate shadowed names
- Undefined identifiers will return NK

## Files Modified

### New Files
- `/Volumes/0/user/claude/src/main/java/org/foolish/fvm/ubc/IdentifierFiroe.java`
- `/Volumes/0/user/claude/ARITHMETIC_ERRORS.md`
- `/Volumes/0/user/claude/UBC_FEATURES.md` (this file)

### Modified Files
- `/Volumes/0/user/claude/src/main/java/org/foolish/fvm/ubc/NKFiroe.java`
- `/Volumes/0/user/claude/src/main/java/org/foolish/fvm/ubc/BinaryFiroe.java`
- `/Volumes/0/user/claude/src/main/java/org/foolish/fvm/ubc/UnaryFiroe.java`
- `/Volumes/0/user/claude/src/main/java/org/foolish/fvm/ubc/IfFiroe.java`
- `/Volumes/0/user/claude/src/main/java/org/foolish/fvm/ubc/FIR.java`
- `/Volumes/0/user/claude/src/test/java/org/foolish/fvm/ubc/UbcApprovalTest.java`

## Next Steps

1. **Fix IfFiroe**: Debug the infinite loop issues in if-expression evaluation
2. **Implement Environment**: Add scope/environment for identifier lookup
3. **Complete IdentifierFiroe**: Implement actual identifier resolution
4. **Type System**: Use characterizations for type annotations
5. **Floating Point**: Add NaN handling as documented in ARITHMETIC_ERRORS.md
