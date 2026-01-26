package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * DerefSearchFiroe is a specialized RegexpSearchFiroe for exact match searches.
 * It is used when the search pattern contains no wildcards (and thus is an exact match)
 * or when explicitly created from a DereferenceExpr.
 */
public class DerefSearchFiroe extends RegexpSearchFiroe {
    private final AST originalAst;

    public DerefSearchFiroe(AST.RegexpSearchExpr regexpSearch) {
        super(regexpSearch);
        this.originalAst = null;
    }

    public DerefSearchFiroe(AST.RegexpSearchExpr syntheticExpr, AST.DereferenceExpr originalExpr) {
        super(syntheticExpr);
        this.originalAst = originalExpr;
    }

    @Override
    public String toString() {
        if (originalAst != null) {
            return originalAst.toString();
        }
        return super.toString();
    }

    /**
     * Checks if the pattern contains any regex wildcards or special characters.
     * If false, the pattern is considered an exact match string.
     */
    public static boolean isExactMatch(String pattern) {
        if (pattern == null) return true;
        for (char c : pattern.toCharArray()) {
            // Check for standard regex metacharacters
            if ("*+?^$[](){} |\\.".indexOf(c) >= 0) {
                return false;
            }
        }
        return true;
    }
}
