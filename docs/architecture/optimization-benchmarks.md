# Lingo Optimization Campaign & Benchmarks (Milestone 38 & 39)

## Optimization Pipeline
1. **Lingo IR Level**:
   - Constant folding of literal operations at compile time.
   - Dead-code elimination after return statements.
2. **LLVM Backend Level**:
   - Instruction combining, devirtualization, and loop unrolling via LLVM optimization passes (`-O1`, `-O2`, `-O3`).
   - Function inlining and dead global elimination.

## Target Platform Expansion (Milestone 41)
- Tier 1: `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`
- Tier 2: `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`
- Tier 3: `wasm32-unknown-unknown`
