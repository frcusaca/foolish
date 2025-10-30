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
        return wrap(targoe, 0);
    }

    public static Firoe wrap(Insoe source_code, int line) {
        switch (source_code) {
            case null -> {
                return new Firoe();
            }
            case Insoe code_node -> {
                AST ast = code_node.ast();
                switch (ast) {
                    case AST.Program p -> {
                        return new ProgramFiroe(code_node, wrap(new Insoe(p.branes())));
                    }
                    case AST.Brane b -> {
                        List<Firoe> stmts = new ArrayList<>();
                        List<AST.Expr> raw_stmts = b.statements();
                        for (int raw_stmts_line = 0; raw_stmts_line < raw_stmts.size(); raw_stmts_line++) {
                            AST.Expr expr = raw_stmts.get(raw_stmts_line);
                            stmts.add(wrap(new Insoe(expr), raw_stmts_line));
                        }
                        return new BraneFiroe(code_node, stmts);
                    }
                    case AST.Branes brs -> {
                        List<Firoe> stmts = new ArrayList<>();
                        int b_l_line = -1;
                        for (AST.Brane br : brs.branes()) {
                            for (AST.Expr expr : br.statements()) {
                                stmts.add(wrap(new Insoe(expr), ++b_l_line));
                            }
                        }
                        return new BraneFiroe(code_node, stmts);
                    }
                    case AST.Assignment a -> {
                        return new AssignmentFiroe(code_node,wrap(new Insoe(a.expr())));
                    }
                    case AST.BinaryExpr be -> {
                        return new BinaryFiroe(code_node, wrap(new Insoe(be.left())), wrap(new Insoe(be.right())));
                    }
                    case AST.UnaryExpr ue -> {
                        return new UnaryFiroe(code_node, wrap(new Insoe(ue.expr())));
                    }
                    case AST.Identifier anid -> {
                        return new IdentifierFiroe(code_node);
                    }
                    case AST.IfExpr iff -> {
                        Firoe condition = wrap(new Insoe(iff.condition()));
                        Firoe thenExpr = wrap(new Insoe(iff.thenExpr()));
                        Firoe elseExpr = wrap(new Insoe(iff.elseExpr()));
                        List<IfFiroe> elseIfs = new ArrayList<>();
                        for (AST.IfExpr e : iff.elseIfs()) {
                            elseIfs.add((IfFiroe) wrap(new Insoe(e)));
                        }
                        return new IfFiroe(code_node, condition, thenExpr, elseExpr, elseIfs);
                    }
                    case AST.UnknownExpr unknown -> {
                        return new Firoe(code_node);
                    }
                    case AST.Literal literal -> {
                        // In this special case, we do not have a Firoe as there is no firoe of processing a literal
                        if (literal instanceof AST.IntegerLiteral il)
                            return Finear.of(il.value());
                        else {
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
