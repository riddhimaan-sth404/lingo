# `lingo_runtime`

The core runtime library providing dynamic memory management, operators, and standard functions for the **Lingo** programming language.

---

## Overview

Transpiled Lingo programs automatically depend on and link against `lingo_runtime`. It provides:

1. **`Value` Tagged Enum**:
   The runtime representation for dynamically-typed variables in Lingo:
   - `Nil`: Represents `None` / null.
   - `Bool(bool)`: Boolean values (`True` / `False`).
   - `Int(i64)`: 64-bit signed integers.
   - `Float(f64)`: 64-bit floating point numbers.
   - `Str(String)`: Heap-allocated string values.
   - `List(Rc<RefCell<Vec<Value>>>)`: Dynamic lists with safe shared ownership.
   - `Map(Rc<RefCell<HashMap<String, Value>>>)`: Dynamic dictionary / key-value maps.

2. **No-GC Compile-Time Memory Safety**:
   - Compound dynamic structures (`List`, `Map`) use Rust reference counting (`Rc<RefCell<...>>`) to manage safe shared references without relying on a tracing Garbage Collector (GC).
   - Variables obey Rust lifetime and ownership semantics at compile time.

3. **Operator Overloads**:
   - Full operator trait implementations (`Add`, `Sub`, `Mul`, `Div`, `Rem`, `PartialEq`, `PartialOrd`) across numbers, strings, and collections.
   - Truthiness evaluation (`Value::is_truthy()`) supporting Python-style boolean checks on numbers, strings, and collections.

4. **I/O Functions**:
   - `lingo_print` & `lingo_println` helper functions accepting any type implementing `std::fmt::Display`.
