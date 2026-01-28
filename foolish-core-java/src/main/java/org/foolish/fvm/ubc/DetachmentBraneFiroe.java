package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import java.util.List;

public class DetachmentBraneFiroe extends BraneFiroe {
    private final List<AST.DetachmentBrane> detachments;
    private final BraneFiroe inner;
    private final DetachmentScope detachmentScope;

    public DetachmentBraneFiroe(List<AST.DetachmentBrane> detachments, BraneFiroe inner) {
        super(inner.ast()); // Use inner AST
        this.detachments = detachments;
        this.inner = inner;

        // Create scope for detachments
        this.detachmentScope = new DetachmentScope(detachments);

        // Use inner's memory as our exposed memory (for searches)
        this.braneMemory = inner.braneMemory;

        // Link inner to detachment scope
        this.braneMemory.setParent(detachmentScope.braneMemory);

        // Add tasks to braneMind manually
        // Note: enqueueFirs puts in braneMemory, which is innerMemory.
        // detachmentScope should NOT be in innerMemory.
        this.braneMind.add(detachmentScope);
        indexLookup.put(detachmentScope, this.braneMind.size() - 1);

        this.braneMind.add(inner);
        indexLookup.put(inner, this.braneMind.size() - 1);

        // Set parents
        detachmentScope.setParentFir(this);
        inner.setParentFir(this);
    }

    @Override
    public void ordinateToParentBraneMind(FiroeWithBraneMind parent, int myPos) {
        // Link detachment scope to parent
        detachmentScope.ordinateToParentBraneMind(parent, myPos);
        this.ordinated = true;
    }

    @Override
    protected void initialize() {
        setInitialized();
        // Nothing else to do, tasks are already in braneMind
    }

    // Helper class for Detachment Scope
    private static class DetachmentScope extends BraneFiroe {
        private final List<AST.DetachmentBrane> detachments;

        public DetachmentScope(List<AST.DetachmentBrane> detachments) {
            super(null); // No AST
            this.detachments = detachments;
            // Override memory with DetachmentBraneMemory
            this.braneMemory = new DetachmentBraneMemory(null);
            // Must set owning brane to THIS scope so definitions resolve here
            this.braneMemory.setOwningBrane(this);
        }

        @Override
        protected void initialize() {
            if (isInitialized()) return;
            setInitialized();

            DetachmentBraneMemory mem = (DetachmentBraneMemory) this.braneMemory;

            // Process detachments Left-to-Right
            for (AST.DetachmentBrane db : detachments) {
                for (AST.DetachmentStatement stmt : db.statements()) {
                    if (stmt.expr() != null) {
                         // Definition (Assignment)
                         FIR assign = FIR.createFiroeFromExpr(new AST.Assignment(stmt.identifier(), stmt.expr(), AST.AssignmentOperator.ASSIGN));
                         enqueueFirs(assign);
                    } else {
                         // Rule (Block identifier)
                         Query q = new Query.StrictlyMatchingQuery(stmt.identifier().id(), stmt.identifier().canonicalCharacterization());
                         // Block rule (isAllow = false)
                         mem.addRule(q, false);
                    }
                }
            }
        }
    }
}
