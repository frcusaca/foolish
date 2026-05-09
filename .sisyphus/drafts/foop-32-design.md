# Draft: FOOP-32 UBCb Design

## Confirmed Design Decisions

### NYES State Chain (UBCb)
```
PREMBRYONIC → EMBRYONIC → (BRANING | WODEP)
WODEP → BRANING | ECONSTANIC | WOCONSTANIC | CONSTANT | INDEPENDENT | NK
BRANING → ECONSTANIC | WOCONSTANIC | CONSTANT | INDEPENDENT | NK
```

- **WODEP**: Generic "waiting on dependencies" — pre-constanic, waiting for other FIRs to progress
- **Only one direct transition to WODEP**: from EMBRYONIC, when a search depends on a pre-constanic FIR
- **Parent invariant**: A brane must be at least EMBRYONIC before its children can be looked at or processed. AB walk will never hit a PREMBRYONIC ancestor.

### EMBRYONIC Stage (search resolution)
- Step all children to EMBRYONIC first
- Gather searches without entering another brane (staying at IB/AB boundary)
- **Linear time** — bounded by brane size (finite brane × finite statements)
- Resolve searches within IB (immediate brane): `{a=1; b=a;}` — b gets 1 locally
- Resolve searches in **entire AB chain** (all ancestors, no sibling brane entry): `{a=1; B={b=a};}` — while stepping brane B, root brane resolves B's search for `a`
- If search depends on pre-constanic FIR → **transition to WODEP** (not BRANING)
- If all searches resolved or failed (ECONSTANIC) → **transition to BRANING**

### BRANING Stage
- Perform operator activities if all members are complete
- Check dependencies, move to constanic state when ready
- Standard UBC braning semantics (depth-first within brane)

### StatementFir (new FIR subtype)
- **Is a FIR** — has its own NYES state
- **Fields**:
  - `code`: subspan of parent node's source string
  - `ast`: pointer to AST node in parent's AST
  - `name`: CharacterizedName (string equality for SPA1; characterization added later)
  - Body FIR embedded (stepped as part of StatementFir)
- **Placed in immutable array** — fixed size, members never change
- Enables: regexp name search, seek (`#-2`) operations

### AST Storage in Every FIR
- Every FIR carries both String code (subspan) and AST pointer
- Parser produces AST; compiler attaches AST pointers during FIR creation
- StatementFirs get subspan + AST node pointer from parent brane's AST

### UBCb Compiler
- **UBCb has its own compiler** — uses UBCb's FVM to step FIRs
- **Shared parser only** — parser output (AST) is shared with UBC
- **Optimization levels**:
  - `-O PREMBRYONIC`: persist raw AST + source (minimal stepping)
  - `-O CONSTANIC`: step as far as possible via FVM (may not reach CONSTANT for all FIRs)
- Persisted FIRs written to disk after stepping; allows comparison with UBC output

### WODEP Semantics
- **Definition**: WODEP = "I cannot make progress AND I cannot declare WOCONSTANIC." Dependency is still pre-constanic (NYE).
- **Entered from EMBRYONIC**: When a search depends on a FIR that has not yet reached any constanic state
  - Example: `{a={b=c;}; d=a.b;}` — during root EMBRYONIC, `a` is EMBRYONIC (cannot enter brane). `d` → WODEP.
- **WODEP processing (during BRANING)**: When a dependency's state changes, WODEP fires and re-evaluates:
  - Dependency → BRANING: WODEP → BRANING (dependency now has ordinates, resume stepping)
  - Dependency → WOCONSTANIC: WODEP → WOCONSTANIC (dependency IS constanic, relationship established)
  - Dependency → ECONSTANIC: WODEP → ECONSTANIC (search truly failed in that context)
  - Dependency → CONSTANT: WODEP → CONSTANT (full resolution)
- **WODEP is transient** — it always resolves to another state. WOCONSTANIC is the steady waiting state.
- **WODEP vs WOCONSTANIC boundary**: WODEP = synchronization timing (dependency still NYE). WOCONSTANIC = semantic dependency (relationship established, awaiting resolution).

### Worked Example
```
{a={b=c;}; d=a.b;}
```
1. Root EMBRYONIC: inner brane `{b=c;}` → EMBRYONIC. `d=a.b`: `a` is EMBRYONIC, cannot enter. `d` → **WODEP**.
2. Root BRANING: inner brane steps → WOCONSTANIC (`b=c` is ECONSTANIC). `a` becomes WOCONSTANIC.
3. WODEP fires: `d=a.b` re-evaluates. `a` IS constanic. `d` → **WOCONSTANIC**.
4. Root → WOCONSTANIC.

### Parent/Child Invariant
- Brane must be at least EMBRYONIC before children are looked at
- This means: when a brane is PREMBRYONIC, no child can reference it
- When a brane transitions to EMBRYONIC, children can begin their own EMBRYONIC processing
- AB walk will always find at least EMBRYONIC ancestors (or root)

## Open Questions

1. **WODEP → BRANING re-entry**: When WODEP resolves and transitions to BRANING, does it restart EMBRYONIC first (re-gather searches) or go directly to BRANING?

2. **CharacterizedName and `???`**: How is `???=1` handled in the StatementFir array? Does `???` use a special sentinel value? How does regexp search handle `???`?

3. **Seek (`#-2`) with immutable array**: Does StatementFir support direct index access? How does `a#5` work — is it "5th statement named `a`" or "5th statement from current position"?

4. **WODEP listener mechanism**: When a WODEP brane is waiting on dependency D, how does it learn that D has progressed? Is this via StateChange messages (like current WOCONSTANIC design) or via polling?

5. **Multiple WODEP dependencies**: If a brane enters WODEP waiting on multiple pre-constanic FIRs, does it track all of them? What if one resolves CONSTANT and another resolves ECONSTANIC?

6. **EMBRYONIC search enqueuing**: User mentioned "how to enqueue search for processing, and it should be sequential for now" — what's the enqueue order? Statement order (writing order)? Search depth?

7. **UBCb compiler module boundary**: Should the compiler live in `foolish-ubcb/` alongside the evaluator, or in a separate `foolish-compiler/` that depends on `foolish-ubcb`?

## Scope Boundaries
- INCLUDE: UBCb FVM evaluation, StatementFir, WODEP, EMBRYONIC search resolution, AB walk, compiler persistence
- EXCLUDE: Characterization (future), distributed evaluation (future), network transport (future)
- EXCLUDE: UBC changes (UBC remains unchanged, UBCb is separate)
