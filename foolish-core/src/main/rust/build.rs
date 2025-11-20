use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../src/main/antlr4/Foolish.g4");
    println!("cargo:rerun-if-changed=../../pom.xml");

    const ANTLR_TOOL_URL: &str = "https://github.com/rrevenantt/antlr4rust/releases/download/antlr4-4.8-2-Rust0.3.0-beta/antlr4-4.8-2-SNAPSHOT-complete.jar";
    const ANTLR_TOOL_JAR: &str = "antlr4-4.8-2-SNAPSHOT-complete.jar";

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("unable to locate project root");

    let antlr_tool_dir = project_root.join("target/antlr-rust");
    let antlr_tool_path = antlr_tool_dir.join(ANTLR_TOOL_JAR);

    if !antlr_tool_path.exists() {
        fs::create_dir_all(&antlr_tool_dir).expect("unable to create antlr tool directory");

        let tool_path_str = antlr_tool_path
            .to_str()
            .expect("antlr tool path contains invalid UTF-8");

        let download_commands = [
            ("curl", vec!["-L", "-o", tool_path_str, ANTLR_TOOL_URL]),
            ("wget", vec!["-O", tool_path_str, ANTLR_TOOL_URL]),
        ];

        let mut downloaded = false;
        for (command, args) in download_commands {
            let status = Command::new(command)
                .args(&args)
                .status();

            match status {
                Ok(status) if status.success() => {
                    downloaded = true;
                    break;
                }
                Ok(status) => {
                    println!("cargo:warning=Download command '{}' failed with status: {}", command, status);
                }
                Err(e) => {
                    println!("cargo:warning=Failed to execute command '{}': {}. Is it installed and in your PATH?", command, e);
                }
            }
        }

        if !downloaded {
            panic!(
                "unable to download ANTLR Rust tool; install curl or wget and ensure network access"
            );
        }
    }

    let status = Command::new("mvn")
        .arg("-q")
        .arg("exec:exec@generate-sources-rust")
        .current_dir(&project_root)
        .status()
        .expect("failed to execute mvn");

    if !status.success() {
        panic!("mvn generate-sources-rust failed");
    }

    let generated_dir = project_root.join("target/generated-sources/antlr-rust");
    if !generated_dir.exists() {
        panic!(
            "expected generated sources at {}",
            generated_dir.display()
        );
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    fs::create_dir_all(&out_dir).expect("unable to create OUT_DIR");

    let mut module_entries = Vec::new();
    for entry in fs::read_dir(&generated_dir).expect("failed to read generated directory") {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("invalid UTF-8 in generated filename");
        let module_name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("invalid UTF-8 in generated filename");

        let destination = out_dir.join(file_name);
        fs::copy(&path, &destination).expect("failed to copy generated file");
        module_entries.push((module_name.to_owned(), file_name.to_owned()));
    }

    if module_entries.is_empty() {
        panic!(
            "no Rust sources were generated in {}",
            generated_dir.display()
        );
    }

    module_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mod_path = out_dir.join("antlr_mod.rs");
    let mut mod_file = File::create(&mod_path).expect("failed to write antlr_mod.rs");
    for (module_name, file_name) in module_entries {
        writeln!(mod_file, "#[path = \"{}\"] pub mod {};", file_name, module_name)
            .expect("failed to write module entry");
    }
}
