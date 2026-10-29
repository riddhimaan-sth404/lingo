# Lingo Concurrency Model

## Concurrency Guarantees
- Safe Lingo code guarantees freedom from data races at runtime.
- Concurrency isolation relies on distinct thread-local state with message passing.
- Shared references across concurrency boundaries require explicit synchronization or immutable guarantees.
