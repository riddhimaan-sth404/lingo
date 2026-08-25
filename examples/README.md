# Lingo Examples

This directory contains sample programs demonstrating various language features of **Lingo (`.ln`)**.

---

## Example Programs

### 1. `hello.ln` - Dynamic Typing & Basics
Demonstrates:
- Dynamic variable assignments without explicit type annotations.
- Python-like string concatenation and arithmetic operator precedence.
- List creation (`[1, 2, 3, 4, 5]`) and dynamic printing.
- Python-style block indentation for `if`/`else` control flow.

**Run command:**
```bash
cargo run -- run examples/hello.ln
```

---

### 2. `rust_crates_and_types.ln` - Static Types & Structs
Demonstrates:
- Declaring structs with `class Person:`.
- Explicit Rust type hints (`let x: i32 = 40`).
- Seamless mixing of dynamic and statically typed variables.
- Struct instantiation (`Person { name: ..., age: ... }`) and property access.

**Run command:**
```bash
cargo run -- run examples/rust_crates_and_types.ln
```

---

### 3. `crates_interop.ln` - Rust Std & Crates Interoperability
Demonstrates:
- Direct imports from Rust standard library / Cargo crates (`import std.collections.HashMap`).
- Generic type annotations (`HashMap<String, i32>`).
- Static constructor method calls (`HashMap.new()`).
- Iterating over Rust data structures using `for key in map.keys():`.

**Run command:**
```bash
cargo run -- run examples/crates_interop.ln
```

---

## Running All Examples

On Windows, execute the batch runner:

```cmd
run-all.bat
```
