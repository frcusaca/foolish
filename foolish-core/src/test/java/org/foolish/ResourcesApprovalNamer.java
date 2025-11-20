package org.foolish;

import org.approvaltests.namer.StackTraceNamer;

/**
 * Custom ApprovalNamer that stores approval files in src/test/resources
 * instead of src/test/java or src/test/scala alongside test classes.
 */
public class ResourcesApprovalNamer extends StackTraceNamer {
    @Override
    public String getSourceFilePath() {
        String path = super.getSourceFilePath();
        // Handle both Java and Scala test directories
        return path.replace("/java/", "/resources/")
                   .replace("/scala/", "/resources/");
    }
}
