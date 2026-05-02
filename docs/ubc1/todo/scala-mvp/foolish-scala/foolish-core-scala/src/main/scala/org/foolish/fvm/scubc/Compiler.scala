package org.foolish.fvm.scubc

import io.circe.Printer
import io.circe.syntax.*
import org.antlr.v4.runtime.{CharStreams, CommonTokenStream}
import org.foolish.ast.{AST, ASTBuilder}
import org.foolish.grammar.{FoolishLexer, FoolishParser}

// Phase 1 deliverable: a compiler that turns Foolish source into serialized FIR JSON.
//
// Pipeline: source -> ANTLR parse tree -> Java AST -> Scala AST (FoolishAstn)
//        -> FIR (sealed Scala case classes) -> JSON via Circe
//
// No evaluation happens here. Every FIR comes out in `Initialized` state, except
// literal integers which are `Constant` directly (per design: "42 should compile to
// Constant(42)") and `???` which is `NK`.
//
// Phase 2 will read the JSON back, walk the FIR tree, and step states forward.
object Compiler:

  // Convenience: parse a Foolish source string into the Java AST.
  def parse(source: String): AST.Program =
    val input  = CharStreams.fromString(source)
    val lexer  = new FoolishLexer(input)
    val tokens = new CommonTokenStream(lexer)
    val parser = new FoolishParser(tokens)
    new ASTBuilder().visitProgram(parser.program()).asInstanceOf[AST.Program]

  // Phase 1 entry point: source -> serialized FIR JSON.
  def compileToJson(source: String): String =
    val program  = parse(source)
    val scalaAst = FoolishAst.fromProgram(program)
    val fir      = compileAstToFir(scalaAst)
    Printer.spaces2.print(fir.asJson)

  // Compile a Scala AST tree to a FIR tree. Stub for P1b — fill in per phase.
  // P1c+ will build this out node-by-node:
  //   IntLitAstn       -> ConstantIntFir
  //   BraneAstn        -> NormalBraneFir
  //   AssignmentAstn   -> StatementFir(name, body)
  //   IdentifierAstn   -> SearchFir(pattern = "^id$", Backward, anchored = false, None)
  //   BinaryExprAstn   -> BinaryOpFir
  //   ... etc
  def compileAstToFir(ast: ProgramAstn): Fir =
    // Stub: every Phase 1 implementation step replaces a `???` here with a real translation.
    NKFir(reason = "compiler not yet implemented")
