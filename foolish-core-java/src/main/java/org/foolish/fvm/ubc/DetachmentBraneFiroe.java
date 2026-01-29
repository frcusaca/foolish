package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import java.util.Set;
import java.util.HashSet;
import java.util.stream.Collectors;

/**
 * Detachment Brane FIR wraps a regular brane and filters identifier resolution.
 * Syntax: [a, b, c]{expr} creates DetachmentBraneFiroe wrapping {expr}
 * <p>
 * The detached identifiers (a, b, c) are prevented from coordinating during
 * the wrapped brane's initial evaluation. Once the brane reaches CONSTANIC,
 * the filter becomes inactive (one-time effect).
 * <p>
 * <b>M-Brane Semantics:</b>
 * M-brane is the default detachment type: [a,b,c] or explicit [-a,b,c]
 * <ul>
 *   <li>Affects the immediately subsequent brane or chain of detachments</li>
 *   <li>Blocks identifier coordination <b>once</b> until wrapped brane reaches CONSTANIC</li>
 *   <li>Once CONSTANIC, filter becomes idle permanently</li>
 *   <li>Detachment does NOT reactivate when brane is referenced in CMFir</li>
 *   <li>Only SFF bracket (future feature) would cause reactivation</li>
 * </ul>
 */
public class DetachmentBraneFiroe extends FiroeWithBraneMind {
    private final Set<String> detachedIdentifiers;
    private boolean filterActive = true;

    /**
     * Creates a DetachmentBraneFiroe from an AST DetachmentBrane node.
     * The detachment brane itself doesn't wrap another brane - that happens
     * via concatenation at the statement level.
     *
     * @param ast the DetachmentBrane AST node
     */
    public DetachmentBraneFiroe(AST.DetachmentBrane ast) {
        super(ast);

        // Extract detached identifier names from AST
        this.detachedIdentifiers = ast.statements().stream()
            .map(stmt -> stmt.identifier().id())
            .collect(Collectors.toSet());

        // Set up this brane's memory with filtering
        this.braneMemory.setOwningBrane(this);
    }

    @Override
    protected void initialize() {
        setInitialized();

        // Enqueue the detachment statements (assignments or standalone identifiers)
        for (AST.DetachmentStatement stmt : ((AST.DetachmentBrane) ast).statements()) {
            // Create assignment FIR for each detachment statement
            // Note: Parser creates ??? (UnknownExpr) for standalone identifiers
            FIR fir = new AssignmentFiroe(new AST.Assignment(stmt.identifier(), stmt.expr()));
            enqueueFirs(fir);
        }
    }

    @Override
    public int step() {
        if (!isInitialized()) {
            initialize();
            return 1;
        }

        int work = super.step();

        // Once all detachment statements are evaluated, deactivate filter
        if (this.isConstanic() && filterActive) {
            filterActive = false;
        }

        return work;
    }

    /**
     * Checks if an identifier should be filtered (blocked) during search.
     * Used by BraneMemory during identifier resolution.
     *
     * @param identifierName the identifier name to check
     * @return true if this identifier should be blocked from coordination
     */
    public boolean shouldFilter(String identifierName) {
        return filterActive && detachedIdentifiers.contains(identifierName);
    }

    /**
     * Returns true if the detachment filter is currently active.
     * The filter starts active and becomes inactive once the detachment
     * reaches CONSTANIC state.
     *
     * @return true if filtering is active
     */
    public boolean isFilterActive() {
        return filterActive;
    }

    @Override
    public String toString() {
        return "DetachmentBrane[" + String.join(", ", detachedIdentifiers) +
               " filterActive=" + filterActive + "]";
    }
}
