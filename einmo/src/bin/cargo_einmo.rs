//! `cargo-einmo` — the cargo-subcommand alias binary.
//!
//! Installing `einmo` (`cargo install einmo`) yields this alias so that
//! `cargo einmo …` works identically to `einmo …`. Cargo invokes a
//! subcommand binary as `cargo-einmo einmo <args…>`, so the alias strips the
//! injected `einmo` argument and delegates to the same CLI entry point.

fn main() -> std::process::ExitCode {
    // When run as `cargo einmo <args>`, argv is `["cargo-einmo", "einmo", <args>]`.
    // Strip the injected subcommand name so the shared parser sees `<args>` only.
    let mut args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if args.get(1).map(|a| a == "einmo").unwrap_or(false) {
        args.remove(1);
    }
    einmo::cli_main(args)
}
