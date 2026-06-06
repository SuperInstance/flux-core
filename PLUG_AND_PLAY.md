# PLUG_AND_PLAY — flux-core-tmp

> *Integration guide for incorporating flux-core-tmp into your SuperInstance fleet setup.*

## Dependency

```toml
[dependencies]
flux_core_tmp = "0.1.0"
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | yes | Standard library support |
| `alloc` | yes | Allocator support (for no_std) |

## Integration Patterns

### Basic Usage

```rust
use flux_core_tmp::*;
```

### With the Ternary Ecosystem

This crate works naturally with:
- [ternary-core](https://github.com/SuperInstance/ternary-core) for Z₃ arithmetic
- [ternary-types](https://github.com/SuperInstance/ternary-types) for type-level encodings

## Configuration

flux-core-tmp requires minimal configuration. Where configuration is needed:
- No runtime configuration files needed — pure library
- Feature gates control optional dependencies

## Compatibility

- **Rust edition**: 2021+
- **Targets**: All tier-1 Rust targets (x86_64, aarch64, ARM Cortex)
- **no_std**: Not applicable
