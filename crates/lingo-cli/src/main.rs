use clap::{Parser as ClapParser, Subcommand};
use lingo_driver::Driver;
use lingo_fmt::Formatter;
use lingo_pkg::PackageManager;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(ClapParser)]
#[command(name = "lingo")]
#[command(
    about = "The Lingo Programming Language Ecosystem & Compiler",
    long_about = "Lingo - Statically compiled language combining Pythonic syntax with native LLVM performance and memory safety."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate source syntax, names, and types without linking
    Check {
        /// Path to .ln source file
        file: PathBuf,
    },
    /// Compile a .ln program to a native standalone executable
    Build {
        /// Path to .ln source file
        file: PathBuf,
        /// Output executable path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Build and execute a .ln program directly
    Run {
        /// Path to .ln source file
        file: PathBuf,
    },
    /// Discover and run test functions in .ln files
    Test {
        /// Path to .ln source file or directory
        #[arg(default_value = "examples")]
        path: PathBuf,
    },
    /// Format a .ln source file with canonical indentation
    Fmt {
        /// Path to .ln source file
        file: PathBuf,
        /// Overwrite file in-place
        #[arg(short, long)]
        write: bool,
    },
    /// Generate API documentation for a .ln module
    Doc {
        /// Path to .ln source file
        file: PathBuf,
    },
    /// Clean compiler temporary build artifacts and cache
    Clean,
    /// Explain a Lingo compiler diagnostic or concept
    Explain {
        /// Diagnostic identifier or keyword (e.g. type-mismatch, off-side-rule, ffi)
        topic: String,
    },
    /// Package management operations
    Pkg {
        #[command(subcommand)]
        action: PkgCommands,
    },
    /// Start the interactive Lingo REPL
    Repl,
}

#[derive(Subcommand)]
enum PkgCommands {
    /// Initialize a new Lingo project manifest
    Init {
        /// Project directory / name
        name: String,
    },
    /// Build a project from lingo.toml manifest
    Build,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut driver = Driver::new();

    match cli.command {
        Commands::Check { file } => {
            driver.check(&file)?;
        }
        Commands::Build { file, output } => {
            driver.build(&file, output.as_deref())?;
        }
        Commands::Run { file } => {
            driver.run(&file)?;
        }
        Commands::Test { path } => {
            run_tests(&mut driver, &path)?;
        }
        Commands::Fmt { file, write } => {
            format_file(&file, write)?;
        }
        Commands::Doc { file } => {
            generate_doc(&file)?;
        }
        Commands::Clean => {
            clean_build_artifacts()?;
        }
        Commands::Explain { topic } => {
            explain_topic(&topic);
        }
        Commands::Pkg { action } => match action {
            PkgCommands::Init { name } => {
                let dir = Path::new(&name);
                let manifest = PackageManager::init_project(dir, &name)?;
                println!("📦 Created new Lingo package at {:?}", manifest);
            }
            PkgCommands::Build => {
                let current_dir = std::env::current_dir()?;
                let manifest = PackageManager::load_manifest(&current_dir)?;
                let entry_path = current_dir.join(&manifest.entry);
                println!("📦 Building package '{}' (v{}) ...", manifest.name, manifest.version);
                driver.build(&entry_path, None)?;
            }
        },
        Commands::Repl => {
            start_repl(&mut driver)?;
        }
    }

    Ok(())
}

fn run_tests(driver: &mut Driver, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Running Lingo test runner on {:?} ...\n", path);
    let mut test_files = Vec::new();

    if path.is_file() {
        test_files.push(path.to_path_buf());
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if p.extension().map_or(false, |ext| ext == "ln") {
                test_files.push(p);
            }
        }
    }

    let mut passed = 0;
    let mut failed = 0;

    for file in &test_files {
        print!("  test {:<35} ... ", file.file_name().unwrap().to_string_lossy());
        io::stdout().flush()?;

        match driver.run(file) {
            Ok(_) => {
                println!("ok");
                passed += 1;
            }
            Err(_) => {
                println!("FAILED");
                failed += 1;
            }
        }
    }

    println!("\nTest result: {} passed; {} failed; finished in all suites.\n", passed, failed);
    if failed > 0 {
        return Err("One or more tests failed".into());
    }
    Ok(())
}

fn format_file(file: &Path, write: bool) -> Result<(), Box<dyn std::error::Error>> {
    let src = fs::read_to_string(file)?;
    let formatted = Formatter::format_source(&src);

    if write {
        fs::write(file, formatted)?;
        println!("✨ Formatted {:?}", file);
    } else {
        print!("{formatted}");
    }
    Ok(())
}

fn generate_doc(file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let src = fs::read_to_string(file)?;
    println!("# Documentation for `{}`\n", file.file_name().unwrap().to_string_lossy());

    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("def ") || trimmed.starts_with("extern def ") {
            println!("### `{trimmed}`\n");
        } else if trimmed.starts_with('#') {
            println!("> {}\n", &trimmed[1..].trim());
        }
    }
    Ok(())
}

fn clean_build_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("lingo_build");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    println!("🧹 Cleaned build cache and temporary files.");
    Ok(())
}

fn explain_topic(topic: &str) {
    match topic.to_lowercase().as_str() {
        "off-side-rule" | "indentation" => {
            println!("--- [Explanation: Indentation & Layout] ---");
            println!("Lingo uses indentation to define code blocks (the off-side rule).");
            println!("Each block level must use 4 spaces. Tabs are disallowed to prevent layout ambiguity.");
        }
        "type-mismatch" | "types" => {
            println!("--- [Explanation: Type System & Constraints] ---");
            println!("Variables are dynamic by default. Adding type constraints with `let x: i64`");
            println!("opts into strict compile-time type verification, disallowing incompatible assignments.");
        }
        "ffi" | "c-abi" => {
            println!("--- [Explanation: Foreign-Function Interface (FFI)] ---");
            println!("Lingo interacts with external native libraries via standard C ABI calling conventions.");
            println!("Use `extern def func_name(arg: int) -> int` to bind foreign native functions.");
        }
        "concurrency" | "channel" => {
            println!("--- [Explanation: Concurrency & Channels] ---");
            println!("Lingo prevents data races in safe code by isolating concurrent threads and using");
            println!("message-passing channels (`Channel()`, `channel_send`, `channel_recv`) with deep cloning.");
        }
        _ => {
            println!("No dedicated explanation found for '{}'. Try: indentation, types, ffi, concurrency.", topic);
        }
    }
}

fn start_repl(driver: &mut Driver) -> Result<(), Box<dyn std::error::Error>> {
    println!("Lingo 1.0 Interactive REPL");
    println!("Type 'help' for instructions, 'clear' to reset session, or 'exit' to quit.\n");

    let stdin = io::stdin();
    let temp_file = std::env::temp_dir().join("lingo_repl.ln");
    let mut session_defs: Vec<String> = Vec::new();

    loop {
        print!(">>> ");
        io::stdout().flush()?;

        let mut first_line = String::new();
        if stdin.read_line(&mut first_line)? == 0 {
            break;
        }

        let trimmed_first = first_line.trim();
        if trimmed_first == "exit" || trimmed_first == "quit" {
            break;
        }
        if trimmed_first == "help" {
            println!("Lingo REPL Help:");
            println!("  - Enter any expression (e.g., 2 + 2, len(\"hello\"), {{\"a\": 1}}) to evaluate.");
            println!("  - Enter function definitions (def f(x): ...) across multiple lines.");
            println!("  - Press Enter on an empty line to finish a multi-line block.");
            println!("  - Type 'clear' to reset previous definitions in this session.");
            println!("  - Type 'exit' to leave the REPL.\n");
            continue;
        }
        if trimmed_first == "clear" {
            session_defs.clear();
            println!("Session cleared.\n");
            continue;
        }
        if trimmed_first.is_empty() {
            continue;
        }

        let mut code_block = first_line.clone();
        if trimmed_first.ends_with(':') {
            loop {
                print!("... ");
                io::stdout().flush()?;
                let mut continuation_line = String::new();
                if stdin.read_line(&mut continuation_line)? == 0 {
                    break;
                }
                if continuation_line.trim().is_empty() {
                    break;
                }
                code_block.push_str(&continuation_line);
            }
        }

        let trimmed_block = code_block.trim();
        let is_func_def = trimmed_block.starts_with("def ");
        let is_extern = trimmed_block.starts_with("extern ");

        if is_func_def || is_extern {
            let mut test_code = String::new();
            for def in &session_defs {
                test_code.push_str(def);
                test_code.push('\n');
            }
            test_code.push_str(trimmed_block);
            test_code.push('\n');
            test_code.push_str("def main():\n    return 0\n");

            fs::write(&temp_file, &test_code)?;
            match driver.build(&temp_file, None) {
                Ok(_) => {
                    session_defs.push(trimmed_block.to_string());
                    println!("Defined function.");
                }
                Err(_) => {
                    eprintln!("Error compiling definition.");
                }
            }
            continue;
        }

        let is_stmt = trimmed_block.starts_with("let ") || trimmed_block.contains('=');
        let mut full_program = String::new();
        for def in &session_defs {
            full_program.push_str(def);
            full_program.push('\n');
        }
        full_program.push_str("def main():\n");
        if is_stmt {
            for line in trimmed_block.lines() {
                full_program.push_str(&format!("    {line}\n"));
            }
        } else {
            full_program.push_str(&format!("    println({trimmed_block})\n"));
        }

        fs::write(&temp_file, &full_program)?;
        let _ = driver.run(&temp_file);
    }

    let _ = fs::remove_file(&temp_file);
    Ok(())
}
