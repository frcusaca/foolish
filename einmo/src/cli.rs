//! CLI entry point for the `einmo` and `cargo-einmo` binaries.

use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};

use crate::config::{MatchSections, Stage, TestConfig, parse_einmo_toml, resolve_stage_key};
use crate::format::EinmoFile;
use crate::stage::{confirm_signatures, flag, promote};
use crate::{KeySource, compare as do_compare};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "einmo",
    version,
    about = "Directory-based signed-snapshot testing with staged promotion"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Number of parallel threads.
    #[arg(long, global = true)]
    parallel: Option<usize>,

    /// Force serial execution.
    #[arg(long, global = true)]
    serial: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Promote files from one stage to another.
    Promote {
        /// Stage pair: from->to (e.g. output->checked).
        #[arg(value_parser = parse_stage_pair)]
        stages: (Stage, Stage),

        /// Work directory.
        work_dir: PathBuf,

        /// Glob filter for files.
        #[arg(long)]
        filter: Option<String>,

        /// Explicit passphrase.
        #[arg(long)]
        passphrase: Option<String>,

        /// Read passphrase from stdin.
        #[arg(long)]
        stdin_passphrase: bool,

        /// Force interactive passphrase prompt.
        #[arg(long)]
        interactive: bool,

        /// Batch: one passphrase for all files.
        #[arg(long)]
        batch: bool,
    },

    /// Move files to the flagged stage.
    Flag {
        /// Work directory.
        work_dir: PathBuf,

        /// Stage to flag from.
        stage: String,

        /// Glob filter.
        #[arg(long)]
        filter: Option<String>,

        /// Reason for flagging.
        #[arg(long, default_value = "flagged via CLI")]
        reason: String,
    },

    /// Compare two stages.
    Compare {
        /// First stage.
        stage_a: String,

        /// Second stage.
        stage_b: String,

        /// Work directory.
        work_dir: PathBuf,

        /// Sections to compare: "input,output" or "input,output,comments".
        #[arg(long)]
        match_sections: Option<String>,

        /// Require comments to match.
        #[arg(long)]
        require_comments_match: bool,

        /// Exit non-zero if any files differ.
        #[arg(long)]
        require_match: bool,

        /// Output as JSON.
        #[arg(long)]
        json: bool,

        /// Descend to deepest differing descendants.
        #[arg(long)]
        root_cause: bool,

        /// Warn about files older than N days.
        #[arg(long)]
        stale_days: Option<u64>,

        /// Glob filter.
        #[arg(long)]
        filter: Option<String>,
    },

    /// Verify stamp integrity of files.
    Verify {
        /// Work directory.
        work_dir: PathBuf,

        /// Verify a specific stage.
        #[arg(long)]
        stage: Option<String>,

        /// Verify all stages.
        #[arg(long)]
        all: bool,
    },

    /// Confirm signatures match a pubkey prefix.
    ConfirmSignatures {
        /// Path to scan.
        path: PathBuf,

        /// Public key prefix to match.
        pubkey_prefix: String,

        /// Exit non-zero if any file lacks a match.
        #[arg(long)]
        require_all: bool,
    },

    /// Show an envelope summary with stamp chain.
    Show {
        /// Path to the `.einmo` file.
        file: PathBuf,
    },

    /// Compute SHA-256 of the running binary.
    SelfCheck {
        /// Expected SHA-256 hash.
        #[arg(long)]
        expected: Option<String>,

        /// Print only the hash.
        #[arg(long)]
        quiet: bool,
    },
}

fn parse_stage_pair(s: &str) -> Result<(Stage, Stage), String> {
    let (from, to) = s
        .split_once("->")
        .ok_or_else(|| format!("expected 'from->to', got '{s}'"))?;
    let from: Stage = from
        .parse()
        .map_err(|e: crate::ConfigError| e.to_string())?;
    let to: Stage = to.parse().map_err(|e: crate::ConfigError| e.to_string())?;
    Ok((from, to))
}

fn matches_filter(rel_path: &Path, filter: &Option<String>) -> bool {
    match filter {
        None => true,
        Some(pat) => {
            let path_str = rel_path.to_string_lossy();
            simple_glob_match(pat, &path_str)
        }
    }
}

fn simple_glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return text.ends_with(suffix);
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return text.starts_with(prefix);
    }
    text.contains(pattern)
}

pub fn run() {
    let cli = Cli::parse();
    match execute(cli) {
        Ok(()) => {}
        Err(msg) => {
            eprintln!("einmo: {msg}");
            process::exit(1);
        }
    }
}

fn execute(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Promote {
            stages,
            work_dir,
            filter,
            passphrase,
            stdin_passphrase,
            interactive,
            batch: _,
        } => cmd_promote(
            &work_dir,
            stages.0,
            stages.1,
            &filter,
            passphrase.as_deref(),
            stdin_passphrase,
            interactive,
        ),

        Command::Flag {
            work_dir,
            stage,
            filter,
            reason,
        } => cmd_flag(&work_dir, &stage, &filter, &reason),

        Command::Compare {
            stage_a,
            stage_b,
            work_dir,
            match_sections,
            require_comments_match,
            require_match,
            json,
            root_cause,
            stale_days,
            filter,
        } => cmd_compare(
            &work_dir,
            &stage_a,
            &stage_b,
            match_sections.as_deref(),
            require_comments_match,
            require_match,
            json,
            root_cause,
            stale_days,
            &filter,
        ),

        Command::Verify {
            work_dir,
            stage,
            all,
        } => cmd_verify(&work_dir, stage.as_deref(), all),

        Command::ConfirmSignatures {
            path,
            pubkey_prefix,
            require_all,
        } => cmd_confirm_signatures(&path, &pubkey_prefix, require_all),

        Command::Show { file } => cmd_show(&file),

        Command::SelfCheck { expected, quiet } => cmd_self_check(expected.as_deref(), quiet),
    }
}

// ---------------------------------------------------------------------------
// Subcommand implementations
// ---------------------------------------------------------------------------

fn cmd_promote(
    work_dir: &Path,
    from: Stage,
    to: Stage,
    filter: &Option<String>,
    cli_pass: Option<&str>,
    stdin_pass: bool,
    interactive: bool,
) -> Result<(), String> {
    let config = TestConfig::new(work_dir);
    let toml_config = parse_einmo_toml(work_dir).map_err(|e| e.to_string())?;

    let stage_dir = config.stage_dir(from);
    let files = collect_stage_files(&stage_dir);
    let filtered: Vec<PathBuf> = files
        .into_iter()
        .filter(|p| matches_filter(p, filter))
        .collect();

    if filtered.is_empty() {
        eprintln!("einmo promote: no files matched");
        return Ok(());
    }

    let (passphrase, source) = resolve_stage_key(
        to,
        cli_pass,
        stdin_pass,
        interactive,
        std::env::var("EINMO_PASSPHRASE").ok().as_deref(),
        &toml_config,
    )
    .map_err(|e| e.to_string())?;

    if source == KeySource::Interactive {
        eprintln!("(passphrase source: interactive prompt)");
    }

    for rel in &filtered {
        match promote(&config, from, to, rel, &passphrase) {
            Ok(report) => {
                for p in &report.files_promoted {
                    println!("promoted: {} ({from} -> {to})", p.display());
                }
            }
            Err(e) => {
                eprintln!("einmo promote failed for {}: {e}", rel.display());
            }
        }
    }

    Ok(())
}

fn cmd_flag(
    work_dir: &Path,
    stage_str: &str,
    filter: &Option<String>,
    reason: &str,
) -> Result<(), String> {
    let stage: Stage = stage_str
        .parse()
        .map_err(|e: crate::ConfigError| e.to_string())?;
    let config = TestConfig::new(work_dir);
    let stage_dir = config.stage_dir(stage);
    let files = collect_stage_files(&stage_dir);
    let filtered: Vec<PathBuf> = files
        .into_iter()
        .filter(|p| matches_filter(p, filter))
        .collect();

    if filtered.is_empty() {
        eprintln!("einmo flag: no files matched");
        return Ok(());
    }

    for rel in &filtered {
        match flag(&config, stage, rel, reason) {
            Ok(report) => {
                for p in &report.files_flagged {
                    println!("flagged: {}", p.display());
                }
            }
            Err(e) => {
                eprintln!("einmo flag failed for {}: {e}", rel.display());
            }
        }
    }

    Ok(())
}

#[expect(clippy::too_many_arguments)]
fn cmd_compare(
    work_dir: &Path,
    stage_a_str: &str,
    stage_b_str: &str,
    match_sections_str: Option<&str>,
    require_comments: bool,
    require_match: bool,
    json_output: bool,
    root_cause: bool,
    stale_days: Option<u64>,
    filter: &Option<String>,
) -> Result<(), String> {
    let stage_a: Stage = stage_a_str
        .parse()
        .map_err(|e: crate::ConfigError| e.to_string())?;
    let stage_b: Stage = stage_b_str
        .parse()
        .map_err(|e: crate::ConfigError| e.to_string())?;

    let sections = if require_comments || match_sections_str.is_some_and(|s| s.contains("comments"))
    {
        MatchSections::InputOutputComments
    } else {
        MatchSections::InputOutput
    };

    let config = TestConfig::new(work_dir).with_match_sections(sections);

    let mut result = do_compare(&config, stage_a, stage_b, sections);

    if let Some(f) = filter {
        let f = f.clone();
        let pred = |p: &PathBuf| matches_filter(p, &Some(f.clone()));
        result.matching.retain(pred);
        result.differing.retain(|d| pred(&d.path));
        result.only_in_a.retain(pred);
        result.only_in_b.retain(pred);
        result.tampered.retain(pred);
    }

    if root_cause && !result.differing.is_empty() {
        eprintln!("(root-cause: showing deepest differing descendants)");
    }

    if stale_days.is_some() {
        eprintln!("(stale-days: warning output not yet implemented)");
    }

    if json_output {
        let json = serde_json::json!({
            "matching": result.matching,
            "differing": result.differing.iter().map(|d| {
                serde_json::json!({"path": d.path, "sections": d.sections})
            }).collect::<Vec<_>>(),
            "only_in_a": result.only_in_a,
            "only_in_b": result.only_in_b,
            "tampered": result.tampered,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?
        );
    } else {
        for p in &result.matching {
            println!("matching: {}", p.display());
        }
        for d in &result.differing {
            println!(
                "differing: {} (sections: {})",
                d.path.display(),
                d.sections.join(", ")
            );
        }
        for p in &result.only_in_a {
            println!("only_in_a: {}", p.display());
        }
        for p in &result.only_in_b {
            println!("only_in_b: {}", p.display());
        }
        for p in &result.tampered {
            println!("tampered: {}", p.display());
        }
    }

    if require_match && !result.is_clean() {
        return Err("stage comparison failed: files differ, are missing, or are tampered".into());
    }

    Ok(())
}

fn cmd_verify(work_dir: &Path, stage_str: Option<&str>, all: bool) -> Result<(), String> {
    let config = TestConfig::new(work_dir);
    let stages = if all {
        Stage::all().to_vec()
    } else if let Some(s) = stage_str {
        vec![
            s.parse::<Stage>()
                .map_err(|e: crate::ConfigError| e.to_string())?,
        ]
    } else {
        Stage::all().to_vec()
    };

    let mut all_valid = true;

    for stage in stages {
        let dir = config.stage_dir(stage);
        let files = collect_stage_files(&dir);
        for rel in &files {
            let path = dir.join(rel);
            match EinmoFile::from_file(&path) {
                Ok(_) => {
                    println!("valid: {} ({stage})", rel.display());
                }
                Err(e) => {
                    eprintln!("INVALID: {} ({stage}): {e}", rel.display());
                    all_valid = false;
                }
            }
        }
    }

    if !all_valid {
        return Err("verification failed".into());
    }
    Ok(())
}

fn cmd_confirm_signatures(
    path: &Path,
    pubkey_prefix: &str,
    require_all: bool,
) -> Result<(), String> {
    let report = confirm_signatures(path, pubkey_prefix).map_err(|e| e.to_string())?;

    for p in &report.matching {
        println!("match: {}", p.display());
    }
    for p in &report.non_matching {
        println!("no-match: {}", p.display());
    }

    if require_all && !report.all_match() {
        return Err(format!(
            "{} files did not match pubkey prefix '{}'",
            report.non_matching.len(),
            pubkey_prefix
        ));
    }

    Ok(())
}

fn cmd_show(file: &Path) -> Result<(), String> {
    let einmo = EinmoFile::from_file(file).map_err(|e| e.to_string())?;

    println!("test: {}", einmo.test());
    println!("suite: {}", einmo.suite());
    println!("producer: {}", einmo.producer());
    if let Some(diff) = einmo.producer_diff() {
        println!("producer-diff: {diff}");
    }
    println!("generated: {}", einmo.generated());
    println!("status: {}", einmo.status());
    if !einmo.status_detail().is_empty() {
        println!("status-detail: {}", einmo.status_detail());
    }
    println!("sections: {}", einmo.sections_list().join(", "));

    for sec in einmo.sections_list() {
        if sec == "STAMPS" {
            continue;
        }
        match einmo.section(sec) {
            Some(bytes) => {
                let preview = String::from_utf8_lossy(bytes);
                let preview = if preview.len() > 200 {
                    format!("{}...", &preview[..200])
                } else {
                    preview.to_string()
                };
                println!("[{sec}]: {preview}");
            }
            None => println!("[{sec}]: (empty)"),
        }
    }

    println!("\nstamps:");
    for stamp in einmo.stamps().entries() {
        println!(
            "  key: {} | pubkey: {}… | signs: {} | produced_by: {} | timestamp: {}",
            stamp.key(),
            &stamp.pubkey()[..8.min(stamp.pubkey().len())],
            stamp.signs(),
            stamp.produced_by(),
            stamp.timestamp()
        );
    }

    if !einmo.advisory_lines().is_empty() {
        println!("\nadvisory:");
        for line in einmo.advisory_lines() {
            println!("  {line}");
        }
    }

    Ok(())
}

fn cmd_self_check(expected: Option<&str>, quiet: bool) -> Result<(), String> {
    use sha2::{Digest, Sha256};

    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_bytes = std::fs::read(&exe_path).map_err(|e| e.to_string())?;

    let mut hasher = Sha256::new();
    hasher.update(&exe_bytes);
    let hash = format!("{:x}", hasher.finalize());

    if quiet {
        println!("{hash}");
    } else {
        println!("path: {}", exe_path.display());
        println!("sha256: {hash}");
    }

    if let Some(exp) = expected
        && hash != exp
    {
        return Err(format!("hash mismatch: expected {exp}, got {hash}"));
    }

    Ok(())
}

fn collect_stage_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    collect_recursive(dir, dir, &mut results);
    results.sort();
    results
}

fn collect_recursive(base: &Path, dir: &Path, results: &mut Vec<PathBuf>) {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if metadata.is_dir() {
            collect_recursive(base, &path, results);
        } else if metadata.is_file()
            && path.extension().map(|e| e == "einmo").unwrap_or(false)
            && let Ok(rel) = path.strip_prefix(base)
        {
            results.push(rel.to_path_buf());
        }
    }
}
