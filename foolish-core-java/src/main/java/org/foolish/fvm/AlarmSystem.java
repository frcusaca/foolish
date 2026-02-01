package org.foolish.fvm;

import org.foolish.fvm.ubc.*;
import org.apache.commons.lang3.tuple.Pair;

import java.util.Optional;

/**
 * Alarm system for reporting diagnostic messages with configurable severity thresholds.
 * <p>
 * Alarms are displayed to STDERR if their severity meets or exceeds the current threshold.
 * The threshold can be set via the {@code alarming_level} variable in the brane context.
 * <p>
 * <b>Severity Levels:</b>
 * <ul>
 *   <li>{@link #NOT} (0) - Not an alarm, always suppressed</li>
 *   <li>{@link #BARELY} (1) - Minor informational message</li>
 *   <li>{@link #MILD} (3) - Moderate warning</li>
 *   <li>{@link #HAIR_RAISING} (5) - Significant warning</li>
 *   <li>{@link #PANIC} (10) - Critical error</li>
 * </ul>
 * <p>
 * <b>Output Format:</b>
 * <pre>
 * [ALARM {severity}] Index:{foolishIndex} {location} - {message}
 * </pre>
 * Example:
 * <pre>
 * [ALARM 5] Index:[0,1,2] test.foo:line 15 Brane@10 - Division by zero
 * </pre>
 */
public class AlarmSystem {
    public static final int NOT = 0;
    public static final int BARELY = 1;
    public static final int MILD = 3;
    public static final int HAIR_RAISING = 5;
    public static final int PANIC = 10;

    private static final String VAR_ALARMING_LEVEL = "alarming_level";

    /**
     * Raises an alarm from an AlarmCode enum with optional context details.
     * <p>
     * This is the preferred method for raising alarms as it ensures consistent
     * formatting and standardized error codes.
     * <p>
     * Output format:
     * <pre>
     * [ALARM {severity}] {code} Index:{index} {location} - {genericMessage} ({details})
     * </pre>
     * Example:
     * <pre>
     * [ALARM 3] M0101 Index:[0#1#2] test.foo:line 15 - Detaching identifier not referenced in subsequent brane (symbol 'a')
     * </pre>
     *
     * @param fir     The FIR where the alarm originated, or null if not available.
     * @param alarm   The AlarmCode describing the error condition.
     * @param details Optional specific details to append (e.g., "symbol 'a'"), or null.
     */
    public static void raise(FIR fir, AlarmCode alarm, String details) {
        String indexInfo = "";
        String locationInfo = "";

        if (fir != null) {
            // Get FoolishIndex
            FoolishIndex index = fir.getMyIndex();
            if (index != null) {
                indexInfo = "Index:" + index.toString() + " ";
            }

            // Get location description
            locationInfo = fir.getLocationDescription() + " - ";
        }

        // Build message with code
        String message = alarm.getFormattedCode() + " " + indexInfo + locationInfo + alarm.getGenericMessage();
        if (details != null && !details.isEmpty()) {
            message += " (" + details + ")";
        }

        // Get braneMemory for threshold lookup
        BraneMemory context = null;
        if (fir != null) {
            BraneFiroe brane = fir.getMyBrane();
            if (brane != null) {
                context = brane.getBraneMemory();
            }
        }

        int threshold = NOT;
        if (context != null) {
            threshold = resolveAlarmingLevel(context);
        }

        if (alarm.getSeverity() >= threshold) {
            System.err.println("[ALARM " + alarm.getSeverity() + "] " + message);
        }
    }

    /**
     * Raises an alarm from an AlarmCode enum with BraneMemory context.
     *
     * @param context The BraneMemory context, or null if not available.
     * @param alarm   The AlarmCode describing the error condition.
     * @param details Optional specific details to append (e.g., "symbol 'a'"), or null.
     */
    public static void raise(BraneMemory context, AlarmCode alarm, String details) {
        String indexInfo = "";
        if (context != null) {
            FiroeWithBraneMind owner = context.getOwningBrane();
            if (owner != null) {
                FoolishIndex index = owner.getMyIndex();
                if (index != null) {
                    indexInfo = "Index:" + index.toString() + " ";
                }
            }
        }

        String message = alarm.getFormattedCode() + " " + indexInfo + alarm.getGenericMessage();
        if (details != null && !details.isEmpty()) {
            message += " (" + details + ")";
        }

        int threshold = NOT;
        if (context != null) {
            threshold = resolveAlarmingLevel(context);
        }

        if (alarm.getSeverity() >= threshold) {
            System.err.println("[ALARM " + alarm.getSeverity() + "] " + message);
        }
    }

    /**
     * Raises an alarm with FIR context for precise location reporting.
     * Includes FoolishIndex, source location, and brane context.
     * <p>
     * This method provides richer context than {@link #raise(BraneMemory, String, int)}
     * by including the FoolishIndex and location description from the FIR.
     *
     * @param fir      The FIR where the alarm originated (must not be null).
     * @param message  The message to display.
     * @param severity The severity of the alarm.
     */
    public static void raiseFromFir(FIR fir, String message, int severity) {
        String indexInfo = "";
        String locationInfo = "";

        // Get FoolishIndex
        FoolishIndex index = fir.getMyIndex();
        if (index != null) {
            indexInfo = "Index:" + index.toString() + " ";
        }

        // Get location description
        locationInfo = fir.getLocationDescription() + " - ";

        // Get braneMemory for threshold lookup via containing brane
        BraneMemory context = null;
        BraneFiroe brane = fir.getMyBrane();
        if (brane != null) {
            context = brane.getBraneMemory();
        }

        int threshold = NOT;
        if (context != null) {
            threshold = resolveAlarmingLevel(context);
        }

        if (severity >= threshold) {
            System.err.println("[ALARM " + severity + "] " + indexInfo + locationInfo + message);
        }
    }

    /**
     * Raises an alarm with the given message and severity.
     * The alarm is displayed if the severity is greater than or equal to the current alarming_level.
     *
     * @param context  The current BraneMemory context, or null if not available (e.g. parsing phase).
     * @param message  The message to display.
     * @param severity The severity of the alarm.
     */
    public static void raise(BraneMemory context, String message, int severity) {
        int threshold = NOT;

        if (context != null) {
            threshold = resolveAlarmingLevel(context);
        }

        if (severity >= threshold) {
            // Try to get index from owning brane
            String indexInfo = "";
            if (context != null) {
                FiroeWithBraneMind owner = context.getOwningBrane();
                if (owner != null) {
                    FoolishIndex index = owner.getMyIndex();
                    if (index != null) {
                        indexInfo = "Index:" + index.toString() + " ";
                    }
                }
            }
            System.err.println("[ALARM " + severity + "] " + indexInfo + message);
        }
    }

    private static int resolveAlarmingLevel(BraneMemory context) {
        Query query = new Query.StrictlyMatchingQuery(VAR_ALARMING_LEVEL, "");

        Optional<Pair<Integer, FIR>> result = context.get(query, context.size());

        if (result.isPresent()) {
            FIR fir = result.get().getValue();
            FIR resolvedValue = null;

            if (fir instanceof AssignmentFiroe assignment) {
                // Get the result of the assignment
                resolvedValue = assignment.getResult();
            } else if (fir instanceof ValueFiroe) {
                // The value might be stored directly (e.g. if resolved previously or optimization)
                resolvedValue = fir;
            }

            if (resolvedValue != null && resolvedValue instanceof ValueFiroe vf) {
                try {
                    return (int) vf.getValue();
                } catch (Exception e) {
                    // Fallback
                }
            }
        }

        return NOT; // Default if not found
    }
}
