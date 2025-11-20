package org.foolish.fvm.scubc

import org.approvaltests.Approvals
import org.approvaltests.integrations.junit5.JupiterApprovals
import org.approvaltests.namer.ApprovalNamer
import org.approvaltests.writers.ApprovalTextWriter
import org.foolish.ApprovalTestRunner
import org.junit.jupiter.api.{DynamicTest, TestFactory}
import java.io.File
import java.nio.file.Files
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
        try {
          // Read input code
          val code = Files.readString(file.toPath).trim

          // Execute tester - produces complete .approved.foo content
          val interpreter = ScalaUbcInterpreter()
          val approvalContent = interpreter.execute(code)

          // Create custom namer for output location
          val namer = new CustomScUbcApprovalNamer("org.foolish.fvm.scubc", testName)

          // Verify with ApprovalTests using namer
          Approvals.verify(new ApprovalTextWriter(approvalContent, "foo"), namer)
        } catch {
          case e: Exception =>
            throw new RuntimeException(s"Failed to run approval test for $testName", e)
        }
      })
    }

  /**
   * Custom ApprovalNamer that places files in test resources by package.
   */
  private class CustomScUbcApprovalNamer(packageName: String, testName: String) extends ApprovalNamer {
    private var additionalInfo: String = ""

    override def getApprovalName(): String = testName

    override def getSourceFilePath(): String = {
      val packagePath = packageName.replace('.', File.separatorChar)
      "src" + File.separator + "test" + File.separator + "resources" + File.separator + packagePath
    }

    override def getApprovedFile(extensionWithDot: String): File =
      new File(getSourceFilePath(), getApprovalName() + ".approved" + extensionWithDot)

    override def getReceivedFile(extensionWithDot: String): File =
      new File(getSourceFilePath(), getApprovalName() + ".received" + extensionWithDot)

    override def addAdditionalInformation(info: String): ApprovalNamer = {
      this.additionalInfo = info
      this
    }

    override def getAdditionalInformation(): String = additionalInfo
  }
