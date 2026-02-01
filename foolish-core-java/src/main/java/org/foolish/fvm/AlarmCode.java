package org.foolish.fvm;

/**
 * Standardized alarm codes with severity, unique identifier, and generic message.
 * <p>
 * Each alarm has:
 * <ul>
 *   <li><b>Severity:</b> One of {@link AlarmSystem}'s levels (BARELY, MILD, HAIR_RAISING, PANIC)</li>
 *   <li><b>Code:</b> Unique hexadecimal identifier within severity level (e.g., 0x0001)</li>
 *   <li><b>Generic Message:</b> Gerund-based description of the error condition</li>
 * </ul>
 * <p>
 * <b>Usage:</b>
 * <pre>
 * AlarmSystem.raise(fir, AlarmCode.DETACHING_UNRESOLVED_ORDINATE, "symbol 'a'");
 * // Output: [ALARM 3] A0301 Index:[0#1#2] file.foo:line 10 - Detaching identifier not referenced in subsequent brane (symbol 'a')
 * </pre>
 */
public enum AlarmCode {
    // === BARELY (Level 1) ===
    // Informational messages, minor issues

    /** Accessing value before full evaluation */
    ACCESSING_PREMATURE_VALUE(AlarmSystem.BARELY, 0x0001,
        "Accessing value before full evaluation"),

    /** Using deprecated syntax or feature */
    USING_DEPRECATED_FEATURE(AlarmSystem.BARELY, 0x0002,
        "Using deprecated syntax or feature"),

    // === MILD (Level 3) ===
    // Moderate warnings, likely programming errors

    /** Detaching identifier not referenced in subsequent brane */
    DETACHING_UNRESOLVED_ORDINATE(AlarmSystem.MILD, 0x0101,
        "Detaching identifier not referenced in subsequent brane"),

    /** Detaching identifier without SF mark */
    DETACHING_WITHOUT_SF_MARK(AlarmSystem.MILD, 0x0102,
        "Detaching identifier without SF mark"),

    /** Shadowing variable from outer scope */
    SHADOWING_OUTER_VARIABLE(AlarmSystem.MILD, 0x0103,
        "Shadowing variable from outer scope"),

    /** Reassigning immutable binding */
    REASSIGNING_IMMUTABLE_BINDING(AlarmSystem.MILD, 0x0104,
        "Reassigning immutable binding"),

    /** Missing identifier in current context */
    MISSING_IDENTIFIER(AlarmSystem.MILD, 0x0105,
        "Missing identifier in current context"),

    // === HAIR_RAISING (Level 5) ===
    // Significant warnings, semantic violations

    /** Violating constraint C1: modifying FIR after CONSTANIC */
    VIOLATING_IMMUTABILITY_CONSTRAINT(AlarmSystem.HAIR_RAISING, 0x0201,
        "Violating immutability constraint (modifying FIR after CONSTANIC)"),

    /** Violating constraint C5: non-empty braneMind at CONSTANIC */
    VIOLATING_PRIMED_CONSTRAINT(AlarmSystem.HAIR_RAISING, 0x0202,
        "Violating PRIMED constraint (non-empty braneMind at CONSTANIC)"),

    /** Exceeding maximum brane depth */
    EXCEEDING_BRANE_DEPTH(AlarmSystem.HAIR_RAISING, 0x0203,
        "Exceeding maximum brane depth limit"),

    /** Circular parent chain detected */
    DETECTING_CIRCULAR_PARENT_CHAIN(AlarmSystem.HAIR_RAISING, 0x0204,
        "Detecting circular parent chain"),

    /** Invalid state transition */
    TRANSITIONING_INVALID_STATE(AlarmSystem.HAIR_RAISING, 0x0205,
        "Transitioning to invalid state"),

    // === PANIC (Level 10) ===
    // Critical errors, system integrity failures

    /** Dividing by zero */
    DIVIDING_BY_ZERO(AlarmSystem.PANIC, 0x0301,
        "Dividing by zero"),

    /** Encountering null where value expected */
    ENCOUNTERING_NULL_VALUE(AlarmSystem.PANIC, 0x0302,
        "Encountering null value where non-null expected"),

    /** Corrupting internal state */
    CORRUPTING_INTERNAL_STATE(AlarmSystem.PANIC, 0x0303,
        "Corrupting internal data structure state"),

    /** Failing type assertion */
    FAILING_TYPE_ASSERTION(AlarmSystem.PANIC, 0x0304,
        "Failing critical type assertion"),

    /** Reaching unreachable code path */
    REACHING_UNREACHABLE_CODE(AlarmSystem.PANIC, 0x0305,
        "Reaching unreachable code path");

    private final int severity;
    private final int code;
    private final String genericMessage;

    AlarmCode(int severity, int code, String genericMessage) {
        this.severity = severity;
        this.code = code;
        this.genericMessage = genericMessage;
    }

    /**
     * Gets the severity level for this alarm.
     * @return severity level (BARELY, MILD, HAIR_RAISING, or PANIC)
     */
    public int getSeverity() {
        return severity;
    }

    /**
     * Gets the unique hexadecimal code for this alarm within its severity level.
     * @return code as integer (e.g., 0x0101)
     */
    public int getCode() {
        return code;
    }

    /**
     * Gets the generic message describing this alarm condition.
     * Uses gerund form: "Detaching...", "Violating...", "Missing..."
     * @return generic message
     */
    public String getGenericMessage() {
        return genericMessage;
    }

    /**
     * Formats the alarm code as a severity letter + hex code.
     * Examples: "A0301" (BARELY 0x0001), "M0101" (MILD 0x0101), "H0201" (HAIR_RAISING 0x0201), "P0301" (PANIC 0x0301)
     *
     * @return formatted code string
     */
    public String getFormattedCode() {
        char severityLetter;
        if (severity == AlarmSystem.BARELY) {
            severityLetter = 'A';
        } else if (severity == AlarmSystem.MILD) {
            severityLetter = 'M';
        } else if (severity == AlarmSystem.HAIR_RAISING) {
            severityLetter = 'H';
        } else if (severity == AlarmSystem.PANIC) {
            severityLetter = 'P';
        } else {
            severityLetter = '?';
        }

        return String.format("%c%04X", severityLetter, code);
    }

    @Override
    public String toString() {
        return getFormattedCode() + ": " + genericMessage;
    }
}
