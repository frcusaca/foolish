package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}
// Scala 3 / Java sealed-record interop quirk: AST.Concatenation cannot be referenced
// via the qualified path; it must be imported by name. Other AST.* nested records work
// either way. Do not "clean up" this import — removing it breaks the build.
import org.foolish.ast.AST.Concatenation
import scala.jdk.CollectionConverters.*

// Scala sealed hierarchy — convert from Java AST once at entry, evaluate entirely in Scala.
//
// Naming convention: every Scala case class begins with `F` (for Foolish) to avoid
// shadowing the Java AST.* nested types in pattern matches. Without this, Scala 3
// resolves bare names like `Identifier` to the local case class and Java pattern
// matching breaks. Do NOT rename them to match the Java side.
//
// See 02_implementor_reference.md for the full node inventory.

sealed trait FoolishExpr

case class FIntLit(value: Long)                                                extends FoolishExpr
case class FIdentifier(characterizations: List[String], id: String)            extends FoolishExpr
case class FBrane(characterizations: List[String], stmts: List[FoolishExpr])   extends FoolishExpr
case class FBinaryExpr(op: String, left: FoolishExpr, right: FoolishExpr)      extends FoolishExpr
case class FUnaryExpr(op: String, expr: FoolishExpr)                           extends FoolishExpr
case class FConcatenation(elements: List[FoolishExpr])                         extends FoolishExpr
case class FAssignment(id: FIdentifier, op: FAssignOp, rhs: FoolishExpr)       extends FoolishExpr
case class FDotSearch(anchor: FoolishExpr, name: FIdentifier)                  extends FoolishExpr
case class FRegexSearch(anchor: FoolishExpr, op: SearchOperator, pattern: String) extends FoolishExpr
case class FIndexAccess(anchor: FoolishExpr, index: Int)                       extends FoolishExpr
case class FOneShotSearch(anchor: FoolishExpr, op: SearchOperator)             extends FoolishExpr
case class FUnanchoredSeek(offset: Int)                                        extends FoolishExpr  // offset already negative
case object FNKLit                                                              extends FoolishExpr  // ??? literal
case class FTopLevel(branes: List[FoolishExpr])                                extends FoolishExpr
case class FNotImplemented(reason: String)                                     extends FoolishExpr  // MVP3-deferred nodes

enum FAssignOp:
  case Normal
  case TailSugar   // id =$ rhs
  case HeadSugar   // id =^ rhs

case class FProgram(expr: FTopLevel)

object FoolishAst:

  def fromJava(expr: AST.Expr): FoolishExpr = expr match
    case lit: AST.IntegerLiteral      => FIntLit(lit.value())
    case id: AST.Identifier           => FIdentifier(id.characterizations().asScala.toList, id.id())
    case br: AST.Brane                => FBrane(br.characterizations().asScala.toList,
                                                 br.statements().asScala.toList.map(fromJava))
    case bin: AST.BinaryExpr          => FBinaryExpr(bin.op(), fromJava(bin.left()), fromJava(bin.right()))
    case un: AST.UnaryExpr            => FUnaryExpr(un.op(), fromJava(un.expr()))
    case cat: Concatenation           => FConcatenation(cat.elements().asScala.toList.map(fromJava))
    case deref: AST.DereferenceExpr   =>
      val name = fromJava(deref.coordinate()).asInstanceOf[FIdentifier]
      FDotSearch(fromJava(deref.anchor()), name)
    case re: AST.RegexpSearchExpr     => FRegexSearch(fromJava(re.anchor()), re.operator(), re.pattern())
    case seek: AST.SeekExpr           => FIndexAccess(fromJava(seek.anchor()), seek.offset())
    case us: AST.UnanchoredSeekExpr   => FUnanchoredSeek(us.offset())
    case os: AST.OneShotSearchExpr    => FOneShotSearch(fromJava(os.anchor()), os.operator())
    case _: AST.UnknownExpr           => FNKLit
    case asgn: AST.Assignment         =>
      val rhs = fromJava(asgn.expr())
      val op  = asgn.expr() match
        case oneShot: AST.OneShotSearchExpr =>
          oneShot.operator() match
            case SearchOperator.TAIL => FAssignOp.TailSugar
            case SearchOperator.HEAD => FAssignOp.HeadSugar
            case _                   => FAssignOp.Normal
        case _ => FAssignOp.Normal
      FAssignment(fromJava(asgn.identifier()).asInstanceOf[FIdentifier], op, rhs)
    case branes: AST.Branes           => FTopLevel(branes.branes().asScala.toList.map(fromJava))
    // MVP3-deferred nodes — preserved as markers so the evaluator can produce a
    // structured "not yet implemented" result without crashing the harness.
    case _: AST.IfExpr                => FNotImplemented("if-then-else removed in UBC2")
    case _: AST.DetachmentBrane       => FNotImplemented("detachment deferred to MVP3")
    case _: AST.SearchUP              => FNotImplemented("↑ search deferred to MVP3")
    case _: AST.StayFoolishExpr       => FNotImplemented("SF marker deferred to MVP3")
    case _: AST.StayFullyFoolishExpr  => FNotImplemented("SFF marker deferred to MVP3")

  def fromProgram(p: AST.Program): FProgram =
    FProgram(fromJava(p.branes()).asInstanceOf[FTopLevel])
