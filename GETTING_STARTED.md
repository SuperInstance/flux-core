# Getting Started — flux-core-tmp

> *Estimated time to complete: 5 minutes*

## Prerequisites

- **Rust 1.75+** (MSRV)
- Cargo (included with Rust)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
flux_core_tmp = "0.1.0"
```

Or build from source:

```bash
git clone https://github.com/SuperInstance/flux-core-tmp.git
cd flux-core-tmp
cargo build --release
cargo test
```

## Core Concepts

This crate is part of the SuperInstance other ecosystem. It provides:

- Core functionality for the SuperInstance fleet

## Quick Start

```rust
use {flux_core_tmp};
// Create and use the primary functionality
let result = some_function();
println!("{:?}", result);
```

## Running Tests

```bash
cargo test
```

## Next Steps

- [ARCHITECTURE.md](./ARCHITECTURE.md) — Internal design and data flow
- [PLUG_AND_PLAY.md](./PLUG_AND_PLAY.md) — Integration and configuration
- [CONTRIBUTING.md](./CONTRIBUTING.md) — How to contribute

## Ecosystem

This crate is part of the **[SuperInstance Fleet](https://github.com/SuperInstance)**.
- [ternary-core](https://github.com/SuperInstance/ternary-core) — shared ternary traits
- [cocapn](https://github.com/SuperInstance/cocapn) — repo-first agent infrastructure
- [construct-core](https://github.com/SuperInstance/construct-core) — hardware-agnostic agent runtime
