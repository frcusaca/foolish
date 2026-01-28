package org.foolish.fvm.ubc;

import org.foolish.ApprovalTestRunner;
import org.junit.jupiter.api.parallel.Execution;
import org.junit.jupiter.api.parallel.ExecutionMode;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;

import java.io.File;
import java.util.stream.Stream;

/**
 * Approval tests for the Java UBC implementation.
 *
 * Reads .foo input files, executes them with Java UBC, and generates
 * approval files in org/foolish/fvm/ubc/ for verification.
 */
@Execution(ExecutionMode.CONCURRENT)
public class UbcApprovalTest {

    private final ApprovalTestRunner runner = new ApprovalTestRunner(
        new JavaUbcInterpreter(),
        "org/foolish/fvm/inputs",  // Shared input directory
        "org/foolish/fvm/ubc"      // Java approval output directory
    );

    static Stream<Arguments> provideInputFiles() {
        String filter = System.getProperty("foolish.test.filter");

        Stream<File> fileStream = ApprovalTestRunner.findInputFiles("org/foolish/fvm/inputs").stream();

        if (filter != null && !filter.isBlank()) {
            fileStream = fileStream.filter(f -> f.getName().contains(filter));
        }

        return fileStream.map(file -> Arguments.of(file, file.getName().replace(".foo", "")));
    }

    @ParameterizedTest(name = "{index}: {1}")
    @MethodSource("provideInputFiles")
    void approvalTests(File inputFile, String testName) {
        runner.runApprovalTest(inputFile, testName);
    }
}
