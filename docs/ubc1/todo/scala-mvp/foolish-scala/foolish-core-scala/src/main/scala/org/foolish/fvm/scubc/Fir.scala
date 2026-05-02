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
// Lifecycle of a FIR:
//   * Phase 1 (compilation): every FIR starts in `Initialized` state. No computation.
//   * Phase 2 (UBC stepping): the evaluator transitions state Initialized → Constant
//     or Initialized → Constanic by stepping the FIR tree.

// -----------------------------------------------------------------------------
// FIR state — the evaluation lifecycle.
// -----------------------------------------------------------------------------

enum FirState:
  case Initialized                  // freshly compiled, evaluation not yet attempted
  case Constant                     // fully evaluated to a definite value
  case Constanic                    // CONSTANIC ("not known yet" — may resolve in new context)
  case NK                           // definitively Not Known (div-by-zero, anchored search miss on CONSTANT brane, etc.)

object FirState:
  given Encoder[FirState] = Encoder.encodeString.contramap(_.toString)
  given Decoder[FirState] = Decoder.decodeString.emap:
    case "Initialized" => Right(Initialized)
    case "Constant"    => Right(Constant)
    case "Constanic"   => Right(Constanic)
    case "NK"          => Right(NK)
    case other         => Left(s"Unknown FirState: $other")

// -----------------------------------------------------------------------------
// FIR algebra — sealed hierarchy. All Phase 1 variants live here.
// -----------------------------------------------------------------------------

sealed trait Fir:
  def state: FirState

// A literal integer value. Compiled directly to Constant state — no evaluation needed.
case class ConstantIntFir(value: Long, state: FirState = FirState.Constant) extends Fir

// A normal brane: ordered list of named or anonymous statements.
// Phase 1: NormalBrane is the only brane variant. ConcatenationBrane, DetachmentBrane
// come later.
case class NormalBraneFir(
  characterizations: List[String],
  statements:        List[StatementFir],
  state:             FirState = FirState.Initialized
) extends Fir

// A statement inside a brane. Anonymous statements have name = None (e.g., bare `42`
// inside `{42; x = 1}`).
case class StatementFir(
  name: Option[String],
  body: Fir,
  state: FirState = FirState.Initialized
)

// Binary operator expression. Phase 1: AST tree only — no arithmetic computation.
// Phase 2: evaluator collapses to ConstantIntFir if both operands are Constant.
case class BinaryOpFir(
  op:    String,    // "+", "-", "*", "/", "%"
  left:  Fir,
  right: Fir,
  state: FirState = FirState.Initialized
) extends Fir

// Unary operator expression (e.g., `-42`).
case class UnaryOpFir(
  op:    String,    // "+", "-", "*"
  expr:  Fir,
  state: FirState = FirState.Initialized
) extends Fir

// Search operations. All searches in Phase 1 are pattern-based — bare identifiers like
// `a_config` compile to a regex of `^a_config$`.
//
// `anchored` distinguishes the two semantic flavors:
//   - false: unanchored (bare identifier or `#-N`). Searches IB, then walks up AB chain.
//            Not finding it produces CONSTANIC.
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
  pattern:   String,                     // regex pattern; bare names become "^name$"
  direction: SearchDirection,
  anchored:  Boolean,
  anchor:    Option[Fir],                // None for unanchored; Some(brane) for anchored
  state:     FirState = FirState.Initialized
) extends Fir

// Index access: `brane#N` (anchored) or `#-N` (unanchored seek).
case class IndexFir(
  index:    Int,
  anchored: Boolean,
  anchor:   Option[Fir],
  state:    FirState = FirState.Initialized
) extends Fir

// Head (`^`) and tail (`$`) — degenerate searches that always find the first/last.
case class HeadTailFir(
  isHead:   Boolean,                     // true = head (^), false = tail ($)
  anchored: Boolean,
  anchor:   Option[Fir],
  state:    FirState = FirState.Initialized
) extends Fir

// Identifier reference — at compilation time, this is an unanchored backward search.
// Compilation desugars `x` into SearchFir(pattern = "^x$", Backward, anchored = false, anchor = None).
// We keep this separate node only when characterizations are present, since
// `type'name` carries semantic info beyond the bare name. Otherwise prefer SearchFir.
case class CharacterizedRefFir(
  characterizations: List[String],
  pattern:           String,
  state:             FirState = FirState.Initialized
) extends Fir

// The literal `???` (NK from source).
case class NKFir(
  reason: String,
  state:  FirState = FirState.NK
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
