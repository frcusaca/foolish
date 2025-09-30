package org.foolish.fvm;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;

/**
 * Utilities for constructing {@link Midoe} trees from arbitrary {@link Targoe}
 * instances.  Each resulting {@code Midoe} has its originating {@link Targoe}
 * at the bottom of its progress heap.
 */
public final class MidoeVm {
    private MidoeVm() {
    }

    /**
     * Wraps the given target in a {@link Midoe}, attaching it to the progress heap.
     */
    public static Midoe wrap(Insoe targoe) {
        return wrap(targoe, 0);
    }

    public static Midoe wrap(Insoe source_code, int line) {
        switch (source_code) {
            case null -> {
                return new Midoe();
            }
            case Insoe code_node -> {
                AST ast = code_node.ast();
                switch (ast) {
                    case AST.Program p -> {
                        return new ProgramMidoe(code_node, wrap(new Insoe(p.branes())));
                    }
                    case AST.Brane b -> {
                        List<Midoe> stmts = new ArrayList<>();
                        List<AST.Expr> raw_stmts = b.statements();
                        for (int raw_stmts_line = 0; line < raw_stmts.size(); raw_stmts_line++) {
                            AST.Expr expr = raw_stmts.get(raw_stmts_line);
                            stmts.add(wrap(new Insoe(expr), line));
                        }
                        return new BraneMidoe(code_node, stmts);
                    }
                    case AST.Branes brs -> {
                        List<Midoe> stmts = new ArrayList<>();
                        int b_l_line = -1;
                        for (AST.Brane br : brs.branes()) {
                            for (AST.Expr expr : br.statements()) {
                                stmts.add(wrap(new Insoe(expr), ++b_l_line));
                            }
                        }
                        return new BraneMidoe(code_node, stmts);
                    }
                    case AST.Assignment a -> {
                        return new AssignmentMidoe(code_node,wrap(new Insoe(a.expr())));
                    }
                    case AST.BinaryExpr be -> {
                        return new BinaryMidoe(code_node, wrap(new Insoe(be.left())), wrap(new Insoe(be.right())));
                    }
                    case AST.UnaryExpr ue -> {
                        return new UnaryMidoe(code_node, wrap(new Insoe(ue.expr())));
                    }
                    case AST.Identifier anid -> {
                        return new IdentifierMidoe(code_node);
                    }
                    case AST.IfExpr iff -> {
                        Midoe condition = wrap(new Insoe(iff.condition()));
                        Midoe thenExpr = wrap(new Insoe(iff.thenExpr()));
                        Midoe elseExpr = wrap(new Insoe(iff.elseExpr()));
                        List<IfMidoe> elseIfs = new ArrayList<>();
                        for (AST.IfExpr e : iff.elseIfs()) {
                            elseIfs.add((IfMidoe) wrap(new Insoe(e)));
                        }
                        return new IfMidoe(code_node, condition, thenExpr, elseExpr, elseIfs);
                    }
                    case AST.UnknownExpr unknown -> {
                        return new Midoe(code_node);
                    }
                    case AST.Literal literal -> {
                        // In this special case, we do not have a Midoe as there is no middoe of processing a literal
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
