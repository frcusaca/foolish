package org.foolish.fvm;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;

/**
 * Utilities for constructing {@link Firoe} trees from arbitrary {@link Targoe}
 * instances.  Each resulting {@code Firoe} has its originating {@link Targoe}
 * at the bottom of its progress heap.
 */
public final class FiroeVm {
    private FiroeVm() {
    }

    /**
     * Wraps the given target in a {@link Firoe}, attaching it to the progress heap.
     */
    public static Firoe wrap(Insoe targoe) {
        return wrap(targoe, 0, null);
    }

    public static Firoe wrap(Insoe source_code, int line) {
        return wrap(source_code, line, null);
    }

    /**
     * Wraps the given target in a {@link Firoe} with parent context.
     *
     * @param source_code The Insoe to wrap
     * @param line The line number in the source
     * @param parent The parent Firoe context for upward search (null for top level)
     * @return The wrapped Firoe
     */
    public static Firoe wrap(Insoe source_code, int line, Firoe parent) {
        switch (source_code) {
            case null -> {
                return new Firoe();
            }
            case Insoe code_node -> {
                AST ast = code_node.ast();
                switch (ast) {
                    case AST.Program p -> {
                        return new ProgramFiroe(code_node, wrap(new Insoe(p.branes()), 0, null));
                    }
                    case AST.Brane b -> {
                        // Create the BraneFiroe that will be the parent for nested statements
                        BraneFiroe braneFiroe = new BraneFiroe(code_node, List.of());
                        List<Firoe> stmts = new ArrayList<>();
                        List<AST.Expr> raw_stmts = b.statements();
                        for (int raw_stmts_line = 0; raw_stmts_line < raw_stmts.size(); raw_stmts_line++) {
                            AST.Expr expr = raw_stmts.get(raw_stmts_line);
                            stmts.add(wrap(new Insoe(expr), raw_stmts_line, braneFiroe));
                        }
                        return new BraneFiroe(code_node, stmts);
                    }
                    case AST.DetachmentBrane ignored -> {
                        throw new UnsupportedOperationException("Detachment branes are not implemented yet");
                    }
                    case AST.Branes brs -> {
                        // Create the BraneFiroe that will be the parent for nested statements
                        BraneFiroe braneFiroe = new BraneFiroe(code_node, List.of());
                        List<Firoe> stmts = new ArrayList<>();
                        int b_l_line = -1;
                        for (AST.Characterizable br : brs.branes()) {
                            if (br instanceof AST.Brane brane) {
                                for (AST.Expr expr : brane.statements()) {
                                    stmts.add(wrap(new Insoe(expr), ++b_l_line, braneFiroe));
                                }
                            } else if (br instanceof AST.DetachmentBrane) {
                                throw new UnsupportedOperationException("Detachment branes are not implemented yet");
                            } else if (br instanceof AST.SearchUP searchUp) {
                                // SearchUP doesn't have statements, handle as a single expression
                                stmts.add(wrap(new Insoe(searchUp), ++b_l_line, braneFiroe));
                            }
                        }
                        return new BraneFiroe(code_node, stmts);
                    }
                    case AST.Assignment a -> {
                        return new AssignmentFiroe(code_node, wrap(new Insoe(a.expr()), 0, parent));
                    }
                    case AST.BinaryExpr be -> {
                        return new BinaryFiroe(code_node, wrap(new Insoe(be.left()), 0, parent), wrap(new Insoe(be.right()), 0, parent));
                    }
                    case AST.UnaryExpr ue -> {
                        return new UnaryFiroe(code_node, wrap(new Insoe(ue.expr()), 0, parent));
                    }
                    case AST.Identifier anid -> {
                        return new IdentifierFiroe(code_node);
                    }
                    case AST.IfExpr iff -> {
                        Firoe condition = wrap(new Insoe(iff.condition()), 0, parent);
                        Firoe thenExpr = wrap(new Insoe(iff.thenExpr()), 0, parent);
                        Firoe elseExpr = wrap(new Insoe(iff.elseExpr()), 0, parent);
                        List<IfFiroe> elseIfs = new ArrayList<>();
                        for (AST.IfExpr e : iff.elseIfs()) {
                            elseIfs.add((IfFiroe) wrap(new Insoe(e), 0, parent));
                        }
                        return new IfFiroe(code_node, condition, thenExpr, elseExpr, elseIfs);
                    }
                    case AST.UnknownExpr unknown -> {
                        return new Firoe(code_node);
                    }
                    case AST.SearchUP searchUp -> {
                        return new SearchUpFiroe(code_node, parent);
                    }
                    case AST.Literal literal -> {
                        // In this special case, we do not have a Firoe as there is no firoe of processing a literal
                        if (literal instanceof AST.IntegerLiteral il) {
                            return Finear.of(il.value());
                        } else if (literal instanceof AST.BooleanLiteral bl) {
                            return Finear.ofBoolean(bl.value());
                        } else {
                            throw new IllegalArgumentException("Unknown literal type: " + literal);
                        }
                    }
                    default -> {
                        throw new IllegalArgumentException("Unknown AST type: " + ast);
                    }
                }
            }
        }
    }
}
