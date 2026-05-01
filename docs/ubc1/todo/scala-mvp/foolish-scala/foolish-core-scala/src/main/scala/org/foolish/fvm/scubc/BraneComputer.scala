package org.foolish.fvm.scubc

import org.antlr.v4.runtime.{CharStreams, CommonTokenStream}
import org.foolish.ast.{AST, ASTBuilder}
import org.foolish.grammar.{FoolishLexer, FoolishParser}

// Entry point for Foolish evaluation.
// P1b stub: parses source, converts to Scala AST, returns placeholder output.
// Replace the body of evaluate() as each phase is implemented.
object BraneComputer:

  def parse(source: String): AST.Program =
    val input  = CharStreams.fromString(source)
    val lexer  = new FoolishLexer(input)
    val tokens = new CommonTokenStream(lexer)
    val parser = new FoolishParser(tokens)
    new ASTBuilder().visitProgram(parser.program()).asInstanceOf[AST.Program]

  def evaluate(source: String): String =
    val javaAst = parse(source)
    val program = FoolishAst.fromProgram(javaAst)
    // TODO: implement evaluation — see 01_implementation_schedule.md P2 onward
    s"!!INPUT!!\n$source\n\n!!!\nNOT IMPLEMENTED YET\n!!!\n"
