---
id: ADR-005
title: "TurboCode serialization via serde + bincode"
status: DECIDED
impact: HIGH for FR-001 (type), FR-007 (page layout)
date: 2026-04-03
---
# ADR-005: TurboCode serialization via serde + bincode

## Context

We need to store `TurboCode` in Postgres as raw bytes — both in the `tqvector` varlena type (heap) and in TqElementTuple (index pages).

## Investigation Results

### TurboCode is fully serializable

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TurboCode {
    pub polar_code: PolarCode,
    pub residual_sketch: QjlSketch,
}

pub struct PolarCode {
    pub dim: usize,
    pub bits: u8,
    pub radii: Vec<f32>,           // dim/2 f32 values
    pub angle_indices: Vec<u16>,    // dim/2 u16 values
}

pub struct QjlSketch {
    pub dim: usize,
    pub projections: usize,
    pub signs: Vec<i8>,            // `projections` i8 values (+1 or -1)
}
```

All fields are public. All types derive `Serialize + Deserialize`.

### Byte size calculation (1536-dim, 4-bit, projections=384)

```
PolarCode:
  dim: 8 bytes (usize)
  bits: 1 byte
  radii: 768 × 4 = 3,072 bytes
  angle_indices: 768 × 2 = 1,536 bytes
  → 4,617 bytes

QjlSketch:
  dim: 8 bytes
  projections: 8 bytes
  signs: 384 × 1 = 384 bytes
  → 400 bytes

Total TurboCode: ~5,017 bytes (bincode, no length prefixes)
```

**Note**: this is larger than the original architecture doc's ~768 byte estimate. The architecture doc assumed bit-packed storage. The `turbo-quant` crate stores radii as f32 (not quantized) and signs as i8 (not bit-packed).

### Compression opportunities

The crate stores data in a convenient but not space-optimal format:
- `radii` as f32 (4 bytes each) — could be f16 for storage (2 bytes)
- `signs` as i8 (1 byte each) — could be bit-packed (÷8)
- `dim`, `bits`, `projections` metadata repeated per code — could be stored once per column

For v0.1, use bincode serialization as-is. Optimize storage format later if needed.

## Decision

**Use bincode for serialization.** Fast, compact, deterministic, widely used.

### Wire format for tqvector type

```
[header: 11 bytes][bincode(TurboCode): variable]

Header:
  dim: u16 (2 bytes)
  bits: u8 (1 byte)
  projections: u16 (2 bytes)
  seed: u64 (8 bytes) — NOT stored; carried at column/index level

Wait — seed is needed to reconstruct the quantizer for scoring. We need it per-code
or per-index. Per-index is better (one seed for the whole column).
```

### Revised wire format

```
[dim: u16][bits: u8][projections: u16][bincode(TurboCode): variable]

Total header: 5 bytes
```

The `seed` is stored as an index-level parameter (WITH clause) or column-level default, not per-vector. The quantizer is reconstructed once per query from `(dim, bits, projections, seed)`.

### Add to Cargo.toml

```toml
bincode = "1"
```
