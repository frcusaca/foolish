package org.foolish.fvm.ubc;

/**
 * ExecutionContext holds runtime context information for FIR execution,
 * such as the source filename being executed.
 * <p>
 * This context is threaded through the UBC and FIR hierarchy to enable
 * comprehensive error reporting with file location information.
 */
public class ExecutionContext {
    private static final ThreadLocal<ExecutionContext> CURRENT = new ThreadLocal<>();

    private final String sourceFilename;

    /**
     * Creates an execution context with the given source filename.
     *
     * @param sourceFilename the name of the .foo file being executed (e.g., "test.foo")
     */
    public ExecutionContext(String sourceFilename) {
        this.sourceFilename = sourceFilename;
    }

    /**
     * Gets the source filename for this execution context.
     *
     * @return the source filename
     */
    public String getSourceFilename() {
        return sourceFilename;
    }

    /**
     * Sets the current thread's execution context.
     *
     * @param context the execution context to set
     */
    public static void setCurrent(ExecutionContext context) {
        CURRENT.set(context);
    }

    /**
     * Gets the current thread's execution context.
     *
     * @return the current execution context, or null if none is set
     */
    public static ExecutionContext getCurrent() {
        return CURRENT.get();
    }

    /**
     * Clears the current thread's execution context.
     */
    public static void clearCurrent() {
        CURRENT.remove();
    }
}
