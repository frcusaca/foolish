# UBC2 Cloning and Message Fate Analysis

## Parent Attachment Sequence (FIR Creation)
1. Parent's step() fires UNINITIALIZED case -> calls initialize()
2. initialize() calls createFiroeFromExpr() for each AST statement (child created, parentFir=null)
3. storeFirs(child) sets child.parentFir = parent, adds to braneMemory, records indexLookup
4. If child has braneMind: ordinateToParentBraneMind() links child.braneMemory.parent -> parent.braneMemory
5. Parent transitions to INITIALIZED; child has parent reference before ever stepping

## Constanic Cloning Sequence (CMFir.startPhaseB)
1. CMFir detects o.atConstanic()
2. o2 = o.cloneConstanic(this, Optional.of(Nyes.INITIALIZED))
3. For BraneFiroe: copy constructor recursively cloneConstanic each child with clone as parent
4. Each child's braneMemory re-ordinated to clone's braneMemory
5. startPhaseB() links clone's braneMemory.parent -> CMFir's containing brane's braneMemory
6. Clone starts at INITIALIZED, skips initialize(), proceeds through CHECKED/PRIMED/EVALUATING

## Message Fate During Cloning
- Clone gets fresh empty queues (no inheritance)
- Original's pending messages are orphaned (original is constanic, will not step)
- Clone re-sends fresh messages through new parent chain
- StateChange listener registrations are NOT inherited; clone's children re-register on cloned targets
- This is correct: original targets and cloned targets are different objects

## Key Gaps Identified (2026-03)
1. No WOCONSTANIC in Nyes enum - design-forward only
2. Current code always resets clones to INITIALIZED (no CONSTANIC vs WOCONSTANIC distinction)
3. Message-passing is design-only; code uses synchronous braneMemory traversal
4. WOCONSTANIC clone listener re-registration protocol under-specified
5. setParentFir() has no runtime guards despite design doc showing depth/constanic checks
6. Protocol doc terminology misaligned with design doc (SearchRequest vs FulfillSearch)
