# ealloc

[![crates.io](https://img.shields.io/crates/v/ealloc.svg)](https://crates.io/crates/ealloc)
[![docs.rs](https://docs.rs/ealloc/badge.svg)](https://docs.rs/ealloc)

A `no_std` memory allocator with pluggable free-block management strategies, built on [`ecore`](https://crates.io/crates/ecore).

Unlike general-purpose allocators that make one-size-fits-all trade-offs, ealloc lets you **fine-tune every aspect** at compile time via type parameters — block size limits, alignment granularity, pointer width, and search strategy — so you can optimize for your specific application: minimize fragmentation, shrink metadata overhead, or maximize throughput, all without runtime cost. With metadata as low as single-digit bytes per free block, ealloc makes dynamic memory allocation practical even on MCUs with only a few kilobytes of SRAM.

## Design Rationale: Why Not Standalone Allocation Algorithms?

Traditional allocators couple the "block-finding strategy" with "block splitting/coalescing," forcing each algorithm (TLSF, FirstFit, HalfTree…) to re-implement the same split-and-merge logic.

ealloc **fully decouples** these two concerns:

```Text
┌──────────────────────────────────────┐
│         LeanFlexAllocator             │
│  ┌────────────────────────────────┐  │
│  │  Block splitting               │  │
│  │  Block coalescing              │  │
│  │  Alignment handling            │  │
│  │  realloc / grow / shrink       │  │
│  │  Mutex                         │  │
│  └───────────┬────────────────────┘  │
│              │ depends on only 3 APIs│
│              ▼                       │
│  ┌────────────────────────────────┐  │
│  │     FreeBlockManager           │  │
│  │  • take_out(size) → block      │  │
│  │  • register(block)             │  │
│  │  • unregister(block)           │  │
│  └────────────────────────────────┘  │
└──────────────────────────────────────┘
```

- **`LeanFlexAllocator`** handles the logic common to all allocation strategies: take a free block from `FreeBlockManager` → split as needed → return to caller; on deallocation, coalesce adjacent free blocks → register back. **This logic is identical regardless of the search strategy.**
- **`FreeBlockManager`** only cares about *how free blocks are organized*: TLSF's two-level bitmap? HalfTree's binary tree? Or a simple FirstFit linked list? Just implement the three core operations: `take_out` / `register` / `unregister`.

This separation yields two key benefits:
1. **Adding a new strategy is trivial**: implement just three APIs — no need to rewrite splitting/coalescing code.
2. **Common logic is written once**: splitting, coalescing, alignment handling, realloc optimization, etc. are maintained in a single place.

> Analogy: `FreeBlockManager` is to the allocator what a hash function is to `HashMap` — swap the strategy by swapping one component.

## Allocation Strategies

| Type | Description | Node Size |
|------|-------------|-----------|
| `Tlsf` | Two-Level Segregated Fit — O(1) allocation, best general-purpose choice | 2 pointers |
| `HalfTree` | Binary tree-based half-tree algorithm, pairs well with `DesignatedVictim` | 3 pointers |
| `FirstFit` | Simple first-fit linked list search, lowest overhead | 1 pointer |
| `SimpleFirstFit` | Minimal implementation, suitable for testing and controlled environments | 1 pointer |

### DesignatedVictim Wrapper

`DesignatedVictim<MIN_SIZE, MAX_SIZE, Manager>` is a generic `FreeBlockManager` wrapper. It maintains a "designated victim" (caching the most recently freed small block) on top of the inner manager, similar to `dlmalloc`'s DV mechanism. Small frees are preferentially cached rather than immediately returned to the inner manager; subsequent allocations of the same size hit in O(1), reducing fragmentation and improving locality. Recommended pairing with `HalfTree`: `DesignatedVictim<64, 256, HalfTree<...>>`.

## Configurable Parameters

ealloc pursues zero-overhead at compile time — all parameters are determined via generics with no virtual dispatch at runtime. There are three core parameters: `StateElement`, `Element`, and `OptiPtr`. They are interrelated and jointly determine the allocator's memory overhead, management range, and fragmentation characteristics.

### Parameter Interaction Overview

The table below shows the effects of different parameter combinations using typical embedded scenarios (assuming TLSF with 2 pointers per node):

| Scenario | StateElement | Element | OptiPtr | Node Size | Max Block | Address Range | Suitable Memory |
|----------|:---:|:---:|:---:|:---:|:---:|:---:|------|
| Tiny MCU | `u8` | `u16` | `NonZero<u16>` | 4 B | 254 B | 128 KiB | ≤ 64 KB SRAM |
| Small MCU | `u8` | `u32` | `NonZero<u16>` | 4 B | 508 B | 256 KiB | ≤ 256 KB SRAM |
| Medium MCU | `u16` | `u32` | `NonZero<u16>` | 4 B | ~128 KiB | 256 KiB | ≤ 512 KB SRAM |
| Medium MCU | `u16` | `u64` | `NonZero<u16>` | 4 B | ~256 KiB | 512 KiB | ≤ 1 MB SRAM |
| Large MCU | `usize` | `u64` | `NonZero<u16>` | 4 B | platform limit | 512 KiB | multi-MB SRAM |
| General | `usize` | `u64` | `NonNull<u64>` | 16 B | platform limit | full space | any |

> **Key relationships**: Max block = f(StateElement, Element); Node overhead = f(OptiPtr); Address range = f(OptiPtr, Element). In embedded scenarios, `NonZero<u16>` is almost always the best choice — only 2 bytes per pointer, with an address range sufficient for most MCU SRAM.

### `StateElement` — Controls Max Block Size

Determines the bit-width of the block size counter. Must satisfy `size_of::<Self>() <= size_of::<usize>()`. Block size is counted in `Element` units; the bit-width limits the maximum number of elements in a single block. For embedded use, typically choose `u8` or `u16`:

| StateElement | Max Elements | Max Block (Element=u32) | Max Block (Element=u64) | Typical Scenario |
|:---:|:---:|:---:|:---:|------|
| `u8` | 127 | 508 B | ~1 KiB | Tiny MCU (≤ 64 KB) |
| `u16` | 32,767 | ~128 KiB | ~256 KiB | Small/medium MCU (≤ 1 MB) |
| `usize` | — | `isize::MAX` | `isize::MAX` | Unlimited / general-purpose |

> Max elements = `StateElement::MAX >> 1` (one bit reserved for free/used flag). When `size_of::<StateElement>() == size_of::<usize>()`, the count stores **bytes** directly instead of elements, so max block is `isize::MAX`. Smaller `StateElement` saves per-block header storage (2 × StateElement per block).

### `Element` — Alignment Granularity and Internal Fragmentation

Memory is organized in units of `Element`. All allocation sizes and addresses are multiples of `size_of::<Element>()`. `Element` can be any type (not limited to primitive integers), e.g. `u32`, `u64`, `[u32; 2]`, as long as `size % align == 0` and `align >= 2`.

Every allocated block carries a fixed overhead: a **Joint** (2 × `StateElement`) at each boundary for coalescing, plus the free-block **Node** at the head when the block is free. A free block must be large enough to hold both the Node and the tail Joint, setting the *minimum block size* = `Node + Joint::SIZE` (rounded up to Element alignment).

The table below uses TLSF with `NonZero<u16>` + `u16` StateElement as a concrete example (Node = 4 B, Joint = 4 B):

| Element | Granularity | Min Free Block | Block for 18 B alloc | Usable | Frag | Addr Range (NonZero\<u16\>) |
|:---:|:---:|:---:|:---:|:---:|:---:|------|
| `u16` | 2 B | 4u × 2 = 8 B | 11u × 2 = 22 B | 18 B | 0 B | 128 KiB |
| `u32` | 4 B | 2u × 4 = 8 B | 6u × 4 = 24 B | 20 B | 2 B | 256 KiB |
| `u64` | 8 B | 1u × 8 = 8 B | 3u × 8 = 24 B | 20 B | 2 B | 512 KiB |

> Block size = `ceil((user_size + Joint::SIZE), align)`, Usable = block − Joint::SIZE, Frag = Usable − user_size. Fragmentation only occurs when rounding pushes the block beyond what the user requested plus overhead.

- **Smaller Element**: finer granularity, lower fragmentation floor, but smaller manageable memory range for a given `StateElement`, and smaller `OptiPtr` address range.
- **Larger Element**: larger management and address ranges, but higher minimum allocation size and more internal fragmentation for small allocations.

**Over-aligned allocations**: when the requested alignment exceeds `align_of::<Element>()`, the allocator falls back to a slower **over-aligned path** that reserves extra elements at the block front for pointer realignment. This path is fully supported but carries a performance cost — choose `Element` alignment to match your most common allocation alignment, not necessarily `max_align_t`. The trade-off is: larger `Element` (higher alignment) = fewer over-aligned fallbacks, but more fragmentation for small objects.

### `OptiPtr` — Narrowed Pointers

Free block nodes store pointers to other free blocks. In embedded scenarios, using full-width pointers (4–8 bytes) for these links is extremely wasteful. ealloc supports narrowed pointers via the `OptimizedPtr` trait:

| Pointer Type | Width (32-bit) | Width (64-bit) | Address Range | Use Case |
|--------------|:---:|:---:|------|------|
| `NonNull<Element>` | 4 B | 8 B | Full address space | General / heap |
| `NonZero<u16>` | 2 B | 2 B | 64K Elements | Embedded MCU |
| `NonZero<u32>` | — | 4 B | 4G Elements | 64-bit large memory |
| `FixedBasePtr<NonZero<u16>>` | 2 B | 2 B | 64K Elements (fixed base) | Static memory pool |

Taking TLSF as an example (node = 2 pointers + block state), per-node cost for each pointer width:

| OptiPtr | Node Size | 100 Free Blocks Total Overhead |
|:---:|:---:|:---:|
| `NonNull<u64>` | 20 B | 2000 B |
| `NonNull<u32>` | 12 B | 1200 B |
| `NonZero<u16>` | 6 B | 600 B |

In MCU scenarios, the number of free blocks can reach dozens to hundreds; the memory saved by narrowed pointers is significant. `NonZero<u16>` covers virtually all MCU SRAM sizes and is the most common choice.

### TLSF-Specific Parameters

```rust,ignore
Tlsf<TlsfParms<StateElement, Element, OptiPtr, FlMap, SlMap>, FL, SL>
```

- **`FlMap` / `SlMap`**: Integer types for first-level and second-level bitmaps. Typically `FlMap` uses `u16`/`u32`, `SlMap` uses `u8`.
- **`FL` / `SL`**: Number of first-level and second-level bins. `SL` must be a power of two. Use `TlsfParms::calc_two_level_for_size(max_region_size)` for automatic calculation.

### HalfTree-Specific Parameters

```rust,ignore
HalfTree<StateElement, Element, OptiPtr, BINS_COUNT>
```

- **`BINS_COUNT`**: Number of bins, which determines the maximum block size that can be managed. Each bin corresponds to a size class (power of two). Must be less than `StateElement::BITS`.

## Quick Example

```rust
use ealloc::{LeanFlexAllocator, NoopMutex, TlsfParms, Tlsf};
use core::num::NonZero;

// StateElement=u16  → max single block ~256 KB (Element=u64)
// Element=u64       → 8-byte alignment granularity
// OptiPtr=u16       → node pointers only 2 bytes, saving memory
// FlMap=u16         → supports up to 16 first-level bits
// FL=13, SL=8       → 13×8 two-level matrix
type Alloc = LeanFlexAllocator<NoopMutex, Tlsf<TlsfParms<u16, u64, NonZero<u16>, u16>, 13, 8>>;

static mut BUF: [u64; 4096] = [0; 4096];  // 32 KiB
let alc = Alloc::new(NoopMutex::new(), Tlsf::new());
// unsafe { alc.init_with_slice(&mut BUF).unwrap(); }
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | | Enable `std::sync::Mutex` as `AllocatorMutex` |

## MSRV

Rust 1.87+

## License

MIT — see [repository](https://github.com/pbl-pw/ecore) for full license.
