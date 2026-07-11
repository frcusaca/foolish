pub mod cli;
pub mod compare;
pub mod config;
pub mod format;
pub mod signature;
pub mod snapshot_suite;
pub mod stage;
pub mod verify;

// Re-exports for convenience.
pub use compare::{ComparisonResult, DiffEntry, compare};
pub use config::{
    ConfigError, EinmoTomlConfig, KeySource, MatchSections, Perspective, PerspectiveOf, Stage,
    StageDirs, TestConfig, parse_einmo_toml, resolve_stage_key,
};
pub use format::EinmoFile;
pub use signature::{SignatureError, Stamp, StampStatus, Stamps};
pub use snapshot_suite::{
    EinmoSuite, Evaluator, FileResult, SuiteError, TestResults, compute_diff,
};
pub use stage::{
    FlagReport, PromotionReport, SignatureReport, StageError, flag, promote, resolve_reference,
    topo_sort_inputs,
};
pub use verify::{FileVerification, StampVerification, VerifyReport};
