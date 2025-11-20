package org.foolish.fvm.scubc

import org.antlr.v4.runtime.{CharStream, CharStreams, CommonTokenStream}
import org.foolish.UbcTester
import org.foolish.ast.{AST, ASTBuilder, ASTFormatter}
import org.foolish.grammar.{FoolishLexer, FoolishParser}
import scala.jdk.CollectionConverters.*

/**
 * Scala implementation of the UBC tester.
 * Produces complete .approved.foo files with input code and test results.
 */
class ScalaUbcInterpreter extends UbcTester:

  override def execute(code: String): String =
    // Parse the code
    val input: CharStream = CharStreams.fromString(code)
    val lexer = FoolishLexer(input)
    val tokens = CommonTokenStream(lexer)
    val parser = FoolishParser(tokens)
    val tree = parser.program()
    val ast = ASTBuilder().visit(tree)

    // Extract first brane
    val program = ast.asInstanceOf[AST.Program]
    val branes = program.branes()
    val firstBrane = branes.branes().asScala.head
    val brane = firstBrane match
      case b: AST.Brane => b
      case _ => throw RuntimeException(s"Expected a brane but got: ${firstBrane.getClass}")

    // Create UBC and run
    val ubc = UnicelluarBraneComputer(brane)
    val stepCount = ubc.runToCompletion()
    val finalResult = ubc.getRootBrane

    // Format as complete .foo file
    val output = StringBuilder()
    output.append("!!INPUT!!\n")
    output.append(code).append("\n\n")
    output.append("!!!\n")

    output.append("PARSED AST:\n")
    output.append(ASTFormatter().format(ast)).append("\n\n")

    output.append("UBC EVALUATION:\n")
    output.append(s"Steps taken: $stepCount\n\n")

    output.append("FINAL RESULT:\n")
    output.append(Sequencer4Human().sequence(finalResult)).append("\n\n")

    output.append("COMPLETION STATUS:\n")
    output.append(s"Complete: ${ubc.isComplete}")

    output.append("\n!!!\n")

    output.toString

  override def getName: String = "Scala UBC"
