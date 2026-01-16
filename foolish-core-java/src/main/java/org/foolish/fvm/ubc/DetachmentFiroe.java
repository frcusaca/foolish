package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

/**
 * DetachmentFiroe represents a detachment brane in the UBC system.
 * <p>
 * Detachment branes block identifier resolution from parent scopes.
 * Syntax: [identifier1 = default1; identifier2; ...]
 * <p>
 * When concatenated with a regular brane, the detachment brane prevents
 * the listed identifiers from being resolved in parent branes.
 * <p>
 * <b>IMPORTANT: Blocking cascades to all child branes.</b> Once an identifier is blocked
 * at a detachment level, that blocking is effective for the entire brane subtree below it.
 * This includes all nested branes within the detached brane. For example:
 * <pre>
 * [α, β]{
 *   result = α;      // α is blocked from parent scope
 *   nested = {
 *     x = α;         // α is ALSO blocked here (cascades to children)
 *   };
 * }
 * </pre>
 * <p>
 * <b>Detachment Brane Chaining (Left-to-Right Combination):</b>
 * <p>
 * Multiple adjacent detachment branes combine left-to-right before applying to a regular brane.
 * The chain stops when a regular brane is encountered. The leftmost detachment wins for conflicts.
 * <p>
 * Examples:
 * <ul>
 * <li>{@code [a][b]{...}} → combines both, blocks a and b</li>
 * <li>{@code [a=1][a=2]{...}} → left wins, a=1 (not a=2)</li>
 * <li>{@code [a=1][b=2][c=3]{...}} → all three combine, all defaults preserved</li>
 * <li>{@code [a]{[b]{...}}} → NO combination, [b] is inside a brane so it's separate</li>
 * </ul>
 * <p>
 * The combination semantics ensure that detachment chains are processed consistently,
 * with the leftmost (earliest) declaration taking precedence. This is implemented via
 * {@link #combineWith(DetachmentFiroe)} which creates a merged DetachmentFiroe.
 * <p>
 * <b>Types of Detachment Branes</b> (from NAMES_SEARCHES_N_BOUNDS.md):
 * <ul>
 * <li><b>Type 1: Backward Search</b> {@code [a, b]} - Blocks backward identifier resolution (implemented)</li>
 * <li><b>Type 2: Forward Search</b> {@code [/a, #N]} - Blocks forward search into brane (TODO)</li>
 * <li><b>Type 3: P-Brane</b> {@code [+a, b]} - Selective binding (undetachment) (TODO)</li>
 * </ul>
 * <p>
 * From the documentation:
 * "The detachment brane dissociates the ensuing brane on its right side from
 * the context, decontextualizing [the identifiers]. [They] become unbound symbols.
 * From the timeline perspective, the detachment snips off dependency DAG edges."
 */
public class DetachmentFiroe extends FiroeWithBraneMind {
    private final List<CharacterizedIdentifier> blockedIdentifiers;
    private final List<DetachmentDefault> defaults;

    /**
     * Represents a detachment statement with optional default value.
     */
    private record DetachmentDefault(CharacterizedIdentifier identifier, FIR defaultValue) {
    }

    public DetachmentFiroe(AST.DetachmentBrane detachmentBrane) {
        super(detachmentBrane);
        this.blockedIdentifiers = new ArrayList<>();
        this.defaults = new ArrayList<>();

        // Extract blocked identifiers and defaults from the AST
        for (AST.DetachmentStatement stmt : detachmentBrane.statements()) {
            AST.Identifier astId = stmt.identifier();
            CharacterizedIdentifier identifier = new CharacterizedIdentifier(
                    astId.id(),
                    astId.canonicalCharacterization()
            );

            blockedIdentifiers.add(identifier);

            // If there's a default value, create a FIR for it
            if (!(stmt.expr() instanceof AST.UnknownExpr)) {
                FIR defaultFir = createFiroeFromExpr(stmt.expr());
                defaults.add(new DetachmentDefault(identifier, defaultFir));
            }
        }
    }

    /**
     * Returns the list of identifiers that should be blocked from parent resolution.
     */
    public List<CharacterizedIdentifier> getBlockedIdentifiers() {
        return blockedIdentifiers;
    }

    /**
     * Returns the list of default values for detached identifiers.
     */
    public List<DetachmentDefault> getDefaults() {
        return defaults;
    }

    @Override
    protected void initialize() {
        if (isInitialized()) return;
        setInitialized();

        // Enqueue default value FIRs for evaluation
        for (DetachmentDefault def : defaults) {
            enqueueFirs(def.defaultValue);
        }
    }

    @Override
    public boolean isNye() {
        return !isInitialized() || super.isNye();
    }

    @Override
    public void step() {
        if (!isInitialized()) {
            initialize();
            return;
        }

        super.step();
    }

    /**
     * Applies this detachment to a target brane, blocking identifiers from parent resolution.
     * <p>
     * This method should be called when concatenating a detachment brane with a regular brane.
     * It modifies the target brane's memory to block the specified identifiers from parent searches.
     *
     * @param targetBrane The brane to apply detachment to
     */
    public void applyDetachmentTo(FiroeWithBraneMind targetBrane) {
        // Create a list of blocked identifier queries using unified matching logic
        // This uses CharacterizedIdentifier.equals() for consistent matching
        List<Query> blockedQueries = blockedIdentifiers.stream()
                .map(id -> new Query.StrictlyMatchingQuery(id.getId(), id.getCharacterization()))
                .collect(Collectors.toList());

        // Apply blocking to the target brane's memory
        targetBrane.braneMemory.setBlockedIdentifiers(blockedQueries);

        // Apply default values to the target brane
        for (DetachmentDefault def : defaults) {
            // Create an assignment FIR for the default value
            AST.Assignment assignment = new AST.Assignment(
                    new AST.Identifier(
                            List.of(),  // TODO: handle characterizations
                            def.identifier.getId()
                    ),
                    def.defaultValue.ast() instanceof AST.Expr expr ? expr : AST.UnknownExpr.INSTANCE
            );
            targetBrane.enqueueFirs(new AssignmentFiroe(assignment));
        }
    }

    /**
     * Combines this detachment brane with another one, implementing left-override semantics.
     * <p>
     * <b>Combination Rules (Left-to-Right, Left Wins):</b>
     * <ul>
     * <li>When `[a=1][a=2]`, the left `a=1` overrides the right `a=2`</li>
     * <li>When `[a][b]`, both `a` and `b` are blocked</li>
     * <li>When `[a=1][b=2]`, both defaults are kept</li>
     * <li>This (left) takes precedence over other (right) for conflicts</li>
     * </ul>
     * <p>
     * This enables chaining: `[a][b][c]{...}` combines all three detachments
     * before applying to the brane.
     *
     * @param other The right-side detachment brane to combine with
     * @return A new DetachmentFiroe representing the combined detachment
     */
    public DetachmentFiroe combineWith(DetachmentFiroe other) {
        // Create combined lists with left-override semantics
        List<CharacterizedIdentifier> combinedBlocked = new ArrayList<>(this.blockedIdentifiers);
        List<DetachmentDefault> combinedDefaults = new ArrayList<>(this.defaults);

        // Add identifiers from 'other' that aren't already in 'this' (left wins)
        for (CharacterizedIdentifier otherId : other.blockedIdentifiers) {
            if (!combinedBlocked.contains(otherId)) {
                combinedBlocked.add(otherId);
            }
        }

        // Add defaults from 'other' that aren't already in 'this' (left wins)
        for (DetachmentDefault otherDefault : other.defaults) {
            boolean alreadyHasDefault = combinedDefaults.stream()
                    .anyMatch(def -> def.identifier.equals(otherDefault.identifier));
            if (!alreadyHasDefault) {
                combinedDefaults.add(otherDefault);
            }
        }

        // Create a combined DetachmentFiroe
        return new DetachmentFiroe(combinedBlocked, combinedDefaults);
    }

    /**
     * Private constructor for creating combined detachment branes.
     * Used by combineWith() to create the result without parsing AST.
     */
    private DetachmentFiroe(List<CharacterizedIdentifier> blockedIds, List<DetachmentDefault> defaults) {
        super(null); // No AST for combined detachments
        this.blockedIdentifiers = new ArrayList<>(blockedIds);
        this.defaults = new ArrayList<>(defaults);
    }

    @Override
    public String toString() {
        StringBuilder sb = new StringBuilder();
        sb.append("[\n");
        for (int i = 0; i < blockedIdentifiers.size(); i++) {
            CharacterizedIdentifier id = blockedIdentifiers.get(i);
            sb.append("  ").append(id.getId());

            // Find matching default
            boolean hasDefault = false;
            for (DetachmentDefault def : defaults) {
                if (def.identifier.equals(id)) {
                    sb.append(" = ").append(def.defaultValue);
                    hasDefault = true;
                    break;
                }
            }
            if (!hasDefault) {
                sb.append(" = ???");
            }
            sb.append(";\n");
        }
        sb.append("]");
        return sb.toString();
    }
}
