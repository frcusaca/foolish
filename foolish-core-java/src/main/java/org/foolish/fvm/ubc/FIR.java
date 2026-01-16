package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import java.util.ArrayList;
import java.util.List;
import java.util.Collections;

/**
 * Foolish Internal Representation (FIR).
 * The FIR is the internal representation of computation that holds an AST
 * and tracks evaluation progress.
 */
public abstract class FIR {
    protected final AST ast;
    protected final String comment;
    private boolean initialized;
    private Nyes nyes;

    protected FIR(AST ast, String comment) {
        this.ast = ast;
        this.comment = comment;
        this.initialized = false;
        this.nyes = Nyes.UNINITIALIZED;
    }

    protected FIR(AST ast) {
        this(ast, null);
    }

    /**
     * Returns the AST node associated with this FIR.
     */
    public AST ast() {
        return ast;
    }

    /**
     * Returns the optional comment for this FIR.
     */
    public String comment() {
        return comment;
    }

    /**
     * Returns whether this FIR has been initialized.
     */
    protected boolean isInitialized() {
        return initialized;
    }

    /**
     * Sets the initialized state of this FIR.
     */
    protected void setInitialized() {
        this.initialized = true;
    }

    /**
     * Returns the current NYE state of this FIR.
     */
    protected Nyes getNyes() {
        return nyes;
    }

    /**
     * Sets the NYE state of this FIR.
     * All changes to a Firoe's Nyes must be made through this method.
     */
    protected void setNyes(Nyes nyes) {
        this.nyes = nyes;
    }

    /**
     * Perform one step of evaluation on this FIR.
     * This method should advance the FIR's evaluation state by one step.
     * For simple values that don't require stepping, this can be a no-op.
     */
    public abstract void step();

    /**
     * Query method returning false if an additional step on this FIR does not change it.
     * Returns true when an additional step would change the FIR.
     * Not Yet Evaluated (NYE) indicates the FIR requires further evaluation steps.
     */
    public abstract boolean isNye();

    /**
     * Query method returning false only when all identifiers are bound.
     * Returns true if there are unbound identifiers (abstract state).
     */
    public abstract boolean isAbstract();

    /**
     * Gets the value from this FIR if it represents a simple value.
     * For ValueFiroe and evaluated expressions, returns the integer value.
     *
     * @return the integer value
     * @throws UnsupportedOperationException if this FIR doesn't support getValue
     * @throws IllegalStateException         if this FIR is not fully evaluated
     */
    public long getValue() {
        throw new UnsupportedOperationException("getValue not supported for " + getClass().getSimpleName());
    }

    /**
     * Creates a FIR from an AST expression.
     */
    protected static FIR createFiroeFromExpr(AST.Expr expr) {
        switch (expr) {
            case AST.IntegerLiteral literal -> {
                return new ValueFiroe(expr, literal.value());
            }
            case AST.BinaryExpr binary -> {
                return new BinaryFiroe(binary);
            }
            case AST.UnaryExpr unary -> {
                return new UnaryFiroe(unary);
            }
            case AST.IfExpr ifExpr -> {
                return new IfFiroe(ifExpr);
            }
            case AST.Brane brane -> {
                return new BraneFiroe(brane);
            }
            case AST.Assignment assignment -> {
                return new AssignmentFiroe(assignment);
            }
            case AST.Identifier identifier -> {
                return new IdentifierFiroe(identifier);
            }
            case AST.RegexpSearchExpr regexpSearch -> {
                return new RegexpSearchFiroe(regexpSearch);
            }
            case AST.OneShotSearchExpr oneShotSearch -> {
                return new OneShotSearchFiroe(oneShotSearch);
            }
            case AST.SeekExpr seekExpr -> {
                // TODO: Implement SeekFiroe when needed
                return new NKFiroe();
            }
            case AST.DetachmentBrane detachment -> {
                // Standalone DetachmentBrane maps to a BraneFiroe with modifications and no statements.
                List<BraneMemory.QueryModification> mods = extractQueryModifications(detachment);
                return new BraneFiroe(null, Collections.emptyList(), mods);
            }
            case AST.Branes branes -> {
                return createFiroeFromBranes(branes);
            }
            default -> {
                // Placeholder for unsupported types
                return new NKFiroe();
            }
        }
    }

    private static List<BraneMemory.QueryModification> extractQueryModifications(AST.DetachmentBrane detachment) {
        List<BraneMemory.QueryModification> mods = new ArrayList<>();
        for (AST.DetachmentStatement stmt : detachment.statements()) {
             // Check for P-brane '+' prefix in identifier
             String id = stmt.identifier().id();
             BraneMemory.Modification modification = BraneMemory.Modification.BLOCK;

             // Simple detection of + prefix
             if (id.startsWith("+")) {
                 modification = BraneMemory.Modification.ALLOW;
                 id = id.substring(1);
             }

             mods.add(new BraneMemory.QueryModification(new Query.RegexpQuery(id), modification));
        }
        return mods;
    }

    private static FIR createFiroeFromBranes(AST.Branes branes) {
        List<AST.Characterizable> list = branes.branes();
        if (list.isEmpty()) {
            return new BraneFiroe(null); // Empty brane
        }

        // Process from Right to Left to handle right-associativity of detachments
        FIR acc = null;

        for (int i = list.size() - 1; i >= 0; i--) {
            AST.Characterizable elem = list.get(i);

            switch (elem) {
                case AST.DetachmentBrane db -> {
                    List<BraneMemory.QueryModification> mods = extractQueryModifications(db);
                    if (acc == null) {
                        // Orphan detachment at end
                        acc = new BraneFiroe(null, Collections.emptyList(), mods);
                    } else if (acc instanceof BraneFiroe bf && bf.ast() == null && bf.comment() == null) { // Synthetic brane
                         bf.addQueryModifications(mods);
                    } else {
                         // acc is a standard brane or complex FIR.
                         // Inject modifications if it's a BraneFiroe, otherwise wrap.
                         if (acc instanceof BraneFiroe bf) {
                             bf.addQueryModifications(mods);
                         } else {
                             acc = new BraneFiroe(null, Collections.singletonList(acc), mods);
                         }
                    }
                }
                case AST.Brane sb -> {
                    // Standard Brane
                    BraneFiroe sbFiroe = (BraneFiroe) createFiroeFromExpr(sb);

                    if (acc == null) {
                        acc = sbFiroe;
                    } else if (acc instanceof BraneFiroe bf) {
                        // acc needs sbFiroe as predecessor.
                        bf.prepend(sbFiroe);
                    } else {
                         // acc is not a BraneFiroe? Fallback: wrap
                         List<FIR> seq = new ArrayList<>();
                         seq.add(sbFiroe);
                         seq.add(acc);
                         acc = new BraneFiroe(null, seq, Collections.emptyList());
                    }
                }
                case AST.Characterizable other -> {
                    // Other characterizable (e.g. SearchUP). Treat as expression.
                    FIR exprFiroe = createFiroeFromExpr((AST.Expr) other);
                    if (acc == null) {
                        acc = exprFiroe;
                    } else {
                        List<FIR> seq = new ArrayList<>();
                        seq.add(exprFiroe);
                        seq.add(acc);
                        acc = new BraneFiroe(null, seq, Collections.emptyList());
                    }
                }
            }
        }

        return acc;
    }
}
