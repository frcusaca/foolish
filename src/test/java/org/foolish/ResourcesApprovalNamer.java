package org.foolish;

import org.approvaltests.namer.StackTraceNamer;

/**
 * Custom ApprovalNamer that stores approval files in src/test/resources
 * instead of src/test/java alongside test classes.
 */
public class ResourcesApprovalNamer extends StackTraceNamer {
    @Override
    public String getSourceFilePath() {
        return super.getSourceFilePath().replace("/java/", "/resources/");
    }
}
