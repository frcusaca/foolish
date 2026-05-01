package org.foolish.fvm.scubc

import org.foolish.UbcTester

// Bridges BraneComputer into the ApprovalTestRunner harness.
// Replace BraneComputer.evaluate() as phases are implemented; this file stays unchanged.
class FoolishInterpreter extends UbcTester:
  override def execute(code: String): String = BraneComputer.evaluate(code)
  override def getName: String = "Foolish Scala MVP"
