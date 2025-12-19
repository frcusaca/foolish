package org.foolish.fvm.scubc

import org.foolish.ApprovalCrossValidationTest

/**
 * Integration test running Cross Validation in the Scala module.
 * Points to the Java resources in the sibling module.
 */
class ScUbcCrossValidationIT extends ApprovalCrossValidationTest:

  override protected def getJavaApprovalPath(): String =
    "../foolish-core-java/src/test/resources/org/foolish/fvm/ubc"
