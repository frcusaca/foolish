//! Configuration for einmo test suites.
//!
//! Defines [`Stage`], [`StageDirs`], [`TestConfig`], and related types for
//! configuring a test suite's directory layout, comparison behaviour, and
//! perspectives.

use std::fmt;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from configuration parsing or validation.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// A stage name does not match `[A-Za-z0-9_-]+`.
    #[error("invalid stage name '{0}': must match [A-Za-z0-9_-]+")]
    InvalidStageName(String),
    /// A string could not be parsed as a stage.
    #[error("unknown stage: '{0}'")]
    UnknownStage(String),
    /// I/O error reading from stdin or `/dev/tty`.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Config file parse error.
    #[error("config file error: {0}")]
    ConfigFile(String),
}

// ---------------------------------------------------------------------------
// Stage
// ---------------------------------------------------------------------------

/// The four promotion stages in the einmo pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    Output,
    Checked,
    Flagged,
    Verified,
}

impl Stage {
    /// Returns the default directory name for this stage.
    pub fn dir_name(&self) -> &'static str {
        match self {
            Self::Output => "output",
            Self::Checked => "checked",
            Self::Flagged => "flagged",
            Self::Verified => "verified",
        }
    }

    /// All four stages in canonical order.
    pub fn all() -> [Stage; 4] {
        [Self::Output, Self::Checked, Self::Flagged, Self::Verified]
    }
}

impl FromStr for Stage {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "output" => Ok(Self::Output),
            "checked" => Ok(Self::Checked),
            "flagged" => Ok(Self::Flagged),
            "verified" => Ok(Self::Verified),
            other => Err(ConfigError::UnknownStage(other.to_owned())),
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.dir_name())
    }
}

// ---------------------------------------------------------------------------
// StageDirs
// ---------------------------------------------------------------------------

/// Directory names for each stage.
#[derive(Debug, Clone)]
pub struct StageDirs {
    output: String,
    checked: String,
    flagged: String,
    verified: String,
}

impl Default for StageDirs {
    fn default() -> Self {
        Self {
            output: "output".into(),
            checked: "checked".into(),
            flagged: "flagged".into(),
            verified: "verified".into(),
        }
    }
}

impl StageDirs {
    /// Create custom stage directories, validating each name.
    pub fn new(
        output: impl Into<String>,
        checked: impl Into<String>,
        flagged: impl Into<String>,
        verified: impl Into<String>,
    ) -> Result<Self, ConfigError> {
        let dirs = Self {
            output: output.into(),
            checked: checked.into(),
            flagged: flagged.into(),
            verified: verified.into(),
        };
        dirs.validate()?;
        Ok(dirs)
    }

    /// Returns the directory name for the given stage.
    pub fn dir_name(&self, stage: Stage) -> &str {
        match stage {
            Stage::Output => &self.output,
            Stage::Checked => &self.checked,
            Stage::Flagged => &self.flagged,
            Stage::Verified => &self.verified,
        }
    }

    /// Validate all directory names match `[A-Za-z0-9_-]+`.
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_name(&self.output)?;
        validate_name(&self.checked)?;
        validate_name(&self.flagged)?;
        validate_name(&self.verified)?;
        Ok(())
    }
}

/// Validate a name matches `[A-Za-z0-9_-]+`.
fn validate_name(name: &str) -> Result<(), ConfigError> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        Err(ConfigError::InvalidStageName(name.to_owned()))
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// MatchSections
// ---------------------------------------------------------------------------

/// Which sections to compare between stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchSections {
    /// Compare INPUT and all OUTPUT sections (COMMENTS ignored).
    InputOutput,
    /// Compare INPUT, all OUTPUT sections, and COMMENTS.
    InputOutputComments,
}

// ---------------------------------------------------------------------------
// Key resolution
// ---------------------------------------------------------------------------

/// How a passphrase was obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    CliFlag,
    StdinPassphrase,
    EnvVar,
    ConfigFile,
    Interactive,
}

/// Per-stage signing configuration from `einmo.toml`.
#[derive(Debug, Clone, Default)]
pub struct EinmoTomlConfig {
    pub signing: SigningConfig,
}

#[derive(Debug, Clone, Default)]
pub struct SigningConfig {
    pub configured_key: Option<String>,
    pub stages: std::collections::HashMap<String, StageSigningConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct StageSigningConfig {
    pub passphrase: Option<String>,
}

/// Parse `einmo.toml` from the work directory (or `.config/einmo.toml`).
pub fn parse_einmo_toml(work_dir: &Path) -> Result<EinmoTomlConfig, ConfigError> {
    let candidates = [
        work_dir.join("einmo.toml"),
        work_dir.join(".config").join("einmo.toml"),
    ];
    for path in &candidates {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| ConfigError::ConfigFile(format!("read {}: {e}", path.display())))?;
            return parse_toml_content(&content);
        }
    }
    Ok(EinmoTomlConfig::default())
}

fn parse_toml_content(content: &str) -> Result<EinmoTomlConfig, ConfigError> {
    let value: toml::Value = content
        .parse()
        .map_err(|e| ConfigError::ConfigFile(format!("TOML parse: {e}")))?;

    let mut config = EinmoTomlConfig::default();

    if let Some(signing) = value.get("signing") {
        if let Some(ck) = signing.get("configured-key").and_then(|v| v.as_str()) {
            config.signing.configured_key = Some(ck.to_string());
        }

        if let Some(table) = signing.as_table() {
            for (stage_name, stage_val) in table {
                if stage_name == "configured-key" {
                    continue;
                }
                if let Some(stage_table) = stage_val.as_table() {
                    let pass = stage_table
                        .get("passphrase")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    config
                        .signing
                        .stages
                        .insert(stage_name.clone(), StageSigningConfig { passphrase: pass });
                }
            }
        }
    }

    Ok(config)
}

/// Resolve the stage key passphrase using the 5-tier cascade.
///
/// Precedence:
/// 1. `--passphrase <v>` (explicit, non-interactive)
/// 2. `--stdin-passphrase` (read one line from stdin)
/// 3. `EINMO_PASSPHRASE` env var
/// 4. `einmo.toml` `[signing.<stage>] passphrase`
/// 5. Interactive prompt on `/dev/tty`
///
/// `--interactive` forces the prompt (skips tiers 1–4).
/// Explicit empty string = "set to empty" (well-known computer key), NOT "unset".
pub fn resolve_stage_key(
    stage: Stage,
    cli_pass: Option<&str>,
    stdin_pass: bool,
    interactive: bool,
    env: Option<&str>,
    toml_config: &EinmoTomlConfig,
) -> Result<(String, KeySource), ConfigError> {
    if interactive {
        return read_tty_passphrase(stage).map(|p| (p, KeySource::Interactive));
    }

    if let Some(pass) = cli_pass {
        return Ok((pass.to_string(), KeySource::CliFlag));
    }

    if stdin_pass {
        let pass = read_stdin_line()?;
        return Ok((pass, KeySource::StdinPassphrase));
    }

    if let Some(pass) = env {
        return Ok((pass.to_string(), KeySource::EnvVar));
    }

    let stage_key = stage.dir_name();
    if let Some(stage_cfg) = toml_config.signing.stages.get(stage_key)
        && let Some(pass) = &stage_cfg.passphrase
    {
        return Ok((pass.clone(), KeySource::ConfigFile));
    }

    read_tty_passphrase(stage).map(|p| (p, KeySource::Interactive))
}

fn read_stdin_line() -> Result<String, ConfigError> {
    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(line.trim_end_matches('\n').to_string())
}

fn read_tty_passphrase(stage: Stage) -> Result<String, ConfigError> {
    match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
    {
        Ok(file) => {
            eprint!("Enter passphrase for stage '{}': ", stage);
            let mut line = String::new();
            std::io::BufReader::new(file).read_line(&mut line)?;
            Ok(line.trim_end_matches('\n').to_string())
        }
        Err(e) => Err(ConfigError::Io(e)),
    }
}

// ---------------------------------------------------------------------------
// Perspective
// ---------------------------------------------------------------------------

/// What a perspective section derives from.
#[derive(Debug, Clone)]
pub enum PerspectiveOf {
    /// Derives from the INPUT section.
    Input,
    /// Derives from the i-th OUTPUT section (0-indexed).
    Output(usize),
}

/// A named perspective: a pure text→text transform over a section.
///
/// Perspectives produce derived views of an einmo file's content (e.g.
/// a brane-name view for Foolish output).
#[derive(Debug, Clone)]
pub struct Perspective {
    name: String,
    of: PerspectiveOf,
    extract: fn(&str) -> String,
}

impl Perspective {
    /// Create a new perspective.
    pub fn new(name: impl Into<String>, of: PerspectiveOf, extract: fn(&str) -> String) -> Self {
        Self {
            name: name.into(),
            of,
            extract,
        }
    }

    /// The perspective name (used as the section name in the `.einmo` file).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What this perspective derives from.
    pub fn of(&self) -> &PerspectiveOf {
        &self.of
    }

    /// Apply the extract transform to input text.
    pub fn apply(&self, text: &str) -> String {
        (self.extract)(text)
    }
}

// ---------------------------------------------------------------------------
// TestConfig
// ---------------------------------------------------------------------------

/// Configuration for an einmo test suite.
///
/// Created via [`TestConfig::new`] with sensible defaults; override fields
/// via the builder methods.
#[derive(Debug)]
pub struct TestConfig {
    /// Root directory of the test suite.
    work_dir: PathBuf,
    /// Name of the input directory (default: "input").
    input_dir: String,
    /// Stage directory names.
    stages: StageDirs,
    /// Pairs of stages that must match (enforced by `compare`).
    require_correspondence: Vec<(Stage, Stage)>,
    /// Which sections to compare.
    match_sections: MatchSections,
    /// File encoding (default: "utf-8").
    encoding: String,
    /// Section separator (default: "①\n").
    separator: String,
    /// Named perspectives (pure transforms).
    perspectives: Vec<Perspective>,
    /// Parallel thread count (`None` = serial).
    parallel: Option<usize>,
    /// Dependent separator (default: "++").
    dependent_separator: String,
    /// Diff limit in characters (default: 2000 = 25×80).
    diff_limit: usize,
}

impl TestConfig {
    /// Create a new config with defaults for the given work directory.
    pub fn new(work_dir: impl Into<PathBuf>) -> Self {
        Self {
            work_dir: work_dir.into(),
            input_dir: "input".into(),
            stages: StageDirs::default(),
            require_correspondence: Vec::new(),
            match_sections: MatchSections::InputOutput,
            encoding: "utf-8".into(),
            separator: "①\n".into(),
            perspectives: Vec::new(),
            parallel: None,
            dependent_separator: "++".into(),
            diff_limit: 2000,
        }
    }

    // -- Accessors ----------------------------------------------------------

    /// Root directory of the test suite.
    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }

    /// Input directory name.
    pub fn input_dir(&self) -> &str {
        &self.input_dir
    }

    /// Stage directory names.
    pub fn stages(&self) -> &StageDirs {
        &self.stages
    }

    /// Required correspondence pairs.
    pub fn require_correspondence(&self) -> &[(Stage, Stage)] {
        &self.require_correspondence
    }

    /// Match sections setting.
    pub fn match_sections(&self) -> MatchSections {
        self.match_sections
    }

    /// File encoding.
    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    /// Section separator.
    pub fn separator(&self) -> &str {
        &self.separator
    }

    /// Configured perspectives.
    pub fn perspectives(&self) -> &[Perspective] {
        &self.perspectives
    }

    /// Parallel thread count.
    pub fn parallel(&self) -> Option<usize> {
        self.parallel
    }

    /// Dependent separator.
    pub fn dependent_separator(&self) -> &str {
        &self.dependent_separator
    }

    /// Diff limit in characters.
    pub fn diff_limit(&self) -> usize {
        self.diff_limit
    }

    /// Full path to the given stage's directory.
    pub fn stage_dir(&self, stage: Stage) -> PathBuf {
        self.work_dir.join(self.stages.dir_name(stage))
    }

    /// Full path to the input directory.
    pub fn input_path(&self) -> PathBuf {
        self.work_dir.join(&self.input_dir)
    }

    // -- Builders -----------------------------------------------------------

    /// Set the input directory name.
    pub fn with_input_dir(mut self, name: impl Into<String>) -> Self {
        self.input_dir = name.into();
        self
    }

    /// Replace stage directories.
    pub fn with_stages(mut self, stages: StageDirs) -> Self {
        self.stages = stages;
        self
    }

    /// Add a required correspondence pair.
    pub fn require(mut self, a: Stage, b: Stage) -> Self {
        self.require_correspondence.push((a, b));
        self
    }

    /// Set match sections.
    pub fn with_match_sections(mut self, ms: MatchSections) -> Self {
        self.match_sections = ms;
        self
    }

    /// Set encoding.
    pub fn with_encoding(mut self, enc: impl Into<String>) -> Self {
        self.encoding = enc.into();
        self
    }

    /// Set separator.
    pub fn with_separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Add a perspective.
    pub fn with_perspective(mut self, p: Perspective) -> Self {
        self.perspectives.push(p);
        self
    }

    /// Set parallel thread count.
    pub fn with_parallel(mut self, n: usize) -> Self {
        self.parallel = Some(n);
        self
    }

    /// Set dependent separator.
    pub fn with_dependent_separator(mut self, sep: impl Into<String>) -> Self {
        self.dependent_separator = sep.into();
        self
    }

    /// Set diff limit.
    pub fn with_diff_limit(mut self, limit: usize) -> Self {
        self.diff_limit = limit;
        self
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_dir_names() {
        assert_eq!(Stage::Output.dir_name(), "output");
        assert_eq!(Stage::Checked.dir_name(), "checked");
        assert_eq!(Stage::Flagged.dir_name(), "flagged");
        assert_eq!(Stage::Verified.dir_name(), "verified");
    }

    #[test]
    fn stage_from_str_roundtrip() {
        for stage in Stage::all() {
            let s = stage.dir_name();
            let parsed: Stage = s.parse().unwrap();
            assert_eq!(parsed, stage);
        }
    }

    #[test]
    fn stage_from_str_unknown() {
        let result: Result<Stage, _> = "nope".parse();
        assert!(result.is_err());
    }

    #[test]
    fn stage_display() {
        assert_eq!(format!("{}", Stage::Output), "output");
        assert_eq!(format!("{}", Stage::Verified), "verified");
    }

    #[test]
    fn stage_dirs_default() {
        let dirs = StageDirs::default();
        assert_eq!(dirs.dir_name(Stage::Output), "output");
        assert_eq!(dirs.dir_name(Stage::Checked), "checked");
        assert_eq!(dirs.dir_name(Stage::Flagged), "flagged");
        assert_eq!(dirs.dir_name(Stage::Verified), "verified");
    }

    #[test]
    fn stage_dirs_custom_valid() {
        let dirs = StageDirs::new("gen", "review", "hold", "sign").unwrap();
        assert_eq!(dirs.dir_name(Stage::Output), "gen");
        assert_eq!(dirs.dir_name(Stage::Verified), "sign");
    }

    #[test]
    fn stage_dirs_invalid_name_rejected() {
        assert!(StageDirs::new("out put", "checked", "flagged", "verified").is_err());
        assert!(StageDirs::new("output", "checked!", "flagged", "verified").is_err());
        assert!(StageDirs::new("", "checked", "flagged", "verified").is_err());
    }

    #[test]
    fn stage_dirs_underscore_dash_allowed() {
        let dirs = StageDirs::new("my-output", "my_checked", "flagged", "verified").unwrap();
        assert_eq!(dirs.dir_name(Stage::Output), "my-output");
        assert_eq!(dirs.dir_name(Stage::Checked), "my_checked");
    }

    #[test]
    fn test_config_defaults() {
        let cfg = TestConfig::new("/tmp/suite");
        assert_eq!(cfg.work_dir(), Path::new("/tmp/suite"));
        assert_eq!(cfg.input_dir(), "input");
        assert_eq!(cfg.match_sections(), MatchSections::InputOutput);
        assert_eq!(cfg.encoding(), "utf-8");
        assert_eq!(cfg.separator(), "①\n");
        assert!(cfg.perspectives().is_empty());
        assert_eq!(cfg.parallel(), None);
        assert_eq!(cfg.dependent_separator(), "++");
        assert_eq!(cfg.diff_limit(), 2000);
        assert!(cfg.require_correspondence().is_empty());
    }

    #[test]
    fn test_config_stage_dir() {
        let cfg = TestConfig::new("/tmp/suite");
        assert_eq!(
            cfg.stage_dir(Stage::Output),
            PathBuf::from("/tmp/suite/output")
        );
        assert_eq!(
            cfg.stage_dir(Stage::Checked),
            PathBuf::from("/tmp/suite/checked")
        );
        assert_eq!(
            cfg.stage_dir(Stage::Flagged),
            PathBuf::from("/tmp/suite/flagged")
        );
        assert_eq!(
            cfg.stage_dir(Stage::Verified),
            PathBuf::from("/tmp/suite/verified")
        );
    }

    #[test]
    fn test_config_input_path() {
        let cfg = TestConfig::new("/tmp/suite");
        assert_eq!(cfg.input_path(), PathBuf::from("/tmp/suite/input"));
    }

    #[test]
    fn test_config_builders() {
        let cfg = TestConfig::new("/tmp/s")
            .with_input_dir("tests")
            .with_match_sections(MatchSections::InputOutputComments)
            .with_encoding("ascii")
            .with_separator("!!\n")
            .with_parallel(4)
            .with_dependent_separator("::")
            .with_diff_limit(1000)
            .require(Stage::Output, Stage::Checked);

        assert_eq!(cfg.input_dir(), "tests");
        assert_eq!(cfg.match_sections(), MatchSections::InputOutputComments);
        assert_eq!(cfg.encoding(), "ascii");
        assert_eq!(cfg.separator(), "!!\n");
        assert_eq!(cfg.parallel(), Some(4));
        assert_eq!(cfg.dependent_separator(), "::");
        assert_eq!(cfg.diff_limit(), 1000);
        assert_eq!(
            cfg.require_correspondence(),
            &[(Stage::Output, Stage::Checked)]
        );
    }

    #[test]
    fn test_config_stage_dir_with_custom_stages() {
        let stages = StageDirs::new("gen", "review", "hold", "sign").unwrap();
        let cfg = TestConfig::new("/s").with_stages(stages);
        assert_eq!(cfg.stage_dir(Stage::Output), PathBuf::from("/s/gen"));
        assert_eq!(cfg.stage_dir(Stage::Verified), PathBuf::from("/s/sign"));
    }

    #[test]
    fn test_config_perspective() {
        let p = Perspective::new("names", PerspectiveOf::Input, |s| s.to_uppercase());
        let cfg = TestConfig::new("/s").with_perspective(p);
        assert_eq!(cfg.perspectives().len(), 1);
        assert_eq!(cfg.perspectives()[0].name(), "names");
        assert_eq!(cfg.perspectives()[0].apply("hello"), "HELLO");
    }

    #[test]
    fn validate_name_rejects_empty() {
        assert!(validate_name("").is_err());
    }

    #[test]
    fn validate_name_rejects_spaces() {
        assert!(validate_name("has space").is_err());
    }

    #[test]
    fn validate_name_accepts_alphanumeric() {
        assert!(validate_name("output123").is_ok());
    }

    #[test]
    fn validate_name_accepts_dash_underscore() {
        assert!(validate_name("my-stage_v2").is_ok());
    }

    // -- Key resolution tests ------------------------------------------------

    #[test]
    fn cli_overrides_env() {
        let toml = EinmoTomlConfig::default();
        let (pass, source) = resolve_stage_key(
            Stage::Output,
            Some("cli-pass"),
            false,
            false,
            Some("env-pass"),
            &toml,
        )
        .unwrap();
        assert_eq!(pass, "cli-pass");
        assert_eq!(source, KeySource::CliFlag);
    }

    #[test]
    fn env_overrides_config() {
        let mut toml = EinmoTomlConfig::default();
        toml.signing.stages.insert(
            "output".into(),
            StageSigningConfig {
                passphrase: Some("config-pass".into()),
            },
        );
        let (pass, source) =
            resolve_stage_key(Stage::Output, None, false, false, Some("env-pass"), &toml).unwrap();
        assert_eq!(pass, "env-pass");
        assert_eq!(source, KeySource::EnvVar);
    }

    #[test]
    fn per_stage_config_used() {
        let mut toml = EinmoTomlConfig::default();
        toml.signing.stages.insert(
            "verified".into(),
            StageSigningConfig {
                passphrase: Some("verified-pass".into()),
            },
        );
        let (pass, source) =
            resolve_stage_key(Stage::Verified, None, false, false, None, &toml).unwrap();
        assert_eq!(pass, "verified-pass");
        assert_eq!(source, KeySource::ConfigFile);
    }

    #[test]
    fn empty_vs_unset_distinction() {
        let toml = EinmoTomlConfig::default();
        let (pass, source) =
            resolve_stage_key(Stage::Output, Some(""), false, false, None, &toml).unwrap();
        assert_eq!(pass, "");
        assert_eq!(source, KeySource::CliFlag);

        let (pass, source) =
            resolve_stage_key(Stage::Output, None, false, false, Some(""), &toml).unwrap();
        assert_eq!(pass, "");
        assert_eq!(source, KeySource::EnvVar);
    }

    #[test]
    fn parse_toml_with_signing() {
        let content = r#"
[signing]
configured-key = "my-key"

[signing.output]
passphrase = ""

[signing.verified]
passphrase = "human-only"
"#;
        let config = parse_toml_content(content).unwrap();
        assert_eq!(config.signing.configured_key.as_deref(), Some("my-key"));
        assert_eq!(
            config.signing.stages["output"].passphrase.as_deref(),
            Some("")
        );
        assert_eq!(
            config.signing.stages["verified"].passphrase.as_deref(),
            Some("human-only")
        );
    }

    #[test]
    fn parse_toml_missing_file_returns_default() {
        let config = parse_einmo_toml(Path::new("/nonexistent/dir")).unwrap();
        assert!(config.signing.configured_key.is_none());
        assert!(config.signing.stages.is_empty());
    }
}
