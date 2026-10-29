use lingo_codegen_llvm::LlvmGenerator;
use lingo_diagnostics::Diagnostic;
use lingo_ir::{IrBuilder, Optimizer};
use lingo_lexer::Lexer;
use lingo_parser::Parser;
use lingo_resolve::Resolver;
use lingo_source::{SourceFile, SourceManager};
use lingo_types::TypeChecker;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Driver {
    pub source_manager: SourceManager,
}

impl Driver {
    pub fn new() -> Self {
        Self {
            source_manager: SourceManager::new(),
        }
    }

    pub fn compile_to_llvm_ir(&mut self, file: &SourceFile) -> Result<String, Diagnostic> {
        let mut lexer = Lexer::new(file);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()?;

        let mut resolver = Resolver::new();
        resolver.resolve_program(&program)?;

        let mut type_checker = TypeChecker::new();
        type_checker.check_program(&program)?;

        let mut ir_builder = IrBuilder::new();
        let mut ir_module = ir_builder.build_module(&program);

        Optimizer::optimize_module(&mut ir_module);

        let mut llvm_gen = LlvmGenerator::new();
        let llvm_ir = llvm_gen.generate_module(&ir_module);

        Ok(llvm_ir)
    }

    pub fn check(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let file = self.source_manager.load_file(path)?;
        match self.compile_to_llvm_ir(&file) {
            Ok(_) => {
                println!("✅ Check passed: {:?}", path);
                Ok(())
            }
            Err(diag) => {
                eprintln!("{}", diag.render(&file));
                Err("Compilation check failed".into())
            }
        }
    }

    pub fn build(
        &mut self,
        source_path: &Path,
        output_path: Option<&Path>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let file = self.source_manager.load_file(source_path)?;
        let llvm_ir = match self.compile_to_llvm_ir(&file) {
            Ok(ir) => ir,
            Err(diag) => {
                eprintln!("{}", diag.render(&file));
                return Err("Compilation failed".into());
            }
        };

        let temp_dir = std::env::temp_dir().join("lingo_build");
        std::fs::create_dir_all(&temp_dir)?;

        let base_name = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let ir_path = temp_dir.join(format!("{base_name}.ll"));
        std::fs::write(&ir_path, &llvm_ir)?;

        let exe_name = if cfg!(windows) {
            format!("{base_name}.exe")
        } else {
            base_name.to_string()
        };

        let out_exe = output_path
            .map(PathBuf::from)
            .unwrap_or_else(|| source_path.with_file_name(exe_name));

        // Locate lingo_runtime static library
        let runtime_lib = self.find_runtime_lib()?;

        // Locate clang compiler executable
        let clang_path = self.find_clang_compiler()?;

        let mut cmd = Command::new(&clang_path);
        cmd.arg(&ir_path)
            .arg("-o")
            .arg(&out_exe)
            .arg(&runtime_lib);

        if cfg!(windows) {
            cmd.arg("-luserenv")
                .arg("-lws2_32")
                .arg("-ladvapi32")
                .arg("-lbcrypt")
                .arg("-lntdll")
                .arg("-lkernel32");
        }

        let output = cmd.output().map_err(|e| {
            format!(
                "Failed to invoke clang compiler ({:?}): {e}. Make sure LLVM/clang is installed.",
                clang_path
            )
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("LLVM Native Compilation error:\n{stderr}").into());
        }

        println!("⚡ Built native executable: {:?}", out_exe);
        Ok(out_exe)
    }

    pub fn run(&mut self, source_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let exe_path = self.build(source_path, None)?;
        println!("🚀 Running {:?} ...\n", exe_path.file_name().unwrap());

        let status = Command::new(&exe_path).status()?;
        if !status.success() {
            eprintln!("\nProgram exited with code: {:?}", status.code());
        }
        Ok(())
    }

    fn find_clang_compiler(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let candidates = [
            PathBuf::from("clang.exe"),
            PathBuf::from("clang"),
            PathBuf::from("C:\\Users\\1385r\\llvm\\bin\\clang.exe"),
            PathBuf::from("C:\\Program Files\\LLVM\\bin\\clang.exe"),
            PathBuf::from("/usr/bin/clang"),
            PathBuf::from("/usr/local/bin/clang"),
        ];

        for cand in &candidates {
            if cand.is_absolute() && cand.exists() {
                return Ok(cand.clone());
            }
            if let Ok(output) = Command::new(cand).arg("--version").output() {
                if output.status.success() {
                    return Ok(cand.clone());
                }
            }
        }

        Err("Could not find clang executable. Please install LLVM/clang or ensure it is on PATH.".into())
    }

    fn find_runtime_lib(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let candidates = [
            PathBuf::from("target/release/lingo_runtime.lib"),
            PathBuf::from("target/debug/lingo_runtime.lib"),
            PathBuf::from("target/release/liblingo_runtime.a"),
            PathBuf::from("target/debug/liblingo_runtime.a"),
            PathBuf::from("../target/release/lingo_runtime.lib"),
            PathBuf::from("../target/debug/lingo_runtime.lib"),
            PathBuf::from("../../target/debug/lingo_runtime.lib"),
        ];

        for cand in &candidates {
            if cand.exists() {
                return Ok(std::fs::canonicalize(cand)?);
            }
        }

        // If not built yet, run cargo build -p lingo-runtime
        println!("⚙️ Compiling lingo-runtime static library...");
        let build_status = Command::new("cargo")
            .arg("build")
            .arg("-p")
            .arg("lingo-runtime")
            .status()?;

        if !build_status.success() {
            return Err("Failed to build lingo-runtime static library".into());
        }

        for cand in &candidates {
            if cand.exists() {
                return Ok(std::fs::canonicalize(cand)?);
            }
        }

        Err("Could not find lingo_runtime static library".into())
    }
}
