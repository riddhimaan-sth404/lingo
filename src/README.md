# Lingo Compiler Internals (`src/`)

This directory contains the source code for the **Lingo** compiler and transpiler (`lingoc`).

---

## Compiler Pipeline

The compilation process converts `.ln` source files into optimized Rust code (`.rs`):

```
.ln Source Code
      │
      ▼
   Lexer  (src/lexer.rs)          ──► Indentation tracking stack, emits INDENT/DEDENT
      │
      ▼
   Parser (src/parser.rs)         ──► Recursive descent & Pratt expression parsing
      │
      ▼
    AST   (src/ast.rs)            ──► Abstract Syntax Tree representation
      │
      ▼
  Codegen (src/codegen.rs)        ──► Scope tracking, type inference, generates Rust (.rs)
      │
      ▼
CLI Driver (src/main.rs)          ──► Assembles Cargo project & executes via Cargo
```

---

## Modules

- **`token.rs`**: Token definitions, keywords, operators, and layout tokens (`Newline`, `Indent`, `Dedent`, `Eof`).
- **`lexer.rs`**: Whitespace-aware tokenizer maintaining an indentation stack (Python off-side rule).
- **`ast.rs`**: AST definitions for declarations, statements, expressions, and type annotations.
- **`parser.rs`**: Precedence-climbing parser handling blocks, classes, functions, generics, and expressions.
- **`codegen.rs`**: Emits Rust source code with automatic variable scoping (`let mut`), dynamic `Value` wrapping, static type retention, and crates.io import resolution.
- **`main.rs`**: CLI entry point supporting `lingo run` and `lingo build`.
