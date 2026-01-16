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
     * Checks if this query blocks another query.
     * This is used for detachment/blocking logic.
     */
    default boolean blocks(Query other) {
        // By default, assume no blocking unless implemented
        // Since we don't have a universal way to check query overlap yet.
        // We will implement specific logic in subclasses.
        return false;
    }

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
        public boolean blocks(Query other) {
            if (other instanceof StrictlyMatchingQuery strict) {
                return this.equals(strict);
            }
            // A specific identifier blocks a regex if the identifier matches the regex? No.
            // A blocker (this) blocks a query (other) if anything 'other' matches is also matched by 'this'.
            // For strict query, it only blocks if 'other' is also strict and equal.
            // If 'other' is a regex, we can't easily know if 'other' ONLY matches this.
            // So we conservatively return false for regex.
            return false;
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
        public boolean blocks(Query other) {
            // A Regex blocker blocks 'other' if 'other' falls within the regex.
            if (other instanceof StrictlyMatchingQuery strict) {
                // Check if the strict identifier matches this regex pattern
                String fullName = strict.toString();
                String nameOnly = strict.getId();
                return pattern.matcher(fullName).find() || pattern.matcher(nameOnly).find();
            } else if (other instanceof RegexpQuery regex) {
                // Hard to determine if one regex blocks another completely.
                // For now, if patterns are identical, we say yes.
                return this.originalPattern.equals(regex.originalPattern);
            }
            return false;
        }

        @Override
        public String toString() {
            return "RegexpQuery[" + originalPattern + " -> " + pattern.pattern() + "]";
        }
    }
}
