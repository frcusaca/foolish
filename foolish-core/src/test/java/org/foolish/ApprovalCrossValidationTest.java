package org.foolish;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer;
import org.junit.jupiter.api.Order;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Cross-validation test that verifies Java and Scala UBC implementations
 * produce identical output for all test cases.
 *
 * This test runs during 'mvn test' to ensure both implementations
 * produce identical output.
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
public class ApprovalCrossValidationTest {

    private static final String JAVA_APPROVAL_PATH = "src/test/resources/org/foolish/fvm/ubc";
    private static final String SCALA_APPROVAL_PATH = "src/test/resources/org/foolish/fvm/scubc";

    @Test
    @Order(Integer.MAX_VALUE)  // Run this test last
    public void javaAndScalaApprovalsShouldMatch() throws IOException {
        List<String> differences = new ArrayList<>();
        int matchCount = 0;
        int mismatchCount = 0;

        // Find all Java approval files
        List<File> javaApprovals = findApprovalFiles(JAVA_APPROVAL_PATH);

        for (File javaFile : javaApprovals) {
            String testName = javaFile.getName();

            // Find corresponding Scala approval file
            File scalaFile = new File(SCALA_APPROVAL_PATH, testName);

            if (!scalaFile.exists()) {
                differences.add("Missing Scala approval for: " + testName);
                mismatchCount++;
                continue;
            }

            // Compare file contents
            String javaContent = Files.readString(javaFile.toPath());
            String scalaContent = Files.readString(scalaFile.toPath());

            if (javaContent.equals(scalaContent)) {
                matchCount++;
            } else {
                differences.add("Mismatch in " + testName);
                mismatchCount++;

                // Optionally show diff details (uncomment for debugging)
                // System.err.println("\n=== Difference in " + testName + " ===");
                // System.err.println("Java length: " + javaContent.length());
                // System.err.println("Scala length: " + scalaContent.length());
            }
        }

        // Report results
        System.out.println("\n=== Approval Cross-Validation Results ===");
        System.out.println("Matches: " + matchCount);
        System.out.println("Mismatches: " + mismatchCount);

        if (!differences.isEmpty()) {
            System.err.println("\nDifferences found:");
            differences.forEach(System.err::println);
        }

        // Fail if any mismatches
        assertEquals(0, mismatchCount,
            "Java and Scala approval files should be identical. Found " + mismatchCount + " mismatches.");
    }

    private List<File> findApprovalFiles(String path) throws IOException {
        Path dir = Path.of(path);
        if (!Files.exists(dir)) {
            return List.of();
        }

        return Files.list(dir)
            .filter(p -> p.getFileName().toString().endsWith(".approved.foo"))
            .map(Path::toFile)
            .sorted()
            .collect(Collectors.toList());
    }
}
