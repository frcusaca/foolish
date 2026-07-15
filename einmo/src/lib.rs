//! Einmo — directory-based, cryptographically signed snapshot testing with a
//! staged promotion pipeline (`output` → `checked` → `flagged` / `verified`).
//!
//! Einmo is **standalone**: it depends on no other workspace crate and
//! reimplements its signing/format machinery from scratch, so it can be
//! promoted to its own repository later (FOOP-92 §1).
//!
//! # The four stages
//!
//! Each test suite has a work directory with an `input/` tree and four stage
//! directories (`output/`, `checked/`, `flagged/`, `verified/`) that mirror
//! the `input/` tree at any depth. Every generated output is timestamped and
//! signed; promotion between stages appends a stamp and is a deliberate act,
//! never an automated `accept`.
//!
//! # Verify-on-inspect
//!
//! Any operation that reads a `.einmo` file verifies *all* its stamps first;
//! a tampered file is refused, never operated on. The pure verification path
//! lives in [`verify`] with no filesystem/tty/Argon2 dependency (WASM-ready).

mod cli;
mod compare;
mod config;
mod einmo_suite;
mod error;
mod format;
mod signature;
mod stage;
mod transitions;
mod verify;

pub use cli::cli_main;
pub use compare::{ComparisonResult, DiffEntry, MatchSections, compare};
pub use config::{KeySource, Perspective, PerspectiveOf, StageDirs, TestConfig, resolve_stage_key};
pub use einmo_suite::{
    EinmoSuite, Evaluator, FileResult, Problem, SuiteIntegrity, TestResults, ValidationLevel,
    check_suite_integrity,
};
pub use error::EinmoError;
pub use format::{EinmoFile, Metadata, Section, Status};
pub use signature::{Stamp, StampRole, Stamps};
pub use stage::Stage;
pub use transitions::{
    FlagReport, PromotionReport, SignatureReport, confirm_signatures, flag, promote,
};
pub use verify::{
    FileVerification, StampVerification, VerificationReport, verify, verify_all, verify_bytes,
};
