//! Prototype E: LLVM IR Generation Prototype
//! Demonstrates generation of valid LLVM IR text for primitives, arithmetic, and native calls.

pub struct LlvmProtoEmitter {
    ir: String,
}

impl LlvmProtoEmitter {
    pub fn new() -> Self {
        let mut ir = String::new();
        ir.push_str("; Lingo LLVM IR Prototype\n");
        ir.push_str("target triple = \"x86_64-pc-windows-msvc\"\n\n");
        ir.push_str("declare void @lingo_println_i64(i64)\n");
        ir.push_str("declare void @lingo_println_str(ptr, i64)\n\n");
        Self { ir }
    }

    pub fn emit_arithmetic_function(&mut self, name: &str, a: i64, b: i64) -> &str {
        self.ir.push_str(&format!(
            "define i64 @{name}() {{\nentry:\n  %1 = add i64 {a}, {b}\n  ret i64 %1\n}}\n\n"
        ));
        &self.ir
    }

    pub fn finish(mut self) -> String {
        self.ir.push_str("define i32 @main() {\nentry:\n  ret i32 0\n}\n");
        self.ir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llvm_ir_emission() {
        let mut emitter = LlvmProtoEmitter::new();
        emitter.emit_arithmetic_function("compute", 10, 20);
        let ir = emitter.finish();
        assert!(ir.contains("define i64 @compute()"));
        assert!(ir.contains("add i64 10, 20"));
        assert!(ir.contains("define i32 @main()"));
    }
}
