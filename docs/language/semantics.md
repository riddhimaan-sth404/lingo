# Lingo Core Semantic Rules

## Evaluation Order
- Expressions are evaluated strictly from left to right.
- In function calls `f(a, b, c)`, `f` is evaluated first, then `a`, `b`, and `c` in sequence.
- Short-circuiting applies to boolean operators `and` and `or`.

## Variables and Mutation
- Unadorned assignments `x = val` bind in the current lexical scope.
- Reassignment to dynamic variables preserves the new type dynamically:
  ```python
  x = 100        # x has runtime type Int
  x = "a string" # x has runtime type Str
  ```
- Constrained variables declared with `let` cannot change their underlying type:
  ```python
  let x: i64 = 42
  x = 10         # valid
  # x = "hello"  # Compile-time error: TypeMismatch
  ```

## Truthiness
Truthiness determines condition evaluation in `if` and `while`:
- `nil` is falsy.
- Booleans evaluate to their value (`false` is falsy, `true` is truthy).
- Integers: `0` is falsy, all non-zero integers are truthy.
- Floats: `0.0` is falsy, non-zero floats are truthy.
- Strings: `""` (empty string) is falsy, non-empty strings are truthy.
- Collections: empty lists and maps are falsy, non-empty are truthy.

## Equality Semantics
- Equality `==` checks value equality for primitives and strings.
- Collections evaluate element-wise equality.
- Dynamic `Value` equality compares both type tags and data values.
