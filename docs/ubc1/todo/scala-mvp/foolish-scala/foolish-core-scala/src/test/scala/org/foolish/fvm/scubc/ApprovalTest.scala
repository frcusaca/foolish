package org.foolish.fvm.scubc

import org.foolish.ApprovalTestRunner
import org.junit.jupiter.api.parallel.{Execution, ExecutionMode}
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.{Arguments, MethodSource}
import java.io.File
import java.util.stream.Stream

// Approval test harness for the Foolish Scala MVP evaluator.
//
// Discovers all .foo files under src/test/resources/org/foolish/fvm/inputs/,
// runs each through FoolishInterpreter, and compares to
// src/test/resources/org/foolish/fvm/scubc/<name>.approved.foo.
//
// To run a single test filtered by input file name substring:
//   mvn test -Dtest=ApprovalTest -Dfoolish.test.filter=Shadow
//
// Approval workflow (never bulk-approve — review each diff):
//   diff -y --color foo.received.foo foo.approved.foo
//   mv foo.received.foo foo.approved.foo
@Execution(ExecutionMode.CONCURRENT)
class ApprovalTest:

  private val runner = ApprovalTestRunner(
    FoolishInterpreter(),
    "org/foolish/fvm/inputs",
    "org/foolish/fvm/scubc"
  )

  @ParameterizedTest(name = "{1}")
  @MethodSource(Array("provideInputFiles"))
  def approvalTests(inputFile: File, testName: String): Unit =
    runner.runApprovalTest(inputFile, testName)

object ApprovalTest:
  @MethodSource
  def provideInputFiles(): Stream[Arguments] =
    val filter = System.getProperty("foolish.test.filter")
    var files  = ApprovalTestRunner.findInputFiles("org/foolish/fvm/inputs").stream()
    if filter != null && !filter.isBlank then
      files = files.filter(f => f.getName.contains(filter))
    files.map(f => Arguments.of(f, f.getName.replace(".foo", "")))
