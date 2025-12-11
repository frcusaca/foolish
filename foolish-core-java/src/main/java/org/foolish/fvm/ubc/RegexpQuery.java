package org.foolish.fvm.ubc;

import java.util.regex.Pattern;

public final class RegexpQuery implements Query {
    private final Pattern pattern;

    public RegexpQuery(String regex) {
        String finalRegex = regex;
        if (!regex.contains("^") && !regex.contains("$")) {
            finalRegex = "^" + regex + "$";
        }
        this.pattern = Pattern.compile(finalRegex);
    }

    @Override
    public boolean matches(FIR brane_line) {
        if (brane_line instanceof AssignmentFiroe ass) {
             String target = ass.getLhs().getCharacterization() + ass.getLhs().getId();
             return pattern.matcher(target).find();
        }
        return false;
    }
}
