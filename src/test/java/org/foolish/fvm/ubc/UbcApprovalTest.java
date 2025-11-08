package org.foolish.fvm.ubc;

import org.foolish.ApprovalTestRunner;
import org.foolish.UbcTester;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

import java.io.File;
import java.util.List;
import java.util.stream.Stream;

/**
 * Approval tests for the Java UBC implementation.
 *
 * Reads .foo input files, executes them with Java UBC, and generates
 * approval files in org/foolish/fvm/ubc/ for verification.
 */
public class UbcApprovalTest {

    private final ApprovalTestRunner runner = new ApprovalTestRunner(
        new JavaUbcInterpreter(),
        "org/foolish/fvm/inputs",  // Shared input directory
        "org/foolish/fvm/ubc"      // Java approval output directory
    );

    @TestFactory
    Stream<DynamicTest> approvalTests() {
        List<File> inputFiles = runner.findInputFiles();

        return inputFiles.stream().map(file -> {
            String testName = file.getName().replace(".foo", "");
            return DynamicTest.dynamicTest(testName, () -> {
                try {
                    runner.runApprovalTest(file, testName);
                } catch (AssertionError e) {
                    // Re-throw with more descriptive message including test name
                    throw new AssertionError("Test '" + testName + "' failed: " + e.getMessage(), e);
                }
            });
        });
    }
}
