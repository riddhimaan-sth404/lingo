# Lingo (`.ln`)

**Lingo** is a modern programming language combining Rust's memory safety, speed, and features with Python's clean syntax and dynamic typing flexibility.

## Features

- 🐍 **Python-like Syntax:** Semantic indentation (off-side rule) for clean block structure.
- ⚡ **No GC Memory Safety:** Dynamic objects use reference counting (`Rc<RefCell<...>>`) to adhere to Rust's compile-time borrow checker with zero runtime garbage collection pauses.
- 🔀 **Dynamic by Default + Static Hints:** Unannotated variables default to dynamic runtime values (`Value`). Adding type hints (e.g. `x: i32`) opts directly into zero-cost native Rust types.
- 📦 **Seamless `crates.io` Interop:** Import standard library modules or third-party Cargo crates natively (`import std.collections.HashMap`).
- 🔄 **Bootstrapped Transpiler Architecture:** Implemented in Rust, transpiling `.ln` source code into idiomatic Rust code (`.rs`) compiled via `cargo`.

---

## Quick Start

### Installation & Prerequisites
Requires a working Rust toolchain (`rustc` and `cargo`).

```bash
git clone https://github.com/riddhimaan-sth404/lingo.git
cd lingo
cargo build --release
```

### Running Examples

Execute all sample programs using the batch runner:

```cmd
run-all.bat
```

Or run individual `.ln` files directly:

```bash
cargo run -- run examples/hello.ln
cargo run -- run examples/rust_crates_and_types.ln
cargo run -- run examples/crates_interop.ln
```

---

## Language Syntax Guide

### Dynamic & Static Variables
```python
def main():
    # Dynamic typing by default
    greeting = "Hello from Lingo!"
    println(greeting)

    # Static type hints
    let x: i32 = 40
    let y: i32 = 2
    let sum: i32 = x + y
    println("Static sum: " + sum)
```

### Structs & Methods
```python
class Person:
    name: String
    age: i32

def main():
    let p = Person { name: "Alice".to_string(), age: 30 }
    println("Person name: " + p.name)
```

### Rust Ecosystem Interop
```python
import std.collections.HashMap

def main():
    let mut map: HashMap<String, i32> = HashMap.new()
    map.insert("apples".to_string(), 5)
    map.insert("oranges".to_string(), 12)

    for key in map.keys():
        println(key)
```

---

## License

MIT / Apache-2.0
