# Unicellular Brane Computer (UBC)

The UBC package implements a step-by-step evaluation system for the Foolish programming language based on the design specified in the main [README.MD](../../../../../README.MD).

## Architecture

### Core Components

1. **FIR (Foolish Internal Representation)**
   - Base class for all internal representations
   - Tracks AST and evaluation progress
   - Query methods: `underevaluated()`, `isAbstract()`
   - Value accessors: `getValue()`, `getEnvironment()`

2. **FiroeWithBraneMind**
   - Contains a `braneMind` queue for breadth-first evaluation
   - Manages step-by-step execution of nested expressions

3. **FiroeWithoutBraneMind**
   - For finalized values that don't need further evaluation
   - Always returns `underevaluated() == false`

### Expression Types

- **ValueFiroe**: Integer literal values
- **BraneFiroe**: Brane containers with multiple statements
- **BinaryFiroe**: Binary operators (+, -, *, /)
- **UnaryFiroe**: Unary operators (-, !)
- **IfFiroe**: Conditional if-then-else expressions
- **SearchUpFiroe**: Search-up (↑) operations for parent brane references

### Main Computer

**UnicelluarBraneComputer** - The execution engine:
- Initialized with a Brane `Insoe` and optional Ancestral Brane (AB) context
- `step()` method advances evaluation one step at a time
- `runToCompletion()` executes until all expressions are evaluated
- Maintains two contexts:
  - **AB (Ancestral Brane)**: Parent context for scoping
  - **IB (Immediate Brane)**: Current accumulated context

## Usage

### Using the UBC Programmatically

```java
import org.foolish.ast.AST;
import org.foolish.fvm.Insoe;
import org.foolish.ubc.*;

// Create a brane AST
AST.Brane brane = new AST.Brane(List.of(
    new AST.BinaryExpr("+",
        new AST.IntegerLiteral(10L),
        new AST.IntegerLiteral(20L)
    )
));

// Create UBC
Insoe insoe = new Insoe(brane);
UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);

// Run to completion
ubc.runToCompletion();

// Get result
BraneFiroe rootBrane = ubc.getRootBrane();
FIR result = rootBrane.getExpressionFiroes().get(0);
long value = result.getValue(); // 30
```

### Using the REPL

Run the interactive REPL:
```bash
mvn exec:java -Dexec.mainClass="org.foolish.ubc.UbcRepl"
```

Example session:
```
Foolish UBC REPL
Using Unicellular Brane Computer
Type Foolish expressions (Ctrl+D to exit)

{5;}
=> 5

{10 + 20;}
=> 30

{(5 + 3) * 2;}
=> 16

{1; 2; 3 + 4;}
=> 7
```

### Step-by-Step Execution

```java
UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);

// Step through execution
while (ubc.step()) {
    // Inspect state between steps if needed
    System.out.println("Still evaluating...");
}

System.out.println("Complete: " + ubc.isComplete());
```

## Design Principles

### Breadth-First Evaluation

The `braneMind` queue ensures expressions within a brane are evaluated breadth-first, matching the UBC design from the README:

1. BraneFiroe creates Expression Firoes from AST
2. Each Firoe is enqueued into braneMind
3. BraneFiroe steps through the queue
4. Nested expressions manage their own braneMind queues

### Finite Steps

Each `step()` call makes finite, deterministic progress:
- Initialization step: Convert AST to Firoes
- Operand creation steps: Create sub-expression Firoes
- Evaluation steps: Step nested expressions
- Final step: Compute result

### Value Extraction

Expression Firoes can extract values from completed sub-expressions:
```java
// Nested: (1 + 2) * 3
// - Inner BinaryFiroe evaluates to 3
// - Outer BinaryFiroe calls leftFiroe.getValue() to get 3
// - Result: 9
```

## Testing

Run UBC-specific tests:
```bash
mvn test -Dtest=UnicelluarBraneComputerTest
mvn test -Dtest=UbcReplTest
```

All tests:
```bash
mvn test
```

## Differences from FVM

| Feature | UBC | FVM |
|---------|-----|-----|
| Evaluation | Step-by-step breadth-first | Full recursive |
| Control | `step()` method | Single `evaluate()` |
| Contexts | AB + IB explicit | Environment implicit |
| Queue | braneMind per Firoe | No explicit queue |
| Result | Last expression value | Full Firoe tree |

## Future Enhancements

Planned additions to match full UBC specification:
- [ ] Identifier resolution with AB/IB contexts
- [ ] Assignment statements with environment updates
- [ ] Detachment brane support
- [ ] Complete SearchUp implementation with parent brane recursion
- [ ] Environment freezing for fully evaluated branes
- [ ] Abstract brane detection and handling

## See Also

- [Main README](../../../../../README.MD) - Full UBC specification
- [package-info.java](package-info.java) - Package documentation
- [UnicelluarBraneComputer.java](UnicelluarBraneComputer.java) - Main implementation
