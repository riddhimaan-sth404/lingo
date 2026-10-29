# Lingo Safety Invariants & Security Review (Milestone 30)

## 1. Memory Safety Model
Lingo guarantees memory safety in safe programs through:
1. **Tagged Pointer ABI**: Dynamic values are stored as a 16-byte `LingoValue` struct containing an explicit discriminator `tag: i64` and payload `data: i64`. Operations check the tag prior to dereferencing.
2. **Deep Cloning across Concurrency Boundaries**: Data transmitted through `Channel` instances undergoes recursive value cloning for non-copy types (`String`, `List`), ensuring that memory is not concurrently mutated without synchronization.
3. **No Uninitialized Memory**: All variables are either bound to a default `Nil` value or initialized on declaration.

## 2. Supply-Chain & Unsafe Rust Audit
All `unsafe` blocks in `crates/lingo-runtime` are strictly limited to:
- Interacting with atomic reference counters (`AtomicUsize`).
- Safe dereferencing of verified `Box::into_raw` pointers.
- Null-pointer guards on all incoming string and collection parameters.
