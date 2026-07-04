# Changelog

All notable changes to the ecore workspace will be documented in this file.

## [0.10.0] - 2026-07-06

### ecore
- Core library for `no_std` bit-level operations
- Bit integers (`u1`..`u128`, `i1`..`i128`) with full arithmetic
- Bitfield structs and enums via `#[bitfld]` attribute
- Ranged integers (`RInt`) with compile-time bounds checking
- Enum maps (`EnumMap`) for efficient lookup tables
- Variable-length integer encoding (`VarInt`)
- Endian-aware type representations (`LEndian`, `BEndian`, `Unalign`)

### ecore-macro
- `#[derive(MapEnum)]` — Enum-to-array mapping derive macro
- `#[bitfld]` — Bitfield struct/enum attribute macro
- `map_enum!` — Const-friendly enum mapping macro
- `rint!` — Compile-time ranged integer type generator
- `#[derive(BitsCast)]` — Bit-level cast derive macro
- `#[derive(IterEnumDiscriminants)]` — Enum discriminant iteration

### ealloc
- `no_std` memory allocator with pluggable free-block managers
- `Tlsf` — TLSF (Two-Level Segregated Fit) allocator
- `FirstFit` — Simple first-fit allocator
- `HalfTree` — Half-tree based allocator
- `SimpleFirstFit` — Minimal first-fit for testing
- Custom mutex abstraction (`AllocatorMutex`)
- `FixedBasePtr` for fixed-address memory regions
