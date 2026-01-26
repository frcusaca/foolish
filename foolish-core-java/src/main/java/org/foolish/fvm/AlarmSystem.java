package org.foolish.fvm;

import org.foolish.fvm.ubc.*;
import org.apache.commons.lang3.tuple.Pair;

import java.util.Optional;

public class AlarmSystem {
    public static final int NOT = 0;
    public static final int BARELY = 1;
    public static final int MILD = 3;
    public static final int HAIR_RAISING = 5;
    public static final int PANIC = 10;

    private static final String VAR_ALARMING_LEVEL = "alarming_level";

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
            System.err.println("[ALARM " + severity + "] " + message);
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
