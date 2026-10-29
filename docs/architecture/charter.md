# Lingo Project Charter & Scope

## 1. Project Purpose & Scope
Lingo is a statically compiled programming language engineered to combine the clean syntax and developer ergonomics of Python with native compilation performance, deterministic resource management, and zero default tracing garbage collection.

### Primary Use Cases
1. **Command-Line & Systems Utilities**: Fast startup, zero runtime dependency overhead, single self-contained binaries.
2. **Application Backends & High-Performance Services**: Predictable latency, data-race-free concurrency, memory safety without GC pauses.
3. **General-Purpose Programming**: Rapid scripting and prototyping through dynamic variables, progressively hardened with explicit type annotations.

### Design Non-Goals (First Release Scope)
- **No Tracing Garbage Collector**: Lingo will not incorporate a stop-the-world or concurrent tracing collector.
- **No Universal Crate/Package Wrapping**: Foreign-language interop will proceed via strict, safe C ABI bindings rather than unvalidated native macro or memory layout assumptions.
- **No JIT Compiler in 1.0**: Compilation targets Ahead-Of-Time (AOT) machine code via LLVM.
- **No Unscoped Dynamic Magic**: Dynamic variables exist to ease ergonomics, but cannot break defined type-safety barriers or undefined behavior guarantees.
