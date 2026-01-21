package org.foolish.fvm.scubc

sealed trait FiroeState

object FiroeState {
  case class Unknown() extends FiroeState
  case class Value(fir: FIR) extends FiroeState
  case class Constanic() extends FiroeState
}
