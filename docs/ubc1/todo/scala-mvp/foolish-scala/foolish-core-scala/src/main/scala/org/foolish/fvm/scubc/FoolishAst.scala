package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}
// Scala 3 / Java sealed-record interop quirk: AST.Concatenation cannot be referenced
// via the qualified path; it must be imported by name. Other AST.* nested records work
// either way. Do not "clean up" this import — removing it breaks the build.
import org.foolish.ast.AST.Concatenation
import scala.jdk.CollectionConverters.*

// Scala sealed hierarchy — convert from Java AST once at entry, evaluate entirely in Scala.
//
// Naming convention: every Scala case class ends with `Astn` (AST Node) to avoid
// shadowing the Java AST.* nested types in pattern matches. Without this, Scala 3
// resolves bare names like `Identifier` to the local case class and Java pattern
// matching breaks. Do NOT rename them to match the Java side.
//
// See 02_implementor_reference.md for the full node inventory.

sealed trait FoolishAstn

case class IntLitAstn(value: Long)                                                  extends FoolishAstn
case class IdentifierAstn(characterizations: List[String], id: String)              extends FoolishAstn
case class BraneAstn(characterizations: List[String], stmts: List[FoolishAstn])     extends FoolishAstn
case class BinaryExprAstn(op: String, left: FoolishAstn, right: FoolishAstn)        extends FoolishAstn
case class UnaryExprAstn(op: String, expr: FoolishAstn)                             extends FoolishAstn
case class ConcatenationAstn(elements: List[FoolishAstn])                           extends FoolishAstn
case class AssignmentAstn(id: IdentifierAstn, op: AssignOpAstn, rhs: FoolishAstn)   extends FoolishAstn
case class DotSearchAstn(anchor: FoolishAstn, name: IdentifierAstn)                 extends FoolishAstn
case class RegexSearchAstn(anchor: FoolishAstn, op: SearchOperator, pattern: String) extends FoolishAstn
case class IndexAccessAstn(anchor: FoolishAstn, index: Int)                         extends FoolishAstn
case class OneShotSearchAstn(anchor: FoolishAstn, op: SearchOperator)               extends FoolishAstn
case class UnanchoredSeekAstn(offset: Int)                                          extends FoolishAstn  // offset already negative
case object NKLitAstn                                                                extends FoolishAstn  // ??? literal
case class TopLevelAstn(branes: List[FoolishAstn])                                  extends FoolishAstn
case class NotImplementedAstn(reason: String)                                       extends FoolishAstn  // MVP3-deferred nodes

enum AssignOpAstn:
  case Normal
  case TailSugar   // id =$ rhs
  case HeadSugar   // id =^ rhs

case class ProgramAstn(expr: TopLevelAstn)

object FoolishAst:

  def fromJava(expr: AST.Expr): FoolishAstn = expr match
    case lit: AST.IntegerLiteral      => IntLitAstn(lit.value())
    case id: AST.Identifier           => IdentifierAstn(id.characterizations().asScala.toList, id.id())
    case br: AST.Brane                => BraneAstn(br.characterizations().asScala.toList,
                                                    br.statements().asScala.toList.map(fromJava))
    case bin: AST.BinaryExpr          => BinaryExprAstn(bin.op(), fromJava(bin.left()), fromJava(bin.right()))
    case un: AST.UnaryExpr            => UnaryExprAstn(un.op(), fromJava(un.expr()))
    case cat: Concatenation           => ConcatenationAstn(cat.elements().asScala.toList.map(fromJava))
    case deref: AST.DereferenceExpr   =>
      val name = fromJava(deref.coordinate()).asInstanceOf[IdentifierAstn]
      DotSearchAstn(fromJava(deref.anchor()), name)
    case re: AST.RegexpSearchExpr     => RegexSearchAstn(fromJava(re.anchor()), re.operator(), re.pattern())
    case seek: AST.SeekExpr           => IndexAccessAstn(fromJava(seek.anchor()), seek.offset())
    case us: AST.UnanchoredSeekExpr   => UnanchoredSeekAstn(us.offset())
    case os: AST.OneShotSearchExpr    => OneShotSearchAstn(fromJava(os.anchor()), os.operator())
    case _: AST.UnknownExpr           => NKLitAstn
    case asgn: AST.Assignment         =>
      val rhs = fromJava(asgn.expr())
      val op  = asgn.expr() match
        case oneShot: AST.OneShotSearchExpr =>
          oneShot.operator() match
            case SearchOperator.TAIL => AssignOpAstn.TailSugar
            case SearchOperator.HEAD => AssignOpAstn.HeadSugar
            case _                   => AssignOpAstn.Normal
        case _ => AssignOpAstn.Normal
      AssignmentAstn(fromJava(asgn.identifier()).asInstanceOf[IdentifierAstn], op, rhs)
    case branes: AST.Branes           => TopLevelAstn(branes.branes().asScala.toList.map(fromJava))
    // MVP3-deferred nodes — preserved as markers so the evaluator can produce a
    // structured "not yet implemented" result without crashing the harness.
    case _: AST.IfExpr                => NotImplementedAstn("if-then-else removed in UBC2")
    case _: AST.DetachmentBrane       => NotImplementedAstn("detachment deferred to MVP3")
    case _: AST.SearchUP              => NotImplementedAstn("↑ search deferred to MVP3")
    case _: AST.StayFoolishExpr       => NotImplementedAstn("SF marker deferred to MVP3")
    case _: AST.StayFullyFoolishExpr  => NotImplementedAstn("SFF marker deferred to MVP3")

  def fromProgram(p: AST.Program): ProgramAstn =
    ProgramAstn(fromJava(p.branes()).asInstanceOf[TopLevelAstn])
