---
id: ADR-005
title: "TurboCode serialization via serde + bincode"
status: SUPERSEDED
superseded_by: ADR-006
impact: HIGH for FR-001 (type), FR-007 (page layout)
date: 2026-04-03
---
# ADR-005: TurboCode serialization via serde + bincode

**SUPERSEDED by ADR-006.** The turbo-quant crate and its PolarCode/TurboCode structs were dropped entirely. The own quantizer uses a simple bit-packed wire format — no serde or bincode needed.

## Original Context

The turbo-quant crate stored codes as `PolarCode { radii: Vec<f32>, angle_indices: Vec<u16> } + QjlSketch { signs: Vec<i8> }`, totaling ~5,017 bytes per 1536-dim 4-bit code when serialized via bincode.

## Resolution

ADR-006 decided to implement the quantizer core directly using the MSE+QJL approach. The wire format is now:

```
[mse_packed: ceil(dim * (bits-1) / 8) bytes][qjl_packed: ceil(dim / 8) bytes]
```

At 1536-dim 4-bit: 576 + 192 = **768 bytes** (6.5x smaller than turbo-quant's format). See FR-001 for the complete wire format specification.
