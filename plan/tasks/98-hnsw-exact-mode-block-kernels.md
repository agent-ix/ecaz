# Task 98: HNSW TurboQuant Exact-Score Mode Block Kernels (TiledLut + Int8Approx)

Status: complete (2026-06-10, closeout `reviews/task-98/003-closeout-matrix/`,
reviewer-approved: Phase A width distribution decisive — HNSW exact-mode
flushes reach width >=32 in <0.1% of cases, so the SVE conditional resolves
to skip per the task's own rule; AVX2 variants recorded into Task 99 as the
Intel-lane handoff. Key fact for Task 99: HNSW exact-mode payoff is governed
by partial-width behavior, not 32-wide block frequency)
Owner: coder (to be assigned). Phase III parallel.
Priority: 3 (HNSW-specific; per-frontier batches limit end-to-end win)

## Why

HNSW's `TurboQuantExactScoreMode` enum in
`src/am/ec_hnsw/scan.rs` dispatches the no-prefilter
scoring path across three branches:

- `FullLut` — per-dimension 16-entry LUT lookup. Identical
  algebra to Task 87's `lut32` kernel. Task 87 already routes
  this through `CandidateBatch`; the kernel applies.
- `TiledLut` — LUT lookup with dimensions grouped into tiles
  for cache reuse. Different access pattern.
- `Int8Approx` — int8-quantized approximation of the LUT.
  Integer arithmetic instead of f32.

`TiledLut` and `Int8Approx` are HNSW-specific exact-score modes
chosen for HNSW's traversal pattern. They are NOT full f32
vector scoring — they remain compressed-domain. Task 87
explicitly deferred `TiledLut` and `Int8Approx` at packet 002.
Task 98 picks them up.

HNSW per-frontier batches are 8–32 candidates, smaller than
IVF posting-list chunks. Block kernels apply only when the
frontier batch reaches 32. Smaller flushes fall through to
scalar. End-to-end win is therefore gated on how often the
frontier produces ≥ 32 candidates.

## Scope

### In scope

1. **`TiledLut` block kernel** at
   `src/quant/tiled_lut32/scalar.rs` + NEON + AVX2. SVE
   conditional on Phase A measurement showing meaningful
   frontier-batch ≥ 32 share.
2. **`Int8Approx` block kernel** at
   `src/quant/int8_approx32/scalar.rs` + NEON + AVX2.
   SVE conditional.
3. **`QuantCodec` registration** of both kernels in HNSW's
   TurboQuant exact-score dispatch.
4. **Per-(corpus × ISA) measurement** on HNSW real10k / 50k /
   100k surfaces for both modes.
5. **Per-frontier batch-width distribution measurement** on
   Phase A (scalar). Determines whether SVE + Graviton 4
   measurement is worth running for Phase C.
6. **Recall byte-equal** per ADR-076.
7. **Closeout matrix** with explicit batch-width-distribution
   evidence and per-ISA scoring-share calls.

### Out of scope

- `FullLut` — already covered by Task 87.
- Generic gamma-aware fallback — covered by Task 97.
- PqFastScan on HNSW (Task 94 territory).
- RaBitQ on HNSW (Task 93 territory).
- AVX-512.

## Acceptance criteria

1. `src/quant/tiled_lut32/` and `src/quant/int8_approx32/`
   modules live with scalar + NEON + AVX2 + SVE (if Phase
   A justifies the cloud-bench cost).
2. HNSW TurboQuant exact-score dispatch routes through
   Task 91's selected `QuantCodec` batch method for both
   `TiledLut` and `Int8Approx` modes at batches ≥ 32.
3. Recall byte-equal at every cell.
4. **Documented per-frontier batch-width distribution** per
   corpus size: histogram of flush widths, fraction ≥ 32, mean
   batch width. This is the missing data Phase 7 needed to
   make the HNSW kernel call.
5. Scoring-share latency improves where the kernel fires.
   Per-ISA stop condition < 1.5× → document and continue.
6. End-to-end p50/p95/p99 measured; small end-to-end wins
   acceptable given the batch-width distribution.
7. `pg_test` surfaces for HNSW pass.
8. Safety docs on intrinsic-using modules.

## Phases

### Phase A — Scalar block kernels + batch-width distribution

- Land scalar `tiled_lut32` and `int8_approx32` kernels.
- Route HNSW dispatch through `QuantCodec`.
- Measure per-frontier batch-width distribution on real10k /
  50k / 100k.
- Decision point: if measured ≥ 32 flush share < 20% on all
  corpora, scope down Task 98 to scalar + one ISA (likely
  AVX2 on Intel desktop) and skip Phase C SVE cloud cost.

### Phase B — NEON variants + ARM measurement

- Land NEON variants for both kernels.
- Graviton 4 measurement with SVE disabled or the NEON dispatch path
  forced. A cheaper ARM host may be used only as supplemental sanity
  evidence.

### Phase C — SVE variants + Graviton 4 measurement (conditional)

- Only if Phase A batch-width distribution justifies.
- AWS Graviton 4 measurement; snapshot + destroy.

### Phase D — AVX2 variants + Intel desktop measurement

- Land AVX2 variants.
- Intel desktop measurement.

### Phase E — Closeout matrix with batch-width disclosure

- Aggregate matrix per (mode × corpus × ISA).
- Batch-width distribution prominent in the closeout call.
- Explicit acknowledgment of end-to-end vs scoring-share
  decoupling on HNSW per the Phase 5 design note.
- Status flip.

## Per-AM validation gate

For each (mode × corpus) cell:

1. Recall byte-equal at bench level.
2. Where kernel fires (batches ≥ 32): scoring-share latency
   improves ≥ 1.5× per ISA (relaxed from Task 93/94's 2×
   because HNSW's per-frontier batches are at the lower edge
   of the kernel-friendly range).
3. End-to-end p50/p95/p99 measured; no regression beyond noise.
   End-to-end *improvement* is not a gate because per-frontier
   batches limit the share — only "no regression" is required.
4. Storage unchanged.
5. `pg_test` HNSW surfaces pass.

## Stop conditions

- If batch-width distribution shows < 20% of flushes ≥ 32 on
  all corpora: scope down to scalar + AVX2 only. Don't pay for
  SVE + NEON cloud measurement.
- If `TiledLut` or `Int8Approx` algebra exceeds ADR-076 ULP
  tolerance under SIMD reorder: document and tighten to
  Option A on that mode only.
- If recall byte-equality fails: BLOCK + triage.

## Coordination

- **Depends on Task 91 Phase 4** (HNSW migration onto
  `QuantCodec`).
- **Depends on Task 92** infrastructure.
- **Lower priority than 93/94** because HNSW per-frontier
  batches limit end-to-end payoff. Schedule after 93/94 land
  or as fill-in work for an under-utilized coder.
- **Consumed by Task 99** with explicit batch-width disclosure.

## References

- Task 87 packet 002 (HNSW `TiledLut` + `Int8Approx` original
  deferral)
- Task 87 packets 020/022/023/024 (HNSW zero-counter context).
  Task 98 must start from a valid HNSW TurboQuant exact-mode
  benchmark surface after Task 91's HNSW `QuantCodec` migration;
  if counters still do not fire on that surface, Phase A first
  resolves instrumentation rather than treating Task 87's
  FullLut-oriented closeout as sufficient evidence.
- ADR-018 (HNSW quantized graph quality)
- ADR-076 (universal block kernel pattern — Task 92)

## Estimated size

Medium. 4–6 weeks for one coder, possibly less if Phase A
scope-down skips Phase C. The HNSW prerequisite is a valid
TurboQuant exact-mode benchmark and counter surface after Task
91 migration; if that does not land cleanly, Task 98 inherits
the instrumentation investigation cost.
