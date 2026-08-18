mod ast;
mod codegen;
mod lexer;
mod parser;
mod token;

use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(ClapParser)]
#[command(name = "lingo")]
#[command(about = "The Lingo Programming Language Transpiler & Runner", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile and execute a .ln program
    Run {
        /// Path to .ln source file
        file: PathBuf,
    },
    /// Compile a .ln program to Rust source code
    Build {
        /// Path to .ln source file
        file: PathBuf,
        /// Output .rs file or project dir
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            run_file(&file)?;
        }
        Commands::Build { file, output } => {
            build_file(&file, output.as_deref())?;
        }
    }

    Ok(())
}

fn compile_source(source: &str) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {}", e))?;

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse_program().map_err(|e| format!("Parser error: {}", e))?;

    let mut codegen = codegen::CodeGenerator::new();
    let rust_code = codegen.generate(&ast);

    Ok((rust_code, codegen.imported_crates))
}

fn run_file(file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(file)
        .map_err(|e| format!("Could not read file {:?}: {}", file, e))?;

    let (rust_code, imported_crates) = compile_source(&source)?;

    let build_dir = PathBuf::from("target/lingo_run");
    fs::create_dir_all(build_dir.join("src"))?;

    // Write generated main.rs
    fs::write(build_dir.join("src/main.rs"), rust_code)?;

    // Determine path to lingo_runtime
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    let runtime_path = manifest_dir.join("crates/lingo_runtime");
    let runtime_path_str = runtime_path.to_str().unwrap().replace('\\', "/");

    let mut cargo_toml = format!(
        r#"[package]
name = "lingo_app"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
lingo_runtime = {{ path = "{}" }}
"#,
        runtime_path_str
    );

    for crate_name in imported_crates {
        cargo_toml.push_str(&format!("{} = \"*\"\n", crate_name));
    }

    fs::write(build_dir.join("Cargo.toml"), cargo_toml)?;

    println!("⚡ Compiling and executing via Cargo...");
    let status = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .current_dir(&build_dir)
        .status()?;

    if !status.success() {
        eprintln!("Execution failed with status: {}", status);
    }

    Ok(())
}

fn build_file(file: &Path, output: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(file)
        .map_err(|e| format!("Could not read file {:?}: {}", file, e))?;

    let (rust_code, _) = compile_source(&source)?;

    let out_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| file.with_extension("rs"));

    fs::write(&out_path, rust_code)?;
    println!("✅ Transpiled {:?} -> {:?}", file, out_path);

    Ok(())
}
