package org.foolish;

import org.approvaltests.Approvals;
import org.approvaltests.core.Options;
import org.approvaltests.namer.ApprovalNamer;
import org.approvaltests.namer.NamerWrapper;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.stream.Collectors;

/**
 * Shared approval testing framework for UBC implementations.
 *
 * This utility:
 * 1. Discovers .foo input files in a shared directory (single source of truth)
 * 2. Runs a UBC tester on each input
 * 3. Generates .approved.foo files (complete Foolish programs with test results)
 * 4. Uses ApprovalTests framework for verification
 *
 * Input file: org/foolish/fvm/inputs/simpleInteger.foo
 * {5;}
 *
 * Output file format: org/foolish/fvm/ubc/simpleInteger.approved.foo
 * !!INPUT!!
 * {5;}
 *
 * !!!
 * PARSED AST:
 * ...
 *
 * UBC EVALUATION:
 * Steps taken: 1
 *
 * FINAL RESULT:
 * ...
 *
 * COMPLETION STATUS:
 * Complete: true
 * !!!
 */
public class ApprovalTestRunner {

    private final UbcTester interpreter;
    private final String inputPath;
    private final String outputPath;

    /**
     * Creates a new approval test runner.
     *
     * @param interpreter The UBC tester implementation to use
     * @param inputPath Path to .foo input files (e.g., "org/foolish/fvm/inputs")
     * @param outputPath Path for .approved.foo output files (e.g., "org/foolish/fvm/ubc")
     */
    public ApprovalTestRunner(UbcTester interpreter, String inputPath, String outputPath) {
        this.interpreter = interpreter;
        this.inputPath = inputPath;
        this.outputPath = outputPath;
    }

    /**
     * Finds all .foo input files in the shared input directory.
     */
    public List<File> findInputFiles() {
        return findInputFiles(inputPath);
    }

    /**
     * Static helper to find all .foo input files in the shared input directory.
     */
    public static List<File> findInputFiles(String inputPath) {
        try {
            Path dir = Path.of("src/test/resources", inputPath);
            if (!Files.exists(dir)) {
                Files.createDirectories(dir);
                return List.of();
            }
            return Files.list(dir)
                .filter(p -> p.getFileName().toString().endsWith(".foo"))
                .map(Path::toFile)
                .sorted()
                .collect(Collectors.toList());
        } catch (IOException e) {
            throw new RuntimeException("Failed to list input test files", e);
        }
    }

    /**
     * Runs approval test for a single .foo input file.
     *
     * @param inputFile The .foo input file
     * @param testName The test name for the approval file
     */
    public void runApprovalTest(File inputFile, String testName) {
        try {
            // Read input code
            String code = Files.readString(inputFile.toPath()).trim();

            // Execute tester - produces complete .approved.foo content
            // Pass the filename for error reporting if the interpreter supports it
            String approvalContent;
            if (interpreter instanceof org.foolish.fvm.ubc.JavaUbcInterpreter javaInterpreter) {
                approvalContent = javaInterpreter.execute(code, inputFile.getName());
            } else {
                approvalContent = interpreter.execute(code);
            }

            // Create custom namer for output location
            String packageName = outputPath.replace('/', '.');
            String packagePath = packageName.replace('.', File.separatorChar);
            String sourceFilePath = "src" + File.separator + "test" + File.separator + "resources" +
                   File.separator + packagePath + File.separator;

            // Verify with ApprovalTests using Options and NamerWrapper
            // Note: chaining forFile() is necessary because withExtension returns Options, not FileOptions
            Approvals.verify(approvalContent, new Options()
                .forFile().withExtension(".foo")
                .forFile().withNamer(new NamerWrapper(
                    () -> testName,
                    () -> sourceFilePath
                )));

        } catch (IOException e) {
            throw new RuntimeException("Failed to read input file: " + inputFile, e);
        }
    }
}
