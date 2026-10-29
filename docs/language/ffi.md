# Foreign-Function Interface (FFI) & Bridge Contract

## C ABI Foundation
Lingo interfaces with native foreign code through standard C calling conventions (`cdecl` / `system`).
- Primitives (`i64`, `f64`, pointers) pass directly via platform registers.
- Strings cross boundaries as null-terminated pointers or explicit length-pointer pairs.
- Foreign panics and unwinding are intercepted at the boundary to prevent undefined behavior.
