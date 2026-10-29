# Lingo Language Specification v0.1

## 1. Program Entry & Modules
Every Lingo executable defines a `def main():` function. Source files act as modules.

## 2. Indentation & Layout
Lingo uses significant whitespace (Python off-side rule):
- Standard indent is 4 spaces.
- Blank lines and full-line comments (`# ...`) do not affect indentation state.
- Inconsistent mixing of tabs and spaces is a compile-time syntax error.
- Line continuation is allowed within unclosed parentheses `()`, brackets `[]`, and braces `{}`.

## 3. Literals & Primitives
- Integers: `42`, `-7`, `0xFF` (64-bit signed `i64`).
- Floats: `3.14159`, `1e-4` (64-bit IEEE-754 `f64`).
- Booleans: `true`, `false`.
- Strings: `"Hello, Lingo!"` (UTF-8 valid).
- Nil/None: `nil`.

## 4. Variables & Scoping
- **Dynamic Variables**: Declared on initial assignment without `let`:
  ```python
  x = 10
  x = "hello"  # valid dynamic reassignment
  ```
- **Constrained Variables**: Declared with explicit type annotation using `let`:
  ```python
  let count: i32 = 0
  # count = "invalid" -> rejected at compile time
  ```
- **Lexical Scope**: Variables are scoped to their enclosing block / function. Inner scopes may shadow outer scopes.

## 5. Control Flow
- Conditionals: `if cond: ... elif cond: ... else: ...`
- Loops: `while cond: ...` and `for item in collection: ...`
- Functions: `def name(arg1, arg2: i32) -> ret_type: ...`
