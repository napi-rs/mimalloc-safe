# `mimalloc-safe`

Forked from https://github.com/purpleprotocol/mimalloc_rust

[![Latest Version]][crates.io] [![Documentation]][docs.rs]

A drop-in global allocator wrapper around the [mimalloc](https://github.com/microsoft/mimalloc) allocator.
Mimalloc is a general purpose, performance oriented allocator built by Microsoft.

## Usage

```rust
use mimalloc_safe::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
```

## Requirements

A **C** compiler is required for building [mimalloc](https://github.com/microsoft/mimalloc) with cargo.

## Usage with secure mode

Using secure mode adds guard pages,
randomized allocation, encrypted free lists, etc. The performance penalty is usually
around 10% according to [mimalloc](https://github.com/microsoft/mimalloc)
own benchmarks.

To enable secure mode, put in `Cargo.toml`:

```ini
[dependencies]
mimalloc-safe = { version = "*", features = ["secure"] }
```

[crates.io]: https://crates.io/crates/mimalloc-safe
[Latest Version]: https://img.shields.io/crates/v/mimalloc-safe.svg
[Documentation]: https://docs.rs/mimalloc-safe/badge.svg
[docs.rs]: https://docs.rs/mimalloc-safe
