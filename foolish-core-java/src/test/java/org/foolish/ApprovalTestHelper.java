package org.foolish;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.stream.Collectors;

/**
 * Helper utilities for approval testing with .foo files.
 *
 * The .foo files are valid Foolish programs with expected output in block comments:
 * - Everything before "!!" is the input Foolish code
 * - Everything between "!! ... !!" is the expected output
 */
public class ApprovalTestHelper {

    public static class TestCase {
        public final String name;
        public final String input;
        public final String expectedOutput;

        public TestCase(String name, String input, String expectedOutput) {
            this.name = name;
            this.input = input;
            this.expectedOutput = expectedOutput;
        }
    }

    /**
     * Parses a .foo test file into input code and expected output.
     *
     * @param file the .foo file to parse
     * @return TestCase containing the input and expected output
     */
    public static TestCase parseTestFile(File file) throws IOException {
        String content = Files.readString(file.toPath());

        // Extract name from filename (e.g., "simpleIntegerIsApproved.approved.foo" -> "simpleIntegerIsApproved")
        String name = file.getName().replace(".approved.foo", "");

        // Find the block comment delimiter
        int blockStart = content.indexOf("!!");
        if (blockStart == -1) {
            throw new IllegalArgumentException("Test file must contain !! block comment: " + file.getName());
        }

        // Input is everything before the first !!
        String input = content.substring(0, blockStart).trim();

        // Expected output is between !! and !!
        int blockEnd = content.indexOf("!!", blockStart + 2);
        if (blockEnd == -1) {
            throw new IllegalArgumentException("Test file must have closing !! delimiter: " + file.getName());
        }

        String expectedOutput = content.substring(blockStart + 2, blockEnd).trim();

        return new TestCase(name, input, expectedOutput);
    }

    /**
     * Finds all .approved.foo files in the given directory.
     *
     * @param resourcePath the package path (e.g., "org/foolish/fvm/ubc")
     * @return list of .foo test files
     */
    public static List<File> findApprovalFiles(String resourcePath) {
        try {
            Path dir = Path.of("src/test/resources", resourcePath);
            return Files.list(dir)
                .filter(p -> p.getFileName().toString().endsWith(".approved.foo"))
                .map(Path::toFile)
                .sorted()
                .collect(Collectors.toList());
        } catch (IOException e) {
            throw new RuntimeException("Failed to list approval test files", e);
        }
    }
}
