package org.foolish.fvm.scubc

import org.foolish.ast.AST
import org.foolish.fvm.v1.Insoe
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Assertions.*
import scala.jdk.CollectionConverters.*

class UnicelluarBraneComputerTest:

  @Test
  def testSimpleIntegerBrane(): Unit =
    val brane = AST.Brane(List(AST.IntegerLiteral(1L), AST.IntegerLiteral(2L), AST.IntegerLiteral(3L)).asJava)
    val insoe = Insoe(brane)
    val ubc = UnicelluarBraneComputer(insoe)
    val steps = ubc.runToCompletion()
    assertTrue(ubc.isComplete)
    assertTrue(steps >= 0)

  @Test
  def testBinaryExpression(): Unit =
    val expr = AST.BinaryExpr("+", AST.IntegerLiteral(1L), AST.IntegerLiteral(2L))
    val brane = AST.Brane(List(expr).asJava)
    val insoe = Insoe(brane)
    val ubc = UnicelluarBraneComputer(insoe)
    ubc.runToCompletion()
    assertTrue(ubc.isComplete)
    val rootBrane = ubc.getRootBrane
    val expressions = rootBrane.getExpressionFiroes
    assertEquals(1, expressions.size)
    val binaryFiroe = expressions.head.asInstanceOf[BinaryFiroe]
    assertEquals(3L, binaryFiroe.getValue)

  @Test
  def testUnaryExpression(): Unit =
    val expr = AST.UnaryExpr("-", AST.IntegerLiteral(5L))
    val brane = AST.Brane(List(expr).asJava)
    val insoe = Insoe(brane)
    val ubc = UnicelluarBraneComputer(insoe)
    ubc.runToCompletion()
    assertTrue(ubc.isComplete)
    val unaryFiroe = ubc.getRootBrane.getExpressionFiroes.head.asInstanceOf[UnaryFiroe]
    assertEquals(-5L, unaryFiroe.getValue)

  @Test
  def testNestedBinaryExpression(): Unit =
    val innerExpr = AST.BinaryExpr("+", AST.IntegerLiteral(1L), AST.IntegerLiteral(2L))
    val outerExpr = AST.BinaryExpr("*", innerExpr, AST.IntegerLiteral(3L))
    val brane = AST.Brane(List(outerExpr).asJava)
    val insoe = Insoe(brane)
    val ubc = UnicelluarBraneComputer(insoe)
    ubc.runToCompletion()
    assertTrue(ubc.isComplete)
    val binaryFiroe = ubc.getRootBrane.getExpressionFiroes.head.asInstanceOf[BinaryFiroe]
    assertEquals(9L, binaryFiroe.getValue)

  @Test
  def testStepByStep(): Unit =
    val expr = AST.BinaryExpr("+", AST.IntegerLiteral(10L), AST.IntegerLiteral(20L))
    val brane = AST.Brane(List(expr).asJava)
    val insoe = Insoe(brane)
    val ubc = UnicelluarBraneComputer(insoe)
    assertFalse(ubc.isComplete)
    var stepCount = 0
    while ubc.step() do
      stepCount += 1
      if stepCount > 100 then fail("Too many steps")
    assertTrue(ubc.isComplete)
    assertTrue(stepCount > 0)
