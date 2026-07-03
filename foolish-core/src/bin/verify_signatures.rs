use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use clap::Parser;
use foolish_core::signature::{
    SnapshotVerification, parse_snapshot_footer, sign_snapshot, verify_snapshot,
};

#[derive(Parser)]
#[command(
    name = "verify_signatures",
    about = "Verify (and optionally re-sign) Ed25519 triple-signatures in Foolish snapshot files.\n\
             Arguments may be files or directories; directories are scanned for .snap/.snap.new.\n\
             Defaults to snapshot_tests/approved/ when no arguments are given."
)]
struct Args {
    /// Read passphrase from stdin (hidden). Default is empty string (computer/AI key).
    #[arg(long)]
    stdin_passphrase: bool,

    /// Re-sign each file with the current passphrase and write the new footer back in place.
    #[arg(long)]
    write_verified: bool,

    /// Append a comment line to the COMMENTS block, then re-sign and write the file.
    /// May be supplied multiple times; each value becomes its own line.
    #[arg(long = "add-comment", value_name = "TEXT")]
    add_comments: Vec<String>,

    /// Files or directories to process.
    /// A file argument is checked directly; a directory is scanned for .snap/.snap.new files.
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let passphrase = if args.stdin_passphrase {
        eprint!("Passphrase: ");
        let stdin = io::stdin();
        stdin
            .lock()
            .lines()
            .next()
            .and_then(|l| l.ok())
            .unwrap_or_default()
    } else {
        String::new()
    };

    let paths: Vec<PathBuf> = if args.paths.is_empty() {
        vec![PathBuf::from("snapshot_tests/approved")]
    } else {
        args.paths
    };

    let mut files: Vec<PathBuf> = Vec::new();
    for path in &paths {
        if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() {
            let mut dir_files: Vec<PathBuf> = match fs::read_dir(path) {
                Ok(entries) => entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| is_snap_file(p))
                    .collect(),
                Err(err) => {
                    eprintln!("Cannot read directory {}: {}", path.display(), err);
                    std::process::exit(1);
                }
            };
            dir_files.sort();
            files.extend(dir_files);
        } else {
            eprintln!("Not a file or directory: {}", path.display());
            std::process::exit(1);
        }
    }

    let mut any_fail = false;
    let writing = args.write_verified || !args.add_comments.is_empty();

    for file in &files {
        let (cols, annotation) = if writing {
            match write_verified_file(file, &passphrase, &args.add_comments) {
                Ok(v) => (columns_from_verification(&v), "[written]".to_string()),
                Err(e) => (columns_error(), format!("[ERROR: {e}]")),
            }
        } else {
            match check_file(file, &passphrase) {
                Check::Verified(v) => (columns_from_verification(&v), String::new()),
                Check::Unsigned => (columns_unsigned(), String::new()),
                Check::ParseError(e) | Check::IoError(e) => {
                    (columns_error(), format!("[ERROR: {e}]"))
                }
            }
        };

        let annotation_sep = if annotation.is_empty() {
            String::new()
        } else {
            format!("  {annotation}")
        };
        println!("{}  {}{}", cols, file.display(), annotation_sep);

        if cols.contains(" no") || cols.contains("ERR") {
            any_fail = true;
        }
    }

    if any_fail {
        std::process::exit(1);
    }
}

// ─── status column formatting ─────────────────────────────────────────────────

fn columns_from_verification(v: &SnapshotVerification) -> String {
    let k = if v.key_match { "yes" } else { " no" };
    let f = if v.foolish_ok { "yes" } else { " no" };
    let h = if v.hs_ok { "yes" } else { " no" };
    let c = if v.comments_ok { "yes" } else { " no" };
    format!("key match: {k}, foolish: {f}, hs: {h}, comments: {c}")
}

fn columns_unsigned() -> String {
    "key match: n/a, foolish: n/a, hs: n/a, comments: n/a".to_string()
}

fn columns_error() -> String {
    "key match: ERR, foolish: ERR, hs: ERR, comments: ERR".to_string()
}

// ─── file operations ──────────────────────────────────────────────────────────

enum Check {
    Verified(SnapshotVerification),
    Unsigned,
    ParseError(String),
    IoError(String),
}

fn check_file(path: &Path, passphrase: &str) -> Check {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => return Check::IoError(e.to_string()),
    };

    let Some(sig) = parse_snapshot_footer(&text) else {
        return Check::Unsigned;
    };

    let Some((input, hs_outputs, comments)) = extract_snapshot_content(&text) else {
        return Check::ParseError("could not parse INPUT/RESULT/COMMENTS blocks".to_string());
    };

    Check::Verified(verify_snapshot(
        passphrase,
        &input,
        &hs_outputs,
        &comments,
        &sig,
    ))
}

/// Re-sign (and optionally append comments to) a snapshot file, writing it back in place.
///
/// `extra_comments` lines are appended to the existing COMMENTS block before re-signing.
fn write_verified_file(
    path: &Path,
    passphrase: &str,
    extra_comments: &[String],
) -> Result<SnapshotVerification, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read error: {e}"))?;

    let (input, hs_outputs, mut comments) = extract_snapshot_content(&text)
        .ok_or_else(|| "could not parse INPUT/RESULT/COMMENTS blocks".to_string())?;

    for line in extra_comments {
        if !comments.is_empty() {
            comments.push('\n');
        }
        comments.push_str(line);
    }

    let new_sig = sign_snapshot(passphrase, &input, &hs_outputs, &comments);
    let comments_block = format!("COMMENTS:\n```markdown\n{}\n```", comments.trim_end());
    let updated = replace_comments_and_footer(&text, &comments_block, &new_sig.format_footer());

    fs::write(path, &updated).map_err(|e| format!("write error: {e}"))?;

    Ok(verify_snapshot(
        passphrase,
        &input,
        &hs_outputs,
        &comments,
        &new_sig,
    ))
}

/// Replace the COMMENTS block and SIGNATURES section in a snapshot file.
///
/// Cuts from the last `COMMENTS:` line; falls back to `SIGNATURES:` or `Public key:`
/// for files that predate the COMMENTS block.
fn replace_comments_and_footer(text: &str, new_comments_block: &str, new_footer: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let cut = lines
        .iter()
        .rposition(|l| *l == "COMMENTS:")
        .or_else(|| lines.iter().rposition(|l| *l == "SIGNATURES:"))
        .or_else(|| lines.iter().rposition(|l| l.starts_with("Public key: ")));
    let body = match cut {
        Some(pos) => lines[..pos].join("\n"),
        None => text.trim_end_matches('\n').to_string(),
    };
    format!(
        "{}\n{}\n{}\n",
        body.trim_end_matches('\n'),
        new_comments_block,
        new_footer
    )
}

fn is_snap_file(p: &Path) -> bool {
    p.extension().and_then(|e| e.to_str()) == Some("snap")
        || p.to_string_lossy().ends_with(".snap.new")
}

/// Extract the raw input, HS output blocks, and comments block from a snapshot file.
fn extract_snapshot_content(text: &str) -> Option<(String, Vec<String>, String)> {
    let mut input: Option<String> = None;
    let mut hs_outputs: Vec<String> = Vec::new();
    let mut comments: Option<String> = None;
    let mut in_foolish = false;
    let mut in_hfssnap = false;
    let mut in_markdown = false;
    let mut block: Vec<&str> = Vec::new();

    for line in text.lines() {
        match line {
            "```foolish" => {
                in_foolish = true;
                block.clear();
            }
            "```hfssnap" => {
                in_hfssnap = true;
                block.clear();
            }
            "```markdown" => {
                in_markdown = true;
                block.clear();
            }
            "```" if in_foolish => {
                input = Some(block.join("\n"));
                in_foolish = false;
                block.clear();
            }
            "```" if in_hfssnap => {
                hs_outputs.push(block.join("\n"));
                in_hfssnap = false;
                block.clear();
            }
            "```" if in_markdown => {
                comments = Some(block.join("\n"));
                in_markdown = false;
                block.clear();
            }
            _ if in_foolish || in_hfssnap || in_markdown => {
                block.push(line);
            }
            _ => {}
        }
    }

    let input = input?;
    let comments = comments.unwrap_or_default();
    Some((input, hs_outputs, comments))
}
