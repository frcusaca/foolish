//! The test runner: [`EinmoSuite`] drives an [`Evaluator`] over the `input/`
//! tree, assembles signed `.einmo` envelopes, and writes them to `output/`.
//!
//! The trait is einmo's own — `Vec<String>`, no dependency on any Foolish
//! crate. Adapters (in zweimomo) format their interpreter's values to strings
//! before returning them; einmo never interprets body content.

use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};

use crate::config::{PerspectiveOf, TestConfig};
use crate::error::{EinmoError, Result};
use crate::format::{EinmoFile, Metadata, Section, Status};
use crate::signature::{Stamps, derive_keypair};
use crate::stage::{Stage, ensure_parent_dir, mirror_input_path};

/// A language-agnostic evaluator: source text in, formatted output chunks out.
///
/// Returning `Err(String)` signals the input could not be parsed/accepted
/// (recorded as `status: input-error`); a *panic* during `evaluate` is caught
/// by the suite and recorded as `status: output-error`. An expected error
/// *value* (a division-by-zero alarm, "infinite loop detected") is a normal
/// `Ok` output — the suite marks it `status: normal`.
///
/// `Sync` is required so `evaluate_all` can share one evaluator across threads;
/// adapters construct their (`!Send`) interpreter *inside* `evaluate`, per call.
pub trait Evaluator: Sync {
    /// Evaluate `source`, returning one formatted string per top-level result.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` when the input cannot be parsed or accepted.
    fn evaluate(&self, source: &str) -> std::result::Result<Vec<String>, String>;
}

/// The outcome of evaluating one input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResult {
    /// The mirror-relative output path.
    pub rel_path: PathBuf,
    /// The harness status recorded in the envelope.
    pub status: Status,
    /// `true` if the file was written, stamped, and re-verified successfully.
    pub written_and_verified: bool,
    /// `true` if this test was skipped because its catastrophe crumb was
    /// acknowledged by `ignore_catastrophe_crumbs`.
    pub ignored: bool,
    /// A detail line when something went wrong (write/verify/diff-limit).
    pub detail: Option<String>,
}

/// The aggregate result of an `evaluate_all` run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestResults {
    /// Per-file results, in deterministic path order.
    pub files: Vec<FileResult>,
    /// Correspondence failures (from `require_correspondence`), if enforced.
    pub correspondence_failures: Vec<String>,
    /// Suite-shape violations: extraneous inputs and orphaned artifacts.
    pub integrity: SuiteIntegrity,
}

/// The escalating validation levels (FOOP-64 §"The escalating validation
/// levels").
///
/// The levels **escalate — they do not replace**: each performs everything the
/// level below it requires, plus its own. `Verified ⊃ Checked ⊃ Output`.
///
/// There is **no default**: a suite states which level it produces and
/// validates, because only the suite's author knows whether an unpopulated
/// `verified/` means "not signed yet" (fine at the `Checked` level) or "the
/// merge gate is incomplete" (fatal at the `Verified` level). The CLI, built on
/// this API, may default — the library may not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationLevel {
    /// The suite is well-formed and evaluates: no extraneous files, non-empty,
    /// every input evaluates to a self-verifying `output/` artifact, no
    /// orphans there.
    Output,
    /// …plus a reviewed baseline: `output` ↔ `checked` match up exactly, their
    /// content is identical, and `checked/`'s signatures verify.
    Checked,
    /// …plus human attestation: `checked` ↔ `verified` match up exactly, their
    /// content is identical, `verified/`'s signatures verify under the
    /// reviewer's key, and no stamp carries the computer key.
    Verified,
}

impl ValidationLevel {
    /// This level and every level it escalates from, in ascending order.
    ///
    /// The engine walks these in order, so a lower level's problems are always
    /// reported before the higher level's.
    #[must_use]
    pub fn escalation(self) -> &'static [ValidationLevel] {
        match self {
            ValidationLevel::Output => &[ValidationLevel::Output],
            ValidationLevel::Checked => &[ValidationLevel::Output, ValidationLevel::Checked],
            ValidationLevel::Verified => &[
                ValidationLevel::Output,
                ValidationLevel::Checked,
                ValidationLevel::Verified,
            ],
        }
    }

    /// The stage this level judges (`Output` → `output/`, …).
    #[must_use]
    pub fn stage(self) -> Stage {
        match self {
            ValidationLevel::Output => Stage::Output,
            ValidationLevel::Checked => Stage::Checked,
            ValidationLevel::Verified => Stage::Verified,
        }
    }

    /// The level this one escalates from, if any.
    #[must_use]
    pub fn escalates_from(self) -> Option<ValidationLevel> {
        match self {
            ValidationLevel::Output => None,
            ValidationLevel::Checked => Some(ValidationLevel::Output),
            ValidationLevel::Verified => Some(ValidationLevel::Checked),
        }
    }

    /// Parse a level from its lowercase name (for CLI/env).
    ///
    /// # Errors
    ///
    /// Returns [`EinmoError::Config`] for an unrecognized name.
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "output" => Ok(ValidationLevel::Output),
            "checked" => Ok(ValidationLevel::Checked),
            "verified" => Ok(ValidationLevel::Verified),
            other => Err(EinmoError::Config(format!(
                "unknown validation level {other:?} (output | checked | verified)"
            ))),
        }
    }
}

impl std::fmt::Display for ValidationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ValidationLevel::Output => "output",
            ValidationLevel::Checked => "checked",
            ValidationLevel::Verified => "verified",
        })
    }
}

/// Everything that can be wrong with a suite, grouped by the level that finds
/// it (FOOP-64 §"The escalating validation levels").
///
/// Einmo is deliberately unforgiving about its own tree: silently skipping
/// files is how a corpus rots — the insta corpus this replaced hid a test that
/// had **never compiled** for months, because the harness `eprintln!`d the
/// error and moved on.
///
/// Every variant names the offending path and describes the fault; **none
/// carries an excerpt of file content**. The artifacts are signed; a problem
/// report must not reproduce their bodies. A reviewer reads them through
/// `einmo body` or `poor_einmo.sh`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Problem {
    // ── Output level ────────────────────────────────────────────────────
    /// O1 — a file under `input/` that is not a test input (editor swap/backup,
    /// `.DS_Store`, …). Skipped for discovery so it cannot become a phantom
    /// test, but never ignored.
    ExtraneousFile { path: PathBuf },
    /// O2 — the suite discovered no inputs at all. A vacuous pass is not a pass.
    EmptySuite,
    /// O3/O4 — an artifact could not be written, or failed to verify against
    /// its own bytes immediately after writing.
    ArtifactUnsound { path: PathBuf, detail: String },
    /// O5/C3/V3 — a `.einmo` whose `input/` file no longer exists: a signed
    /// record that can never be re-derived or reviewed.
    OrphanedArtifact { stage: Stage, path: PathBuf },

    // ── Checked / Verified levels: a pairwise (left, right) comparison ──
    //
    // A level escalates by comparing two stages: `left` is the established
    // side it escalates *from* (output at the Checked level, checked at the
    // Verified level) and `right` is the side it escalates *to*. The variants
    // below name which side is at fault, so a report never leaves the reader
    // guessing which direction the fault runs.
    //
    /// C2/V2 — the artifact is absent from the **left** stage entirely: the
    /// right side holds a record the left does not.
    LeftMissingEntirely {
        left: Stage,
        right: Stage,
        path: PathBuf,
    },
    /// C2/V2 — the artifact is absent from the **right** stage entirely: the
    /// left side has not been promoted through.
    RightMissingEntirely {
        left: Stage,
        right: Stage,
        path: PathBuf,
    },
    /// C5/V5 — the two sides hold this section with different content. One
    /// problem per differing section; names the section, never its content.
    SectionDifference {
        left: Stage,
        right: Stage,
        path: PathBuf,
        section: String,
    },
    /// C4/V4 — an artifact's stamp chain does not verify: tampered, corrupt,
    /// or a broken chain. Refused, never compared.
    SignatureInvalid {
        stage: Stage,
        path: PathBuf,
        detail: String,
    },

    // ── Verified level only ─────────────────────────────────────────────
    /// V6 — a `stage:verified` stamp is not the configured reviewer's key.
    WrongSigner {
        path: PathBuf,
        expected_prefix: String,
        found: String,
    },
    /// V7 — a `stage:verified` stamp carries the well-known computer key: an
    /// AI attested where a human was required.
    ComputerKeyAttestation { path: PathBuf },
}

impl Problem {
    /// The level whose requirements this problem violates.
    #[must_use]
    pub fn level(&self) -> ValidationLevel {
        match self {
            Problem::ExtraneousFile { .. }
            | Problem::EmptySuite
            | Problem::ArtifactUnsound { .. } => ValidationLevel::Output,
            Problem::OrphanedArtifact { stage, .. } | Problem::SignatureInvalid { stage, .. } => {
                match stage {
                    Stage::Verified => ValidationLevel::Verified,
                    Stage::Checked => ValidationLevel::Checked,
                    _ => ValidationLevel::Output,
                }
            }
            Problem::LeftMissingEntirely { right, .. }
            | Problem::RightMissingEntirely { right, .. }
            | Problem::SectionDifference { right, .. } => match right {
                Stage::Verified => ValidationLevel::Verified,
                _ => ValidationLevel::Checked,
            },
            Problem::WrongSigner { .. } | Problem::ComputerKeyAttestation { .. } => {
                ValidationLevel::Verified
            }
        }
    }

    /// The offending path, if the problem has one.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Problem::EmptySuite => None,
            Problem::ExtraneousFile { path }
            | Problem::ArtifactUnsound { path, .. }
            | Problem::OrphanedArtifact { path, .. }
            | Problem::LeftMissingEntirely { path, .. }
            | Problem::RightMissingEntirely { path, .. }
            | Problem::SectionDifference { path, .. }
            | Problem::SignatureInvalid { path, .. }
            | Problem::WrongSigner { path, .. }
            | Problem::ComputerKeyAttestation { path } => Some(path),
        }
    }

    /// What to do about it.
    #[must_use]
    pub fn remedy(&self) -> &'static str {
        match self {
            Problem::ExtraneousFile { .. } => {
                "remove it (editor swap/backup files belong outside the suite)"
            }
            Problem::EmptySuite => "check the suite's input/ directory and its configured path",
            Problem::ArtifactUnsound { .. } => {
                "investigate: the harness could not produce a sound artifact"
            }
            Problem::OrphanedArtifact { .. } => {
                "delete it, or `einmo flag <suite> <stage> <file>` to retire it"
            }
            Problem::LeftMissingEntirely { .. } => {
                "the right side holds a record the left does not: delete the stray, or restore the left"
            }
            Problem::RightMissingEntirely { .. } => {
                "review the diff, then promote the left side through"
            }
            Problem::SectionDifference { .. } => {
                "review the diff: repair the code, or promote after review"
            }
            Problem::SignatureInvalid { .. } => {
                "the artifact is tampered or corrupt: regenerate and re-promote"
            }
            Problem::WrongSigner { .. } => "re-sign with the reviewer's key",
            Problem::ComputerKeyAttestation { .. } => {
                "a human must sign: `einmo promote checked->verified <suite> --interactive`"
            }
        }
    }
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::ExtraneousFile { path } => write!(
                f,
                "{}: not a test input (a test input is a file someone deliberately named)",
                path.display()
            ),
            Problem::EmptySuite => write!(f, "the suite discovered no inputs"),
            Problem::ArtifactUnsound { path, detail } => {
                write!(f, "{}: not written+verified: {detail}", path.display())
            }
            Problem::OrphanedArtifact { stage, path } => write!(
                f,
                "{}/{}: orphaned — no corresponding file in input/",
                stage.dir_name(),
                path.display()
            ),
            Problem::LeftMissingEntirely { left, right, path } => write!(
                f,
                "{}: missing entirely from {}/ (present in {}/)",
                path.display(),
                left.dir_name(),
                right.dir_name()
            ),
            Problem::RightMissingEntirely { left, right, path } => write!(
                f,
                "{}: missing entirely from {}/ (present in {}/)",
                path.display(),
                right.dir_name(),
                left.dir_name()
            ),
            Problem::SectionDifference {
                left,
                right,
                path,
                section,
            } => write!(
                f,
                "{}: section {section} differs between {}/ and {}/",
                path.display(),
                left.dir_name(),
                right.dir_name()
            ),
            Problem::SignatureInvalid {
                stage,
                path,
                detail,
            } => write!(
                f,
                "{}/{}: signature invalid: {detail}",
                stage.dir_name(),
                path.display()
            ),
            Problem::WrongSigner {
                path,
                expected_prefix,
                found,
            } => write!(
                f,
                "{}: stage:verified signed by {found}, expected key starting {expected_prefix}",
                path.display()
            ),
            Problem::ComputerKeyAttestation { path } => write!(
                f,
                "{}: stage:verified carries the computer key — an AI attested where a human was required",
                path.display()
            ),
        }
    }
}

/// The outcome of validating a suite at a level: sound, or a list of problems.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SuiteIntegrity {
    /// Every problem found, ordered by escalating level then path.
    pub problems: Vec<Problem>,
}

impl SuiteIntegrity {
    /// `true` when the suite satisfies the level it was validated at.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.problems.is_empty()
    }

    /// A reviewer-facing report: one problem per line, with its remedy.
    #[must_use]
    pub fn report(&self) -> String {
        self.problems
            .iter()
            .map(|p| format!("  [{}] {p}\n      → {}\n", p.level(), p.remedy()))
            .collect()
    }
}

impl TestResults {
    /// `true` if the suite's shape is sound, every file was written and
    /// re-verified (or acknowledged as an ignored catastrophe crumb), and every
    /// required correspondence held.
    #[must_use]
    pub fn all_output_written_and_verified(&self) -> bool {
        self.integrity.is_clean()
            && self
                .files
                .iter()
                .all(|f| f.written_and_verified || f.ignored)
            && self.correspondence_failures.is_empty()
    }
}

/// Judge a suite's *shape* without evaluating anything: extraneous inputs and
/// orphaned artifacts (see [`SuiteIntegrity`]).
///
/// This is what `einmo verify` reports alongside signature integrity, and what
/// [`EinmoSuite::evaluate_all`] folds into its results — one implementation, so
/// the CLI and the library can never disagree about what a sound suite is.
///
/// # Errors
///
/// Returns [`EinmoError::Io`] if a tree cannot be walked.
pub fn check_suite_integrity(config: &TestConfig) -> Result<SuiteIntegrity> {
    let (inputs, extraneous) =
        crate::stage::walk_input_tree_reporting(&config.input_path(), config.walk_depth_limit())?;
    let level = config.validation_level();
    EinmoSuite::new(config.clone()).check_integrity(&inputs, extraneous, level)
}

/// A test suite bound to one work directory and configuration.
#[derive(Debug)]
pub struct EinmoSuite {
    config: TestConfig,
}

impl EinmoSuite {
    /// Bind a suite to `config`.
    #[must_use]
    pub fn new(config: TestConfig) -> Self {
        EinmoSuite { config }
    }

    /// The suite's configuration.
    #[must_use]
    pub fn config(&self) -> &TestConfig {
        &self.config
    }

    /// Judge the suite's *shape*: extraneous inputs (R1) and orphaned
    /// artifacts (R2). See [`SuiteIntegrity`] for the rules and rationale.
    ///
    /// `flagged/` is exempt from R2 — flagging is retirement, so a flagged
    /// artifact without an input is a finished job, not an orphan.
    /// Validate the suite at `level`, walking the escalation from the base up
    /// (FOOP-64 §"The escalating validation levels").
    ///
    /// Lower levels' problems are reported before higher ones, so a reviewer
    /// fixes the foundation first: an extraneous file is not hidden behind a
    /// signing complaint.
    fn check_integrity(
        &self,
        inputs: &[PathBuf],
        extraneous: Vec<PathBuf>,
        level: ValidationLevel,
    ) -> Result<SuiteIntegrity> {
        let mut problems: Vec<Problem> = Vec::new();
        let expected: std::collections::HashSet<PathBuf> =
            inputs.iter().map(|p| mirror_input_path(p)).collect();

        for step in level.escalation() {
            match step {
                // ── Output level ───────────────────────────────────────
                ValidationLevel::Output => {
                    // O1 — extraneous files under input/.
                    let mut extras: Vec<PathBuf> = extraneous
                        .iter()
                        .map(|rel| Path::new(self.config.input_dir()).join(rel))
                        .collect();
                    extras.sort();
                    problems.extend(
                        extras
                            .into_iter()
                            .map(|path| Problem::ExtraneousFile { path }),
                    );
                    // O2 — a suite that discovered nothing is not a pass.
                    if inputs.is_empty() {
                        problems.push(Problem::EmptySuite);
                    }
                    // O5 — orphans in output/. (O3/O4 come from the run itself.)
                    problems.extend(self.orphans_of(Stage::Output, &expected)?);
                }
                // ── Checked level: escalates from Output ───────────────
                ValidationLevel::Checked => {
                    problems.extend(self.orphans_of(Stage::Checked, &expected)?); // C3
                    problems.extend(self.stage_pair_problems(Stage::Output, Stage::Checked)?);
                    // C2/C4/C5
                }
                // ── Verified level: escalates from Checked ─────────────
                ValidationLevel::Verified => {
                    problems.extend(self.orphans_of(Stage::Verified, &expected)?); // V3
                    problems.extend(self.stage_pair_problems(Stage::Checked, Stage::Verified)?);
                    // V2/V4/V5
                    problems.extend(self.attestation_problems()?); // V6/V7
                }
            }
        }
        Ok(SuiteIntegrity { problems })
    }

    /// Artifacts in `stage` with no corresponding `input/` file (O5/C3/V3).
    ///
    /// `flagged/` is never asked: it sits outside the escalation — flagging is
    /// retirement, so an input-less flagged artifact is a finished job.
    fn orphans_of(
        &self,
        stage: Stage,
        expected: &std::collections::HashSet<PathBuf>,
    ) -> Result<Vec<Problem>> {
        let dir = self.config.stage_dir(stage);
        let (present, _) =
            crate::stage::walk_input_tree_reporting(&dir, self.config.walk_depth_limit())?;
        let mut out: Vec<Problem> = present
            .into_iter()
            .filter(|rel| !expected.contains(rel))
            .map(|path| Problem::OrphanedArtifact { stage, path })
            .collect();
        out.sort_by(|a, b| a.path().cmp(&b.path()));
        Ok(out)
    }

    /// Do two stages match up exactly, verify, and hold identical content?
    /// (C2/C4/C5 for output↔checked; V2/V4/V5 for checked↔verified.)
    ///
    /// Delegates to `compare`, which already verifies-on-inspect both sides and
    /// compares only the configured sections — STAMPS and metadata are excluded
    /// by design (they carry per-run timestamps; comparing them is the insta
    /// defect this FOOP exists to fix).
    fn stage_pair_problems(&self, left: Stage, right: Stage) -> Result<Vec<Problem>> {
        let (a, b) = (left, right);
        let cmp = crate::compare::compare(&self.config, a, b, self.config.match_sections(), None)?;
        let mut out = Vec::new();
        // `only_in_a` = present left, absent right → the right side is missing.
        for rel in cmp.only_in_a {
            out.push(Problem::RightMissingEntirely {
                left: a,
                right: b,
                path: rel,
            });
        }
        // `only_in_b` = present right, absent left → the left side is missing.
        for rel in cmp.only_in_b {
            out.push(Problem::LeftMissingEntirely {
                left: a,
                right: b,
                path: rel,
            });
        }
        // One problem per differing section: a reviewer fixes sections, not
        // files, and a bare "content differs" hides how much differs.
        for d in cmp.differing {
            for section in d.sections {
                out.push(Problem::SectionDifference {
                    left: a,
                    right: b,
                    path: d.rel_path.clone(),
                    section,
                });
            }
        }
        for rel in cmp.tampered {
            out.push(Problem::SignatureInvalid {
                stage: b,
                path: rel,
                detail: "verify-on-inspect refused this artifact".to_string(),
            });
        }
        out.sort_by(|x, y| x.path().cmp(&y.path()));
        Ok(out)
    }

    /// Is every `verified/` artifact attested by the reviewer, and none by the
    /// computer key? (V6/V7.)
    ///
    /// With no reviewer key configured, V6 cannot be judged — but V7 still can,
    /// and must: an AI-signed `verified/` is detectable regardless.
    fn attestation_problems(&self) -> Result<Vec<Problem>> {
        let dir = self.config.stage_dir(Stage::Verified);
        let (present, _) =
            crate::stage::walk_input_tree_reporting(&dir, self.config.walk_depth_limit())?;
        let expected_prefix = self.config.reviewer_key_prefix().map(str::to_string);
        let mut out = Vec::new();
        for rel in present {
            let path = dir.join(&rel);
            let Ok(file) = EinmoFile::from_file(&path) else {
                continue; // SignatureInvalid is reported by the stage-pair check
            };
            let Some(stamp) = file
                .stamps()
                .entries()
                .iter()
                .find(|s| s.key() == Stage::Verified.stamp_key())
            else {
                continue; // no verified stamp: MissingCounterpart/compare covers it
            };
            let pubkey = stamp.pubkey_hex().to_string();
            if crate::signature::is_computer_key(&pubkey) {
                out.push(Problem::ComputerKeyAttestation { path: rel.clone() });
            } else if let Some(prefix) = &expected_prefix
                && !pubkey.starts_with(prefix.as_str())
            {
                out.push(Problem::WrongSigner {
                    path: rel.clone(),
                    expected_prefix: prefix.clone(),
                    found: pubkey,
                });
            }
        }
        out.sort_by(|x, y| x.path().cmp(&y.path()));
        Ok(out)
    }

    fn is_catastrophe_crumb(&self, out_path: &Path) -> bool {
        match EinmoFile::from_file(out_path) {
            Ok(file) => file
                .metadata()
                .status_detail
                .starts_with("TEST IN PROGRESS"),
            Err(_) => false,
        }
    }

    fn is_ignored(&self, rel: &Path) -> bool {
        let mirror = mirror_input_path(rel);
        self.config
            .ignore_catastrophe_crumbs()
            .iter()
            .any(|p| p == &mirror || p == rel)
    }

    /// If a stale catastrophe crumb sits at `out_path`, decide whether the test
    /// is skipped (ignored), refused, or re-run. Returns `Some(FileResult)`
    /// when the test must NOT proceed to `write_crash_crumb`/evaluation.
    fn check_catastrophe_crumb(&self, rel: &Path, out_path: &Path) -> Option<FileResult> {
        if !self.is_catastrophe_crumb(out_path) {
            return None;
        }
        if self.is_ignored(rel) {
            return Some(FileResult {
                rel_path: mirror_input_path(rel),
                status: Status::OutputError,
                written_and_verified: false,
                ignored: true,
                detail: Some("catastrophe crumb ignored by configuration".into()),
            });
        }
        if !self.config.rerun_catastrophes() {
            return Some(FileResult {
                rel_path: mirror_input_path(rel),
                status: Status::OutputError,
                written_and_verified: false,
                ignored: false,
                detail: Some(format!(
                    "catastrophe crumb detected from previous run; use --ignore-catastrophe-crumbs {} or --rerun-catastrophes to override",
                    mirror_input_path(rel).display()
                )),
            });
        }
        None
    }

    /// Evaluate one input file and write its signed `.einmo` to `output/`.
    ///
    /// # Errors
    ///
    /// Returns [`EinmoError::Io`] on read/write failure, or
    /// [`EinmoError::SeparatorCollision`] if a body contains the separator.
    pub fn evaluate(&self, input_rel: &Path, evaluator: &dyn Evaluator) -> Result<FileResult> {
        let source = self.read_input(input_rel)?;
        let out_path = self
            .config
            .stage_dir(Stage::Output)
            .join(mirror_input_path(input_rel));
        if let Some(gated) = self.check_catastrophe_crumb(input_rel, &out_path) {
            return Ok(gated);
        }
        let _ = self.write_crash_crumb(input_rel, &source, &out_path);
        let outcome = evaluate_capturing(evaluator, &source);
        self.write_output(input_rel, &source, outcome, None)
    }

    /// Evaluate an inlined input (a string in code, not a file on disk).
    ///
    /// `name` becomes the mirror-relative filename. Inline *expected values*
    /// are refused by design (there is no API to supply one).
    ///
    /// # Errors
    ///
    /// As [`EinmoSuite::evaluate`].
    pub fn evaluate_inline(
        &self,
        name: &str,
        input: &str,
        evaluator: &dyn Evaluator,
    ) -> Result<FileResult> {
        let input_rel = Path::new(name);
        let out_path = self
            .config
            .stage_dir(Stage::Output)
            .join(mirror_input_path(input_rel));
        if let Some(gated) = self.check_catastrophe_crumb(input_rel, &out_path) {
            return Ok(gated);
        }
        let _ = self.write_crash_crumb(input_rel, input, &out_path);
        let outcome = evaluate_capturing(evaluator, input);
        self.write_output(input_rel, input, outcome, None)
    }

    /// Discover all inputs, evaluate them (parallel or serial per config),
    /// write all outputs, and enforce `require_correspondence`.
    ///
    /// References of dependent einmos (§4.7) are evaluated before their
    /// dependents so the DIFF can be computed from the same run.
    ///
    /// # Errors
    ///
    /// Returns [`EinmoError::Io`] if the input tree cannot be walked.
    pub fn evaluate_all(&self, evaluator: &dyn Evaluator) -> Result<TestResults> {
        let (inputs, extraneous) = crate::stage::walk_input_tree_reporting(
            &self.config.input_path(),
            self.config.walk_depth_limit(),
        )?;
        // First pass: evaluate every input, capturing raw outputs so dependents
        // can diff against their reference's output from the same run.
        let ordered = topological_order(&inputs, self.config.dependent_separator());
        let suite_start = std::time::Instant::now();

        // KNOWN GAP (FOOP-64, 2026-07-15): `suite_duration_limit` is enforced
        // only on the serial path below -- it is checked before starting each
        // test, so parallel workers all launch before a short budget expires
        // and nothing aborts. Parallel is now the default, so an operator
        // setting EINMO_SUITE_DURATION_LIMIT gets no throttling unless they
        // also force serial. Fixing it properly means a shared deadline the
        // workers poll between tests; deferred, not silently ignored.
        let (raw, suite_skipped, crumb_gated) = if let Some(threads) = self.config.parallel() {
            self.evaluate_raw_parallel(&ordered, evaluator, threads, suite_start)
        } else {
            let mut raw: Vec<(PathBuf, String, EvalOutcome)> = Vec::new();
            let mut skipped = 0usize;
            let mut crumb_gated: Vec<FileResult> = Vec::new();
            for rel in &ordered {
                if let Some(limit) = self.config.suite_duration_limit()
                    && suite_start.elapsed() > limit
                {
                    skipped = ordered.len() - raw.len();
                    break;
                }
                let source = match self.read_input(rel) {
                    Ok(s) => s,
                    Err(e) => {
                        raw.push((rel.clone(), String::new(), EvalOutcome::read_error(&e)));
                        continue;
                    }
                };
                let out_path = self
                    .config
                    .stage_dir(Stage::Output)
                    .join(mirror_input_path(rel));
                if let Some(gated) = self.check_catastrophe_crumb(rel, &out_path) {
                    crumb_gated.push(gated);
                    continue;
                }
                let _ = self.write_crash_crumb(rel, &source, &out_path);
                let test_start = std::time::Instant::now();
                let mut outcome = evaluate_capturing(evaluator, &source);
                if let Some(limit) = self.config.duration_limit() {
                    let elapsed = test_start.elapsed();
                    if elapsed > limit {
                        outcome = EvalOutcome {
                            outputs: vec![],
                            status: Status::OutputError,
                            detail: Some(format!(
                                "exceeded EINMO_DURATION_LIMIT ({}s, actual {}ms)",
                                limit.as_secs(),
                                elapsed.as_millis()
                            )),
                        };
                    }
                }
                raw.push((rel.clone(), source, outcome));
            }
            (raw, skipped, crumb_gated)
        };

        // Second pass: write each output, computing dependent DIFFs. Per-file
        // write/serialize failures (e.g. separator collision) are recorded as
        // a failed `FileResult` rather than aborting the whole run.
        let mut results = TestResults::default();
        for gated in crumb_gated {
            results.files.push(gated);
        }
        for (rel, source, outcome) in &raw {
            let dependent = self.dependent_context(rel, &raw);
            match self.write_output(rel, source, outcome.clone(), dependent) {
                Ok(result) => results.files.push(result),
                Err(e) => results.files.push(FileResult {
                    rel_path: mirror_input_path(rel),
                    status: Status::OutputError,
                    written_and_verified: false,
                    ignored: false,
                    detail: Some(format!("write/serialize error: {e}")),
                }),
            }
        }
        results.files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

        if suite_skipped > 0
            && let Some(limit) = self.config.suite_duration_limit()
        {
            results.correspondence_failures.push(format!(
                "suite duration limit exceeded ({}s), skipped {} remaining tests",
                limit.as_secs(),
                suite_skipped
            ));
        }

        // Enforce configured correspondences via `compare`. Ignored catastrophe
        // crumbs in output are excluded so an acknowledged crumb does not cause
        // a spurious correspondence failure.
        let ignored_paths = self.config.ignore_catastrophe_crumbs();
        let ignored_match = |p: &Path| {
            ignored_paths
                .iter()
                .any(|ip| ip == p || mirror_input_path(ip) == p)
        };
        for &(a, b) in self.config.required_correspondences() {
            let mut cmp =
                crate::compare::compare(&self.config, a, b, self.config.match_sections(), None)?;
            if !ignored_paths.is_empty() {
                cmp.differing.retain(|d| !ignored_match(&d.rel_path));
                cmp.only_in_a.retain(|p| !ignored_match(p));
            }
            if !cmp.is_clean() {
                results.correspondence_failures.push(format!(
                    "{a} vs {b}: {} differing, {} only-in-{a}, {} only-in-{b}, {} tampered",
                    cmp.differing.len(),
                    cmp.only_in_a.len(),
                    cmp.only_in_b.len(),
                    cmp.tampered.len()
                ));
            }
        }

        // Validate the suite's shape LAST: the escalating levels judge output/
        // (and, above the base level, checked/ and verified/), none of which
        // exist until this run has written them.
        results.integrity =
            self.check_integrity(&inputs, extraneous, self.config.validation_level())?;
        Ok(results)
    }

    /// Evaluate a batch of inlined `(name, input)` pairs.
    ///
    /// # Errors
    ///
    /// As [`EinmoSuite::evaluate_all`].
    pub fn evaluate_all_inline(
        &self,
        pairs: &[(&str, &str)],
        evaluator: &dyn Evaluator,
    ) -> Result<TestResults> {
        let mut results = TestResults::default();
        for (name, input) in pairs {
            let outcome = evaluate_capturing(evaluator, input);
            results
                .files
                .push(self.write_output(Path::new(name), input, outcome, None)?);
        }
        Ok(results)
    }

    // ---- internals ----

    fn read_input(&self, input_rel: &Path) -> Result<String> {
        let path = self.config.input_path().join(input_rel);
        std::fs::read_to_string(&path).map_err(|e| EinmoError::io(&path, e))
    }

    /// Write a signed crash-crumb `.einmo` to the output path BEFORE running
    /// the evaluator. If the process crashes during evaluation (panic that
    /// escapes `catch_unwind`, stack overflow, OOM, abort, kill signal), this
    /// signed file remains as the test's output — a forensic signal that can be
    /// verified, compared, and promoted. When the evaluator succeeds,
    /// [`Self::write_output`] overwrites it with the real output.
    fn write_crash_crumb(&self, input_rel: &Path, source: &str, out_path: &Path) -> Result<()> {
        let section_names = vec![
            "INPUT".into(),
            "OUTPUT".into(),
            "COMMENTS".into(),
            "STAMPS".into(),
        ];
        let metadata = Metadata {
            test: input_rel.to_string_lossy().into_owned(),
            suite: self.config.suite_name().to_string(),
            producer: git_commit_sha(),
            producer_diff: git_diff_sha(),
            generated: crate::signature::now_iso8601(),
            status: Status::OutputError,
            status_detail: "TEST IN PROGRESS — if you see this file, the test harness crashed during evaluation. Escalate to human or other agents for support.".into(),
            reference: String::new(),
            sections: section_names,
        };
        let sections = vec![
            Section::new("INPUT", source.to_string()),
            Section::new("OUTPUT", String::new()),
            Section::new("COMMENTS", String::new()),
        ];
        let mut file = EinmoFile::new(
            self.config.encoding(),
            self.config.separator(),
            metadata,
            sections,
            Stamps::new(),
        );
        let (configured, _) = derive_keypair(self.config.configured_passphrase());
        let output_pass = self.config.stage_passphrase(Stage::Output).unwrap_or("");
        let (stage_output, _) = derive_keypair(output_pass);
        let stamps = Stamps::generate(&file.signed_prefix(), &configured, &stage_output);
        file.set_stamps(stamps);

        ensure_parent_dir(out_path)?;
        let bytes = file.serialize()?;
        std::fs::write(out_path, &bytes).map_err(|e| EinmoError::io(out_path, e))?;
        Ok(())
    }

    fn evaluate_raw_parallel(
        &self,
        ordered: &[PathBuf],
        evaluator: &dyn Evaluator,
        threads: usize,
        suite_start: std::time::Instant,
    ) -> (Vec<(PathBuf, String, EvalOutcome)>, usize, Vec<FileResult>) {
        use std::sync::Mutex;
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

        let next = AtomicUsize::new(0);
        let suite_timed_out = AtomicBool::new(false);
        let results: Mutex<Vec<(PathBuf, String, EvalOutcome)>> = Mutex::new(Vec::new());
        let crumb_gated: Mutex<Vec<FileResult>> = Mutex::new(Vec::new());
        let threads = threads.max(1);

        std::thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|| {
                    loop {
                        if suite_timed_out.load(Ordering::Relaxed) {
                            break;
                        }
                        if let Some(limit) = self.config.suite_duration_limit()
                            && suite_start.elapsed() > limit
                        {
                            suite_timed_out.store(true, Ordering::Relaxed);
                            break;
                        }
                        let idx = next.fetch_add(1, Ordering::Relaxed);
                        let Some(rel) = ordered.get(idx) else { break };
                        let out_path = self
                            .config
                            .stage_dir(Stage::Output)
                            .join(mirror_input_path(rel));
                        if let Some(gated) = self.check_catastrophe_crumb(rel, &out_path) {
                            if let Ok(mut g) = crumb_gated.lock() {
                                g.push(gated);
                            }
                            continue;
                        }
                        // Wrap the worker body so ANY panic (read, evaluate,
                        // lock) becomes a failed `EvalOutcome` instead of
                        // poisoning the shared `Mutex`.
                        let entry: (PathBuf, String, EvalOutcome) =
                            match std::panic::catch_unwind(AssertUnwindSafe(|| {
                                let source = match self.read_input(rel) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        return (
                                            rel.clone(),
                                            String::new(),
                                            EvalOutcome::read_error(&e),
                                        );
                                    }
                                };
                                let _ = self.write_crash_crumb(rel, &source, &out_path);
                                let test_start = std::time::Instant::now();
                                let mut outcome = evaluate_capturing(evaluator, &source);
                                if let Some(limit) = self.config.duration_limit() {
                                    let elapsed = test_start.elapsed();
                                    if elapsed > limit {
                                        outcome = EvalOutcome {
                                            outputs: vec![],
                                            status: Status::OutputError,
                                            detail: Some(format!(
                                                "exceeded EINMO_DURATION_LIMIT ({}s, actual {}ms)",
                                                limit.as_secs(),
                                                elapsed.as_millis()
                                            )),
                                        };
                                    }
                                }
                                (rel.clone(), source, outcome)
                            })) {
                                Ok(entry) => entry,
                                Err(panic) => {
                                    let msg = panic_message(&panic);
                                    (
                                        rel.clone(),
                                        String::new(),
                                        EvalOutcome {
                                            outputs: vec![format!("PANIC: {msg}")],
                                            status: Status::OutputError,
                                            detail: Some(msg),
                                        },
                                    )
                                }
                            };
                        if let Ok(mut guard) = results.lock() {
                            guard.push(entry);
                        }
                    }
                });
            }
        });
        let raw = results.into_inner().unwrap();
        let crumb_gated = crumb_gated.into_inner().unwrap();
        let suite_skipped = if suite_timed_out.load(Ordering::Relaxed) {
            ordered.len().saturating_sub(raw.len())
        } else {
            0
        };
        (raw, suite_skipped, crumb_gated)
    }

    /// Assemble, stamp, write, and re-verify one output envelope.
    fn write_output(
        &self,
        input_rel: &Path,
        source: &str,
        outcome: EvalOutcome,
        dependent: Option<DependentContext>,
    ) -> Result<FileResult> {
        self.config.ensure_stage_dirs()?;
        let rel = mirror_input_path(input_rel);
        let out_path = self.config.stage_dir(Stage::Output).join(&rel);

        let mut sections = vec![Section::new("INPUT", source.to_string())];
        let mut section_names = vec!["INPUT".to_string()];
        let mut status = outcome.status;
        let mut status_detail = outcome.detail.clone().unwrap_or_default();

        // OUTPUT sections — one per returned chunk (OUTPUT, OUTPUT[1], …).
        for (i, chunk) in outcome.outputs.iter().enumerate() {
            let name = if i == 0 {
                "OUTPUT".to_string()
            } else {
                format!("OUTPUT[{i}]")
            };
            section_names.push(name.clone());
            sections.push(Section::new(name, chunk.clone()));
        }
        // When there is no output at all, still emit a single empty OUTPUT so
        // the envelope always has one.
        if outcome.outputs.is_empty() {
            section_names.push("OUTPUT".into());
            sections.push(Section::new("OUTPUT", String::new()));
        }

        // Perspectives (§4.5).
        for perspective in self.config.perspectives() {
            let src = match perspective.of {
                PerspectiveOf::Input => Some(source.to_string()),
                PerspectiveOf::Output(i) => outcome.outputs.get(i).cloned(),
            };
            if let Some(text) = src {
                let body = (perspective.extract)(&text);
                section_names.push(perspective.name.to_string());
                sections.push(Section::new(perspective.name.to_string(), body));
            }
        }

        // DIFF section for dependents (§4.7).
        let mut reference_name = String::new();
        if let Some(ctx) = dependent {
            reference_name = ctx.reference_name.clone();
            let (diff_body, diff_status, diff_detail) = ctx.build_diff(self.config.diff_limit());
            section_names.push("DIFF".into());
            sections.push(Section::new("DIFF", diff_body));
            if let Some(s) = diff_status {
                status = s;
                status_detail = diff_detail;
            }
        }

        // COMMENTS is always present (possibly empty).
        section_names.push("COMMENTS".into());
        sections.push(Section::new("COMMENTS", String::new()));
        section_names.push("STAMPS".into());

        let metadata = Metadata {
            test: input_rel.to_string_lossy().into_owned(),
            suite: self.config.suite_name().to_string(),
            producer: git_commit_sha(),
            producer_diff: git_diff_sha(),
            generated: crate::signature::now_iso8601(),
            status,
            status_detail,
            reference: reference_name,
            sections: section_names,
        };

        let mut file = EinmoFile::new(
            self.config.encoding(),
            self.config.separator(),
            metadata,
            sections,
            Stamps::new(),
        );
        // Stamp with compiled + configured + stage:output.
        let (configured, _) = derive_keypair(self.config.configured_passphrase());
        let output_pass = self.config.stage_passphrase(Stage::Output).unwrap_or("");
        let (stage_output, _) = derive_keypair(output_pass);
        let stamps = Stamps::generate(&file.signed_prefix(), &configured, &stage_output);
        file.set_stamps(stamps);

        ensure_parent_dir(&out_path)?;
        let bytes = file.serialize()?;
        std::fs::write(&out_path, &bytes).map_err(|e| EinmoError::io(&out_path, e))?;

        // Re-verify what we just wrote (verify-on-inspect on our own output).
        let written_and_verified = EinmoFile::from_file(&out_path).is_ok();
        let final_status = file.metadata().status;
        Ok(FileResult {
            rel_path: rel,
            status: final_status,
            written_and_verified,
            ignored: false,
            detail: if written_and_verified {
                outcome.detail
            } else {
                Some("re-verification of written output failed".into())
            },
        })
    }

    /// If `rel` is a dependent, locate its reference's raw outcome from `raw`.
    fn dependent_context(
        &self,
        rel: &Path,
        raw: &[(PathBuf, String, EvalOutcome)],
    ) -> Option<DependentContext> {
        let sep = self.config.dependent_separator();
        let reference_rel = reference_of(rel, sep)?;
        // Find the dependent's own outcome and the reference's outcome.
        let own = raw
            .iter()
            .find(|(p, _, _)| p == rel)
            .map(|(_, _, o)| o.clone())?;
        let reference = raw.iter().find(|(p, _, _)| p == &reference_rel);
        Some(DependentContext {
            reference_name: reference_rel.to_string_lossy().into_owned(),
            own_outputs: own.outputs,
            reference_outputs: reference.map(|(_, _, o)| o.outputs.clone()),
            reference_status: reference.map(|(_, _, o)| o.status),
        })
    }
}

/// A captured evaluation outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
struct EvalOutcome {
    outputs: Vec<String>,
    status: Status,
    detail: Option<String>,
}

impl EvalOutcome {
    /// Build an outcome for a failed input read. The harness records this as
    /// `status: output-error` (a harness-level failure, not a successful empty
    /// eval) with no output chunks.
    fn read_error(e: &EinmoError) -> Self {
        EvalOutcome {
            outputs: vec![],
            status: Status::OutputError,
            detail: Some(format!("read error: {e}")),
        }
    }
}

/// Evaluate `source`, catching `Err` (→ input-error) and panics (→ output-error).
fn evaluate_capturing(evaluator: &dyn Evaluator, source: &str) -> EvalOutcome {
    let caught = std::panic::catch_unwind(AssertUnwindSafe(|| evaluator.evaluate(source)));
    match caught {
        Ok(Ok(outputs)) => EvalOutcome {
            outputs,
            status: Status::Normal,
            detail: None,
        },
        Ok(Err(msg)) => EvalOutcome {
            outputs: vec![format!("INPUT ERROR: {msg}")],
            status: Status::InputError,
            detail: Some(msg),
        },
        Err(panic) => {
            let msg = panic_message(&panic);
            EvalOutcome {
                outputs: vec![format!("PANIC: {msg}")],
                status: Status::OutputError,
                detail: Some(msg),
            }
        }
    }
}

/// Extract a human-readable message from a caught panic payload.
fn panic_message(panic: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

/// Context for computing a dependent's DIFF section.
struct DependentContext {
    reference_name: String,
    own_outputs: Vec<String>,
    reference_outputs: Option<Vec<String>>,
    reference_status: Option<Status>,
}

impl DependentContext {
    /// Build the DIFF body plus an optional status override (diff-limit exceed).
    fn build_diff(&self, diff_limit: usize) -> (String, Option<Status>, String) {
        let Some(reference_outputs) = &self.reference_outputs else {
            return (
                "reference unavailable: reference input not found".to_string(),
                None,
                String::new(),
            );
        };
        if self.reference_status != Some(Status::Normal) {
            return (
                "reference unavailable: reference status is not normal".to_string(),
                None,
                String::new(),
            );
        }
        let reference_text = reference_outputs.join("\n");
        let own_text = self.own_outputs.join("\n");
        let diff = deterministic_unified_diff(&reference_text, &own_text);
        if diff.chars().count() > diff_limit {
            let truncated: String = diff.chars().take(diff_limit).collect();
            let detail = format!(
                "DIFF exceeds diff-limit ({} > {diff_limit})",
                diff.chars().count()
            );
            (truncated, Some(Status::OutputError), detail)
        } else {
            (diff, None, String::new())
        }
    }
}

/// A deterministic unified diff (`reference` vs `dependent`), fixed 3-line
/// context, no paths or timestamps — byte-stable across runs.
fn deterministic_unified_diff(reference: &str, dependent: &str) -> String {
    use similar::TextDiff;
    let diff = TextDiff::from_lines(reference, dependent);
    let mut out = String::new();
    out.push_str("--- reference\n");
    out.push_str("+++ dependent\n");
    for group in diff.grouped_ops(3) {
        for op in group {
            for change in diff.iter_changes(&op) {
                let sign = match change.tag() {
                    similar::ChangeTag::Delete => "-",
                    similar::ChangeTag::Insert => "+",
                    similar::ChangeTag::Equal => " ",
                };
                out.push_str(sign);
                out.push_str(change.value());
                if !change.value().ends_with('\n') {
                    out.push('\n');
                }
            }
        }
    }
    out
}

/// The reference input path of a dependent (`base++a++b` → `base++a`), same
/// directory. `None` if `rel` is not a dependent.
fn reference_of(rel: &Path, sep: &str) -> Option<PathBuf> {
    let name = rel.file_name()?.to_string_lossy();
    let last = name.rfind(sep)?;
    let parent = rel.parent();
    let reference_name = &name[..last];
    // Preserve the file extension carried after the last `++segment`? No — the
    // reference input keeps the base name; the extension travels with it since
    // the separator sits before the extension in `base++case.foo`. Strip the
    // trailing `++case` including any extension on the case, then re-attach the
    // original extension.
    let reference_name = reattach_extension(reference_name, &name);
    Some(match parent {
        Some(p) if !p.as_os_str().is_empty() => p.join(reference_name),
        _ => PathBuf::from(reference_name),
    })
}

/// Re-attach the reference's extension. `base++case.foo` → reference is
/// `base.foo`. `stripped` is `base++case` minus the last `++case`, i.e. `base`.
fn reattach_extension(stripped: &str, full_name: &str) -> String {
    let ext = Path::new(full_name)
        .extension()
        .map(|e| e.to_string_lossy().into_owned());
    // `stripped` may itself carry an extension if it was `base++a` from
    // `base++a++b.foo`; normalize by taking its stem and re-adding the ext.
    let stem = Path::new(stripped)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| stripped.to_string());
    match ext {
        Some(e) => format!("{stem}.{e}"),
        None => stem,
    }
}

/// Order inputs so every reference precedes its dependents (topological within
/// each directory). Non-dependents keep their sorted order.
fn topological_order(inputs: &[PathBuf], sep: &str) -> Vec<PathBuf> {
    // Depth = number of `sep` occurrences in the file name; shallower first,
    // then lexicographic. A reference always has fewer separators than its
    // dependents, so this yields a valid topological order.
    let mut ordered = inputs.to_vec();
    ordered.sort_by(|a, b| {
        let da = separator_depth(a, sep);
        let db = separator_depth(b, sep);
        da.cmp(&db).then_with(|| a.cmp(b))
    });
    ordered
}

fn separator_depth(path: &Path, sep: &str) -> usize {
    path.file_name()
        .map(|n| n.to_string_lossy().matches(sep).count())
        .unwrap_or(0)
}

/// The current git commit SHA (short), or `"unknown"` if unavailable.
fn git_commit_sha() -> String {
    run_git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".to_string())
}

/// The SHA-256 of the current `git diff`, or empty when the tree is clean.
fn git_diff_sha() -> String {
    match run_git(&["diff"]) {
        Some(diff) if !diff.is_empty() => {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(diff.as_bytes());
            format!("sha256:{}", hex::encode(hasher.finalize()))
        }
        _ => String::new(),
    }
}

fn run_git(args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git").args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Perspective, PerspectiveOf};

    // A trivial evaluator: echoes the trimmed input as one output chunk; a
    // source of "BOOM" panics; a source starting with "!" errors.
    struct Echo;
    impl Evaluator for Echo {
        fn evaluate(&self, source: &str) -> std::result::Result<Vec<String>, String> {
            if source.trim() == "BOOM" {
                panic!("boom happened");
            }
            if let Some(rest) = source.strip_prefix('!') {
                return Err(format!("cannot parse: {}", rest.trim()));
            }
            Ok(vec![source.trim().to_string()])
        }
    }

    fn suite() -> (tempfile::TempDir, EinmoSuite) {
        let tmp = tempfile::tempdir().unwrap();
        let config = TestConfig::new(tmp.path(), ValidationLevel::Output);
        config.ensure_stage_dirs().unwrap();
        std::fs::create_dir_all(config.input_path()).unwrap();
        (tmp, EinmoSuite::new(config))
    }

    #[test]
    fn evaluate_writes_stamped_verifiable_output() {
        let (_tmp, suite) = suite();
        std::fs::write(suite.config().input_path().join("a.foo"), "{5;}").unwrap();
        let result = suite.evaluate(Path::new("a.foo"), &Echo).unwrap();
        assert!(result.written_and_verified);
        assert_eq!(result.status, Status::Normal);
        let out = suite.config().stage_dir(Stage::Output).join("a.foo.einmo");
        let file = EinmoFile::from_file(&out).unwrap();
        assert_eq!(file.section("INPUT").unwrap().body(), "{5;}");
        assert_eq!(file.section("OUTPUT").unwrap().body(), "{5;}");
    }

    #[test]
    fn err_becomes_input_error_status() {
        let (_tmp, suite) = suite();
        let result = suite.evaluate_inline("bad.foo", "!garbage", &Echo).unwrap();
        assert_eq!(result.status, Status::InputError);
        let out = suite
            .config()
            .stage_dir(Stage::Output)
            .join("bad.foo.einmo");
        let file = EinmoFile::from_file(&out).unwrap();
        assert!(file.metadata().status_detail.contains("cannot parse"));
    }

    #[test]
    fn panic_becomes_output_error_status() {
        let (_tmp, suite) = suite();
        let result = suite.evaluate_inline("boom.foo", "BOOM", &Echo).unwrap();
        assert_eq!(result.status, Status::OutputError);
        assert!(
            result.written_and_verified,
            "a panicking eval still writes a signed file"
        );
        let out = suite
            .config()
            .stage_dir(Stage::Output)
            .join("boom.foo.einmo");
        let file = EinmoFile::from_file(&out).unwrap();
        assert!(file.section("OUTPUT").unwrap().body().contains("PANIC"));
    }

    #[test]
    fn inline_input_captured() {
        let (_tmp, suite) = suite();
        suite.evaluate_inline("x.foo", "{42;}", &Echo).unwrap();
        let out = suite.config().stage_dir(Stage::Output).join("x.foo.einmo");
        let file = EinmoFile::from_file(&out).unwrap();
        assert_eq!(file.section("INPUT").unwrap().body(), "{42;}");
    }

    #[test]
    fn perspective_section_emitted() {
        let (_tmp, _suite) = suite();
        let tmp = tempfile::tempdir().unwrap();
        let config = TestConfig::new(tmp.path(), ValidationLevel::Output).with_perspectives(vec![
            Perspective {
                name: "shout",
                of: PerspectiveOf::Input,
                extract: |s| s.to_uppercase(),
            },
        ]);
        config.ensure_stage_dirs().unwrap();
        let suite = EinmoSuite::new(config);
        suite.evaluate_inline("p.foo", "hello", &Echo).unwrap();
        let out = suite.config().stage_dir(Stage::Output).join("p.foo.einmo");
        let file = EinmoFile::from_file(&out).unwrap();
        assert_eq!(file.section("shout").unwrap().body(), "HELLO");
    }

    #[test]
    fn parallel_and_serial_agree() {
        let tmp1 = tempfile::tempdir().unwrap();
        let tmp2 = tempfile::tempdir().unwrap();
        for (tmp, threads) in [(&tmp1, None), (&tmp2, Some(4))] {
            let config =
                TestConfig::new(tmp.path(), ValidationLevel::Output).with_parallel(threads);
            config.ensure_stage_dirs().unwrap();
            std::fs::create_dir_all(config.input_path()).unwrap();
            for i in 0..6 {
                std::fs::write(
                    config.input_path().join(format!("n{i}.foo")),
                    format!("{{{i};}}"),
                )
                .unwrap();
            }
            let suite = EinmoSuite::new(config);
            let results = suite.evaluate_all(&Echo).unwrap();
            assert_eq!(results.files.len(), 6);
            assert!(results.files.iter().all(|f| f.written_and_verified));
        }
        // Compare the INPUT bodies produced by both modes.
        let read_inputs = |tmp: &tempfile::TempDir| -> Vec<String> {
            let dir = tmp.path().join("output");
            let mut v: Vec<String> = std::fs::read_dir(&dir)
                .unwrap()
                .map(|e| {
                    let f = EinmoFile::from_file(&e.unwrap().path()).unwrap();
                    f.section("INPUT").unwrap().body().to_string()
                })
                .collect();
            v.sort();
            v
        };
        assert_eq!(read_inputs(&tmp1), read_inputs(&tmp2));
    }

    #[test]
    fn reference_resolution_and_chains() {
        assert_eq!(
            reference_of(Path::new("arith++divZero.foo"), "++"),
            Some(PathBuf::from("arith.foo"))
        );
        assert_eq!(
            reference_of(Path::new("dir/base++a++b.foo"), "++"),
            Some(PathBuf::from("dir/base++a.foo"))
        );
        assert_eq!(reference_of(Path::new("plain.foo"), "++"), None);
    }

    #[test]
    fn topological_puts_reference_first() {
        let inputs = vec![
            PathBuf::from("arith++divZero.foo"),
            PathBuf::from("arith.foo"),
        ];
        let ordered = topological_order(&inputs, "++");
        assert_eq!(ordered[0], PathBuf::from("arith.foo"));
    }

    #[test]
    fn dependent_diff_is_generated_and_signed() {
        let (_tmp, suite) = suite();
        std::fs::write(suite.config().input_path().join("base.foo"), "10 20 30").unwrap();
        std::fs::write(
            suite.config().input_path().join("base++case.foo"),
            "10 99 30",
        )
        .unwrap();
        let results = suite.evaluate_all(&Echo).unwrap();
        assert!(results.files.iter().all(|f| f.written_and_verified));
        let dep = suite
            .config()
            .stage_dir(Stage::Output)
            .join("base++case.foo.einmo");
        let file = EinmoFile::from_file(&dep).unwrap();
        assert_eq!(file.metadata().reference, "base.foo");
        let diff = file.section("DIFF").unwrap().body();
        assert!(
            diff.contains("--- reference"),
            "diff header present: {diff}"
        );
        assert!(diff.contains("+++ dependent"));
    }

    #[test]
    fn diff_limit_exceed_fails_and_marks_output_error() {
        let tmp = tempfile::tempdir().unwrap();
        // A tiny diff limit forces the exceed path.
        let mut config = TestConfig::new(tmp.path(), ValidationLevel::Output);
        config = config.with_suite_name("s");
        // Shrink the diff limit via a fresh config path is not exposed; instead
        // craft a large divergence and rely on default 2000 — build a >2000 diff.
        config.ensure_stage_dirs().unwrap();
        std::fs::create_dir_all(config.input_path()).unwrap();
        let big_ref: String = (0..300).map(|i| format!("line {i}\n")).collect();
        let big_dep: String = (0..300).map(|i| format!("DIFFERENT {i}\n")).collect();
        std::fs::write(config.input_path().join("big.foo"), &big_ref).unwrap();
        std::fs::write(config.input_path().join("big++x.foo"), &big_dep).unwrap();
        let suite = EinmoSuite::new(config);
        let results = suite.evaluate_all(&Echo).unwrap();
        let dep = results
            .files
            .iter()
            .find(|f| f.rel_path.as_path() == Path::new("big++x.foo.einmo"))
            .unwrap();
        assert_eq!(dep.status, Status::OutputError, "oversized DIFF must fail");
    }

    #[test]
    fn read_error_produces_output_error_status() {
        let (_tmp, suite) = suite();
        // `evaluate` propagates read failures via `?`.
        std::fs::write(suite.config().input_path().join("real.foo"), "{5;}").unwrap();
        let missing = suite.evaluate(Path::new("nonexistent.foo"), &Echo);
        assert!(missing.is_err(), "evaluate propagates read errors as Err");

        // `evaluate_all` must NOT abort on an unreadable input. A file with
        // invalid-UTF-8 content is discovered by the walk (it is a regular
        // file) but `read_to_string` fails, exercising the read-error →
        // failed `FileResult` path deterministically with no filesystem race.
        std::fs::write(
            suite.config().input_path().join("bad.foo"),
            [0xFF, 0xFE, 0x00, 0xADu8],
        )
        .unwrap();

        let results = suite.evaluate_all(&Echo).unwrap();
        let failed = results
            .files
            .iter()
            .find(|f| f.rel_path.as_path() == Path::new("bad.foo.einmo"))
            .expect("unreadable input produced a FileResult");
        assert_eq!(
            failed.status,
            Status::OutputError,
            "read failure recorded as output-error"
        );
        // The read error is recorded in the signed envelope's `status_detail`
        // (not in `FileResult.detail`, which is reserved for write/verify
        // failures). The envelope itself is still valid and re-verifies.
        assert!(
            failed.written_and_verified,
            "a read-failed input still writes a valid signed envelope"
        );
        let out = suite
            .config()
            .stage_dir(Stage::Output)
            .join("bad.foo.einmo");
        let file = EinmoFile::from_file(&out).unwrap();
        assert!(
            file.metadata().status_detail.contains("read error"),
            "envelope status_detail records the read error: {:?}",
            file.metadata().status_detail
        );

        // The well-formed input still succeeded.
        let ok = results
            .files
            .iter()
            .find(|f| f.rel_path.as_path() == Path::new("real.foo.einmo"))
            .expect("real input produced a FileResult");
        assert!(ok.written_and_verified);
    }

    // ---- suite integrity: einmo is critical of its own file paths ----

    /// An editor swap file in `input/` is reported and fails the suite. It is
    /// still skipped for discovery (no phantom test), but never ignored.
    #[test]
    fn extraneous_input_is_a_violation() {
        let (_tmp, suite0) = suite();
        let config = suite0.config().clone();
        std::fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        std::fs::write(config.input_path().join(".a.foo.swp"), "vim").unwrap();

        let results = EinmoSuite::new(config).evaluate_all(&Echo).unwrap();

        assert_eq!(results.files.len(), 1, "the swap file is not a test");
        assert_eq!(
            results.integrity.problems,
            vec![Problem::ExtraneousFile {
                path: PathBuf::from("input/.a.foo.swp"),
            }]
        );
        assert!(
            !results.all_output_written_and_verified(),
            "an unsound suite shape must fail the gate"
        );
        assert!(results.integrity.report().contains(".a.foo.swp"));
    }

    /// The levels escalate: each carries every requirement of the level below.
    #[test]
    fn levels_escalate_cumulatively() {
        assert_eq!(
            ValidationLevel::Output.escalation(),
            &[ValidationLevel::Output]
        );
        assert_eq!(
            ValidationLevel::Checked.escalation(),
            &[ValidationLevel::Output, ValidationLevel::Checked]
        );
        assert_eq!(
            ValidationLevel::Verified.escalation(),
            &[
                ValidationLevel::Output,
                ValidationLevel::Checked,
                ValidationLevel::Verified
            ]
        );
        assert_eq!(ValidationLevel::Output.escalates_from(), None);
        assert_eq!(
            ValidationLevel::Verified.escalates_from(),
            Some(ValidationLevel::Checked)
        );
        // Ord follows the escalation, so `level >= Checked` is meaningful.
        assert!(ValidationLevel::Verified > ValidationLevel::Checked);
        assert!(ValidationLevel::Checked > ValidationLevel::Output);
    }

    /// The Output level makes no claim about checked/ or verified/: an
    /// unpopulated higher stage is "not promoted yet", not a failure. This is
    /// the bug the escalating levels fix — a dev suite must not go red because
    /// nobody has signed a verified/ corpus.
    #[test]
    fn output_level_ignores_unpopulated_higher_stages() {
        let (_tmp, suite0) = suite();
        let config = suite0
            .config()
            .clone()
            .with_validation_level(ValidationLevel::Output);
        std::fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();

        let results = EinmoSuite::new(config).evaluate_all(&Echo).unwrap();

        assert!(
            results.integrity.is_clean(),
            "empty checked/ and verified/ are not Output-level problems: {:?}",
            results.integrity.problems
        );
        assert!(results.all_output_written_and_verified());
    }

    /// The Checked level demands a reviewed baseline: an unpromoted output is
    /// a missing right-hand side. It still says nothing about verified/.
    #[test]
    fn checked_level_demands_a_baseline_but_not_signatures() {
        let (_tmp, suite0) = suite();
        let config = suite0
            .config()
            .clone()
            .with_validation_level(ValidationLevel::Checked);
        std::fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();

        let results = EinmoSuite::new(config).evaluate_all(&Echo).unwrap();

        assert_eq!(
            results.integrity.problems,
            vec![Problem::RightMissingEntirely {
                left: Stage::Output,
                right: Stage::Checked,
                path: PathBuf::from("a.foo.einmo"),
            }],
            "checked/ is empty, so the output has no counterpart — and verified/ \
             must not be mentioned at this level"
        );
        assert_eq!(
            results.integrity.problems[0].level(),
            ValidationLevel::Checked
        );
    }

    /// Every problem knows which level owns it, so a report can be read
    /// foundation-first.
    #[test]
    fn problems_report_their_level() {
        assert_eq!(
            Problem::ExtraneousFile {
                path: PathBuf::from("input/.x.swp")
            }
            .level(),
            ValidationLevel::Output
        );
        assert_eq!(Problem::EmptySuite.level(), ValidationLevel::Output);
        assert_eq!(
            Problem::ComputerKeyAttestation {
                path: PathBuf::from("a.foo.einmo")
            }
            .level(),
            ValidationLevel::Verified
        );
        assert_eq!(
            Problem::SectionDifference {
                left: Stage::Output,
                right: Stage::Checked,
                path: PathBuf::from("a.foo.einmo"),
                section: "OUTPUT".into(),
            }
            .level(),
            ValidationLevel::Checked
        );
    }

    /// A checked artifact whose input was deleted is a signed baseline for a
    /// test that cannot be re-run — a deal breaker.
    #[test]
    fn orphaned_artifact_is_a_violation() {
        let (_tmp, suite0) = suite();
        let config = suite0
            .config()
            .clone()
            .with_validation_level(ValidationLevel::Checked);
        std::fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        // A checked artifact with no corresponding input.
        let orphan = config.stage_dir(Stage::Checked).join("ghost.foo.einmo");
        ensure_parent_dir(&orphan).unwrap();
        std::fs::write(&orphan, "not even a real envelope").unwrap();

        let results = EinmoSuite::new(config).evaluate_all(&Echo).unwrap();

        assert!(
            results
                .integrity
                .problems
                .contains(&Problem::OrphanedArtifact {
                    stage: Stage::Checked,
                    path: PathBuf::from("ghost.foo.einmo"),
                }),
            "got {:?}",
            results.integrity.problems
        );
        assert!(!results.all_output_written_and_verified());
    }

    /// `flagged/` is exempt: flagging IS retirement, so a flagged artifact with
    /// no input is a completed job, not an orphan.
    #[test]
    fn flagged_orphan_is_not_a_violation() {
        let (_tmp, suite0) = suite();
        let config = suite0.config().clone();
        std::fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        let retired = config.stage_dir(Stage::Flagged).join("retired.foo.einmo");
        ensure_parent_dir(&retired).unwrap();
        std::fs::write(&retired, "retired").unwrap();

        let results = EinmoSuite::new(config).evaluate_all(&Echo).unwrap();

        assert!(
            results.integrity.is_clean(),
            "flagged/ is the terminal sink: {:?}",
            results.integrity.problems
        );
    }

    /// A well-formed suite reports no violations.
    #[test]
    fn clean_suite_has_no_integrity_violations() {
        let (_tmp, suite0) = suite();
        let config = suite0.config().clone();
        std::fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();

        let results = EinmoSuite::new(config).evaluate_all(&Echo).unwrap();

        assert!(results.integrity.is_clean());
        assert!(results.integrity.report().is_empty());
    }

    #[test]
    fn correspondence_failure_reported_until_promoted() {
        let (_tmp, suite0) = suite();
        let config = suite0
            .config()
            .clone()
            .require_correspondence(Stage::Output, Stage::Checked);
        std::fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        let suite = EinmoSuite::new(config);
        let results = suite.evaluate_all(&Echo).unwrap();
        // checked/ is empty → only_in_a → not clean.
        assert!(
            !results.all_output_written_and_verified(),
            "empty checked/ must fail correspondence"
        );
        assert!(!results.correspondence_failures.is_empty());
    }

    // ---- Feature D: subprocess crash tests ----

    /// An evaluator that calls `std::process::abort()` — simulates a
    /// catastrophic crash that `catch_unwind` cannot catch.
    struct AbortEvaluator;
    impl Evaluator for AbortEvaluator {
        fn evaluate(&self, _source: &str) -> std::result::Result<Vec<String>, String> {
            std::process::abort();
        }
    }

    /// An evaluator that does infinite recursion — causes stack overflow
    /// (SIGSEGV), which `catch_unwind` cannot catch.
    struct StackOverflowEvaluator;
    impl Evaluator for StackOverflowEvaluator {
        fn evaluate(&self, _source: &str) -> std::result::Result<Vec<String>, String> {
            fn recurse(n: usize) -> usize {
                if n == 0 { 0 } else { recurse(n - 1) + 1 }
            }
            recurse(usize::MAX);
            Ok(vec!["unreachable".into()])
        }
    }

    #[test]
    fn crash_crumb_survives_process_abort() {
        if std::env::var("EINMO_CRASH_TEST_CHILD_ABORT").is_ok() {
            let dir = std::env::var("EINMO_CRASH_TEST_DIR").unwrap();
            let config = TestConfig::new(dir.as_str(), ValidationLevel::Output);
            let suite = EinmoSuite::new(config);
            let input_dir = std::path::Path::new(&dir).join("input");
            std::fs::create_dir_all(&input_dir).unwrap();
            std::fs::write(input_dir.join("crash.foo"), "trigger").unwrap();
            let _ = suite.evaluate(std::path::Path::new("crash.foo"), &AbortEvaluator);
            return;
        }

        let exe = std::env::current_exe().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let output = std::process::Command::new(&exe)
            .arg("crash_crumb_survives_process_abort")
            .env("EINMO_CRASH_TEST_CHILD_ABORT", "1")
            .env("EINMO_CRASH_TEST_DIR", tmp.path())
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "child should have crashed, got status: {:?}",
            output.status
        );

        let crumb_path = tmp.path().join("output").join("crash.foo.einmo");
        assert!(
            crumb_path.exists(),
            "crash-crumb should exist at {}",
            crumb_path.display()
        );

        let file = EinmoFile::from_file(&crumb_path)
            .expect("crash-crumb must be a valid signed .einmo that passes verification");
        assert_eq!(file.metadata().status, Status::OutputError);
        assert!(
            file.metadata().status_detail.contains("TEST IN PROGRESS"),
            "crash-crumb must contain the warning message: got {}",
            file.metadata().status_detail
        );
        assert!(
            file.stamps().chain_valid(&file.signed_prefix()),
            "crash-crumb stamp chain must be valid"
        );

        let config = TestConfig::new(tmp.path(), ValidationLevel::Output);
        let key = crate::config::KeySource::from_passphrase("");
        let report = crate::promote(&config, Stage::Output, Stage::Checked, &key, None, None)
            .expect("promote should succeed on the signed crash-crumb");
        assert_eq!(report.promoted.len(), 1);

        let checked_path = tmp.path().join("checked").join("crash.foo.einmo");
        let checked = EinmoFile::from_file(&checked_path).unwrap();
        let stamp_keys: Vec<&str> = checked.stamps().entries().iter().map(|s| s.key()).collect();
        assert!(
            stamp_keys.contains(&"stage:checked"),
            "checked file must have stage:checked stamp"
        );
        assert!(
            stamp_keys.contains(&"stage:output"),
            "checked file must still have stage:output stamp"
        );
    }

    #[test]
    fn crash_crumb_survives_stack_overflow() {
        if std::env::var("EINMO_CRASH_TEST_CHILD_STACK").is_ok() {
            let dir = std::env::var("EINMO_CRASH_TEST_DIR").unwrap();
            let config = TestConfig::new(dir.as_str(), ValidationLevel::Output);
            let suite = EinmoSuite::new(config);
            let input_dir = std::path::Path::new(&dir).join("input");
            std::fs::create_dir_all(&input_dir).unwrap();
            std::fs::write(input_dir.join("overflow.foo"), "trigger").unwrap();
            let _ = suite.evaluate(
                std::path::Path::new("overflow.foo"),
                &StackOverflowEvaluator,
            );
            return;
        }

        let exe = std::env::current_exe().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let _ = std::process::Command::new(&exe)
            .arg("crash_crumb_survives_stack_overflow")
            .env("EINMO_CRASH_TEST_CHILD_STACK", "1")
            .env("EINMO_CRASH_TEST_DIR", tmp.path())
            .output()
            .unwrap();
        let crumb_path = tmp.path().join("output").join("overflow.foo.einmo");
        assert!(
            crumb_path.exists(),
            "crash-crumb should survive stack overflow"
        );
        let file =
            EinmoFile::from_file(&crumb_path).expect("crash-crumb must be valid signed .einmo");
        assert!(file.metadata().status_detail.contains("TEST IN PROGRESS"));
    }

    // ---- Feature E: crumb verification test (non-subprocess) ----

    #[test]
    fn crumb_written_before_evaluation() {
        struct CrumbCheckingEvaluator {
            out_path: PathBuf,
        }
        impl Evaluator for CrumbCheckingEvaluator {
            fn evaluate(&self, _source: &str) -> std::result::Result<Vec<String>, String> {
                let content = std::fs::read_to_string(&self.out_path).unwrap();
                assert!(
                    content.contains("TEST IN PROGRESS"),
                    "crumb should exist during evaluation"
                );
                assert!(
                    content.contains("crashed during evaluation"),
                    "crumb should have warning"
                );
                Ok(vec!["ok".into()])
            }
        }
        let tmp = tempfile::tempdir().unwrap();
        let config = TestConfig::new(tmp.path(), ValidationLevel::Output);
        let suite = EinmoSuite::new(config);
        let input_dir = tmp.path().join("input");
        std::fs::create_dir_all(&input_dir).unwrap();
        std::fs::write(input_dir.join("test.foo"), "hello").unwrap();
        let out_path = tmp.path().join("output").join("test.foo.einmo");
        let result = suite
            .evaluate(Path::new("test.foo"), &CrumbCheckingEvaluator { out_path })
            .unwrap();
        assert!(result.written_and_verified);
        let final_content =
            std::fs::read_to_string(tmp.path().join("output").join("test.foo.einmo")).unwrap();
        assert!(
            !final_content.contains("TEST IN PROGRESS"),
            "crumb should be overwritten by real output"
        );
        assert!(
            final_content.contains("hello"),
            "real output should have input content"
        );
    }

    // ---- Feature F: duration limit tests ----

    struct SlowEvaluator(std::time::Duration);
    impl Evaluator for SlowEvaluator {
        fn evaluate(&self, _source: &str) -> std::result::Result<Vec<String>, String> {
            std::thread::sleep(self.0);
            Ok(vec!["slow".into()])
        }
    }

    #[test]
    fn duration_limit_exceeded_fails_test() {
        let tmp = tempfile::tempdir().unwrap();
        let config = TestConfig::new(tmp.path(), ValidationLevel::Output)
            .with_duration_limit(std::time::Duration::from_millis(10));
        let suite = EinmoSuite::new(config);
        let input_dir = tmp.path().join("input");
        std::fs::create_dir_all(&input_dir).unwrap();
        std::fs::write(input_dir.join("slow.foo"), "trigger").unwrap();
        let results = suite
            .evaluate_all(&SlowEvaluator(std::time::Duration::from_millis(100)))
            .unwrap();
        assert_eq!(results.files.len(), 1);
        assert_eq!(results.files[0].status, Status::OutputError);
        assert!(
            results.files[0]
                .detail
                .as_ref()
                .unwrap()
                .contains("EINMO_DURATION_LIMIT"),
            "detail should mention EINMO_DURATION_LIMIT: {:?}",
            results.files[0].detail
        );
    }

    #[test]
    fn suite_duration_limit_aborts_early() {
        let tmp = tempfile::tempdir().unwrap();
        // Serial: the suite-duration check runs *before starting* each test,
        // which only throttles when tests start one at a time. Under parallel
        // evaluation all workers launch before the budget expires, so nothing
        // aborts -- see the parallel-branch note in `evaluate_all`.
        let config = TestConfig::new(tmp.path(), ValidationLevel::Output)
            .with_parallel(Some(1))
            .with_suite_duration_limit(std::time::Duration::from_millis(80));
        let suite = EinmoSuite::new(config);
        let input_dir = tmp.path().join("input");
        std::fs::create_dir_all(&input_dir).unwrap();
        for i in 0..5 {
            std::fs::write(input_dir.join(format!("t{i}.foo")), "x").unwrap();
        }
        let results = suite
            .evaluate_all(&SlowEvaluator(std::time::Duration::from_millis(50)))
            .unwrap();
        assert!(
            results.files.len() < 5,
            "suite limit should have aborted early, got {} files",
            results.files.len()
        );
        assert!(
            !results.correspondence_failures.is_empty(),
            "should have a suite-limit failure message"
        );
    }

    // ---- Feature G: catastrophe crumb detection gate ----

    /// Serialize the env-touching config tests so they cannot race on the
    /// process-global `EINMO_*` environment variables.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn plant_crumb(suite: &EinmoSuite, rel: &str, source: &str) -> PathBuf {
        let out_path = suite
            .config()
            .stage_dir(Stage::Output)
            .join(mirror_input_path(Path::new(rel)));
        suite
            .write_crash_crumb(Path::new(rel), source, &out_path)
            .unwrap();
        assert!(
            suite.is_catastrophe_crumb(&out_path),
            "planted crumb must be detectable"
        );
        out_path
    }

    #[test]
    fn catastrophe_crumb_detected_suite_fails() {
        let (_tmp, suite) = suite();
        std::fs::write(suite.config().input_path().join("a.foo"), "{5;}").unwrap();
        plant_crumb(&suite, "a.foo", "{5;}");
        let results = suite.evaluate_all(&Echo).unwrap();
        let failed = results
            .files
            .iter()
            .find(|f| f.rel_path.as_path() == Path::new("a.foo.einmo"))
            .expect("crumb-gated result present");
        assert!(!failed.written_and_verified);
        assert!(!failed.ignored);
        assert!(
            failed
                .detail
                .as_ref()
                .unwrap()
                .contains("catastrophe crumb detected"),
            "detail: {:?}",
            failed.detail
        );
        assert!(!results.all_output_written_and_verified());
    }

    #[test]
    fn catastrophe_crumb_ignored_suite_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let config = TestConfig::new(tmp.path(), ValidationLevel::Output)
            .with_ignore_catastrophe_crumbs(vec![PathBuf::from("a.foo.einmo")]);
        config.ensure_stage_dirs().unwrap();
        std::fs::create_dir_all(config.input_path()).unwrap();
        let suite = EinmoSuite::new(config);
        std::fs::write(suite.config().input_path().join("a.foo"), "{5;}").unwrap();
        plant_crumb(&suite, "a.foo", "{5;}");
        let results = suite.evaluate_all(&Echo).unwrap();
        let ignored = results
            .files
            .iter()
            .find(|f| f.rel_path.as_path() == Path::new("a.foo.einmo"))
            .expect("ignored result present");
        assert!(ignored.ignored);
        assert!(!ignored.written_and_verified);
        assert!(
            ignored
                .detail
                .as_ref()
                .unwrap()
                .contains("ignored by configuration")
        );
        assert!(results.all_output_written_and_verified());
    }

    #[test]
    fn catastrophe_crumb_rerun_overwrites() {
        let tmp = tempfile::tempdir().unwrap();
        let config =
            TestConfig::new(tmp.path(), ValidationLevel::Output).with_rerun_catastrophes(true);
        config.ensure_stage_dirs().unwrap();
        std::fs::create_dir_all(config.input_path()).unwrap();
        let suite = EinmoSuite::new(config);
        std::fs::write(suite.config().input_path().join("a.foo"), "{5;}").unwrap();
        let out_path = plant_crumb(&suite, "a.foo", "{5;}");
        let results = suite.evaluate_all(&Echo).unwrap();
        let ok = results
            .files
            .iter()
            .find(|f| f.rel_path.as_path() == Path::new("a.foo.einmo"))
            .expect("rerun result present");
        assert!(ok.written_and_verified);
        assert!(!ok.ignored);
        assert_eq!(ok.status, Status::Normal);
        let file = EinmoFile::from_file(&out_path).unwrap();
        assert!(
            !file.metadata().status_detail.contains("TEST IN PROGRESS"),
            "crumb should be overwritten with real output"
        );
        assert_eq!(file.section("OUTPUT").unwrap().body(), "{5;}");
    }

    // ---- Feature G: 4-tier configuration precedence ----

    #[test]
    fn toml_suite_section_parsed() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            for v in [
                "EINMO_WALK_DEPTH_LIMIT",
                "EINMO_DURATION_LIMIT",
                "EINMO_SUITE_DURATION_LIMIT",
                "EINMO_RERUN_CATASTROPHES",
                "EINMO_IGNORE_CATASTROPHE_CRUMBS",
            ] {
                std::env::remove_var(v);
            }
        }
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("einmo.toml"),
            "[suite]\nwalk_depth_limit = 10\nduration_limit = 5\nsuite_duration_limit = 100\nrerun_catastrophes = true\nignore_catastrophe_crumbs = [\"a.foo.einmo\", \"b.foo.einmo\"]\n",
        )
        .unwrap();
        let config = TestConfig::new(tmp.path(), ValidationLevel::Output);
        assert_eq!(config.walk_depth_limit(), 10);
        assert_eq!(
            config.duration_limit(),
            Some(std::time::Duration::from_secs(5))
        );
        assert_eq!(
            config.suite_duration_limit(),
            Some(std::time::Duration::from_secs(100))
        );
        assert!(config.rerun_catastrophes());
        let ignored = config.ignore_catastrophe_crumbs();
        assert!(ignored.iter().any(|p| p == &PathBuf::from("a.foo.einmo")));
        assert!(ignored.iter().any(|p| p == &PathBuf::from("b.foo.einmo")));
    }

    #[test]
    fn per_suite_toml_beats_crate_wise() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("EINMO_WALK_DEPTH_LIMIT");
        }
        let parent = tempfile::tempdir().unwrap();
        std::fs::write(
            parent.path().join("einmo.toml"),
            "[suite]\nwalk_depth_limit = 32\n",
        )
        .unwrap();
        let work_dir = parent.path().join("suite");
        std::fs::create_dir_all(&work_dir).unwrap();
        std::fs::write(
            work_dir.join("einmo.toml"),
            "[suite]\nwalk_depth_limit = 16\n",
        )
        .unwrap();
        let config = TestConfig::new(&work_dir, ValidationLevel::Output);
        assert_eq!(config.walk_depth_limit(), 16);
    }

    #[test]
    fn env_beats_builder() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("EINMO_WALK_DEPTH_LIMIT", "10");
        }
        let tmp = tempfile::tempdir().unwrap();
        let config = TestConfig::new(tmp.path(), ValidationLevel::Output).with_walk_depth_limit(20);
        assert_eq!(config.walk_depth_limit(), 10);
        unsafe {
            std::env::remove_var("EINMO_WALK_DEPTH_LIMIT");
        }
    }
}
