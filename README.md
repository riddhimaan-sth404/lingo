# Lingo (`.ln`)

**Lingo** is a statically compiled programming language combining the approachable, clean syntax of Python with native compilation performance through LLVM, low runtime overhead, and strong memory-safety foundations without a tracing garbage collector.

---

## 🚀 Key Architectural Features (Phases 1 to 5)

- 🐍 **Python-Inspired Indentation:** Semantic indentation (off-side rule) with structured layout tracking.
- ⚡ **Native Ahead-Of-Time (AOT) Compilation via LLVM:** Independent compiler lowering Lingo AST -> High-Level IR (HIR) -> Constant-folding Optimizer -> Verified LLVM IR -> Standalone Native Executables (`.exe` on Windows, ELF on Linux).
- 🔀 **Dynamic Typing by Default + Explicit Type Constraints:** Variables without annotations are dynamically typed values (`LingoValue`), while variables declared with `let x: i64` enforce strict compile-time type checking.
- 🛡️ **Zero Tracing Garbage Collection:** Memory safety achieved through deterministic ownership, reference counting for dynamic heap entities, and RAII cleanup without stop-the-world GC pauses.
- 🏗️ **Clean Modular Architecture:**
  - `crates/lingo-source`: File management, source spans, and line-column mapping.
  - `crates/lingo-diagnostics`: Rich compiler diagnostics with source line snippets and pointer carets.
  - `crates/lingo-lexer`: Whitespace-aware tokenizer tracking indentation stacks (`Indent`, `Dedent`, `Newline`).
  - `crates/lingo-ast`: Span-annotated Abstract Syntax Tree.
  - `crates/lingo-parser`: Recursive-descent & Pratt expression parser with operator precedence.
  - `crates/lingo-resolve`: Lexical scoping, symbol tables, and declaration checking.
  - `crates/lingo-types`: Static & dynamic type system contract and constraint validation.
  - `crates/lingo-ir`: High-level Control-Flow Graph (CFG) IR with constant-folding optimizations.
  - `crates/lingo-codegen-llvm`: Target triple code generator emitting verified LLVM IR text (`.ll`).
  - `crates/lingo-runtime`: Native C ABI runtime library managing tagged values, strings, lists, and I/O.
  - `crates/lingo-driver`: Compilation orchestration, LLVM backend compilation, and native linking.
  - `crates/lingo-cli`: Unified command-line interface (`lingo check`, `lingo build`, `lingo run`).

---

## 🛠️ Quick Start

### Prerequisites
- A working Rust toolchain (`rustc` and `cargo`).
- LLVM toolchain (`clang` and `lld`).

### Building the Compiler
```bash
cargo build --release
```

### Running Examples
Run all sample programs using the batch runner:
```cmd
run-all.bat
```

Or run individual `.ln` source files directly:
```bash
cargo run -- run examples/hello.ln
cargo run -- run examples/functions_and_types.ln
cargo run -- run examples/control_flow.ln
```

Or compile directly into a standalone binary:
```bash
cargo run -- build examples/hello.ln -o hello.exe
.\hello.exe
```

---

## 📖 Language Syntax Guide

### Functions, Math & Loops
```python
def factorial(n):
    result = 1
    current = n
    while current > 1:
        result = result * current
        current = current - 1
    return result

def main():
    fact = factorial(5)
    println("factorial(5) = " + fact)
```

### Dynamic & Constrained Variables
```python
def main():
    # Dynamic variable: can change runtime types
    x = 100
    println("x as int: " + x)
    x = "Now x is a dynamic string!"
    println("x as string: " + x)

    # Statically constrained variable: rejected at compile time if mismatched
    let count: i64 = 42
    println("Static count: " + count)
```

### Conditionals & Lists
```python
def main():
    items = [10, 20, 30, 40]
    println("First element: " + items[0])

    if items[0] > 5:
        println("Greater than 5")
    else:
        println("Smaller")
```

---

## 🧪 Testing

Run all unit, integration, and compiler regression tests:
```bash
cargo test --workspace
cargo test --test compile_tests
```

---

## License

MIT / Apache-2.0
