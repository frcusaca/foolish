use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

use foolish_core::Sequencer;
use foolish_ubcb::UbcbEngine;

pub fn format_result(result: &foolish_ubcb::EvaluationResult, states: bool) -> String {
    if result.statements.is_empty() {
        return "{}".to_string();
    }
    let mut lines = vec![];
    for stmt in &result.statements {
        let fir = foolish_core::clone_steppable(&stmt.fir);
        let formatted = Sequencer::format(&fir);
        match &stmt.name {
            Some(name) => lines.push(format!("{name} = {formatted}")),
            None => lines.push(formatted),
        }
    }
    format!("{{\n  {}\n}}", lines.join(",\n  "))
}

#[derive(Parser)]
#[command(name = "foolish-ubcb-cli")]
#[command(about = "Foolish UBCb CLI — message-passing brane computer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate .foo source and print result
    Run {
        /// Path to .foo source file
        file: PathBuf,
        /// Show NYES states alongside values
        #[arg(long)]
        states: bool,
    },
    /// Interactive REPL
    Repl,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { file, states } => cmd_run(&file, states),
        Commands::Repl => cmd_repl(),
    }
}

fn cmd_run(file: &PathBuf, states: bool) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate(&source)
        .with_context(|| "Evaluation failed")?;
    println!("{}", format_result(&result, states));
    Ok(())
}

fn cmd_repl() -> anyhow::Result<()> {
    println!("Foolish UBCb REPL — type {{ to start a brane, evaluated to completion");
    let mut engine = UbcbEngine::new();
    let mut buf = String::new();
    let mut depth = 0i32;
    loop {
        let prompt = if depth > 0 { ".. " } else { "> " };
        print!("{}", prompt);
        std::io::stdout().flush()?;

        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(0) => { println!(); return Ok(()); }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => { println!(); return Ok(()); }
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
            match engine.evaluate(&buf) {
                Ok(result) => {
                    println!("=> {}", format_result(&result, false));
                }
                Err(e) => eprintln!("Error: {e}"),
            }
            buf.clear();
        }
    }
}
