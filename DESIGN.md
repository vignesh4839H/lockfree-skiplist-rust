# Technical Design Document

## Hazard Pointer & Memory Ordering
This lock-free skip list implementation leverages Rust's explicit memory ordering via `std::sync::atomic::Ordering`.

* **Logical vs Physical Deletion**: Nodes are marked logically prior to unlinking to avoid structural race conditions under contention.
* **Acquire-Release Semantics**: `Ordering::Acquire` and `Ordering::Release` ensure pointer state changes are visible across thread boundaries without requiring full sequential consistency barriers (`SeqCst`).
* **Memory Safety**: Unsafe code blocks are encapsulated carefully within safe abstractions, ensuring zero data races while managing manual dynamic allocation.
