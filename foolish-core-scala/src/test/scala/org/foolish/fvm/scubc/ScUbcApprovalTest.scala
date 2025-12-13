package org.foolish.fvm.scubc

import org.foolish.ApprovalTestRunner
import org.junit.jupiter.api.parallel.{Execution, ExecutionMode}
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.{Arguments, MethodSource}
import java.io.File
import java.util.stream.Stream

/**
 * Approval tests for the Scala UBC implementation.
 *
 * Reads .foo input files (shared with Java tests), executes them with Scala UBC,
 * and generates approval files in org/foolish/fvm/scubc/ for verification.
 */
@Execution(ExecutionMode.CONCURRENT)
class ScUbcApprovalTest:

  private val runner = ApprovalTestRunner(
    ScalaUbcInterpreter(),
    "org/foolish/fvm/inputs",  // Shared input directory
    "org/foolish/fvm/scubc"    // Scala approval output directory
  )

  @ParameterizedTest(name = "{1}")
  @MethodSource(Array("provideInputFiles"))
  def approvalTests(inputFile: File, testName: String): Unit =
    runner.runApprovalTest(inputFile, testName)

object ScUbcApprovalTest:
  @MethodSource
  def provideInputFiles(): Stream[Arguments] =
    val filter = System.getProperty("foolish.test.filter")
    var fileStream = ApprovalTestRunner.findInputFiles("org/foolish/fvm/inputs").stream()

    if (filter != null && !filter.isBlank) {
      fileStream = fileStream.filter(f => f.getName.contains(filter))
    }

    fileStream.map(file => Arguments.of(file, file.getName.replace(".foo", "")))
