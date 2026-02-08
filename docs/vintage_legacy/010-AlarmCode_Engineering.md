# AlarmCode Engineering Documentation

## Overview

The AlarmCode system provides standardized error codes for the Foolish VM alarm system. This document describes the engineering rationale, implementation details, and guidelines for extending the system.

## Design Rationale

### Problem Statement

Prior to AlarmCode, alarm messages were constructed ad-hoc with string messages and numeric severity levels:

```java
AlarmSystem.raise(context, "Detaching identifier not referenced in subsequent brane: " + name, AlarmSystem.MILD);
```

This approach had several issues:
1. **Inconsistent messaging** - No standard format for similar errors
2. **No unique identifiers** - Difficult to track specific error types in logs
3. **Hard to categorize** - No systematic organization of error codes
4. **Poor i18n support** - Messages embedded in code
5. **Difficult to document** - No central registry of all possible errors

### Solution: Enum-Based Error Codes

The AlarmCode enum provides:
- **Unique identifiers** - Each error has a hex code within its severity level
- **Standardized messages** - Generic gerund-based descriptions
- **Severity hierarchy** - Built-in severity levels (BARELY, MILD, HAIR_RAISING, PANIC)
- **Formatted codes** - Human-readable codes like "M0101", "H0201"
- **Optional details** - Generic message + optional specific context

## Code Format Specification

### Format: `{SeverityLetter}{HexCode}`

- **Severity Letter**: Single character indicating severity level
  - `A` = BARELY (level 1) - Minor informational messages
  - `M` = MILD (level 3) - Moderate warnings, likely programming errors
  - `H` = HAIR_RAISING (level 5) - Significant warnings, semantic violations
  - `P` = PANIC (level 10) - Critical errors, system integrity failures

- **Hex Code**: 4-digit hexadecimal identifier (0x0001 - 0xFFFF)
  - Each severity level has independent numbering
  - Starts at 0x0001 for first code in each level
  - Allows up to 65,535 unique codes per severity level

### Examples

```
A0001 - Accessing premature value (BARELY)
A0002 - Using deprecated feature (BARELY)
M0101 - Detaching unresolved ordinate (MILD)
M0102 - Detaching without SF mark (MILD)
H0201 - Violating immutability constraint (HAIR_RAISING)
H0202 - Violating PRIMED constraint (HAIR_RAISING)
P0301 - Dividing by zero (PANIC)
P0302 - Encountering null value (PANIC)
```

## Message Conventions

### Gerund-Based Generic Messages

All AlarmCode messages use **gerund form** (verb + -ing) for consistency:

✅ **Correct**:
- "Detaching identifier not referenced in subsequent brane"
- "Violating immutability constraint"
- "Missing identifier in current context"
- "Dividing by zero"

❌ **Incorrect**:
- "Detached identifier..." (past tense)
- "Violation of immutability" (noun form)
- "Identifier missing..." (participle adjective)
- "Division by zero" (noun form)

### Generic Message + Details Pattern

The generic message describes the **error class**, while details provide **specific context**:

```java
// Generic: "Detaching identifier not referenced in subsequent brane"
// Details: "symbol 'a'"
AlarmSystem.raise(fir, AlarmCode.DETACHING_UNRESOLVED_ORDINATE, "symbol 'a'");

// Output:
// [ALARM 3] M0101 Index:[0#1#2] test.foo:line 15 - Detaching identifier not referenced in subsequent brane (symbol 'a')
```

This pattern allows:
- **Log parsing** - Consistent format for automated analysis
- **Error grouping** - Group by code even with different details
- **Context preservation** - Specific variable names, values, etc.

## Code Allocation Strategy

### Current Allocation

| Severity | Range Start | Next Available | Reserved Range |
|----------|-------------|----------------|----------------|
| BARELY   | 0x0001      | 0x0003         | 0x0001-0x00FF  |
| MILD     | 0x0101      | 0x0106         | 0x0100-0x01FF  |
| HAIR_RAISING | 0x0201  | 0x0206         | 0x0200-0x02FF  |
| PANIC    | 0x0301      | 0x0306         | 0x0300-0x03FF  |

### Allocation Guidelines

1. **Start codes at distinct ranges**:
   - BARELY: 0x0001-0x00FF (1-255)
   - MILD: 0x0100-0x01FF (256-511)
   - HAIR_RAISING: 0x0200-0x02FF (512-767)
   - PANIC: 0x0300-0x03FF (768-1023)

2. **Sequential allocation within ranges**:
   - Add new codes sequentially: 0x0001, 0x0002, 0x0003, ...
   - Don't skip numbers unless reserving for related codes

3. **Group related errors**:
   - Detachment errors: M0101, M0102
   - Immutability violations: H0201, H0202
   - Null/value errors: P0302, P0303

4. **Reserve expansion space**:
   - Each severity level has 256 codes reserved
   - If needed, expand to 0x0400+, 0x0500+, etc.

## Adding New Alarm Codes

### Step 1: Choose Severity Level

Determine appropriate severity:

- **BARELY** - User might not care, purely informational
- **MILD** - Likely a bug, should be investigated
- **HAIR_RAISING** - Serious semantic violation, must be fixed
- **PANIC** - Critical failure, system cannot continue safely

### Step 2: Allocate Code Number

Choose next available hex code in the severity range:

```java
// Check current codes in that severity level
// MILD range: 0x0100-0x01FF
// Current: M0101, M0102, M0103, M0104, M0105
// Next: 0x0106

MISSING_REQUIRED_PARAMETER(AlarmSystem.MILD, 0x0106,
    "Missing required parameter in function call"),
```

### Step 3: Write Generic Message

Use gerund form describing the error class:

```java
// Good: Describes what's happening
"Accessing uninitialized variable"
"Calling function with wrong argument count"
"Exceeding maximum recursion depth"

// Bad: Noun form or past tense
"Uninitialized variable access"
"Function called with wrong argument count"
"Maximum recursion depth exceeded"
```

### Step 4: Add to Enum

Insert in appropriate severity section:

```java
public enum AlarmCode {
    // === MILD (Level 3) ===

    DETACHING_UNRESOLVED_ORDINATE(AlarmSystem.MILD, 0x0101,
        "Detaching identifier not referenced in subsequent brane"),

    DETACHING_WITHOUT_SF_MARK(AlarmSystem.MILD, 0x0102,
        "Detaching identifier without SF mark"),

    // ... existing codes ...

    MISSING_REQUIRED_PARAMETER(AlarmSystem.MILD, 0x0106,
        "Missing required parameter in function call"),  // NEW
```

### Step 5: Use in Code

Replace old raise() calls:

```java
// Old:
AlarmSystem.raise(context, "Missing required parameter: " + paramName, AlarmSystem.MILD);

// New:
AlarmSystem.raise(fir, AlarmCode.MISSING_REQUIRED_PARAMETER, "parameter '" + paramName + "'");
```

## Migration Guide

### Finding Candidates

Search for direct raise() calls with string messages:

```bash
# Find all raise() calls with string literals
grep -r "AlarmSystem.raise.*\"" foolish-core-java/src/main/java/

# Common patterns to migrate:
AlarmSystem.raise(context, "...", AlarmSystem.MILD)
AlarmSystem.raiseFromFir(fir, "...", AlarmSystem.HAIR_RAISING)
```

### Migration Pattern

**Before**:
```java
if (depth > MAX_BRANE_DEPTH) {
    AlarmSystem.raiseFromFir(fir,
        "Brane depth " + depth + " exceeds maximum " + MAX_BRANE_DEPTH,
        AlarmSystem.HAIR_RAISING);
}
```

**After**:
```java
if (depth > MAX_BRANE_DEPTH) {
    AlarmSystem.raise(fir,
        AlarmCode.EXCEEDING_BRANE_DEPTH,
        "depth " + depth + " exceeds maximum " + MAX_BRANE_DEPTH);
}
```

### Gradual Migration

1. **Add new AlarmCode** for common error patterns
2. **Keep old methods** for now (backwards compatibility)
3. **Migrate incrementally** - update code as you touch it
4. **Eventually deprecate** old string-based raise() methods
5. **Remove deprecated methods** after full migration

## Output Format

### Standard Format

```
[ALARM {severity}] {code} Index:{index} {location} - {genericMessage} ({details})
```

### Components

- **severity**: Numeric severity level (1, 3, 5, 10)
- **code**: Formatted code (A0001, M0101, H0201, P0301)
- **index**: FoolishIndex seek-like path [0#1#2#3]
- **location**: Source file and line (test.foo:line 15)
- **genericMessage**: From AlarmCode enum
- **details**: Optional specific context (in parentheses)

### Example Output

```
[ALARM 3] M0101 Index:[0#1#2] detachment.foo:line 10 - Detaching identifier not referenced in subsequent brane (symbol 'a')
```

Breakdown:
- Severity: 3 (MILD)
- Code: M0101 (DETACHING_UNRESOLVED_ORDINATE)
- Index: [0#1#2] (root → statement 1 → nested statement 2)
- Location: detachment.foo:line 10
- Generic: "Detaching identifier not referenced in subsequent brane"
- Details: "(symbol 'a')"

## Testing Considerations

### Test with AlarmCode

```java
@Test
public void testDetachmentWithoutReference() {
    // Capture stderr
    ByteArrayOutputStream stderr = new ByteArrayOutputStream();
    System.setErr(new PrintStream(stderr));

    // Trigger alarm
    AlarmSystem.raise(fir, AlarmCode.DETACHING_UNRESOLVED_ORDINATE, "symbol 'x'");

    // Verify output
    String output = stderr.toString();
    assertTrue(output.contains("[ALARM 3]"));
    assertTrue(output.contains("M0101"));
    assertTrue(output.contains("symbol 'x'"));
}
```

### Approval Test Matching

Alarm output appears in `.approved.foo` files:

```
!! [ALARM 3] M0101 Index:[0#1#2] test.foo:line 10 - Detaching identifier not referenced in subsequent brane (symbol 'a')
```

When updating approval tests:
1. Run test to generate `.received.foo`
2. Check that alarm code appears correctly
3. Verify index and location are accurate
4. Approve if correct

## Future Enhancements

### Possible Extensions

1. **I18N Support**:
   ```java
   public String getLocalizedMessage(Locale locale) {
       return ResourceBundle.getBundle("alarms", locale)
           .getString(this.name());
   }
   ```

2. **Structured Logging**:
   ```java
   public Map<String, Object> toStructuredLog() {
       return Map.of(
           "code", getFormattedCode(),
           "severity", severity,
           "message", genericMessage,
           "details", details
       );
   }
   ```

3. **Help URLs**:
   ```java
   public String getHelpUrl() {
       return "https://docs.foolish-lang.org/errors/" + getFormattedCode();
   }
   ```

4. **Suggested Fixes**:
   ```java
   public String getSuggestedFix() {
       // Return hint for fixing this error
   }
   ```

## Related Files

- `AlarmCode.java` - Enum definition
- `AlarmSystem.java` - Alarm raising and threshold checking
- `FoolishIndex.java` - Hierarchical indexing for error locations
- `FIR.java` - getLocationDescription() for source locations

## References

- Project plan: `projects/009-Concatenation_Project.md` (Phase 5)
- Alarm system docs: `AlarmSystem.java` javadoc
- Severity levels: `AlarmSystem.java` constants

---

## Last Updated

**Date**: 2026-02-01
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-5-20250929
**Changes**: Initial creation documenting AlarmCode enum design, implementation, and migration guidelines.
