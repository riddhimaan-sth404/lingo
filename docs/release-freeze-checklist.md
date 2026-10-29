# Lingo 1.0 Release Candidate & Freeze Checklist (Milestones 34 & 35)

## Verification Checklist
- [x] Syntax specification finalized and documented.
- [x] AST, lexer, and Pratt parser handle all language statements and precedence levels.
- [x] Scoping, symbol resolution, and type checking reject invalid programs with source spans.
- [x] High-level CFG IR and constant-folding optimizer verified.
- [x] LLVM code generator produces verifiable target-triple LLVM IR without undefined behavior.
- [x] Native runtime provides tagged values, heap collections, reference counting, and C ABI FFI.
- [x] Concurrency model guarantees freedom from data races through isolated channel message passing.
- [x] Tooling ecosystem implements check, build, run, test, fmt, doc, clean, explain, pkg, and repl.
- [x] Integration, unit, adversarial, and cross-platform test suites passing at 100%.
