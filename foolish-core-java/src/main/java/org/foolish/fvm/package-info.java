/**
 * Unicellular Brane Computer (UBC) implementation.
 *
 * <p>The UBC package implements a step-by-step evaluation system for the Foolish
 * programming language based on the design specified in the README.MD. The UBC
 * represents a simple computational machine that processes branes (code blocks)
 * expression by expression using breadth-first evaluation.
 *
 * <p>A <strong>Foolisher</strong>, for lack of a better concise way to describe them,
 * is a person who develops Foolish. A Foolisher might say a variable is <em>nye</em>
 * (says 'nigh') when they encounter a FIR that has not yet been evaluated fully.
 *
 * <h3>Evaluation State Terminology</h3>
 * <ul>
 *   <li><strong><em>NYE</em></strong> (<em>Not Yet Evaluated</em>, pronounced "nigh") -
 *       Code that has been expressed but not fully evaluated in the current context</li>
 *   <li><strong><em>FK</em></strong> (<em>Fully Known</em>) -
 *       Values or branes that are completely evaluated and known</li>
 *   <li><strong><em>NK</em></strong> (<em>Not Knowable</em>, pronounced "no-no") -
 *       The unknown state represented by {@code ???} in Foolish</li>
 * </ul>
 *
 * <h2>Core Architecture</h2>
 *
 * <h3>FIR (Foolish Internal Representation)</h3>
 * <p>The {@link org.foolish.fvm.ubc.FIR} class hierarchy represents the internal
 * representation of computation:
 * <ul>
 *   <li>{@link org.foolish.fvm.ubc.FiroeWithBraneMind} - FIRs that have a braneMind
 *       queue for managing evaluation tasks (breadth-first execution)</li>
 *   <li>{@link org.foolish.fvm.ubc.FiroeWithoutBraneMind} - FIRs representing
 *       finalized values without evaluation queues</li>
 * </ul>
 *
 * <h3>Expression Types</h3>
 * <p>The following expression types are implemented:
 * <ul>
 *   <li>{@link org.foolish.fvm.ubc.ValueFiroe} - Represents finalized integer values</li>
 *   <li>{@link org.foolish.fvm.ubc.BraneFiroe} - Represents branes with multiple statements</li>
 *   <li>{@link org.foolish.fvm.ubc.BinaryFiroe} - Binary operations (+, -, *, /, etc.)</li>
 *   <li>{@link org.foolish.fvm.ubc.UnaryFiroe} - Unary operations (-, !)</li>
 *   <li>{@link org.foolish.fvm.ubc.IfFiroe} - Conditional if-then-else expressions</li>
 *   <li>{@link org.foolish.fvm.ubc.SearchUpFiroe} - Search-up (↑) operations</li>
 * </ul>
 *
 * <h3>Evaluation Model</h3>
 * <p>The {@link org.foolish.fvm.ubc.UnicelluarBraneComputer} manages evaluation:
 * <ul>
 *   <li>Initialized with a Brane {@link org.foolish.fvm.v1.Insoe} and optional
 *       Ancestral Brane (AB) context</li>
 *   <li>{@link org.foolish.fvm.ubc.UnicelluarBraneComputer#step()} advances evaluation
 *       one step at a time</li>
 *   <li>{@link org.foolish.fvm.ubc.UnicelluarBraneComputer#runToCompletion()} executes
 *       until all expressions are evaluated</li>
 *   <li>Returns frozen {@link org.foolish.fvm.Env} for fully evaluated branes</li>
 *   <li>Returns integer values for expression Firoes</li>
 * </ul>
 *
 * <h3>Key Methods</h3>
 * <ul>
 *   <li>{@code isNye()} - Returns true if additional steps would change the FIR (*NYE* - Not Yet Evaluated)</li>
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
 * @see org.foolish.fvm.ubc.UnicelluarBraneComputer
 * @see org.foolish.fvm.ubc.FIR
 * @since 1.0
 */
package org.foolish.fvm;
