package org.foolish.fvm.scubc

import org.foolish.ApprovalTestRunner
import org.junit.jupiter.api.{DynamicTest, TestFactory}
import java.util.stream.Stream
import scala.jdk.CollectionConverters.*

/**
 * Approval tests for the Scala UBC implementation.
 *
 * Reads .foo input files (shared with Java tests), executes them with Scala UBC,
 * and generates approval files in org/foolish/fvm/scubc/ for verification.
 */
class ScUbcApprovalTest:

  private val runner = ApprovalTestRunner(
    ScalaUbcInterpreter(),
    "org/foolish/fvm/inputs",  // Shared input directory
    "org/foolish/fvm/scubc"    // Scala approval output directory
  )

  @TestFactory
  def approvalTests(): Stream[DynamicTest] =
    val inputFiles = runner.findInputFiles()

    inputFiles.stream().map { file =>
      val testName = file.getName.replace(".foo", "")
      DynamicTest.dynamicTest(testName, () => {
        try
          runner.runApprovalTest(file, testName)
        catch
          case e: AssertionError =>
            // Re-throw with more descriptive message including test name
            throw new AssertionError(s"Test '$testName' failed: ${e.getMessage}", e)
      })
    }
