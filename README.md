# ttmap (Trivial Type Map)

[![Crates.io](https://img.shields.io/crates/v/ttmap.svg)](https://crates.io/crates/ttmap)
[![Documentation](https://docs.rs/ttmap/badge.svg)](https://docs.rs/crate/ttmap/)
[![Build](https://github.com/DoumanAsh/type-map/workflows/Rust/badge.svg)](https://github.com/DoumanAsh/type-map/actions?query=workflow%3ARust)

Trivial type-map implementation

Implementation uses type erased values with type as index.
Due to limitation of `TypeId` only types without non-static references are supported. (in future it can be changed)

## Type erasure

Each inserted value is stored on heap, with type erased pointer, using type as key.
When value is retrieved, type information is used as key and pointer is casted to corresponding the type.
This is safe, because Rust allows cast back and forth between pointers as long as the pointer actually points to the type (which is the case).

Static references are allowed, but in current implementation are stored on heap.
It might be changed in future.

## Hash implementation

The map uses simplified `Hasher` that relies on fact that `TypeId` produces unique values only.
In fact there is no hashing under hood, and type's id is returned as it is.

## Requirements:

- `alloc` with enabled global allocator
