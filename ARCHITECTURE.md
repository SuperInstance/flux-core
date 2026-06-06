# Architecture — flux-core-tmp

> *Internal design, data flow, and extension points.*

## Overview

This crate provides core functionality in the SuperInstance fleet.

## Source Structure

14 Rust source file(s) in `src/`:
- `vm` — module
- `bytecode` — module
- `a2a` — module
- `error` — module
- `vocabulary` — module

## Data Flow

```
Input → flux_core_tmp::transform → Ternary {-1,0,+1} → Output
```

## Design Principles

1. **Zero-dependency where possible** — keep the trust chain minimal
2. **Ternary by default** — all operations expose or consume {-1, 0, +1}
3. **No hidden state** — pure functions over explicit parameters
4. **Fail closed** — errors return safe defaults (typically 0/neutral)
