use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use foolish_core::{Compiler, FirSequencer, clone_steppable, fir_to_json, fir_to_ref, ubc};

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
    /// Compile .foo source to FIR JSON
    Compile {
        /// Path to .foo source file
        file: PathBuf,
    },
    /// Evaluate .foo source and print result
    Run {
        /// Path to .foo source file
        file: PathBuf,
    },
    /// Step-evaluate .foo source (debug output)
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
        Commands::Compile { file } => cmd_compile(&file),
        Commands::Run { file } => cmd_run(&file),
        Commands::Step { file } => cmd_step(&file),
        Commands::Repl => cmd_repl(),
    }
}

fn cmd_compile(file: &PathBuf) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;
    let firs = Compiler::compile(&source)?;
    for fir in &firs {
        let json = fir_to_json(fir).map_err(|e| anyhow::anyhow!("{}", e))?;
        println!("{}", json);
    }
    Ok(())
}

fn cmd_run(file: &PathBuf) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;
    let firs = Compiler::compile(&source)?;
    for fir in &firs {
        let mut fir_ref = fir_to_ref(fir.clone());
        ubc::run_to_completion(&mut fir_ref).with_context(|| "Evaluation failed")?;
        let final_fir = clone_steppable(&fir_ref);
        let output = FirSequencer::format(&final_fir);
        println!("{}", output);
    }
    Ok(())
}

fn cmd_step(file: &PathBuf) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;
    let firs = Compiler::compile(&source)?;
    for (i, fir) in firs.iter().enumerate() {
        println!("[{}] PARSED:", i);
        println!("{}", FirSequencer::format(fir));

        let mut fir_ref = fir_to_ref(fir.clone());
        match ubc::run_to_completion(&mut fir_ref) {
            Ok(()) => {
                let final_fir = clone_steppable(&fir_ref);
                println!("RESULT:");
                println!("{}", FirSequencer::format(&final_fir));
            }
            Err(e) => eprintln!("ERROR: {}", e),
        }
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
            match Compiler::compile(&buf) {
                Ok(firs) => {
                    for fir in &firs {
                        let mut fir_ref = fir_to_ref(fir.clone());
                        match ubc::run_to_completion(&mut fir_ref) {
                            Ok(()) => {
                                let final_fir = clone_steppable(&fir_ref);
                                println!("=> {}", FirSequencer::format(&final_fir));
                            }
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            buf.clear();
        }
    }
}
