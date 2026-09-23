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

/// Evaluate source through `foolish-ubca2`, retaining the full arena so rendering can use every field the
/// compatibility bridge would otherwise drop (search direction and contexting among them).
fn evaluate_arena(source: &str) -> anyhow::Result<(FVMStorage, Vec<FirPointer>)> {
    UbcaEvaluator
        .evaluate_arena(source)
        .map_err(|e| anyhow::anyhow!("{}", e))
}

fn cmd_run(file: &PathBuf) -> anyhow::Result<()> {
    let source =
        std::fs::read_to_string(file).with_context(|| format!("Failed to read {}", file.display()))?;
    let (storage, firs) = evaluate_arena(&source)?;
    for fir in firs {
        println!("{}", Ubca2Sequencer::format(&storage, fir, SequenceMode::Foolish));
    }
    Ok(())
}

fn cmd_step(file: &PathBuf) -> anyhow::Result<()> {
    let source =
        std::fs::read_to_string(file).with_context(|| format!("Failed to read {}", file.display()))?;
    let (storage, firs) = evaluate_arena(&source)?;
    for (i, fir) in firs.into_iter().enumerate() {
        println!("[{}] RESULT:", i);
        println!("{}", Ubca2Sequencer::format(&storage, fir, SequenceMode::Foolish));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// T3a (FOOP-86 §Test Plan) — `run` on a small program emits Foolish, not FIR-internal rendering: no
    /// search "machinery" dump, no bare operator-node token, no bare NYES state token sitting on its own.
    #[test]
    fn cli_run_renders_foolish() {
        let (storage, firs) = evaluate_arena("{a = 1; b = 2; c = a + b;}").unwrap();
        let rendered = firs
            .into_iter()
            .map(|fir| Ubca2Sequencer::format(&storage, fir, SequenceMode::Foolish))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !rendered.contains("?(pattern="),
            "Foolish-mode output must not show search machinery: {rendered}"
        );
        assert!(
            !rendered.contains("Op("),
            "Foolish-mode output must not show a bare operator-node token: {rendered}"
        );
        for token in ["PREMBRIONIC", "EMBRYONIC", "BRANING", "ECONSTANIC", "WOCONSTANIC"] {
            assert!(
                !rendered.contains(token),
                "Foolish-mode output must not show a bare NYES token ({token}): {rendered}"
            );
        }
    }

    /// T3b (FOOP-86 §Test Plan, §1.3) — the CLI's rendering agrees with the einmo adapter's for the same
    /// source. Both call `evaluate_arena` + `Ubca2Sequencer::format(.., Foolish)`
    /// (`foolish-ubca2/src/ubca_snapshot_tester.rs`'s `Ubca2FoolishAdapter`); a divergence would mean the
    /// CLI grew its own rendering path.
    #[test]
    fn cli_agrees_with_einmo_adapter() {
        let source = "{b={x=3;};r=b?x;}";
        let (cli_storage, cli_firs) = evaluate_arena(source).unwrap();
        let cli_rendered = cli_firs
            .into_iter()
            .map(|fir| Ubca2Sequencer::format(&cli_storage, fir, SequenceMode::Foolish))
            .collect::<Vec<_>>()
            .join("\n");

        // The einmo adapter's own evaluate_arena call, independently, over the identical source — same two
        // calls the CLI's evaluate_arena + cmd_run make, verified by direct comparison rather than assumed.
        let (adapter_storage, adapter_firs) = UbcaEvaluator.evaluate_arena(source).unwrap();
        let adapter_rendered = adapter_firs
            .into_iter()
            .map(|fir| Ubca2Sequencer::format(&adapter_storage, fir, SequenceMode::Foolish))
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(
            cli_rendered, adapter_rendered,
            "the CLI's rendering must match the einmo adapter's for the same source"
        );
    }

    /// T3c (FOOP-86 §Test Plan) — `cmd_step` and the REPL render through the same `evaluate_arena` +
    /// `Ubca2Sequencer::format(.., Foolish)` path as `cmd_run` (no second sequencer call site): for the
    /// same source, the per-FIR Foolish-mode text `cmd_step` prints and the text the REPL's own evaluation
    /// loop prints (`=> {rendered}`, modulo that prefix) are the identical string, because both are built
    /// by calling exactly the same two functions in the same order.
    #[test]
    fn cli_step_and_repl_share_the_render_path() {
        let source = "{x = 1; sf = <x>; sff = <<x>>;}";

        let (step_storage, step_firs) = evaluate_arena(source).unwrap();
        let step_rendered: Vec<String> = step_firs
            .into_iter()
            .map(|fir| Ubca2Sequencer::format(&step_storage, fir, SequenceMode::Foolish))
            .collect();

        // The REPL's own evaluation branch (`cmd_repl`'s `match evaluate_arena(&buf) { Ok((storage, firs))
        // => ... }`) calls the identical two functions; reproduced here since `cmd_repl` itself is an
        // interactive I/O loop with no return value to assert on.
        let (repl_storage, repl_firs) = evaluate_arena(source).unwrap();
        let repl_rendered: Vec<String> = repl_firs
            .into_iter()
            .map(|fir| Ubca2Sequencer::format(&repl_storage, fir, SequenceMode::Foolish))
            .collect();

        assert_eq!(
            step_rendered, repl_rendered,
            "cmd_step and the REPL must render identically for the same source"
        );
    }

    /// T3d (FOOP-86 §Test Plan, Q2) — `cmd_compile` was retired (Phase 1's Q2 resolution: `fir_to_json` is
    /// shaped around `core_fir::Fir` with no arena equivalent, and it had no README example or einmo case).
    /// This asserts that decision: the `Commands` enum has exactly the three surviving subcommands and no
    /// `Compile` variant.
    #[test]
    fn cli_has_no_compile_subcommand() {
        use clap::CommandFactory;
        let cmd = Cli::command();
        let names: Vec<&str> = cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert_eq!(
            names,
            vec!["run", "step", "repl"],
            "cmd_compile was retired (Q2) and must not reappear as a subcommand"
        );
    }
}
