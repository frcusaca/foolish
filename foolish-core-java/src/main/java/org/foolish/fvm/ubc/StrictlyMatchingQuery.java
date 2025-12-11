package org.foolish.fvm.ubc;

public final class StrictlyMatchingQuery extends CharacterizedIdentifier implements Query {
    public StrictlyMatchingQuery(String name, String characterization) {
        super(name, characterization);
    }

    public StrictlyMatchingQuery(CharacterizedIdentifier ci) {
        super(ci.getId(), ci.getCharacterization());
    }

    public boolean matches(FIR brane_line) {
        return switch (brane_line) {
            case AssignmentFiroe ass -> {
                CharacterizedIdentifier lhs = ass.getLhs();
                yield lhs.equals(this);
            }
            default ->
                // mostly here is unamed lines in brane.
                    false;
        };
    }
}
