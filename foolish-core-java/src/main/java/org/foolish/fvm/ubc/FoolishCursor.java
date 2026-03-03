package org.foolish.fvm.ubc;

import java.util.List;

import static org.foolish.fvm.ubc.Cursor.*;

/**
 * The Foolish Cursor is a cursor instantiated from a FoolishIndex.
 * It points to a specific BraneFiroe and statement index within that brane.
 * Defaults to beforeLine.
 */
public final class FoolishCursor extends CursorImpl {

    public FoolishCursor(BraneFiroe root, FoolishIndex idx) {
        this(root, idx, 0); // Default to start?
    }

    public FoolishCursor(BraneFiroe root, int index) {
        super(root, index);
    }

    public FoolishCursor(BraneFiroe root, FoolishIndex idx, int offset) {
        // ... simplistic impl, existing FoolishIndex logic was complex
        // Let's assume FoolishIndex logic here is just to find the start point.
        // The original constructor navigated indices.
        BraneFiroe current = root;
        List<Integer> inds = idx.getIndices();
        for (int idxidx = 0; idxidx < inds.size() - 1; idxidx++) {
            FIR tf = current.getMemoryItem(inds.get(idxidx));
            if (!(tf instanceof BraneFiroe brane)) {
                throw new IllegalArgumentException("Invalid path in cursor: " + idxidx + " of " + inds);
            }
            current = brane;
        }
        super(current, inds.get(inds.size() - 1));
    }

}
