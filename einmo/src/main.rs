//! The `einmo` CLI binary — a thin entry point over the library's CLI.

fn main() -> std::process::ExitCode {
    einmo::cli_main(std::env::args_os().collect())
}
