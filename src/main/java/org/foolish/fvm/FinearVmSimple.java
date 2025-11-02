package org.foolish.fvm;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;
import java.util.Set;

/**
 * Evaluation utilities that operate on {@link Firoe} trees and push them toward
 * {@link Finear} results.
 */
public final class FinearVmSimple implements FinearVmAbstract {
    private static final Set<String> BUILT_IN_TYPES = Set.of("integer", "boolean", "brane");
    public FinearVmSimple() {
    }

    public Firoe evaluate(Firoe firoe) {
        return evaluate(firoe, new Env());
    }

    public Firoe evaluate(Firoe firoe, Env env) {
        Firoe result = Finear.UNKNOWN;
        if (firoe instanceof ProgramFiroe pm) {
            result = evaluate(pm.brane(), env);
        } else if (firoe instanceof BraneFiroe bm) {
            int uks = 0;
            int b_l_n = -1;
            List<Firoe> evaluatedStmts = new ArrayList<>();
            for (Firoe stmt : bm.statements()) {
                ++b_l_n;
                Firoe evaluatedStmt;
                if (stmt instanceof BraneFiroe bstmt) {
                    evaluatedStmt = evaluate(bstmt, new Env(env, b_l_n));
                } else {
                    evaluatedStmt = evaluate(stmt, env);
                }
                evaluatedStmts.add(evaluatedStmt);
                if (evaluatedStmt == Finear.UNKNOWN) {
                    ++uks;
                }
            }
            if(uks > 0){
                result = Finear.UNKNOWN;
            } else {
                result = new BraneFiroe(null, evaluatedStmts);
            }
        } else if (firoe instanceof AssignmentFiroe am) {
            Targoe value = evaluate(am.expr(), env);
            if (value instanceof Firoe vm) {
                Characterizable typedId = characterizeAssignment(am.id(), vm);
                env.put(typedId, vm);
                result = new AssignmentFiroe(null, typedId, vm);
            }
        } else if (firoe instanceof BinaryFiroe bm) {
            Targoe l = evaluate(bm.left(), env);
            Targoe r = evaluate(bm.right(), env);
            if (l instanceof Firoe lm && lm instanceof Finear flm && !flm.isUnknown()
                    && r instanceof Firoe rm && rm instanceof Finear frm && !frm.isUnknown()) {
                Object lvObj = flm.value();
                Object rvObj = frm.value();
                result = switch (bm.op()) {
                    case "+" -> Finear.of(requireNumeric(lvObj, bm.op()) + requireNumeric(rvObj, bm.op()));
                    case "-" -> Finear.of(requireNumeric(lvObj, bm.op()) - requireNumeric(rvObj, bm.op()));
                    case "*" -> Finear.of(requireNumeric(lvObj, bm.op()) * requireNumeric(rvObj, bm.op()));
                    case "/" -> Finear.of(requireNumeric(lvObj, bm.op()) / requireNumeric(rvObj, bm.op()));
                    case "==" -> Finear.ofBoolean(equalsValue(lvObj, rvObj));
                    case "<>" -> Finear.ofBoolean(!equalsValue(lvObj, rvObj));
                    case "<=" -> Finear.ofBoolean(requireNumeric(lvObj, bm.op()) <= requireNumeric(rvObj, bm.op()));
                    case ">=" -> Finear.ofBoolean(requireNumeric(lvObj, bm.op()) >= requireNumeric(rvObj, bm.op()));
                    default -> throw new IllegalArgumentException("Unknown op: " + bm.op());
                };
            }
        } else if (firoe instanceof UnaryFiroe um) {
            Targoe res = evaluate(um.expr(), env);
            if (res instanceof Firoe mres && !mres.isUnknown() &&
                    mres instanceof Finear fmres) {
                long v = requireNumeric(fmres.value(), um.op());
                long val = switch (um.op()) {
                    case "+" -> +v;
                    case "-" -> -v;
                    case "*" -> v; // '*' unary no-op for now
                    default -> throw new IllegalArgumentException("Unknown unary op: " + um.op());
                };
                result = Finear.of(val);
            }
        } else if (firoe instanceof IdentifierFiroe im) {
            // NOTE: here, we did not specify a line number. We depend on sequential evaluation to
            // find the right item.
            Targoe res = env.get(im.id());
            if (res instanceof Firoe mr) {
                result = mr;
            } else {
                throw new IllegalStateException("Identifier " + im.id().id() + " resolved too generically to an object:" + res.getClass());
            }
        } else if (firoe instanceof IfFiroe im) {
            switch (eval_cond(im.condition(), im.thenExpr(), env)) {
                case null -> {
                }
                case Finear f when f == Finear.UNKNOWN -> {
                }
                case Firoe m -> {
                    result = m;
                }
            }
            if (result == Finear.UNKNOWN) {
                for (IfFiroe elif : im.elseIfs()) {
                    switch (eval_cond(elif.condition(), elif.thenExpr(), env)) {
                        case null -> {
                        }
                        case Firoe m -> {
                            result = m;
                        }
                    }
                    if (result != Finear.UNKNOWN) {
                        break;
                    }
                }
            }
            if (result == Finear.UNKNOWN) {
                Firoe elseExpr = im.elseExpr();
                if (elseExpr != null) {
                    result = evaluate(elseExpr, env);
                }
            }
        } else if (firoe.base() instanceof Insoe in && in.ast() instanceof AST.UnknownExpr) {
            result = Finear.UNKNOWN;
        } else {
            Targoe base = firoe.base();
            if (base instanceof Finear f) {
                result = f;
            } else {
                result = Finear.UNKNOWN;
            }
        }
        if (firoe.progress_heap().isEmpty() || result != firoe.progress_heap().getLast())
            firoe.progress_heap().add(result);
        return result;
    }

    // Checks an if branch and executes result if the condition is known or true
    // if condition evaluates to true, then the evaluted then expr is returned.
    // if condition is false, then null is returned
    // if condition is unknown, then Unknown is returned without executing then branch
    Firoe eval_cond(Firoe condition, Firoe thenExpr, Env env) {
        ensureBooleanCharacterization(condition);
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

    private long requireNumeric(Object value, String op) {
        if (value instanceof Number number) {
            return number.longValue();
        }
        throw new IllegalStateException("Operation '" + op + "' requires numeric operands but found " + value);
    }

    private boolean equalsValue(Object left, Object right) {
        if (left instanceof Number ln && right instanceof Number rn) {
            return ln.longValue() == rn.longValue();
        }
        return left == null ? right == null : left.equals(right);
    }

    private Characterizable characterizeAssignment(Characterizable id, Firoe value) {
        if (value instanceof BraneFiroe) {
            return prependCharacterization(id, "brane");
        }
        if (value instanceof Finear finear && !finear.isUnknown()) {
            Object raw = finear.value();
            if (raw instanceof Number) {
                return prependCharacterization(id, "integer");
            }
            if (raw instanceof Boolean) {
                return prependCharacterization(id, "boolean");
            }
        }
        return id;
    }

    private Characterizable prependCharacterization(Characterizable id, String characterization) {
        if (id == null) {
            return Characterizable.fromCanonical(characterization);
        }
        String baseCanonical = id.canonical();
        if (baseCanonical == null || baseCanonical.isEmpty()) {
            baseCanonical = id.id();
        }
        if (baseCanonical == null) {
            baseCanonical = "";
        }
        if (baseCanonical.startsWith(characterization + "'")) {
            return id;
        }
        String remainder = baseCanonical;
        int idx = baseCanonical.indexOf("'");
        if (idx > 0) {
            String first = baseCanonical.substring(0, idx);
            if (BUILT_IN_TYPES.contains(first)) {
                remainder = baseCanonical.substring(idx + 1);
            }
        } else if (BUILT_IN_TYPES.contains(baseCanonical)) {
            remainder = "";
        }
        if (remainder.isEmpty()) {
            remainder = id.id() == null ? "" : id.id();
        }
        String canonical = remainder.isEmpty() ? characterization : characterization + "'" + remainder;
        return Characterizable.fromCanonical(canonical);
    }

    private void ensureBooleanCharacterization(Firoe condition) {
        Targoe base = condition.base();
        if (base instanceof Insoe insoe) {
            AST ast = insoe.ast();
            if (!isBooleanCharacterized(ast)) {
                throw new IllegalStateException("IF conditions must have boolean characterization");
            }
        }
    }

    private boolean isBooleanCharacterized(AST ast) {
        if (ast instanceof AST.Characterizable charExpr) {
            return "boolean".equals(charExpr.canonicalCharacterization());
        } else if (ast instanceof AST.BinaryExpr binaryExpr) {
            AST.Identifier characterization = binaryExpr.characterization();
            return characterization != null && "boolean".equals(characterization.cannonicalId());
        } else if (ast instanceof AST.UnaryExpr unaryExpr) {
            AST.Identifier characterization = unaryExpr.characterization();
            return characterization != null && "boolean".equals(characterization.cannonicalId());
        }
        return false;
    }
}
