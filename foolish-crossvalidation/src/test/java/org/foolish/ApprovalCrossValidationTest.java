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
import java.util.Set;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Cross-validation test that verifies Java and Scala UBC implementations
 * produce identical output for all test cases.
 *
 * This test runs as a separate module after both Java and Scala tests complete
 * to ensure both implementations produce identical output.
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
public class ApprovalCrossValidationTest {

    protected String getJavaApprovalPath() {
        return "../foolish-core-java/src/test/resources/org/foolish/fvm/ubc";
    }

    protected String getScalaApprovalPath() {
        return "../foolish-core-scala/src/test/resources/org/foolish/fvm/scubc";
    }

    /**
     * Tests that are known to differ between Java and Scala implementations.
     * These are skipped from cross-validation until the Scala implementation catches up.
     */
    private static final Set<String> KNOWN_DIFFERENCES = Set.of(
        "concatenationBasics.approved.foo"  // Scala doesn't have ConcatenationFiroe yet
    );

    @Test
    @Order(Integer.MAX_VALUE)  // Run this test last
    public void javaAndScalaApprovalsShouldMatch() throws IOException {
        List<String> differences = new ArrayList<>();
        int matchCount = 0;
        int mismatchCount = 0;

        // Find all Java approval files
        List<File> javaApprovals = findApprovalFiles(getJavaApprovalPath());

        for (File javaFile : javaApprovals) {
            String testName = javaFile.getName();

            // Skip known differences (features not yet implemented in Scala)
            if (KNOWN_DIFFERENCES.contains(testName)) {
                System.out.println("Skipping known difference: " + testName);
                continue;
            }

            // Find corresponding Scala approval file
            File scalaFile = new File(getScalaApprovalPath(), testName);

            if (!scalaFile.exists()) {
                differences.add("Missing Scala approval for: " + testName);
                mismatchCount++;
                continue;
            }

            // Compare file contents
            String javaContent = normalize(Files.readString(javaFile.toPath()));
            String scalaContent = normalize(Files.readString(scalaFile.toPath()));

            if (javaContent.equals(scalaContent)) {
                matchCount++;
            } else {
                differences.add("Mismatch in " + testName);
                mismatchCount++;
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

    private String normalize(String content) {
        return content.lines()
                .filter(line -> !line.trim().startsWith("Steps taken:"))
                .collect(Collectors.joining("\n"));
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
