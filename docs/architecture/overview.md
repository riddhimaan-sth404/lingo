# Lingo Compiler & Runtime Architecture Overview

## Compiler Pipeline

```text
       Lingo Source (.ln)
               │
               ▼
         Source Manager       (crates/lingo-source)
               │
               ▼
             Lexer            (crates/lingo-lexer)
               │
               ▼
            Parser            (crates/lingo-parser)
               │
               ▼
          Syntax AST          (crates/lingo-ast)
               │
               ▼
        Name Resolution       (crates/lingo-resolve)
               │
               ▼
       Semantic Analysis      (crates/lingo-types / analysis)
               │
               ▼
      Ownership & Borrowing   (crates/lingo-analysis)
               │
               ▼
        High-Level IR (HIR)   (crates/lingo-ir)
               │
               ▼
         Optimizer Pass       (crates/lingo-ir / opt)
               │
               ▼
        LLVM IR Emitter       (crates/lingo-codegen-llvm)
               │
               ▼
      LLVM Optimization &
      Object Code Emitter     (Target Triple / Machine Code)
               │
               ▼
          Native Linker       (LLD / link.exe)
               │
               ▼
     Standalone Executable    (.exe / elf)
```

## System Components
1. **`lingo-source`**: Source file loader, file ID manager, byte offsets, spans (`Span`), and line-column mapping.
2. **`lingo-diagnostics`**: ANSI diagnostic emitter with underline indicators, severity levels (`Error`, `Warning`, `Note`), and actionable suggestions.
3. **`lingo-lexer`**: Indentation-stack-driven tokenizer producing `TokenKind`, tracking indentation depth with synthetic `Indent` and `Dedent` tokens.
4. **`lingo-ast`**: Strongly-typed syntax tree representation preserving source spans.
5. **`lingo-parser`**: Precedence-climbing Pratt expression parser and recursive-descent statement parser with error-recovery sync points.
6. **`lingo-resolve`**: Lexical scoping, symbol table construction, variable declaration tracking, shadowing, and module-level export verification.
7. **`lingo-types`**: Static and dynamic type representations, constraints, and coercion rules.
8. **`lingo-ir`**: Control-flow graph (CFG) based intermediate representation suitable for optimizations.
9. **`lingo-codegen-llvm`**: LLVM IR builder producing verifiable LLVM IR strings / modules with target machine compilation.
10. **`lingo-runtime`**: Essential runtime library providing tagged dynamic values, standard string/collection operations, and memory primitives.
11. **`lingo-driver` & `lingo-cli`**: CLI orchestration (`lingo build`, `lingo run`, `lingo check`).
