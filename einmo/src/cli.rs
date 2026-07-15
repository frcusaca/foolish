//! The single `einmo` CLI app (FOOP-92 §B.6, Phase 10).
//!
//! Every subcommand calls the library; every read verifies-on-inspect. Stage
//! pairs use the ASCII arrow `->` (`output->checked`); stage names are
//! validated `[A-Za-z0-9_-]+`.

use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use crate::config::{KeyCascadeInputs, KeySource, MatchSections, TestConfig, resolve_stage_key};
use crate::error::{EinmoError, Result};
use crate::format::EinmoFile;
use crate::stage::Stage;

/// The `einmo` command-line interface.
#[derive(Parser, Debug)]
#[command(
    name = "einmo",
    version,
    about = "Signed directory-based snapshot testing"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Promote files between stages (appends the destination stage's stamp).
    Promote(PromoteArgs),
    /// Move files from a stage into flagged/ (advisory line, no stamp).
    Flag(FlagArgs),
    /// Compare two stages over the mirrored tree.
    Compare(CompareArgs),
    /// Verify signatures across a stage (or all stages).
    Verify(VerifyArgs),
    /// Verify signatures on a path (legacy-compatible subcommand name).
    VerifySignatures(VerifyArgs),
    /// Report files carrying a signer whose pubkey starts with a prefix.
    ConfirmSignatures(ConfirmArgs),
    /// Show an envelope's summary and stamp chain.
    Show(ShowArgs),
    /// List the suite's tests and which stages hold each one.
    List(ListArgs),
    /// Print an envelope's signed body sections (verify-on-inspect first).
    Body(BodyArgs),
    /// Compute the SHA-256 of this binary (self-attestation).
    SelfCheck(SelfCheckArgs),
}

#[derive(Args, Debug)]
struct PromoteArgs {
    /// The `<from>-><to>` stage pair, e.g. `output->checked`.
    transition: String,
    /// The suite work directory.
    work_dir: PathBuf,
    /// Specific `.einmo` files to act on (mirror-relative, stage-relative, or
    /// absolute). Use `-` to read paths from stdin (one per line).
    #[arg(num_args = 0.., trailing_var_arg = true)]
    files: Vec<PathBuf>,
    /// Restrict to inputs matching this glob (`*` wildcard).
    #[arg(long)]
    filter: Option<String>,
    /// Explicit passphrase (tier 1). Empty string = the computer key.
    #[arg(long)]
    passphrase: Option<String>,
    /// Read one passphrase line from stdin (tier 2).
    #[arg(long)]
    stdin_passphrase: bool,
    /// Force the interactive prompt (skips tiers 1–4).
    #[arg(long)]
    interactive: bool,
    /// Override the directory-walk depth limit (tier 1).
    #[arg(long)]
    walk_depth_limit: Option<usize>,
    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct FlagArgs {
    /// The suite work directory.
    work_dir: PathBuf,
    /// The stage to flag from.
    stage: String,
    /// Specific `.einmo` files to act on. Use `-` to read paths from stdin.
    #[arg(num_args = 0.., trailing_var_arg = true)]
    files: Vec<PathBuf>,
    /// Restrict to inputs matching this glob.
    #[arg(long)]
    filter: Option<String>,
    /// The advisory reason recorded in the flagged file.
    #[arg(long, default_value = "")]
    reason: String,
    /// Override the directory-walk depth limit (tier 1).
    #[arg(long)]
    walk_depth_limit: Option<usize>,
    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct CompareArgs {
    /// Stage A.
    stage_a: String,
    /// Stage B.
    stage_b: String,
    /// The suite work directory.
    work_dir: PathBuf,
    /// Specific `.einmo` files to act on. Use `-` to read paths from stdin.
    #[arg(num_args = 0.., trailing_var_arg = true)]
    files: Vec<PathBuf>,
    /// Require COMMENTS to match too.
    #[arg(long)]
    require_comments_match: bool,
    /// Exit non-zero if any file differs / is one-sided / is tampered.
    #[arg(long)]
    require_match: bool,
    /// Report the deepest differing descendants (root-cause diagnostic).
    #[arg(long)]
    root_cause: bool,
    /// Override the directory-walk depth limit (tier 1).
    #[arg(long)]
    walk_depth_limit: Option<usize>,
    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct VerifyArgs {
    /// The suite work directory.
    work_dir: PathBuf,
    /// Restrict to one stage.
    #[arg(long)]
    stage: Option<String>,
    /// Verify all stages (the default).
    #[arg(long)]
    all: bool,
    /// Specific `.einmo` files to act on. Use `-` to read paths from stdin.
    #[arg(num_args = 0.., trailing_var_arg = true)]
    files: Vec<PathBuf>,
    /// Maximum recursion depth for directory walks.
    #[arg(long)]
    walk_depth_limit: Option<usize>,
    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct ConfirmArgs {
    /// A directory (or file) of `.einmo` files.
    path: PathBuf,
    /// The pubkey hex prefix to match.
    pubkey_prefix: String,
    /// Exit non-zero if any file lacks a matching signer.
    #[arg(long)]
    require_all: bool,
    /// Override the directory-walk depth limit (tier 1).
    #[arg(long)]
    walk_depth_limit: Option<usize>,
    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct ShowArgs {
    /// The `.einmo` file to show.
    file: PathBuf,
    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct ListArgs {
    /// The suite work directory.
    work_dir: PathBuf,
    /// Only tests whose mirror-relative path contains this substring.
    #[arg(long)]
    filter: Option<String>,
    /// Only tests whose stage bodies are not all identical (ignores stamps and
    /// metadata, exactly as `compare` does). Absent artifacts count as differing.
    #[arg(long)]
    differing: bool,
    /// Emit machine-readable JSON (one object per line).
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct BodyArgs {
    /// The `.einmo` file whose body to print.
    file: PathBuf,
    /// Print only this section (e.g. `INPUT`, `OUTPUT`, `COMMENTS`).
    #[arg(long)]
    section: Option<String>,
    /// Do not print `=== NAME ===` headers between sections.
    #[arg(long)]
    bare: bool,
}

#[derive(Args, Debug)]
struct SelfCheckArgs {
    /// Exit non-zero if the computed hash does not match this value.
    #[arg(long)]
    expected: Option<String>,
    /// Print only the hash.
    #[arg(long)]
    quiet: bool,
}

/// The shared CLI entry point used by both the `einmo` and `cargo-einmo` bins.
///
/// Returns a process exit code; every error is reported to stderr.
#[must_use]
pub fn cli_main(args: Vec<OsString>) -> ExitCode {
    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(e) => {
            // clap prints help/version to stdout with a success code.
            let _ = e.print();
            return if e.use_stderr() {
                ExitCode::from(2)
            } else {
                ExitCode::SUCCESS
            };
        }
    };
    match dispatch(cli.command) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("einmo: {e}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(command: Command) -> Result<ExitCode> {
    match command {
        Command::Promote(a) => cmd_promote(a),
        Command::Flag(a) => cmd_flag(a),
        Command::Compare(a) => cmd_compare(a),
        Command::Verify(a) | Command::VerifySignatures(a) => cmd_verify(a),
        Command::ConfirmSignatures(a) => cmd_confirm(a),
        Command::Show(a) => cmd_show(a),
        Command::List(a) => cmd_list(a),
        Command::Body(a) => cmd_body(a),
        Command::SelfCheck(a) => cmd_self_check(a),
    }
}

/// Parse an `<from>-><to>` transition into a stage pair.
fn parse_transition(s: &str) -> Result<(Stage, Stage)> {
    let (from, to) = s
        .split_once("->")
        .ok_or_else(|| EinmoError::Config(format!("transition {s:?} must be `<from>-><to>`")))?;
    Ok((Stage::parse(from.trim())?, Stage::parse(to.trim())?))
}

/// Expand any `-` entries in `files` by pulling paths from `stdin_lines` (one
/// per line, blanks skipped). Non-`-` entries are kept verbatim. An empty
/// `files` yields an empty result.
fn resolve_files_from_iter(
    files: Vec<PathBuf>,
    stdin_lines: impl Iterator<Item = String>,
) -> Vec<PathBuf> {
    if files.is_empty() {
        return Vec::new();
    }
    if !files.iter().any(|p| p.to_string_lossy() == "-") {
        return files;
    }
    let stdin_paths: Vec<PathBuf> = stdin_lines
        .filter(|line| !line.trim().is_empty())
        .map(PathBuf::from)
        .collect();
    let mut result = Vec::new();
    for f in files {
        if f.to_string_lossy() == "-" {
            result.extend(stdin_paths.clone());
        } else {
            result.push(f);
        }
    }
    result
}

/// Resolve the `files` positional list, reading stdin when `-` appears.
///
/// # Errors
///
/// Returns [`EinmoError::Io`] if stdin cannot be read.
fn resolve_files(files: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    if !files.iter().any(|p| p.to_string_lossy() == "-") {
        return Ok(files);
    }
    let mut input = String::new();
    std::io::stdin()
        .lock()
        .read_to_string(&mut input)
        .map_err(|e| EinmoError::io("<stdin>", e))?;
    Ok(resolve_files_from_iter(
        files,
        input.lines().map(String::from),
    ))
}

/// Convert a resolved `Vec<PathBuf>` into the `Option<&[PathBuf]>` the library
/// functions expect (`None` when empty so the walk/filter path is used).
fn files_ref(files: &[PathBuf]) -> Option<&[PathBuf]> {
    if files.is_empty() { None } else { Some(files) }
}

fn cmd_promote(args: PromoteArgs) -> Result<ExitCode> {
    let (from, to) = parse_transition(&args.transition)?;
    let mut config = TestConfig::new(&args.work_dir);
    if let Some(limit) = args.walk_depth_limit {
        config = config.with_walk_depth_limit(limit);
    }
    let key = resolve_promotion_key(to, &args, &config)?;
    let files = resolve_files(args.files)?;
    let report = crate::promote(
        &config,
        from,
        to,
        &key,
        args.filter.as_deref(),
        files_ref(&files),
    )?;

    // Warn on any non-human verified attestation.
    for promoted in &report.promoted {
        if promoted.non_human {
            eprintln!(
                "einmo: warning: {} verified under a well-known computer key (non-human attestation)",
                promoted.rel_path.display()
            );
        }
    }
    if args.json {
        println!(
            "{{\"promoted\":{},\"non_human\":{}}}",
            report.promoted.len(),
            report.promoted.iter().filter(|p| p.non_human).count()
        );
    } else {
        println!("promoted {} file(s) {from}->{to}", report.promoted.len());
    }
    Ok(ExitCode::SUCCESS)
}

/// Resolve the destination stage's key (only `*->verified` truly needs one; the
/// `output->checked`/config defaults resolve without a prompt).
fn resolve_promotion_key(to: Stage, args: &PromoteArgs, config: &TestConfig) -> Result<KeySource> {
    let stdin_line = if args.stdin_passphrase {
        Some(read_stdin_line()?)
    } else {
        None
    };
    let env_pass = std::env::var("EINMO_PASSPHRASE").ok();
    let inputs = KeyCascadeInputs {
        cli_passphrase: args.passphrase.as_deref(),
        stdin_passphrase: stdin_line.as_deref(),
        interactive: args.interactive,
        env_passphrase: env_pass.as_deref(),
    };
    resolve_stage_key(to, &inputs, config, prompt_tty)
}

fn cmd_flag(args: FlagArgs) -> Result<ExitCode> {
    let stage = Stage::parse(&args.stage)?;
    let mut config = TestConfig::new(&args.work_dir);
    if let Some(limit) = args.walk_depth_limit {
        config = config.with_walk_depth_limit(limit);
    }
    let files = resolve_files(args.files)?;
    let report = crate::flag(
        &config,
        stage,
        args.filter.as_deref(),
        &args.reason,
        files_ref(&files),
    )?;
    if args.json {
        println!("{{\"flagged\":{}}}", report.flagged.len());
    } else {
        println!("flagged {} file(s) from {stage}", report.flagged.len());
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_compare(args: CompareArgs) -> Result<ExitCode> {
    let a = Stage::parse(&args.stage_a)?;
    let b = Stage::parse(&args.stage_b)?;
    let mut config = TestConfig::new(&args.work_dir);
    if let Some(limit) = args.walk_depth_limit {
        config = config.with_walk_depth_limit(limit);
    }
    let sections = if args.require_comments_match {
        MatchSections::InputOutputComments
    } else {
        MatchSections::InputOutput
    };
    let files = resolve_files(args.files)?;
    let result = crate::compare(&config, a, b, sections, files_ref(&files))?;

    if args.json {
        println!(
            "{{\"matching\":{},\"differing\":{},\"only_in_a\":{},\"only_in_b\":{},\"tampered\":{}}}",
            result.matching.len(),
            result.differing.len(),
            result.only_in_a.len(),
            result.only_in_b.len(),
            result.tampered.len()
        );
    } else {
        println!(
            "{a} vs {b}: {} matching, {} differing, {} only-in-{a}, {} only-in-{b}, {} tampered",
            result.matching.len(),
            result.differing.len(),
            result.only_in_a.len(),
            result.only_in_b.len(),
            result.tampered.len()
        );
        for entry in &result.differing {
            println!(
                "  differing {} [{}]",
                entry.rel_path.display(),
                entry.sections.join(", ")
            );
        }
        if args.root_cause {
            for root in crate::compare::root_causes(&result) {
                println!("  root-cause {}", root.display());
            }
        }
    }
    if args.require_match && !result.is_clean() {
        eprintln!("einmo: {a} does not match {b}.");
        eprintln!("  burden: the producer of the divergent output must repair or escalate.");
        return Ok(ExitCode::FAILURE);
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_verify(args: VerifyArgs) -> Result<ExitCode> {
    let mut config = TestConfig::new(&args.work_dir);
    if let Some(limit) = args.walk_depth_limit {
        config = config.with_walk_depth_limit(limit);
    }
    let stage = match &args.stage {
        Some(s) => Some(Stage::parse(s)?),
        None => None,
    };
    let files = resolve_files(args.files)?;
    let report = crate::verify(&config, stage, files_ref(&files))?;
    if args.json {
        println!(
            "{{\"files\":{},\"failures\":{}}}",
            report.files.len(),
            report.failures()
        );
    } else {
        println!(
            "verified {} file(s), {} failure(s)",
            report.files.len(),
            report.failures()
        );
        for f in report.files.iter().filter(|f| !f.ok) {
            println!(
                "  FAILED {} ({})",
                f.rel_path.display(),
                f.detail.as_deref().unwrap_or("")
            );
        }
    }
    Ok(if report.all_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn cmd_confirm(args: ConfirmArgs) -> Result<ExitCode> {
    let report = crate::confirm_signatures(&args.path, &args.pubkey_prefix)?;
    if args.json {
        println!(
            "{{\"matched\":{},\"unmatched\":{}}}",
            report.matched.len(),
            report.unmatched.len()
        );
    } else {
        println!(
            "{} file(s) match prefix {:?}, {} do not",
            report.matched.len(),
            args.pubkey_prefix,
            report.unmatched.len()
        );
    }
    Ok(if args.require_all && !report.all_matched() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn cmd_show(args: ShowArgs) -> Result<ExitCode> {
    let file = EinmoFile::from_file(&args.file)?;
    let meta = file.metadata();
    if args.json {
        let stamps: Vec<String> = file
            .stamps()
            .entries()
            .iter()
            .map(|s| {
                format!(
                    "{{\"key\":\"{}\",\"pubkey\":\"{}\"}}",
                    s.key(),
                    s.pubkey_hex()
                )
            })
            .collect();
        println!(
            "{{\"test\":\"{}\",\"status\":\"{}\",\"stamps\":[{}]}}",
            meta.test,
            file.metadata().status,
            stamps.join(",")
        );
    } else {
        println!("test:     {}", meta.test);
        println!("suite:    {}", meta.suite);
        println!("producer: {}", meta.producer);
        println!("status:   {}", meta.status);
        if !meta.reference.is_empty() {
            println!("reference: {}", meta.reference);
        }
        println!("sections: {}", meta.sections.join(", "));
        println!("stamps:");
        for stamp in file.stamps().entries() {
            println!(
                "  {} pubkey={}… {} [{}]",
                stamp.key(),
                &stamp.pubkey_hex()[..stamp.pubkey_hex().len().min(8)],
                stamp.timestamp(),
                stamp.produced_by()
            );
        }
        if let Some(adv) = file.advisory() {
            println!("advisory: {adv}");
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// The signed body of an envelope: every section except STAMPS.
///
/// This is what `compare` matches on, so it is what a reviewer should read —
/// stamps and metadata (timestamps, keys) are deliberately excluded.
/// Verify-on-inspect applies: a tampered file is refused, never rendered.
fn body_sections(file: &EinmoFile, only: Option<&str>) -> Vec<(String, String)> {
    file.sections()
        .iter()
        .filter(|s| !s.name().eq_ignore_ascii_case("STAMPS"))
        .filter(|s| only.is_none_or(|w| s.name().eq_ignore_ascii_case(w)))
        .map(|s| (s.name().to_string(), s.body().to_string()))
        .collect()
}

fn cmd_body(args: BodyArgs) -> Result<ExitCode> {
    // from_file verifies every stamp before returning (verify-on-inspect).
    let file = EinmoFile::from_file(&args.file)?;
    let sections = body_sections(&file, args.section.as_deref());
    if sections.is_empty()
        && let Some(want) = &args.section
    {
        return Err(EinmoError::Parse(format!(
            "no section {want:?} in {}",
            args.file.display()
        )));
    }
    for (name, body) in sections {
        if !args.bare {
            println!("=== {name} ===");
        }
        println!("{body}");
    }
    Ok(ExitCode::SUCCESS)
}

/// Where a test's artifacts exist, and whether their bodies agree.
struct TestRow {
    rel: PathBuf,
    stages: Vec<(Stage, Option<String>)>, // (stage, status if present)
    differing: bool,
}

/// Enumerate the suite's tests across output/checked/verified.
///
/// The union of the input tree and every stage tree, so a test that exists only
/// in a stage (input deleted) or only in `output/` (never promoted) is still
/// listed — the file scan `poor_einmo.sh` needs.
fn scan_tests(config: &TestConfig, filter: Option<&str>) -> Result<Vec<TestRow>> {
    use crate::stage::{mirror_input_path, walk_input_tree};

    const STAGES: [Stage; 3] = [Stage::Output, Stage::Checked, Stage::Verified];

    let mut rels: Vec<PathBuf> = walk_input_tree(&config.input_path(), config.walk_depth_limit())
        .unwrap_or_default()
        .iter()
        .map(|p| mirror_input_path(p))
        .collect();
    // Union in anything present in a stage but absent from input/.
    for stage in STAGES {
        let dir = config.stage_dir(stage);
        if let Ok(found) = walk_input_tree(&dir, config.walk_depth_limit()) {
            rels.extend(found);
        }
    }
    rels.sort();
    rels.dedup();

    let mut rows = Vec::new();
    for rel in rels {
        let shown = rel.to_string_lossy().to_string();
        if filter.is_some_and(|f| !shown.contains(f)) {
            continue;
        }
        let mut stages = Vec::new();
        let mut bodies: Vec<Option<Vec<(String, String)>>> = Vec::new();
        for stage in STAGES {
            let path = config.stage_dir(stage).join(&rel);
            if path.exists() {
                match EinmoFile::from_file(&path) {
                    Ok(f) => {
                        let status = f.metadata().status.to_string();
                        stages.push((stage, Some(status)));
                        bodies.push(Some(body_sections(&f, None)));
                    }
                    Err(_) => {
                        // Tampered/unreadable: report it, never render it.
                        stages.push((stage, Some("TAMPERED".to_string())));
                        bodies.push(None);
                    }
                }
            } else {
                stages.push((stage, None));
                bodies.push(None);
            }
        }
        // Differing unless every stage is present and their bodies agree.
        let differing =
            bodies.iter().any(Option::is_none) || bodies.windows(2).any(|w| w[0] != w[1]);
        rows.push(TestRow {
            rel,
            stages,
            differing,
        });
    }
    Ok(rows)
}

fn cmd_list(args: ListArgs) -> Result<ExitCode> {
    let config = TestConfig::new(&args.work_dir);
    let rows = scan_tests(&config, args.filter.as_deref())?;
    let rows: Vec<&TestRow> = rows
        .iter()
        .filter(|r| !args.differing || r.differing)
        .collect();

    for row in &rows {
        let rel = row.rel.to_string_lossy();
        if args.json {
            let stages: Vec<String> = row
                .stages
                .iter()
                .map(|(s, st)| {
                    format!(
                        "\"{}\":{}",
                        s.dir_name(),
                        st.as_ref()
                            .map_or_else(|| "null".to_string(), |v| format!("\"{v}\""))
                    )
                })
                .collect();
            println!(
                "{{\"test\":\"{}\",\"differing\":{},{}}}",
                rel,
                row.differing,
                stages.join(",")
            );
        } else {
            let marks: Vec<String> = row
                .stages
                .iter()
                .map(|(s, st)| {
                    let mark = st.as_ref().map_or("—", |v| match v.as_str() {
                        "normal" => "ok",
                        other => other,
                    });
                    format!("{}:{}", s.dir_name(), mark)
                })
                .collect();
            println!(
                "{}\t{}\t{}",
                rel,
                if row.differing { "differ" } else { "same" },
                marks.join(" ")
            );
        }
    }
    if !args.json {
        eprintln!("{} test(s)", rows.len());
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_self_check(args: SelfCheckArgs) -> Result<ExitCode> {
    let exe = std::env::current_exe().map_err(|e| EinmoError::io("<current_exe>", e))?;
    let hash = sha256_file(&exe)?;
    if args.quiet {
        println!("{hash}");
    } else {
        println!("{} sha256:{hash}", exe.display());
    }
    // An expected hash may come from --expected or a sidecar `einmo.sha256`.
    let expected = args.expected.or_else(|| read_sidecar_hash(&exe));
    if let Some(expected) = expected
        && !expected.eq_ignore_ascii_case(&hash)
    {
        eprintln!("einmo: self-check mismatch (expected {expected}, got {hash})");
        return Ok(ExitCode::FAILURE);
    }
    Ok(ExitCode::SUCCESS)
}

fn read_sidecar_hash(exe: &Path) -> Option<String> {
    let sidecar = exe.with_file_name("einmo.sha256");
    std::fs::read_to_string(sidecar)
        .ok()
        .map(|s| s.split_whitespace().next().unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
}

fn sha256_file(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path).map_err(|e| EinmoError::io(path, e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

/// Read one line from stdin (for `--stdin-passphrase`).
fn read_stdin_line() -> Result<String> {
    use std::io::BufRead;
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| EinmoError::io("<stdin>", e))?;
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

/// Prompt for a passphrase on the controlling terminal (cross-platform via
/// rpassword). Used by the stage-key cascade's interactive tier (§B.5).
fn prompt_tty() -> Result<String> {
    rpassword::prompt_password("einmo passphrase: ").map_err(|e| EinmoError::io("<tty>", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_transition_ok_and_errors() {
        assert_eq!(
            parse_transition("output->checked").unwrap(),
            (Stage::Output, Stage::Checked)
        );
        assert!(parse_transition("output>checked").is_err());
        assert!(parse_transition("output->bogus").is_err());
    }

    #[test]
    fn cli_parses_subcommands() {
        // Smoke: the parser accepts each subcommand shape.
        assert!(Cli::try_parse_from(["einmo", "verify", "/tmp/s", "--all"]).is_ok());
        assert!(Cli::try_parse_from(["einmo", "compare", "output", "checked", "/tmp/s"]).is_ok());
        assert!(Cli::try_parse_from(["einmo", "promote", "output->checked", "/tmp/s"]).is_ok());
        assert!(Cli::try_parse_from(["einmo", "self-check", "--quiet"]).is_ok());
    }

    #[test]
    fn cli_parses_list_and_body() {
        assert!(Cli::try_parse_from(["einmo", "list", "/tmp/s"]).is_ok());
        assert!(Cli::try_parse_from(["einmo", "list", "/tmp/s", "--differing", "--json"]).is_ok());
        assert!(Cli::try_parse_from(["einmo", "list", "/tmp/s", "--filter", "foop/23"]).is_ok());
        assert!(Cli::try_parse_from(["einmo", "body", "/tmp/a.einmo"]).is_ok());
        assert!(
            Cli::try_parse_from([
                "einmo",
                "body",
                "/tmp/a.einmo",
                "--section",
                "OUTPUT",
                "--bare"
            ])
            .is_ok()
        );
    }

    /// The body view is what a reviewer reads, so it must exclude the stamp
    /// chain (and therefore the timestamp/key churn that made the legacy insta
    /// corpus structurally red).
    #[test]
    fn body_sections_excludes_stamps() {
        use crate::format::{DEFAULT_SEPARATOR, EinmoFile, Metadata, Section, Status};
        use crate::signature::Stamps;

        let meta = Metadata {
            test: "t.foo".into(),
            suite: "s".into(),
            producer: "abc".into(),
            producer_diff: String::new(),
            generated: "2026-07-15T00:00:00Z".into(),
            status: Status::Normal,
            status_detail: String::new(),
            reference: String::new(),
            sections: vec!["INPUT".into(), "OUTPUT".into(), "STAMPS".into()],
        };
        let file = EinmoFile::new(
            "utf-8",
            DEFAULT_SEPARATOR,
            meta,
            vec![
                Section::new("INPUT", "{3 + 4;}"),
                Section::new("OUTPUT", "{ 7 }"),
                Section::new("STAMPS", "{\"key\":\"stage:output\"}"),
            ],
            Stamps::new(),
        );

        let all = body_sections(&file, None);
        assert_eq!(
            all.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            vec!["INPUT", "OUTPUT"],
            "STAMPS must never reach a reviewer's pane"
        );

        let only = body_sections(&file, Some("output"));
        assert_eq!(only.len(), 1, "--section is case-insensitive");
        assert_eq!(only[0].1, "{ 7 }");
    }

    #[test]
    fn cli_promote_accepts_positional_files() {
        let cli = Cli::try_parse_from([
            "einmo",
            "promote",
            "output->checked",
            "/tmp/s",
            "a.einmo",
            "b.einmo",
        ])
        .unwrap();
        let Command::Promote(a) = cli.command else {
            panic!("expected Promote");
        };
        assert_eq!(
            a.files,
            vec![PathBuf::from("a.einmo"), PathBuf::from("b.einmo")]
        );
    }

    #[test]
    fn cli_promote_accepts_dash_separator() {
        let cli = Cli::try_parse_from([
            "einmo",
            "promote",
            "output->checked",
            "/tmp/s",
            "--",
            "a.einmo",
            "b.einmo",
        ])
        .unwrap();
        let Command::Promote(a) = cli.command else {
            panic!("expected Promote");
        };
        assert_eq!(
            a.files,
            vec![PathBuf::from("a.einmo"), PathBuf::from("b.einmo")]
        );
    }

    #[test]
    fn cli_promote_no_files_is_empty() {
        let cli = Cli::try_parse_from([
            "einmo",
            "promote",
            "output->checked",
            "/tmp/s",
            "--filter",
            "*",
        ])
        .unwrap();
        let Command::Promote(a) = cli.command else {
            panic!("expected Promote");
        };
        assert!(a.files.is_empty());
        assert_eq!(a.filter.as_deref(), Some("*"));
    }

    #[test]
    fn cli_verify_accepts_positional_files() {
        let cli = Cli::try_parse_from(["einmo", "verify", "--all", "/tmp/s", "x.einmo"]).unwrap();
        let Command::Verify(a) = cli.command else {
            panic!("expected Verify");
        };
        assert_eq!(a.files, vec![PathBuf::from("x.einmo")]);
    }

    #[test]
    fn resolve_files_from_iter_empty() {
        assert!(resolve_files_from_iter(Vec::new(), std::iter::empty()).is_empty());
    }

    #[test]
    fn resolve_files_from_iter_no_dash() {
        let files = vec![PathBuf::from("a.einmo"), PathBuf::from("b.einmo")];
        let out = resolve_files_from_iter(files.clone(), ["c".into()].into_iter());
        assert_eq!(out, files);
    }

    #[test]
    fn resolve_files_from_iter_dash_replaced_by_stdin() {
        let out = resolve_files_from_iter(
            vec![PathBuf::from("-")],
            ["a.einmo", "b.einmo"].into_iter().map(String::from),
        );
        assert_eq!(
            out,
            vec![PathBuf::from("a.einmo"), PathBuf::from("b.einmo")]
        );
    }

    #[test]
    fn resolve_files_from_iter_dash_mixed_with_files() {
        let out = resolve_files_from_iter(
            vec![
                PathBuf::from("pre.einmo"),
                PathBuf::from("-"),
                PathBuf::from("post.einmo"),
            ],
            ["a.einmo", "b.einmo"].into_iter().map(String::from),
        );
        assert_eq!(
            out,
            vec![
                PathBuf::from("pre.einmo"),
                PathBuf::from("a.einmo"),
                PathBuf::from("b.einmo"),
                PathBuf::from("post.einmo"),
            ]
        );
    }

    #[test]
    fn resolve_files_from_iter_skips_blank_lines() {
        let out = resolve_files_from_iter(
            vec![PathBuf::from("-")],
            ["a.einmo", "", "  ", "b.einmo"]
                .into_iter()
                .map(String::from),
        );
        assert_eq!(
            out,
            vec![PathBuf::from("a.einmo"), PathBuf::from("b.einmo")]
        );
    }

    #[test]
    fn files_ref_empty_is_none() {
        assert!(files_ref(&[]).is_none());
        let v = vec![PathBuf::from("a.einmo")];
        assert_eq!(files_ref(&v), Some(v.as_slice()));
    }
}
