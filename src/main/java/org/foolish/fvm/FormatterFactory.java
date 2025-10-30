package org.foolish.fvm;

/**
 * Factory for creating commonly used Targoe formatters.
 */
public class FormatterFactory {
    
    private static final TargoeFormatter VERBOSE_FORMATTER = 
        TargoeFormatter.builder()
            .verbose(true)
            .showProgressHeap(false)
            .build();
            
    private static final TargoeFormatter HUMAN_READABLE_FORMATTER = 
        TargoeFormatter.builder()
            .verbose(false)
            .showProgressHeap(false)
            .build();
            
    private static final TargoeFormatter DEBUG_FORMATTER = 
        TargoeFormatter.builder()
            .verbose(true)
            .showProgressHeap(true)
            .build();
    
    /**
     * Returns a formatter that outputs verbose class-based representation.
     * Example: "FiroeBrane(FiroeAssignment(a, 2))"
     */
    public static TargoeFormatter verbose() {
        return VERBOSE_FORMATTER;
    }
    
    /**
     * Returns a formatter that outputs human-readable representation.
     * Example: "{a = 2}"
     */
    public static TargoeFormatter humanReadable() {
        return HUMAN_READABLE_FORMATTER;
    }
    
    /**
     * Returns a formatter that outputs verbose representation with progress heap information.
     * Useful for debugging VM evaluation states.
     */
    public static TargoeFormatter debug() {
        return DEBUG_FORMATTER;
    }
    
    /**
     * Creates a custom formatter with the given configuration.
     */
    public static TargoeFormatter.Builder custom() {
        return TargoeFormatter.builder();
    }
}