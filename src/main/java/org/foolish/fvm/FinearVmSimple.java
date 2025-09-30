package org.foolish.fvm;

import org.foolish.ast.AST;

import static java.lang.Boolean.FALSE;
import static java.lang.Boolean.TRUE;

/**
 * Evaluation utilities that operate on {@link Midoe} trees and push them toward
 * {@link Finear} results.
 */
public final class FinearVmSimple implements FinearVmAbstract {
    public FinearVmSimple() {
    }

    public Midoe evaluate(Midoe midoe) {
        return evaluate(midoe, new Env());
    }

    public Midoe evaluate(Midoe midoe, Env env) {
        Midoe result = Finear.UNKNOWN;
        if (midoe instanceof ProgramMidoe pm) {
            result = evaluate(pm.brane(), env);
        } else if (midoe instanceof BraneMidoe bm) {
            int uks =0;
            int b_l_n = -1;
            for (Midoe stmt : bm.statements()) {
                ++b_l_n;
                if (stmt instanceof BraneMidoe bstmt) {
                    result = evaluate(bstmt, new Env(env, b_l_n));
                } else {
                    result = evaluate(stmt, env);
                }
                if (result == Finear.UNKNOWN) {
                    ++uks;
                }
            }
            if(uks>0){
                result=Finear.UNKNOWN;
            }else{
                result = midoe;
            }
        } else if (midoe instanceof AssignmentMidoe am) {
            Targoe value = evaluate(am.expr(), env);
            if (value instanceof Midoe vm) {
                env.put(am.id().id(), vm);
                result = new AssignmentMidoe(null, am.id(),vm);
            }
        } else if (midoe instanceof BinaryMidoe bm) {
            Targoe l = evaluate(bm.left(), env);
            Targoe r = evaluate(bm.right(), env);
            if (l instanceof Midoe lm && lm instanceof Finear flm && !flm.isUnknown()
                    && r instanceof Midoe rm && rm instanceof Finear frm && !frm.isUnknown()) {
                long lv = ((Number) flm.value()).longValue();
                long rv = ((Number) frm.value()).longValue();
                long val = switch (bm.op()) {
                    case "+" -> lv + rv;
                    case "-" -> lv - rv;
                    case "*" -> lv * rv;
                    case "/" -> lv / rv;
                    default -> throw new IllegalArgumentException("Unknown op: " + bm.op());
                };
            }
        } else if (midoe instanceof UnaryMidoe um) {
            Targoe res = evaluate(um.expr(), env);
            if (res instanceof Midoe mres && !mres.isUnknown() &&
                    mres instanceof Finear fmres) {
                long v = ((Number) fmres.value()).longValue();
                long val = switch (um.op()) {
                    case "+" -> +v;
                    case "-" -> -v;
                    case "*" -> v; // '*' unary no-op for now
                    default -> throw new IllegalArgumentException("Unknown unary op: " + um.op());
                };
                result = Finear.of(val);
            }
        } else if (midoe instanceof IdentifierMidoe im) {
            // NOTE: here, we did not specify a line number. We depend on sequential evaluation to
            // find the right item.
            Targoe res = env.get(im.id().id());
            if (res instanceof Midoe mr) {
                result = mr;
            } else {
                throw new IllegalStateException("Identifier " + im.id().id() + " resolved too generically to an object:" + res.getClass());
            }
        } else if (midoe instanceof IfMidoe im) {
            switch (eval_cond(im.condition(), im.thenExpr(), env)) {
                case null -> {
                }
                case Finear f when f == Finear.UNKNOWN -> {
                }
                case Midoe m -> {
                    result = m;
                }
            }
            if (result == Finear.UNKNOWN) {
                for (IfMidoe elif : im.elseIfs()) {
                    switch (eval_cond(elif.condition(), elif.thenExpr(), env)) {
                        case null -> {
                        }
                        case Midoe m -> {
                            result = m;
                        }
                    }
                    if (result != Finear.UNKNOWN) {
                        break;
                    }
                }
            }
            if (result == Finear.UNKNOWN) {
                Midoe elseExpr = im.elseExpr();
                if (elseExpr != null) {
                    result = evaluate(elseExpr, env);
                }
            }
        } else if (midoe.base() instanceof Insoe in && in.ast() instanceof AST.UnknownExpr) {
            result = Finear.UNKNOWN;
        } else {
            Targoe base = midoe.base();
            if (base instanceof Finear f) {
                result = f;
            } else {
                result = Finear.UNKNOWN;
            }
        }
        if (result != midoe.progress_heap.getLast())
            midoe.progress_heap().add(result);
        return result;
    }

    // Checks an if branch and executes result if the condition is known or true
    // if condition evaluates to true, then the evaluted then expr is returned.
    // if condition is false, then null is returned
    // if condition is unknown, then Unknown is returned without executing then branch
    Midoe eval_cond(Midoe condition, Midoe thenExpr, Env env) {
        Targoe val = evaluate(condition, env);
        switch (asBoolean(val)) {
            case null -> {
                return Finear.UNKNOWN;
            }
            case Boolean b when b.booleanValue() -> {
                return evaluate(thenExpr, env);
            }
            default /*FALSE*/ -> {
                return null;
            }
        }
    }

    static Boolean asBoolean(Targoe f) {
        if (f instanceof Finear fv) {
            if (fv.isUnknown()) return null;
            Object o = fv.value();
            if (o instanceof Boolean b) return b;
            if (o instanceof Number n) return n.longValue() != 0;
        }
        return null;

    }
}
