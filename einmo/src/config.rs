//! Test-suite configuration, stage-directory layout, perspectives, and the
//! stage-key resolution cascade (FOOP-92 §4.5, §B.5, §8).

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{EinmoError, Result};
use crate::format::{DEFAULT_SEPARATOR, FOOLISH_SEPARATOR};
use crate::stage::{Stage, validate_stage_name};

/// The names of the four stage directories under a work directory.
///
/// Defaults mirror the [`Stage`] variants (`output`/`checked`/`flagged`/
/// `verified`); a suite may override them (validated `[A-Za-z0-9_-]+`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageDirs {
    output: String,
    checked: String,
    flagged: String,
    verified: String,
}

impl Default for StageDirs {
    fn default() -> Self {
        StageDirs {
            output: "output".into(),
            checked: "checked".into(),
            flagged: "flagged".into(),
            verified: "verified".into(),
        }
    }
}

impl StageDirs {
    /// The directory name configured for `stage`.
    #[must_use]
    pub fn name(&self, stage: Stage) -> &str {
        match stage {
            Stage::Output => &self.output,
            Stage::Checked => &self.checked,
            Stage::Flagged => &self.flagged,
            Stage::Verified => &self.verified,
        }
    }

    /// Validate all four directory names.
    ///
    /// # Errors
    ///
    /// Returns [`EinmoError::InvalidStageName`] for any name failing the
    /// `[A-Za-z0-9_-]+` rule.
    pub fn validate(&self) -> Result<()> {
        for stage in Stage::ALL {
            validate_stage_name(self.name(stage))?;
        }
        Ok(())
    }
}

/// Which sections must be byte-identical for two `.einmo` files to match (§5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchSections {
    /// INPUT + every OUTPUT[i] (and DIFF on dependents). COMMENTS excluded.
    InputOutput,
    /// Also require COMMENTS to match.
    InputOutputComments,
}

/// Whether a perspective is derived from the INPUT or from an OUTPUT chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerspectiveOf {
    /// Derived from the INPUT body.
    Input,
    /// Derived from the `i`-th OUTPUT body.
    Output(usize),
}

/// A statically configured text→text view of the input or output (§4.5).
///
/// The transform is a pure `fn` supplied by the test crate; einmo stays
/// language-agnostic (it never parses body content).
#[derive(Clone)]
pub struct Perspective {
    /// The section name this perspective emits.
    pub name: &'static str,
    /// What the perspective is derived from.
    pub of: PerspectiveOf,
    /// The pure text→text transform.
    pub extract: fn(&str) -> String,
}

impl std::fmt::Debug for Perspective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Perspective")
            .field("name", &self.name)
            .field("of", &self.of)
            .finish_non_exhaustive()
    }
}

/// A test-suite configuration.
///
/// Constructed via [`TestConfig::new`] / [`TestConfig::default_for`] and refined
/// with builder-style methods (`with_separator`, `require_correspondence`, …).
#[derive(Debug, Clone)]
pub struct TestConfig {
    work_dir: PathBuf,
    input_dir: String,
    stages: StageDirs,
    require_correspondence: Vec<(Stage, Stage)>,
    match_sections: MatchSections,
    encoding: String,
    separator: String,
    perspectives: Vec<Perspective>,
    parallel: Option<usize>,
    dependent_separator: String,
    diff_limit: usize,
    suite_name: String,
    stage_passphrases: StagePassphrases,
    configured_passphrase: String,
    /// The escalating level this suite produces and validates (FOOP-64).
    /// Required at construction: the library has no default level.
    validation_level: crate::einmo_suite::ValidationLevel,
    /// Hex prefix of the human reviewer's key, for the Verified level (V6).
    reviewer_key_prefix: Option<String>,
    /// Configurable recursion depth limit for the directory walk (Feature A).
    walk_depth_limit: usize,
    /// Per-test duration limit (`EINMO_DURATION_LIMIT`, Feature B).
    duration_limit: Option<Duration>,
    /// Per-suite duration limit (`EINMO_SUITE_DURATION_LIMIT`, Feature B).
    suite_duration_limit: Option<Duration>,
    /// Whether to overwrite and re-run tests that have a stale catastrophe
    /// crumb on disk (Feature G).
    rerun_catastrophes: bool,
    /// Mirror-relative (or input-relative) paths whose catastrophe crumbs are
    /// acknowledged as expected and skipped rather than failing the suite
    /// (Feature G).
    ignore_catastrophe_crumbs: Vec<PathBuf>,
}

/// Per-stage passphrase overrides from `einmo.toml` `[signing.<stage>]`.
///
/// `None` means the stage's passphrase is deliberately unset (→ prompt);
/// `Some("")` means the well-known empty-passphrase key.
#[derive(Debug, Clone, Default)]
struct StagePassphrases {
    output: Option<String>,
    checked: Option<String>,
    verified: Option<String>,
}

impl StagePassphrases {
    fn get(&self, stage: Stage) -> Option<&str> {
        match stage {
            Stage::Output => self.output.as_deref(),
            Stage::Checked => self.checked.as_deref(),
            Stage::Verified => self.verified.as_deref(),
            Stage::Flagged => None, // flagging never signs
        }
    }
}

impl TestConfig {
    /// A configuration for `work_dir`, validating at `level`.
    ///
    /// **`level` is required — the library has no default** (FOOP-64 §"The
    /// escalating validation levels"). Only the suite's author knows whether an
    /// unpopulated `verified/` means "not signed yet" (fine at the `Checked`
    /// level) or "the merge gate is incomplete" (fatal at `Verified`). The CLI,
    /// built on this API, may default; the library may not.
    ///
    /// Other defaults: `input/` input dir, standard stage dirs, `①`+LF
    /// separator, `InputOutput` matching, `output/checked` deployment
    /// passphrases empty, `verified` unset (→ prompt), `++` dependent
    /// separator, 2000-char diff limit.
    #[must_use]
    pub fn new(work_dir: impl Into<PathBuf>, level: crate::einmo_suite::ValidationLevel) -> Self {
        let work_dir = work_dir.into();
        let suite_name = work_dir.to_string_lossy().into_owned();
        let toml = parse_einmo_toml(&work_dir);
        TestConfig {
            work_dir,
            input_dir: "input".into(),
            stages: StageDirs::default(),
            require_correspondence: Vec::new(),
            match_sections: MatchSections::InputOutput,
            encoding: "utf-8".into(),
            separator: DEFAULT_SEPARATOR.into(),
            perspectives: Vec::new(),
            dependent_separator: "++".into(),
            diff_limit: 2000,
            suite_name,
            // Deployment convention: output/checked empty, verified unset.
            // `[signing]` in einmo.toml overrides per stage.
            stage_passphrases: StagePassphrases {
                output: Some(toml.signing.output.clone().unwrap_or_default()),
                checked: Some(toml.signing.checked.clone().unwrap_or_default()),
                verified: toml.signing.verified.clone(),
            },
            configured_passphrase: String::new(),
            validation_level: level,
            reviewer_key_prefix: toml.signing.reviewer_key_prefix.clone(),
            walk_depth_limit: toml
                .suite
                .walk_depth_limit
                .unwrap_or(crate::stage::MAX_WALK_DEPTH),
            duration_limit: toml.suite.duration_limit.map(Duration::from_secs),
            suite_duration_limit: toml.suite.suite_duration_limit.map(Duration::from_secs),
            rerun_catastrophes: toml.suite.rerun_catastrophes.unwrap_or(false),
            parallel: toml.suite.parallel,
            ignore_catastrophe_crumbs: toml
                .suite
                .ignore_catastrophe_crumbs
                .unwrap_or_default()
                .into_iter()
                .map(PathBuf::from)
                .collect(),
        }
    }

    /// Alias kept close to the spec's `TestConfig::default("…")` examples.
    #[must_use]
    pub fn default_for(
        work_dir: impl Into<PathBuf>,
        level: crate::einmo_suite::ValidationLevel,
    ) -> Self {
        Self::new(work_dir, level)
    }

    /// Use the Foolish-suite separator (`"!!"`+LF, a Foolish line comment).
    #[must_use]
    pub fn foolish_separator(mut self) -> Self {
        self.separator = FOOLISH_SEPARATOR.into();
        self
    }

    /// Set a custom section separator.
    #[must_use]
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// Add a required stage correspondence (e.g. `(Output, Checked)` for the
    /// commit gate).
    #[must_use]
    pub fn require_correspondence(mut self, a: Stage, b: Stage) -> Self {
        self.require_correspondence.push((a, b));
        self
    }

    /// Set which sections must match in [`crate::compare`].
    #[must_use]
    pub fn with_match_sections(mut self, m: MatchSections) -> Self {
        self.match_sections = m;
        self
    }

    /// Register the statically configured perspectives (§4.5).
    #[must_use]
    pub fn with_perspectives(mut self, perspectives: Vec<Perspective>) -> Self {
        self.perspectives = perspectives;
        self
    }

    /// Run with `n` threads (serial when `None`).
    #[must_use]
    pub fn with_parallel(mut self, threads: Option<usize>) -> Self {
        self.parallel = threads;
        self
    }

    /// Set the human-readable suite name (metadata `suite:`).
    #[must_use]
    pub fn with_suite_name(mut self, name: impl Into<String>) -> Self {
        self.suite_name = name.into();
        self
    }

    /// Set the recursion depth limit for the input-tree walk (Feature A).
    #[must_use]
    pub fn with_walk_depth_limit(mut self, limit: usize) -> Self {
        self.walk_depth_limit = limit;
        self
    }

    /// Set the per-test duration limit (Feature B).
    #[must_use]
    pub fn with_duration_limit(mut self, limit: Duration) -> Self {
        self.duration_limit = Some(limit);
        self
    }

    /// Set the per-suite duration limit (Feature B).
    #[must_use]
    pub fn with_suite_duration_limit(mut self, limit: Duration) -> Self {
        self.suite_duration_limit = Some(limit);
        self
    }

    /// Set whether stale catastrophe crumbs are overwritten and re-run
    /// (Feature G).
    #[must_use]
    pub fn with_rerun_catastrophes(mut self, v: bool) -> Self {
        self.rerun_catastrophes = v;
        self
    }

    /// Set the list of paths whose catastrophe crumbs are acknowledged as
    /// expected (Feature G).
    #[must_use]
    pub fn with_ignore_catastrophe_crumbs(mut self, paths: Vec<PathBuf>) -> Self {
        self.ignore_catastrophe_crumbs = paths;
        self
    }

    // ---- accessors ----

    /// The work directory.
    #[must_use]
    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }

    /// The absolute path to the `input/` directory.
    #[must_use]
    pub fn input_path(&self) -> PathBuf {
        self.work_dir.join(&self.input_dir)
    }

    /// The input directory's name (relative to the work dir; default `input`).
    #[must_use]
    pub fn input_dir(&self) -> &str {
        &self.input_dir
    }

    /// The escalating level this suite validates at.
    #[must_use]
    pub fn validation_level(&self) -> crate::einmo_suite::ValidationLevel {
        self.validation_level
    }

    /// The reviewer's key prefix for the Verified level (V6), if configured.
    ///
    /// Unset ⇒ V6 cannot be judged; V7 (no computer-key attestation) still is.
    #[must_use]
    pub fn reviewer_key_prefix(&self) -> Option<&str> {
        self.reviewer_key_prefix.as_deref()
    }

    /// Escalate (or lower) the level this suite validates at.
    ///
    /// The *initial* level is required at construction — this only refines an
    /// already-stated choice, e.g. a merge gate escalating a suite config to
    /// the Verified level.
    #[must_use]
    pub fn with_validation_level(mut self, level: crate::einmo_suite::ValidationLevel) -> Self {
        self.validation_level = level;
        self
    }

    /// Set the reviewer's public-key hex prefix (V6).
    #[must_use]
    pub fn with_reviewer_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.reviewer_key_prefix = Some(prefix.into());
        self
    }

    /// The absolute path to a stage's directory.
    #[must_use]
    pub fn stage_dir(&self, stage: Stage) -> PathBuf {
        self.work_dir.join(self.stages.name(stage))
    }

    /// The configured section separator.
    #[must_use]
    pub fn separator(&self) -> &str {
        &self.separator
    }

    /// The configured encoding.
    #[must_use]
    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    /// The suite name for envelope metadata.
    #[must_use]
    pub fn suite_name(&self) -> &str {
        &self.suite_name
    }

    /// The configured match-sections policy.
    #[must_use]
    pub fn match_sections(&self) -> MatchSections {
        self.match_sections
    }

    /// The required stage correspondences.
    #[must_use]
    pub fn required_correspondences(&self) -> &[(Stage, Stage)] {
        &self.require_correspondence
    }

    /// The configured perspectives.
    #[must_use]
    pub fn perspectives(&self) -> &[Perspective] {
        &self.perspectives
    }

    /// The thread count for `evaluate_all`; `None` means run serially.
    ///
    /// Precedence: `EINMO_PARALLEL` env > builder (`with_parallel`) > `einmo.toml`
    /// `[suite] parallel` > **default: the machine's available parallelism**.
    ///
    /// Evaluation is embarrassingly parallel (one independent test per input),
    /// so using every core is the sane default — a suite of 161 inputs should
    /// not idle 15 of 16 cores. Set `EINMO_PARALLEL=0` (or `1`) to force serial;
    /// CI can pin a lower cap that test code cannot override, matching how
    /// every other einmo limit resolves.
    ///
    /// `available_parallelism` respects cgroup/affinity limits, so a container
    /// gets its quota rather than the host's core count. If it cannot be
    /// determined, fall back to serial rather than guessing.
    #[must_use]
    pub fn parallel(&self) -> Option<usize> {
        let threads = std::env::var("EINMO_PARALLEL")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .or(self.parallel)
            .unwrap_or_else(|| {
                std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
            });
        // 0 and 1 both mean "no worker threads": run in the calling thread.
        (threads > 1).then_some(threads)
    }

    /// The dependent-name separator (default `++`).
    #[must_use]
    pub fn dependent_separator(&self) -> &str {
        &self.dependent_separator
    }

    /// The DIFF size limit for dependents (default 2000 = 25×80).
    #[must_use]
    pub fn diff_limit(&self) -> usize {
        self.diff_limit
    }

    /// The Configured Key's passphrase (default empty).
    #[must_use]
    pub(crate) fn configured_passphrase(&self) -> &str {
        &self.configured_passphrase
    }

    /// The per-stage passphrase configured for `stage`, if any.
    #[must_use]
    pub(crate) fn stage_passphrase(&self, stage: Stage) -> Option<&str> {
        self.stage_passphrases.get(stage)
    }

    /// The recursion depth limit for the input-tree walk (Feature A).
    ///
    /// Precedence: `EINMO_WALK_DEPTH_LIMIT` env > builder/toml > default.
    #[must_use]
    pub fn walk_depth_limit(&self) -> usize {
        std::env::var("EINMO_WALK_DEPTH_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(self.walk_depth_limit)
    }

    /// The per-test duration limit, if set (Feature B).
    ///
    /// Precedence: `EINMO_DURATION_LIMIT` env > builder/toml > default.
    #[must_use]
    pub fn duration_limit(&self) -> Option<Duration> {
        std::env::var("EINMO_DURATION_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .or(self.duration_limit)
    }

    /// The per-suite duration limit, if set (Feature B).
    ///
    /// Precedence: `EINMO_SUITE_DURATION_LIMIT` env > builder/toml > default.
    #[must_use]
    pub fn suite_duration_limit(&self) -> Option<Duration> {
        std::env::var("EINMO_SUITE_DURATION_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .or(self.suite_duration_limit)
    }

    /// Whether stale catastrophe crumbs are overwritten and re-run (Feature G).
    ///
    /// Precedence: `EINMO_RERUN_CATASTROPHES` env > builder/toml > default.
    #[must_use]
    pub fn rerun_catastrophes(&self) -> bool {
        std::env::var("EINMO_RERUN_CATASTROPHES")
            .ok()
            .map(|s| matches!(s.as_str(), "1" | "true" | "yes"))
            .or(Some(self.rerun_catastrophes))
            .unwrap_or(false)
    }

    /// Paths whose catastrophe crumbs are acknowledged as expected (Feature G).
    ///
    /// Precedence: `EINMO_IGNORE_CATASTROPHE_CRUMBS` env (colon-separated) >
    /// builder/toml > default.
    #[must_use]
    pub fn ignore_catastrophe_crumbs(&self) -> Vec<PathBuf> {
        std::env::var("EINMO_IGNORE_CATASTROPHE_CRUMBS")
            .ok()
            .map(|s| s.split(':').map(PathBuf::from).collect())
            .unwrap_or_else(|| self.ignore_catastrophe_crumbs.clone())
    }

    /// Ensure the four stage directories exist.
    ///
    /// # Errors
    ///
    /// Returns [`EinmoError::Io`] if creation fails.
    pub fn ensure_stage_dirs(&self) -> Result<()> {
        self.stages.validate()?;
        for stage in Stage::ALL {
            let dir = self.stage_dir(stage);
            std::fs::create_dir_all(&dir).map_err(|e| EinmoError::io(&dir, e))?;
        }
        Ok(())
    }
}

/// A resolved passphrase ready to derive a signing key.
///
/// This is a thin newtype so key material is never confused with arbitrary
/// strings and the cascade's decision is explicit at call sites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySource {
    passphrase: String,
}

impl KeySource {
    /// A key source from an explicit passphrase (empty = the computer key).
    #[must_use]
    pub fn from_passphrase(passphrase: impl Into<String>) -> Self {
        KeySource {
            passphrase: passphrase.into(),
        }
    }

    /// The resolved passphrase.
    #[must_use]
    pub(crate) fn passphrase(&self) -> &str {
        &self.passphrase
    }
}

/// The inputs to the stage-key resolution cascade (§B.5).
///
/// Each field mirrors one tier; the cascade consults them in precedence order.
#[derive(Debug, Default)]
pub struct KeyCascadeInputs<'a> {
    /// Tier 1: `--passphrase <value>`.
    pub cli_passphrase: Option<&'a str>,
    /// Tier 2: one line read from stdin via `--stdin-passphrase`.
    pub stdin_passphrase: Option<&'a str>,
    /// `--interactive`: forces the prompt, skipping tiers 1–4.
    pub interactive: bool,
    /// Tier 3: the `EINMO_PASSPHRASE` environment variable.
    pub env_passphrase: Option<&'a str>,
}

/// Resolve the stage key for a signing operation via the §B.5 cascade.
///
/// Precedence: `--passphrase` > `--stdin-passphrase` > `EINMO_PASSPHRASE` >
/// `einmo.toml [signing.<stage>]` > interactive `/dev/tty` prompt.
/// `--interactive` forces the prompt. An explicit empty string is "set to
/// empty" (the computer key), never "unset".
///
/// The interactive tier is delegated to `prompt`, a closure that reads a line
/// from a tty; injecting it keeps this function testable without a terminal.
///
/// # Errors
///
/// Returns [`EinmoError::NoKey`] if no tier yields a value and the prompt fails.
pub fn resolve_stage_key(
    stage: Stage,
    inputs: &KeyCascadeInputs<'_>,
    config: &TestConfig,
    prompt: impl FnOnce() -> Result<String>,
) -> Result<KeySource> {
    if inputs.interactive {
        return Ok(KeySource::from_passphrase(prompt()?));
    }
    if let Some(p) = inputs.cli_passphrase {
        return Ok(KeySource::from_passphrase(p));
    }
    if let Some(p) = inputs.stdin_passphrase {
        return Ok(KeySource::from_passphrase(p));
    }
    if let Some(p) = inputs.env_passphrase {
        return Ok(KeySource::from_passphrase(p));
    }
    if let Some(p) = config.stage_passphrase(stage) {
        return Ok(KeySource::from_passphrase(p));
    }
    // No tier yielded a value → interactive prompt.
    Ok(KeySource::from_passphrase(prompt()?))
}

#[derive(Debug, Clone, Default)]
pub struct EinmoTomlConfig {
    pub signing: SigningConfig,
    pub suite: SuiteConfig,
}

#[derive(Debug, Clone, Default)]
pub struct SigningConfig {
    pub output: Option<String>,
    pub checked: Option<String>,
    pub verified: Option<String>,
    /// Hex prefix of the human reviewer's key (Verified level, V6).
    pub reviewer_key_prefix: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SuiteConfig {
    pub walk_depth_limit: Option<usize>,
    pub duration_limit: Option<u64>,
    pub suite_duration_limit: Option<u64>,
    pub rerun_catastrophes: Option<bool>,
    pub ignore_catastrophe_crumbs: Option<Vec<String>>,
    /// Thread count for `evaluate_all`. `0` means serial.
    pub parallel: Option<usize>,
}

pub fn parse_einmo_toml(work_dir: &Path) -> EinmoTomlConfig {
    let crate_wide = find_crate_wise_toml(work_dir)
        .and_then(|path| std::fs::read_to_string(&path).ok())
        .and_then(|content| parse_toml_content(&content).ok())
        .unwrap_or_default();

    let candidates = [
        work_dir.join("einmo.toml"),
        work_dir.join(".config").join("einmo.toml"),
    ];
    for path in &candidates {
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(path)
            && let Ok(per_suite) = parse_toml_content(&content)
        {
            return merge_toml(crate_wide, per_suite);
        }
    }
    crate_wide
}

fn find_crate_wise_toml(work_dir: &Path) -> Option<PathBuf> {
    let mut dir = work_dir.parent()?;
    loop {
        let candidate = dir.join("einmo.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        dir = dir.parent()?;
    }
}

fn merge_toml(crate_wide: EinmoTomlConfig, per_suite: EinmoTomlConfig) -> EinmoTomlConfig {
    EinmoTomlConfig {
        signing: SigningConfig {
            reviewer_key_prefix: per_suite
                .signing
                .reviewer_key_prefix
                .clone()
                .or(crate_wide.signing.reviewer_key_prefix),
            ..per_suite.signing
        },
        suite: SuiteConfig {
            walk_depth_limit: per_suite
                .suite
                .walk_depth_limit
                .or(crate_wide.suite.walk_depth_limit),
            duration_limit: per_suite
                .suite
                .duration_limit
                .or(crate_wide.suite.duration_limit),
            suite_duration_limit: per_suite
                .suite
                .suite_duration_limit
                .or(crate_wide.suite.suite_duration_limit),
            rerun_catastrophes: per_suite
                .suite
                .rerun_catastrophes
                .or(crate_wide.suite.rerun_catastrophes),
            parallel: per_suite.suite.parallel.or(crate_wide.suite.parallel),
            ignore_catastrophe_crumbs: per_suite
                .suite
                .ignore_catastrophe_crumbs
                .or(crate_wide.suite.ignore_catastrophe_crumbs),
        },
    }
}

fn parse_toml_content(content: &str) -> Result<EinmoTomlConfig> {
    let value: toml::Value = content
        .parse()
        .map_err(|e| EinmoError::Config(format!("TOML parse: {e}")))?;
    let mut config = EinmoTomlConfig::default();

    if let Some(signing) = value.get("signing").and_then(|v| v.as_table()) {
        if let Some(v) = signing.get("reviewer_key_prefix").and_then(|v| v.as_str()) {
            config.signing.reviewer_key_prefix = Some(v.to_string());
        }
        if let Some(v) = signing.get("output").and_then(|v| v.as_str()) {
            config.signing.output = Some(v.to_string());
        }
        if let Some(v) = signing.get("checked").and_then(|v| v.as_str()) {
            config.signing.checked = Some(v.to_string());
        }
        if let Some(v) = signing.get("verified").and_then(|v| v.as_str()) {
            config.signing.verified = Some(v.to_string());
        }
    }

    if let Some(suite) = value.get("suite").and_then(|v| v.as_table()) {
        if let Some(v) = suite.get("parallel").and_then(|v| v.as_integer()) {
            config.suite.parallel = Some(v.max(0) as usize);
        }
        if let Some(v) = suite.get("walk_depth_limit").and_then(|v| v.as_integer()) {
            config.suite.walk_depth_limit = Some(v as usize);
        }
        if let Some(v) = suite.get("duration_limit").and_then(|v| v.as_integer()) {
            config.suite.duration_limit = Some(v as u64);
        }
        if let Some(v) = suite
            .get("suite_duration_limit")
            .and_then(|v| v.as_integer())
        {
            config.suite.suite_duration_limit = Some(v as u64);
        }
        if let Some(v) = suite.get("rerun_catastrophes").and_then(|v| v.as_bool()) {
            config.suite.rerun_catastrophes = Some(v);
        }
        if let Some(arr) = suite
            .get("ignore_catastrophe_crumbs")
            .and_then(|v| v.as_array())
        {
            config.suite.ignore_catastrophe_crumbs = Some(
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect(),
            );
        }
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> TestConfig {
        TestConfig::new("/tmp/suite", crate::einmo_suite::ValidationLevel::Output)
    }

    #[test]
    fn stage_dirs_default_and_paths() {
        let c = cfg();
        assert_eq!(
            c.stage_dir(Stage::Output),
            PathBuf::from("/tmp/suite/output")
        );
        assert_eq!(
            c.stage_dir(Stage::Verified),
            PathBuf::from("/tmp/suite/verified")
        );
        assert_eq!(c.input_path(), PathBuf::from("/tmp/suite/input"));
    }

    #[test]
    fn builder_sets_fields() {
        let c = cfg()
            .foolish_separator()
            .require_correspondence(Stage::Output, Stage::Checked)
            .with_match_sections(MatchSections::InputOutputComments);
        assert_eq!(c.separator(), FOOLISH_SEPARATOR);
        assert_eq!(
            c.required_correspondences(),
            &[(Stage::Output, Stage::Checked)]
        );
        assert_eq!(c.match_sections(), MatchSections::InputOutputComments);
    }

    #[test]
    fn cascade_cli_overrides_env() {
        let c = cfg();
        let inputs = KeyCascadeInputs {
            cli_passphrase: Some("cli"),
            env_passphrase: Some("env"),
            ..Default::default()
        };
        let key = resolve_stage_key(Stage::Verified, &inputs, &c, || panic!("no prompt")).unwrap();
        assert_eq!(key.passphrase(), "cli");
    }

    #[test]
    fn cascade_env_overrides_stage_config() {
        let c = cfg(); // checked stage default is Some("")
        let inputs = KeyCascadeInputs {
            env_passphrase: Some("env"),
            ..Default::default()
        };
        let key = resolve_stage_key(Stage::Checked, &inputs, &c, || panic!("no prompt")).unwrap();
        assert_eq!(key.passphrase(), "env");
    }

    #[test]
    fn cascade_stage_config_used_when_present() {
        let c = cfg();
        let inputs = KeyCascadeInputs::default();
        // Checked default passphrase is "" → computer key, no prompt.
        let key = resolve_stage_key(Stage::Checked, &inputs, &c, || panic!("no prompt")).unwrap();
        assert_eq!(key.passphrase(), "");
    }

    #[test]
    fn cascade_verified_unset_falls_through_to_prompt() {
        let c = cfg(); // verified passphrase is None
        let inputs = KeyCascadeInputs::default();
        let key = resolve_stage_key(Stage::Verified, &inputs, &c, || Ok("typed".into())).unwrap();
        assert_eq!(key.passphrase(), "typed");
    }

    #[test]
    fn interactive_flag_forces_prompt() {
        let c = cfg();
        let inputs = KeyCascadeInputs {
            cli_passphrase: Some("cli"), // ignored because interactive
            interactive: true,
            ..Default::default()
        };
        let key = resolve_stage_key(Stage::Checked, &inputs, &c, || Ok("prompted".into())).unwrap();
        assert_eq!(key.passphrase(), "prompted");
    }

    #[test]
    fn empty_string_is_set_not_unset() {
        let c = cfg();
        let inputs = KeyCascadeInputs {
            cli_passphrase: Some(""),
            ..Default::default()
        };
        let key = resolve_stage_key(Stage::Verified, &inputs, &c, || panic!("no prompt")).unwrap();
        assert_eq!(
            key.passphrase(),
            "",
            "explicit empty string is the computer key"
        );
    }
}
