package org.foolish.fvm.scubc

import io.circe.parser.decode
import io.circe.syntax.*
import org.scalatest.funsuite.AnyFunSuite
import org.foolish.fvm.scubc.Fir.given

// Phase 1, Test Layer 3: in-memory FIR -> JSON -> in-memory FIR round trip.
//
// Tests are inline (no .foo input file). For each FIR construction, assert that
// `fir == decode[Fir](fir.asJson.noSpaces).right.get`.
//
// Strategy: cover one case class per test, plus a few nested-tree tests.
// New FIR variants must add a roundtrip test here when introduced.
class FirRoundtripTest extends AnyFunSuite:

  private def roundtrip(fir: Fir): Unit =
    val json    = fir.asJson.noSpaces
    val decoded = decode[Fir](json).fold(err => fail(s"decode failed: $err\nJSON: $json"), identity)
    assert(decoded == fir, s"roundtrip mismatch for $fir\nJSON: $json")

  test("ConstantIntFir roundtrips") {
    roundtrip(ConstantIntFir(42))
  }

  test("NormalBraneFir empty roundtrips") {
    roundtrip(NormalBraneFir(characterizations = Nil, statements = Nil))
  }

  test("NormalBraneFir with one anonymous statement roundtrips") {
    val fir = NormalBraneFir(
      characterizations = Nil,
      statements = List(StatementFir(name = None, body = ConstantIntFir(42)))
    )
    roundtrip(fir)
  }

  // TODO: add a test per FIR variant as Phase 1 grows.
