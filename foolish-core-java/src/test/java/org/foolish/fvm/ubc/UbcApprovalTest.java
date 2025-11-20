package org.foolish.fvm.ubc;

import org.approvaltests.Approvals;
import org.approvaltests.integrations.junit5.JupiterApprovals;
import org.approvaltests.namer.ApprovalNamer;
import org.approvaltests.writers.ApprovalTextWriter;
import org.foolish.ApprovalTestRunner;
import org.foolish.UbcTester;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;
import org.junit.jupiter.api.function.Executable;

import java.io.File;
import java.nio.file.Files;
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
            return JupiterApprovals.dynamicTest(
                testName,
                options -> {
                    try {
                        // Read input code
                        String code = Files.readString(file.toPath()).trim();

                        // Execute tester - produces complete .approved.foo content
                        JavaUbcInterpreter interpreter = new JavaUbcInterpreter();
                        String approvalContent = interpreter.execute(code);

                        // Create custom namer for output location
                        ApprovalNamer namer = new CustomUbcApprovalNamer("org.foolish.fvm.ubc", testName);

                        // Verify with ApprovalTests using namer
                        Approvals.verify(new ApprovalTextWriter(approvalContent, "foo"), namer, options);
                    } catch (Exception e) {
                        throw new RuntimeException("Failed to run approval test for " + testName, e);
                    }
                }
            );
        });
    }

    /**
     * Custom ApprovalNamer that places files in test resources by package.
     */
    private static class CustomUbcApprovalNamer extends org.approvaltests.namer.StackTraceNamer {
        private final String packageName;
        private final String testName;

        public CustomUbcApprovalNamer(String packageName, String testName) {
            this.packageName = packageName;
            this.testName = testName;
        }

        @Override
        public String getApprovalName() {
            return testName;
        }

        @Override
        public String getSourceFilePath() {
            String packagePath = packageName.replace('.', File.separatorChar);
            return "src" + File.separator + "test" + File.separator + "resources" +
                   File.separator + packagePath;
        }
    }
}
