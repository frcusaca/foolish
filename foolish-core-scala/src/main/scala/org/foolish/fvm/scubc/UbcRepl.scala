package org.foolish.fvm.scubc

import org.antlr.v4.runtime.{CharStream, CharStreams, CommonTokenStream}
import org.foolish.ast.{AST, ASTBuilder}
import org.foolish.fvm.Env
import org.foolish.fvm.v1.Insoe
import org.foolish.grammar.{FoolishLexer, FoolishParser}
import scala.io.StdIn
import scala.util.{Try, Success, Failure}

/**
 * Simple read-eval-print loop for the Foolish language using the UBC.
 * This REPL uses the Unicellular Brane Computer for evaluation.
 */
object UbcRepl:

  /** Parse the provided source into an AST program */
  def parse(source: String): AST.Program =
    val input: CharStream = CharStreams.fromString(source)
    val lexer = FoolishLexer(input)
    val tokens = CommonTokenStream(lexer)
    val parser = FoolishParser(tokens)
    parser.removeErrorListeners()
    parser.addErrorListener(org.antlr.v4.runtime.ConsoleErrorListener())
    ASTBuilder().visitProgram(parser.program()).asInstanceOf[AST.Program]

  /** Evaluate the given source using UBC, returning the result */
  def eval(source: String, env: Env): Option[BraneFiroe] =
    val ast = parse(source)

    // Extract the brane from the program
    val branes = ast.branes()
    if branes == null || branes.branes().isEmpty then
      return None

    // Get the first brane to evaluate
    import scala.jdk.CollectionConverters.*
    val firstBrane = branes.branes().asScala.headOption
    firstBrane match
      case Some(brane: AST.Brane) =>
        // Create UBC and evaluate
        val braneInsoe = Insoe(brane)
        val ubc = UnicelluarBraneComputer(braneInsoe, env)

        // Run to completion
        ubc.runToCompletion()

        // Return the whole BraneFiroe (the evaluated brane)
        Some(ubc.getRootBrane)

      case _ => None

  def main(args: Array[String]): Unit =
    println("Foolish UBC REPL")
    println("Using Unicellular Brane Computer")
    println("Type Foolish expressions (Ctrl+D to exit)")
    println()

    val env = Env()
    val debug = args.contains("--debug")

    Iterator.continually(Option(StdIn.readLine()))
      .takeWhile(_.isDefined)
      .map(_.get)
      .filterNot(_.isBlank)
      .foreach { line =>
        Try(eval(line, env)) match
          case Success(Some(result)) =>
            println(s"=> $result")
          case Success(None) =>
            () // No output
          case Failure(e) =>
            System.err.println(s"Error: ${e.getMessage}")
            if debug then
              e.printStackTrace()
      }
