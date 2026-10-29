# ADR 0002: LLVM Code-Generation Backend Strategy

## Status
Accepted

## Context
Section 2.2 of the Lingo Specification specifies separating language semantics from LLVM and using LLVM as the native code-generation backend.

## Decision
1. The compiler lowers Syntax AST -> Resolved AST -> Lingo Intermediate Representation (IR).
2. Lingo IR performs semantic validations, constant folding, and control flow optimization.
3. The LLVM backend (`lingo-codegen-llvm`) lowers Lingo IR into valid, verifiable LLVM Intermediate Representation (`.ll`).
4. Native compilation invokes LLVM tools (`clang`/`llc` or internal LLVM pipeline) to emit target object code (`.obj` / `.o`), followed by native linking with LLD (`lld-link` on Windows, `ld.lld` on Linux) into a standalone native executable.
5. Transpilation to Rust source code is explicitly replaced by this direct LLVM code generation pipeline.
