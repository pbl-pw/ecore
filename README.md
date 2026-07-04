# ecore

**`ecore`** is a `no_std` Rust workspace for embedded/bare-metal/systems programming. It supplements the standard `core` library with bit-level types and operations: non-standard-width integers, bitfield structs/enums, range-constrained integers, enum-to-array mapping, and a pluggable memory allocator.

## Sub-crates

### [`ecore`](./ecore) — Core Library [![crates.io](https://img.shields.io/crates/v/ecore.svg)](https://crates.io/crates/ecore) [![docs.rs](https://docs.rs/ecore/badge.svg)](https://docs.rs/ecore)

`ecore` fills the gaps in Rust's standard library for bit-level operations: `bitint` provides `u1..u127` / `i1..i127` non-standard-width integers (native integer storage, zero-overhead, const-friendly); `bitfld` enables declarative macro-based bitfield structs and tag+payload bitfield enums; `ranged` offers compile-time value range constraints via `RInt`; `map_enum` implements O(1) enum-to-array index mapping; plus a generic integer trait system (`int`), endianness/alignment adaptation (`repr`), `nbool`, and more. All modules share a unified trait system, delivering compounding benefits when used together.

See [ecore/README.md](./ecore/README.md) or [docs.rs/ecore](https://docs.rs/ecore).

### [`ecore-macro`](./ecore-macro) — Procedural Macros [![crates.io](https://img.shields.io/crates/v/ecore-macro.svg)](https://crates.io/crates/ecore-macro) [![docs.rs](https://docs.rs/ecore-macro/badge.svg)](https://docs.rs/ecore-macro)

Provides procedural macros for `ecore`: `#[derive(MapEnum)]` auto-generates enum-to-array mapping, `#[bitfld]` defines bitfield layouts, `map_enum!` / `rint!` enable const-friendly type construction, plus `#[derive(BitsCast)]`, `#[derive(IterEnumDiscriminants)]`, and more.

See [ecore-macro/README.md](./ecore-macro/README.md) or [docs.rs/ecore-macro](https://docs.rs/ecore-macro).

### [`ealloc`](./ealloc) — Memory Allocator [![crates.io](https://img.shields.io/crates/v/ealloc.svg)](https://crates.io/crates/ealloc) [![docs.rs](https://docs.rs/ealloc/badge.svg)](https://docs.rs/ealloc)

A `no_std` memory allocator with pluggable strategies. It fully decouples generic logic (block splitting/coalescing) from the free-block search strategy — `LeanFlexAllocator` handles splitting, coalescing, alignment, and realloc, while `FreeBlockManager` only needs to implement three APIs: `take_out` / `register` / `unregister`. Built-in strategies include TLSF (O(1) allocation, real-time friendly), HalfTree (binary tree), and FirstFit (minimal-overhead linked list). All parameters (max block size, alignment granularity, pointer width) are determined at compile time via generics — zero runtime overhead.

See [ealloc/README.md](./ealloc/README.md) or [docs.rs/ealloc](https://docs.rs/ealloc).

## Minimum Supported Rust Version (MSRV)

Rust **1.87** or later (edition 2024).

## License

Licensed under the [MIT License](LICENSE).
