package org.foolish;

/**
 * Interface for UBC testers (Java or Scala implementations).
 *
 * A tester takes Foolish code as input and produces a complete .approved.foo
 * file with the input code followed by a block comment containing test results.
 */
public interface UbcTester {

    /**
     * Executes the given Foolish code and returns a complete .foo file.
     *
     * @param code The Foolish source code to execute
     * @return Complete .foo file content with !!INPUT!! marker, code, and
     *         !!! block comment containing PARSED AST, UBC EVALUATION,
     *         FINAL RESULT, COMPLETION STATUS, and optionally FINAL ENVIRONMENT
     */
    String execute(String code);

    /**
     * Returns the name of this tester implementation (e.g., "Java UBC", "Scala UBC").
     */
    String getName();
}
