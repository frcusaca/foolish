package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.apache.commons.lang3.tuple.Pair;

public class SearchFiroe extends FiroeWithBraneMind {
    private final AST.Expr braneAst;
    private final String operator;
    private final String patternStr;
    private Query query;
    private FIR targetBraneFir;
    private FIR value;

    public SearchFiroe(AST.BraneRegexpSearch ast) {
        super(ast);
        this.braneAst = ast.brane();
        this.operator = ast.operator();
        this.patternStr = ast.pattern();
        this.query = new RegexpQuery(patternStr);
    }

    @Override
    protected void initialize() {
        if (isInitialized()) return;
        setInitialized();

        targetBraneFir = FIR.createFiroeFromExpr(braneAst);
        enqueueFirs(targetBraneFir);
    }

    @Override
    public void step() {
        super.step();

        if (isNye()) {
            if (targetBraneFir == null) {
                // Should not happen for valid AST, but avoid loop
                setNyes(Nyes.RESOLVED);
                return;
            }

            // Check if we can perform the search now (when targetBraneFir is done)
            if (value == null && !targetBraneFir.isNye()) {
                 performSearch();
            }
            return;
        }
    }

    private void performSearch() {
         if (value != null) return;

         FIR actualTarget = targetBraneFir;
         if (actualTarget instanceof IdentifierFiroe idF) {
              actualTarget = idF.getResolvedValue();
         }

         if (actualTarget instanceof AssignmentFiroe ass) {
              actualTarget = ass.getResult();
         }

         if (actualTarget instanceof FiroeWithBraneMind fwbm) {
              BraneMemory mem = fwbm.getMemory();

              if (".".equals(operator) || "?".equals(operator)) {
                  this.value = mem.get(query, Integer.MAX_VALUE)
                                  .map(Pair::getValue)
                                  .orElse(null); // Or NK?
              } else {
                  // TODO: Implement ?? and ..
              }
         }

         if (this.value == null) {
             // Not found or not a valid target?
             // Should we return NK or fail?
             // For now leaving as null or maybe empty ValueFiroe?
             // If we don't set RESOLVED, it will loop forever?
             // SearchFiroe needs to eventually finish.
         }

         setNyes(Nyes.RESOLVED);
    }

    public FIR getValueFir() {
        return value;
    }

    @Override
    public long getValue() {
        if (value == null) {
             throw new IllegalStateException("SearchFiroe not fully evaluated or nothing found");
        }
        return value.getValue();
    }
}
