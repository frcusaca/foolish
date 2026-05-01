package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}
import scala.jdk.CollectionConverters.*

// Scala sealed hierarchy — convert from Java AST once at entry, evaluate entirely in Scala.
// See 02_implementor_reference.md for the full node inventory and conversion sketch.

sealed trait FoolishExpr

case class IntLit(value: Long)                                              extends FoolishExpr
case class Identifier(characterizations: List[String], id: String)         extends FoolishExpr
case class Brane(characterizations: List[String], stmts: List[FoolishExpr]) extends FoolishExpr
case class BinaryExpr(op: String, left: FoolishExpr, right: FoolishExpr)   extends FoolishExpr
case class UnaryExpr(op: String, expr: FoolishExpr)                        extends FoolishExpr
case class Concatenation(elements: List[FoolishExpr])                      extends FoolishExpr
case class Assignment(id: Identifier, op: AssignOp, rhs: FoolishExpr)     extends FoolishExpr
case class DotSearch(anchor: FoolishExpr, name: Identifier)                extends FoolishExpr
case class RegexSearch(anchor: FoolishExpr, op: SearchOperator, pattern: String) extends FoolishExpr
case class IndexAccess(anchor: FoolishExpr, index: Int)                    extends FoolishExpr
case class OneShotSearch(anchor: FoolishExpr, op: SearchOperator)          extends FoolishExpr
case class UnanchoredSeek(offset: Int)                                     extends FoolishExpr  // offset is negative
case object NKLit                                                           extends FoolishExpr  // ??? literal
case class TopLevel(branes: List[FoolishExpr])                             extends FoolishExpr
case class NotImplemented(reason: String)                                  extends FoolishExpr  // deferred MVP3 nodes

enum AssignOp:
  case Normal
  case TailSugar   // id =$ rhs
  case HeadSugar   // id =^ rhs

case class Program(expr: TopLevel)

object FoolishAst:
  def fromJava(expr: AST.Expr): FoolishExpr = expr match
    case lit: AST.IntegerLiteral      => IntLit(lit.value())
    case id: AST.Identifier           => Identifier(id.characterizations().asScala.toList, id.id())
    case br: AST.Brane                => Brane(br.characterizations().asScala.toList,
                                               br.statements().asScala.toList.map(fromJava))
    case bin: AST.BinaryExpr          => BinaryExpr(bin.op(), fromJava(bin.left()), fromJava(bin.right()))
    case un: AST.UnaryExpr            => UnaryExpr(un.op(), fromJava(un.expr()))
    case cat: AST.Concatenation       => Concatenation(cat.elements().asScala.toList.map(fromJava))
    case deref: AST.DereferenceExpr   => DotSearch(fromJava(deref.anchor()),
                                                    fromJava(deref.coordinate()).asInstanceOf[Identifier])
    case re: AST.RegexpSearchExpr     => RegexSearch(fromJava(re.anchor()), re.operator(), re.pattern())
    case seek: AST.SeekExpr           => IndexAccess(fromJava(seek.anchor()), seek.offset())
    case us: AST.UnanchoredSeekExpr   => UnanchoredSeek(us.offset())
    case os: AST.OneShotSearchExpr    => OneShotSearch(fromJava(os.anchor()), os.operator())
    case _: AST.UnknownExpr           => NKLit
    case asgn: AST.Assignment         =>
      val rhs = fromJava(asgn.expr())
      val op  = if asgn.expr().isInstanceOf[AST.OneShotSearchExpr] then
        asgn.expr().asInstanceOf[AST.OneShotSearchExpr].operator() match
          case SearchOperator.TAIL => AssignOp.TailSugar
          case SearchOperator.HEAD => AssignOp.HeadSugar
          case _                   => AssignOp.Normal
      else AssignOp.Normal
      Assignment(fromJava(asgn.identifier()).asInstanceOf[Identifier], op, rhs)
    case branes: AST.Branes           => TopLevel(branes.branes().asScala.toList.map(fromJava))
    // MVP3 — not implemented yet
    case _: AST.IfExpr                => NotImplemented("if-then-else removed in UBC2")
    case _: AST.DetachmentBrane       => NotImplemented("detachment deferred to MVP3")
    case _: AST.SearchUP              => NotImplemented("↑ search deferred to MVP3")
    case _: AST.StayFoolishExpr       => NotImplemented("SF marker deferred to MVP3")
    case _: AST.StayFullyFoolishExpr  => NotImplemented("SFF marker deferred to MVP3")

  def fromProgram(p: AST.Program): Program =
    Program(fromJava(p.branes()).asInstanceOf[TopLevel])
