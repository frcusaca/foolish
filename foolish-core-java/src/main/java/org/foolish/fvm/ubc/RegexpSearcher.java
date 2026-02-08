package org.foolish.fvm.ubc;

import org.apache.commons.lang3.tuple.Pair;
import java.util.Optional;

/**
 * Utility for performing regular expression searches on a cursor.
 * Enforces convention: patterns without anchors (^ or $) are wrapped in ^...$ for full line matching.
 */
public class RegexpSearcher {
    private final String rawPattern;
    private final String effectivePattern;

    public RegexpSearcher(String pattern) {
        this.rawPattern = pattern;
        this.effectivePattern = normalizePattern(pattern);
    }

    private String normalizePattern(String pattern) {
        if (pattern == null) return null;
        if (pattern.isEmpty()) return pattern; // Or ^$ ?
        
        boolean hasStart = pattern.startsWith("^");
        boolean hasEnd = pattern.endsWith("$");
        
        if (!hasStart && !hasEnd) {
            return "^" + pattern + "$";
        }
        return pattern;
    }

    public static boolean isExactMatch(String pattern) {
        if (pattern == null) return true;
        for (char c : pattern.toCharArray()) {
            if ("*+?^$[](){} |~\\.".indexOf(c) >= 0) {
                return false;
            }
        }
        return true;
    }

    public Optional<FIR> search(SearchCursor cursor) {
        Query.RegexpQuery query = new Query.RegexpQuery(effectivePattern);
        
        return cursor.streamCandidates()
            .filter(pair -> {
                FIR fir = pair.getRight();
                return query.matches(fir);
            })
            .findFirst()
            .map(Pair::getValue);
    }

    public String getPattern() {
        return effectivePattern;
    }

    public String getRawPattern() {
        return rawPattern;
    }
}
