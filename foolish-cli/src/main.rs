use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use foolish_ubca2::fvm_storage::{FVMStorage, FirPointer};
use foolish_ubca2::{SequenceMode, Ubca2Sequencer, UbcaEvaluator};

#[derive(Parser)]
#[command(name = "foolish")]
#[command(about = "Foolish language CLI")]
struct Cli {
    /// Signing passphrase for snapshot signing (defaults to empty string)
    #[arg(long, env = "SIGNING_PASSPHRASE", default_value = "")]
    signing_passphrase: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate .foo source and print result
    Run {
        /// Path to .foo source file
        file: PathBuf,
    },
    /// Evaluate .foo source, printing the settled FIR (debug output)
    Step {
        /// Path to .foo source file
        file: PathBuf,
    },
    /// Interactive REPL
    Repl,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { file } => cmd_run(&file),
        Commands::Step { file } => cmd_step(&file),
        Commands::Repl => cmd_repl(),
    }
}

/// Evaluate source through `foolish-ubca2`, retaining the full arena so
/// rendering can use every field the compatibility bridge would otherwise
/// drop (search direction and contexting among them).
fn evaluate_arena(source: &str) -> anyhow::Result<(FVMStorage, Vec<FirPointer>)> {
    UbcaEvaluator
        .evaluate_arena(source)
        .map_err(|e| anyhow::anyhow!("{}", e))
}

fn cmd_run(file: &PathBuf) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;
    let (storage, firs) = evaluate_arena(&source)?;
    for fir in firs {
        println!(
            "{}",
            Ubca2Sequencer::format(&storage, fir, SequenceMode::Foolish)
        );
    }
    Ok(())
}

fn cmd_step(file: &PathBuf) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;
    let (storage, firs) = evaluate_arena(&source)?;
    for (i, fir) in firs.into_iter().enumerate() {
        println!("[{}] RESULT:", i);
        println!(
            "{}",
            Ubca2Sequencer::format(&storage, fir, SequenceMode::Foolish)
        );
    }
    Ok(())
}

fn cmd_repl() -> anyhow::Result<()> {
    println!("Foolish REPL — type {{ to start a brane, evaluated to completion");
    let mut buf = String::new();
    let mut depth = 0i32;
    loop {
        let prompt = if depth > 0 { ".. " } else { "> " };
        print!("{}", prompt);
        std::io::stdout().flush()?;

        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(0) => {
                println!();
                return Ok(());
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                println!();
                return Ok(());
            }
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        }

        for c in line.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        buf.push_str(&line);

        if depth <= 0 && !buf.trim().is_empty() {
            match evaluate_arena(&buf) {
                Ok((storage, firs)) => {
                    for fir in firs {
                        println!(
                            "=> {}",
                            Ubca2Sequencer::format(&storage, fir, SequenceMode::Foolish)
                        );
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            buf.clear();
        }
    }
}
