package org.foolish.fvm.scubc

import org.foolish.ast.AST
import org.foolish.fvm.v1.Insoe
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Assertions.*
import scala.jdk.CollectionConverters.*

class SequencerTest:

  @Test
  def testSequencer4HumanWithSimpleInteger(): Unit =
    val brane = AST.Brane(List(AST.IntegerLiteral(5L)).asJava)
    val insoe = Insoe(brane)
    val ubc = UnicelluarBraneComputer(insoe)
    ubc.runToCompletion()
    val result = ubc.getRootBrane
    val output = Sequencer4Human().sequence(result)
    assertTrue(output.contains("{") && output.contains("}") && output.contains("5") && output.contains("＿"))

  @Test
  def testSequencer4HumanWithMultipleExpressions(): Unit =
    val brane = AST.Brane(List(AST.IntegerLiteral(1L), AST.IntegerLiteral(2L), AST.IntegerLiteral(3L)).asJava)
    val insoe = Insoe(brane)
    val ubc = UnicelluarBraneComputer(insoe)
    ubc.runToCompletion()
    val result = ubc.getRootBrane
    val output = Sequencer4Human().sequence(result)
    assertTrue(output.contains("1") && output.contains("2") && output.contains("3"))
    val lineCount = output.split("\n").length
    assertEquals(5, lineCount)

  @Test
  def testSequencer4HumanWithCustomTabCharacter(): Unit =
    val brane = AST.Brane(List(AST.IntegerLiteral(42L)).asJava)
    val insoe = Insoe(brane)
    val ubc = UnicelluarBraneComputer(insoe)
    ubc.runToCompletion()
    val result = ubc.getRootBrane
    val customSequencer = Sequencer4Human("  ")
    val output = customSequencer.sequence(result)
    assertTrue(output.contains("  42"))
    assertFalse(output.contains("＿"))

  @Test
  def testDefaultTabCharacter(): Unit =
    val sequencer = Sequencer4Human()
    assertEquals("＿", sequencer.getTabChar)
