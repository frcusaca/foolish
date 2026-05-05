# Phase 4 — CLI

> Goal: A command-line tool that wires Phase 1 + Phase 2 + Phase 3 into a
> daily-driver binary. Users can compile, evaluate, and inspect Foolish
> source from the shell, and use the REPL to compose Foolish snippets
> interactively.

---

## Phase 4 Deliverable

A `foolish` binary (from the `foolish-cli` crate) with subcommands:

| Subcommand | Behavior |
|-----------|----------|
| `foolish compile <file.foo>` | Phase 1 only — emit FIR JSON to stdout |
| `foolish run <file.foo>` | Phase 1 + Phase 2 + Phase 3 — emit final evaluation result |
| `foolish step <file.foo>` | Phase 1 + Phase 2 — emit intermediate steps for debugging |
| `foolish repl` | Interactive REPL: each line extends a persistent top-level brane |

---

## Implementation with clap

```toml
# foolish-cli/Cargo.toml
[package]
name = "foolish-cli"
version = "0.1.0"
edition = "2024"

[dependencies]
foolish-core = { path = "../foolish-core" }
clap = { workspace = true }
anyhow = { workspace = true }
```

```rust
// foolish-cli/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "foolish")]
#[command(about = "Foolish language CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile .foo source to FIR JSON
    Compile { file: PathBuf },
    /// Evaluate .foo source and print result
    Run { file: PathBuf },
    /// Step-evaluate .foo source (debug output)
    Step { file: PathBuf },
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
```

---

## REPL Session Model

Each REPL line is appended as a new statement to one persistent
top-level brane. Later lines see earlier names via unanchored backward
search. Per Foolish's writing-order semantics, **earlier
statements do NOT retroactively see names defined by later lines**.

```
> x = 42                ← statement 1, CONSTANT 42
> y = x + 1             ← statement 2, sees x; CONSTANT 43
=> y = 43
> z = missing           ← statement 3, ECONSTANIC
=> z = 🧠??
> missing = 7           ← statement 4, CONSTANT — z stays ECONSTANIC
=> missing = 7
> z2 = missing          ← statement 5, NEW search; finds missing = 7
=> z2 = 7
```

### REPL implementation

```rust
fn cmd_repl() -> anyhow::Result<()> {
    let mut session = Session::new();
    let mut buf = String::new();
    let mut collecting = false;

    loop {
        let prompt = if collecting { ".. " } else { "> " };
        print!("{}", prompt);
        std::io::stdout().flush()?;

        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;

        // Track brace depth for multiline
        buf.push_str(&line);
        let depth = buf.chars().filter(|c| *c == '{').count()
            - buf.chars().filter(|c| *c == '}').count();

        if depth > 0 {
            collecting = true;
            continue;
        }

        let source = buf.clone();
        buf.clear();
        collecting = false;

        match session.evaluate(&source) {
            Ok(output) => println!("=> {}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
```

---

## Phase 4 Exit Criteria

- `foolish run` matches Phase 2 + Phase 3 approval test output.
- REPL handles multiline input, parse errors, concatenation, and ECONSTANIC display.
- A REPL session test demonstrates writing-order semantics.
- `foolish --help` is informative.

---

## Last Updated

**Date**: 2026-05-05
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — Rust Phase 4 CLI plan. Uses clap derive macros
for subcommand parsing. REPL implementation with brace-depth tracking for multiline.
