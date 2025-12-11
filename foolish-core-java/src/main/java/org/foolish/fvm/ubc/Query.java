package org.foolish.fvm.ubc;

import java.util.regex.Pattern;

/**
 * Query represents a search pattern for finding lines in BraneMemory.
 * This is a sealed interface with different query types:
 * - StrictlyMatchingQuery: exact identifier match
 * - RegexpQuery: regular expression pattern match
 */
public sealed interface Query permits Query.StrictlyMatchingQuery, Query.RegexpQuery {
    /**
     * Checks if the given FIR matches this query.
     */
    boolean matches(FIR brane_line);

    /**
     * StrictlyMatchingQuery matches identifiers exactly by name and characterization.
     */
    final class StrictlyMatchingQuery extends CharacterizedIdentifier implements Query {
        public StrictlyMatchingQuery(String name, String characterization) {
            super(name, characterization);
        }

        @Override
        public boolean matches(FIR brane_line) {
            return switch (brane_line) {
                case AssignmentFiroe ass -> {
                    CharacterizedIdentifier lhs = ass.getLhs();
                    yield lhs.equals(this);
                }
                default ->
                    // mostly here is unnamed lines in brane.
                        false;
            };
        }

        @Override
        public String toString() {
            return "StrictQuery[" + super.toString() + "]";
        }
    }

    /**
     * RegexpQuery matches identifiers using a regular expression pattern.
     * If the pattern doesn't start with ^ or end with $, they are added
     * to make it a "whole identifier" match (similar to StrictlyMatchingQuery).
     * If the pattern contains anchors, it uses partial matching within the identifier.
     */
    final class RegexpQuery implements Query {
        private final Pattern pattern;
        private final String originalPattern;
        private final boolean isAnchored;

        public RegexpQuery(String regexPattern) {
            this.originalPattern = regexPattern;
            // Check if pattern has anchoring
            boolean startsWithCaret = regexPattern.startsWith("^");
            boolean endsWithDollar = regexPattern.endsWith("$");
            this.isAnchored = startsWithCaret || endsWithDollar;

            // If no anchoring, add both ^ and $ to make it a whole match
            String finalPattern = regexPattern;
            if (!startsWithCaret && !endsWithDollar) {
                finalPattern = "^" + regexPattern + "$";
            }

            this.pattern = Pattern.compile(finalPattern);
        }

        @Override
        public boolean matches(FIR brane_line) {
            return switch (brane_line) {
                case AssignmentFiroe ass -> {
                    CharacterizedIdentifier lhs = ass.getLhs();
                    String fullName = lhs.toString(); // Includes characterization
                    String nameOnly = lhs.getId();

                    // Try matching against both the full name and name only
                    yield pattern.matcher(fullName).find() || pattern.matcher(nameOnly).find();
                }
                default -> false;
            };
        }

        public Pattern getPattern() {
            return pattern;
        }

        public String getOriginalPattern() {
            return originalPattern;
        }

        public boolean isAnchored() {
            return isAnchored;
        }

        @Override
        public String toString() {
            return "RegexpQuery[" + originalPattern + " -> " + pattern.pattern() + "]";
        }
    }
}
