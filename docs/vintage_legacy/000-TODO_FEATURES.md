# TODO Items

Other features that we want to expand documentation on in the future:
* Implement detachment:
  * Detach : Matain the patterns in the FIR, or extend the memory class to hold patterns and matching logic. Test thoroughly.
  * Undetach: should be implemented by merging undetachment brane with subsequent detachment brane, shrinking into a Detach brane that is the difference. (Still maintain the AST though for future inspection)

* Implement brane concatenation

* Document the call-by-value versus call-by-reference semantics: Foolish is call by value, except for when branes are partially detached, in which case it behaves partially by reference and can take on different values.

* Document When does Foolishness become concrete and when is it still Foolish: The whole system, from bit stream, to foolish source code, to various implementations of the VM are all part of the Foolish programming ecosystem.

* Characterization
  * Characterize some names as system-generated, e.g. branching.
* Conditional
* Loops, Recursions, etc.
* Traceability.
* Enhanced refactoring and intervention in computation.
* Restatable programs for better comprehension and compression.
* Generation of programs.
* Program differentiation.
* Program hyper-optimization.
* AI adaptations to support Foolish language.
* Human-in-the-loop programming.
* Mutable branes. There is not much one can do to a brane. Given any line in the brane: it can be
  deleted, appended, prepended, name changed, or value changed.
* Translation versus Transformation.
* Specify the "Thought Block": how to stop evaluation.
* Native AI facilities: brane2vec, brane2tok, BraneMemory, etc.
* Homomorphic encryption and equivalent "thought block" builtin.
* Asynchronized execution.
* Real-time computations.
* Built-in masking of data from compute.
* Library ecosystem for interactivity with Python/Java/C++/Rust/Go/etc. Other popular open systems
  such as PyTorch, R, Octave, Apache systems such as Ray, Spark, Arrow, standards such as JSON,
  XML, HTML, PPML, ONNX, Protocol Buffers, etc.
