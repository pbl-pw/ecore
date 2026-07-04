# ecore-macro

[![crates.io](https://img.shields.io/crates/v/ecore-macro.svg)](https://crates.io/crates/ecore-macro)
[![docs.rs](https://docs.rs/ecore-macro/badge.svg)](https://docs.rs/ecore-macro)

Procedural macros for the [`ecore`](https://crates.io/crates/ecore) crate.

## Macros

| Macro | Description |
|-------|-------------|
| `#[derive(MapEnum)]` | Generate enum-to-array mapping with optional discriminant enum |
| `#[bitfld(...)]` | Attribute macro for defining bitfield structs and enums |
| `map_enum!` | Const-friendly enum variant → value mapping |
| `rint!` | Compile-time ranged integer type generator |
| `#[derive(BitsCast)]` | Derive bit-level casting between representations |
| `#[derive(IterEnumDiscriminants)]` | Derive iteration over enum discriminants |

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
ecore = "0.10"
ecore-macro = "0.10"
```

## Nightly: const trait impls

Enable the `const-trait` feature to generate `const fn` trait implementations (requires nightly Rust with `#![feature(const_trait_impl)]`):

```toml
ecore-macro = { version = "0.10", features = ["const-trait"] }
```

## MSRV

Rust 1.87+

## License

MIT — see [repository](https://github.com/pbl-pw/ecore) for full license.
