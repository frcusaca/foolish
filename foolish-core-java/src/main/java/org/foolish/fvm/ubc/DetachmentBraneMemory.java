package org.foolish.fvm.ubc;

import org.apache.commons.lang3.tuple.Pair;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

public class DetachmentBraneMemory extends BraneMemory {
    private final List<Rule> rules = new ArrayList<>();

    public DetachmentBraneMemory(BraneMemory parent) {
        super(parent);
    }

    public void addRule(Query pattern, boolean isAllow) {
        rules.add(new Rule(pattern, isAllow));
    }

    @Override
    public Optional<Pair<Integer, FIR>> get(Query query, int fromLine) {
        // First, check local memory (definitions)
        Optional<Pair<Integer, FIR>> local = getLocal(query, fromLine);
        if (local.isPresent()) {
            return local;
        }

        // If not local, look in parent (if any)
        if (getParent() != null) {
            Optional<Pair<Integer, FIR>> result = super.get(query, fromLine);
            if (result.isPresent()) {
                FIR candidate = result.get().getRight();
                // Since local was not found, this result is from parent.
                // Apply filter rules.
                if (isBlocked(candidate)) {
                    return Optional.empty();
                }
                return result;
            }
        }

        return Optional.empty();
    }

    private boolean isBlocked(FIR candidate) {
         for (Rule rule : rules) {
             if (rule.matches(candidate)) {
                 return !rule.isAllow(); // If allow, not blocked. If block, blocked.
             }
         }
         return false; // Default Allow
    }

    public record Rule(Query pattern, boolean isAllow) {
        public boolean matches(FIR candidate) {
             return pattern.matches(candidate);
        }
    }
}
