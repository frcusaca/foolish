/**
 * Unicellular Brane Computer (UBC) implementation.
 *
 * <p>The UBC package implements a step-by-step evaluation system for the Foolish
 * programming language based on the design specified in the README.MD. The UBC
 * represents a simple computational machine that processes branes (code blocks)
 * expression by expression using breadth-first evaluation.
 *
 * <h2>Core Architecture</h2>
 *
 * <h3>FIR (Foolish Internal Representation)</h3>
 * <p>The {@link org.foolish.ubc.FIR} class hierarchy represents the internal
 * representation of computation:
 * <ul>
 *   <li>{@link org.foolish.ubc.FiroeWithBraneMind} - FIRs that have a braneMind
 *       queue for managing evaluation tasks (breadth-first execution)</li>
 *   <li>{@link org.foolish.ubc.FiroeWithoutBraneMind} - FIRs representing
 *       finalized values without evaluation queues</li>
 * </ul>
 *
 * <h3>Expression Types</h3>
 * <p>The following expression types are implemented:
 * <ul>
 *   <li>{@link org.foolish.ubc.ValueFiroe} - Represents finalized integer values</li>
 *   <li>{@link org.foolish.ubc.BraneFiroe} - Represents branes with multiple statements</li>
 *   <li>{@link org.foolish.ubc.BinaryFiroe} - Binary operations (+, -, *, /, etc.)</li>
 *   <li>{@link org.foolish.ubc.UnaryFiroe} - Unary operations (-, !)</li>
 *   <li>{@link org.foolish.ubc.IfFiroe} - Conditional if-then-else expressions</li>
 *   <li>{@link org.foolish.ubc.SearchUpFiroe} - Search-up (↑) operations</li>
 * </ul>
 *
 * <h3>Evaluation Model</h3>
 * <p>The {@link org.foolish.ubc.UnicelluarBraneComputer} manages evaluation:
 * <ul>
 *   <li>Initialized with a Brane {@link org.foolish.fvm.Insoe} and optional
 *       Ancestral Brane (AB) context</li>
 *   <li>{@link org.foolish.ubc.UnicelluarBraneComputer#step()} advances evaluation
 *       one step at a time</li>
 *   <li>{@link org.foolish.ubc.UnicelluarBraneComputer#runToCompletion()} executes
 *       until all expressions are evaluated</li>
 *   <li>Returns frozen {@link org.foolish.fvm.Env} for fully evaluated branes</li>
 *   <li>Returns integer values for expression Firoes</li>
 * </ul>
 *
 * <h3>Key Methods</h3>
 * <ul>
 *   <li>{@code underevaluated()} - Returns true if additional steps would change the FIR</li>
 *   <li>{@code isAbstract()} - Returns true if there are unbound identifiers</li>
 *   <li>{@code getValue()} - Gets the integer value from evaluated expressions</li>
 *   <li>{@code getEnvironment()} - Gets the frozen Env from evaluated branes</li>
 * </ul>
 *
 * <h2>Design Principles</h2>
 * <ul>
 *   <li><strong>Breadth-First Evaluation</strong>: The braneMind queue ensures
 *       expressions are evaluated breadth-first within each brane</li>
 *   <li><strong>Step-by-Step Processing</strong>: Each {@code step()} call makes
 *       finite progress, enabling controlled execution</li>
 *   <li><strong>Two-Context Model</strong>: Ancestral Brane (AB) and Immediate
 *       Brane (IB) contexts support proper scoping</li>
 *   <li><strong>Lazy Initialization</strong>: BraneFiroe initializes on first step,
 *       creating expression Firoes from AST</li>
 * </ul>
 *
 * @see org.foolish.ubc.UnicelluarBraneComputer
 * @see org.foolish.ubc.FIR
 * @since 1.0
 */
package org.foolish.ubc;
