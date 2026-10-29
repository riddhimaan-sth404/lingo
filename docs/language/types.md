# Lingo Type System & Value Semantics

## Dynamic Value Representation
Dynamic values are represented via tagged structures:
- Dynamic variables carry a runtime tag distinguishing `Nil`, `Bool`, `Int`, `Float`, `Str`, `List`, `Map`.
- Operations on untyped variables are checked dynamically or specialized when provably invariant.

## Static Type Constraints
- When a variable or function parameter is annotated (e.g. `x: i32`), the compiler enforces compile-time type checking.
- Conversions between static types and dynamic values are explicit or validated via checked runtime downcasts.

## Memory & Ownership Contract
- Memory safety without tracing GC is achieved through deterministic ownership:
  - Primitives (`i64`, `f64`, `bool`) are Copy.
  - Heap-allocated entities (Strings, Lists, Objects) follow unique ownership with move semantics by default.
  - Shared ownership is handled through explicit reference-counted wrappers with deterministic RAII destruction upon refcount dropping to 0.
- No dangling pointers: References cannot outlive the owned scope.
