# Lingo 1.0 Release Notes (Milestone 36)

## Version: 1.0.0 Stable Release
**Release Date:** October 2026

Lingo 1.0 delivers a high-performance compiled programming language combining Python's expressive syntax with native LLVM ahead-of-time compilation, strong memory safety foundations without a tracing garbage collector, and a complete developer tooling suite.

### Key Highlights
- **Approachability**: Python-inspired semantic indentation with dynamic variables by default and static type hints (`let x: i64`).
- **Performance**: Direct code generation through LLVM yielding standalone native machine executables.
- **Deterministic Memory**: Zero stop-the-world tracing GC pauses; reference-counted dynamic structures with deterministic RAII cleanup.
- **Safe Concurrency**: Data-race-free thread execution with deep-cloned message passing channels.
- **FFI & C ABI**: First-class `extern def` bindings and `lingo-bridge` generator.
- **Full Tooling Suite**: Built-in test runner, code formatter (`lingo fmt`), documentation generator (`lingo doc`), package manager (`lingo pkg`), and interactive REPL (`lingo repl`).

### Compatibility Commitment
Lingo 1.0 commits to semantic versioning. All 1.x releases will maintain backward source compatibility for valid 1.0 Lingo programs.
