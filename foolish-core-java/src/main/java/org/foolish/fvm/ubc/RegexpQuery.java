package org.foolish.fvm.ubc;

import java.util.regex.Pattern;

public class RegexpQuery implements Query {
    private final Pattern pattern;

    public RegexpQuery(String regex) {
        this.pattern = Pattern.compile(regex);
    }

    @Override
    public boolean matches(FIR brane_line) {
        if (brane_line instanceof AssignmentFiroe ass) {
             return pattern.matcher(ass.getId()).matches();
        }
        return false;
    }
}
