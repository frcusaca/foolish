package org.foolish.fvm.scubc

import io.circe.{Decoder, Encoder}
import io.circe.generic.semiauto.{deriveDecoder, deriveEncoder}

// Foolish Internal Representation (FIR) — the compilation target of the Foolish source.
//
// FIRs are sealed Scala case classes representing structure plus a few primitive data
// types. Circe's generic derivation handles serialization. There is no schema to design
// — the contract is "round-trip via Circe": serialize a FIR to JSON, parse it back,
// the result must equal the original (`==` on case classes).
//
// Phase 1 scope: only the FIR types listed below. Phase 2 adds evaluation state
// transitions on these same types. Later phases add more FIR variants
// (ConcatenationFir, DetachmentFir, SystemOperatorFir, etc.).
//
// Lifecycle (the Nyes states — see 00_accumulated_specs.md for full definitions):
//   PREMBRYONIC → EMBRYONIC → BRANING → ECONSTANIC | WOCONSTANIC | CONSTANT | INDEPENDENT
//   NK is a separate terminal state for definitively-unresolvable cases (div-by-zero,
//   anchored search miss on a CONSTANT brane).
//
// Phase 1 emits these subset states only:
//   * EMBRYONIC — every non-literal FIR (work remains for Phase 2's evaluator)
//   * CONSTANT  — integer literals (already known at compile time)
//   * INDEPENDENT — reserved for literals that can never become CONSTANIC by recoordination
//   * NK        — the `???` literal

// -----------------------------------------------------------------------------
// Nyes — "Not Yet Evaluated states." The lifecycle the UBC steps through.
// -----------------------------------------------------------------------------

enum Nyes:
  case PREMBRYONIC                   // holds AST; atomic setup; pre-step
  case EMBRYONIC                     // search resolution / re-resolution in progress
  case BRANING                       // stepping children; waiting on constanic children
  case ECONSTANIC                    // search found nothing after due effort; needs new context to resolve
  case WOCONSTANIC                   // depends on at least one ECONSTANIC FIR
  case CONSTANT                      // fully evaluated, immutable; safe to share
  case INDEPENDENT                   // CONSTANT and immune to recoordination (literals only)
  case NK                            // definitively Not Known (div-by-zero, anchored miss on CONSTANT brane, depth exceeded)

object Nyes:
  given Encoder[Nyes] = Encoder.encodeString.contramap(_.toString)
  given Decoder[Nyes] = Decoder.decodeString.emap:
    case "PREMBRYONIC" => Right(PREMBRYONIC)
    case "EMBRYONIC"   => Right(EMBRYONIC)
    case "BRANING"     => Right(BRANING)
    case "ECONSTANIC"  => Right(ECONSTANIC)
    case "WOCONSTANIC" => Right(WOCONSTANIC)
    case "CONSTANT"    => Right(CONSTANT)
    case "INDEPENDENT" => Right(INDEPENDENT)
    case "NK"          => Right(NK)
    case other         => Left(s"Unknown Nyes: $other")

  // An FIR is "constanic" if it is in any of these states. See 00_accumulated_specs.md.
  def isConstanic(n: Nyes): Boolean = n match
    case ECONSTANIC | WOCONSTANIC | CONSTANT | INDEPENDENT => true
    case _                                                 => false

  // An FIR is "nigh" if it has not yet reached any constanic state.
  def isNigh(n: Nyes): Boolean = n match
    case PREMBRYONIC | EMBRYONIC | BRANING => true
    case _                                 => false

// -----------------------------------------------------------------------------
// FIR algebra — sealed hierarchy. All Phase 1 variants live here.
// -----------------------------------------------------------------------------

sealed trait Fir:
  def state: Nyes

// A literal integer value. Compiled directly to INDEPENDENT — it can never become
// constanic via recoordination (no context could ever change `42`).
case class ConstantIntFir(value: Long, state: Nyes = Nyes.INDEPENDENT) extends Fir

// A normal brane: ordered list of named or anonymous statements.
// Phase 1: NormalBrane is the only brane variant. ConcatenationBrane, DetachmentBrane
// come later.
case class NormalBraneFir(
  characterizations: List[String],
  statements:        List[StatementFir],
  state:             Nyes = Nyes.EMBRYONIC
) extends Fir

// A statement inside a brane. Anonymous statements have name = None (e.g., bare `42`
// inside `{42; x = 1}`).
case class StatementFir(
  name: Option[String],
  body: Fir,
  state: Nyes = Nyes.EMBRYONIC
)

// Binary operator expression. Phase 1: AST tree only — no arithmetic computation.
// Phase 2: evaluator collapses to ConstantIntFir if both operands are CONSTANT.
case class BinaryOpFir(
  op:    String,    // "+", "-", "*", "/", "%"
  left:  Fir,
  right: Fir,
  state: Nyes = Nyes.EMBRYONIC
) extends Fir

// Unary operator expression (e.g., `-42`).
case class UnaryOpFir(
  op:    String,    // "+", "-", "*"
  expr:  Fir,
  state: Nyes = Nyes.EMBRYONIC
) extends Fir

// Search operations. All searches in Phase 1 are pattern-based — bare identifiers like
// `a_config` compile to a regex of `^a_config$`. See FOOP-4.
//
// `anchored` distinguishes the two semantic flavors:
//   - false: unanchored (bare identifier). Searches IB, then walks up AB chain.
//            Not finding it produces ECONSTANIC.
//   - true:  anchored (`brane.name`, `brane?pat`, `brane^`, `brane$`, `brane#N`).
//            Searches only inside the specified brane. Not finding produces NK.
//
// Direction: Backward = from end toward start (default for `?` and `.`).
//            Forward  = from start toward end (used by `~` operator, deferred).
enum SearchDirection:
  case Backward
  case Forward

object SearchDirection:
  given Encoder[SearchDirection] = Encoder.encodeString.contramap(_.toString)
  given Decoder[SearchDirection] = Decoder.decodeString.emap:
    case "Backward" => Right(Backward)
    case "Forward"  => Right(Forward)
    case other      => Left(s"Unknown SearchDirection: $other")

case class SearchFir(
  pattern:   String,                     // regex pattern; bare names become "^name$" (FOOP-4)
  direction: SearchDirection,
  anchored:  Boolean,
  anchor:    Option[Fir],                // None for unanchored; Some(brane) for anchored
  state:     Nyes = Nyes.EMBRYONIC
) extends Fir

// Index access: `brane#N` (anchored) or `#-N` (unanchored seek).
case class IndexFir(
  index:    Int,
  anchored: Boolean,
  anchor:   Option[Fir],
  state:    Nyes = Nyes.EMBRYONIC
) extends Fir

// Head (`^`) and tail (`$`) — degenerate searches that always find the first/last.
case class HeadTailFir(
  isHead:   Boolean,                     // true = head (^), false = tail ($)
  anchored: Boolean,
  anchor:   Option[Fir],
  state:    Nyes = Nyes.EMBRYONIC
) extends Fir

// Identifier reference with characterizations — `type'name`. See FOOP-4.
case class CharacterizedRefFir(
  characterizations: List[String],
  pattern:           String,
  state:             Nyes = Nyes.EMBRYONIC
) extends Fir

// The literal `???` (NK from source).
case class NKFir(
  reason: String,
  state:  Nyes = Nyes.NK
) extends Fir

// -----------------------------------------------------------------------------
// Circe codecs.
// -----------------------------------------------------------------------------

object Fir:
  // Generic derivation handles every case class above. The `Fir` trait gets a
  // tagged-union encoding by default (a "type" discriminator field), which is
  // exactly what we want for round-trip.
  given Encoder[StatementFir] = deriveEncoder
  given Decoder[StatementFir] = deriveDecoder
  given Encoder[Fir]          = deriveEncoder
  given Decoder[Fir]          = deriveDecoder
