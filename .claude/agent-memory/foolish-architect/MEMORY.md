# Foolish Architect Agent Memory

## Key Files
- Grammar: `foolish-parser-java/src/main/antlr4/Foolish.g4`
- FIR base: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/FIR.java`
- BraneMind: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/FiroeWithBraneMind.java`
- BraneFiroe: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/BraneFiroe.java`
- CMFir: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/CMFir.java`
- Nyes enum: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/Nyes.java`
- UBC2 design: `docs/how/ubc2_design.md` (large file, read in 200-line chunks)
- Message protocol: `docs/how/ubc2_message_protocol.md`
- Doc conventions: `docs/DOC_AGENTS.md`

## Architecture Notes
- UBC2 design uses PREMBRYONIC/EMBRYONIC/BRANING/constanic states
- Current Nyes enum: UNINITIALIZED/INITIALIZED/CHECKED/PRIMED/EVALUATING/CONSTANIC/CONSTANT
- No WOCONSTANIC or INDEPENDENT in current enum (design-forward only)
- UBC2 message-passing (FulfillSearch/RespondToSearch/StateChange) is design-only; current code uses synchronous braneMemory parent-chain traversal
- Parent attachment happens in storeFirs() during parent's initialize()
- Constanic cloning triggered by CMFir.startPhaseB() when o.atConstanic()
- Clone always resets to INITIALIZED regardless of CONSTANIC/WOCONSTANIC distinction
- Constraint C5: braneMind must be empty when FIR is constanic (verified in copy constructor)
- setParentFir() in code has no guards; design doc shows depth/constanic checks not yet implemented

## Design Tensions
- Protocol doc uses SearchRequest/SearchResponse; design doc uses FulfillSearch/RespondToSearch
- Protocol doc lists StateRequest but also says it was removed (contradiction)
- WOCONSTANIC cloning listener re-registration protocol is under-specified in design

See `ubc2-cloning-analysis.md` for detailed analysis of cloning and message fate.
