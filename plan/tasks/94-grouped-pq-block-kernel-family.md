# Task 94: Grouped-PQ / PqFastScan Block Kernel Family (All AMs × All ISAs)

Status: in review (2026-06-09; local implementation through `reviews/task-94/015-sve-vector-lane-warning-cleanup/`, final Graviton 4 / benchmark closeout evidence pending approval)
Owner: coder (to be assigned). Phase III parallel — multiple coders OK across Tasks 93–98.
Priority: 2 (highest documented kernel-win ROI in Phase III)

## Why

Grouped-PQ (a.k.a. PqFastScan) is the **canonical block-kernel
quant** in the literature. FAISS PQ4-FastScan papers report 3–8×
scoring-share wins over scalar PQ across IVF and DiskANN-class
indexes. The kernel structure — per-group LUT lookup + accumulate
across groups, with the LUT held in SIMD registers and 32
candidates scored per inner-loop iteration — is the prototype
for every other LUT-based block kernel in this matrix.

Current scoring paths in tree:

- IVF: `IvfQuantizer::score_ip_from_parts` on `IvfPreparedQuery::
  PqFastScan` branch — per-candidate `score_ip_from_parts` over
  grouped codes.
- DiskANN: `DiskannPreparedPrefilter::GroupedPq::score` per node
  during Vamana scan.

IVF and DiskANN posting/page batches are typically 64–256
candidates — well above the 32-block threshold. This is the
highest-ROI cell in the Phase III matrix.

## Scope

### In scope

Implementation note: Phase 1 reviewer feedback approved
`src/quant/grouped_pq_block/` as the module path, replacing the original
`pq_fastscan32/` wording while retaining PqFastScan terminology for the
storage format.

1. **Scalar block kernel** at `src/quant/grouped_pq_block/scalar.rs`
   following the FAISS PQ4-FastScan algorithm (per-group SIMD
   LUT lookup pattern, in scalar form as the bit-equal reference).
2. **NEON variant** under `src/quant/grouped_pq_block/neon.rs`.
   The first landing uses the approved f32 LUT gather / vector accumulate
   shape; packed-table repacks remain follow-up-only if measurements show
   gather stalls dominate. Validate on Graviton 4 with the NEON path forced,
   plus any cheaper ARM sanity host if available.
3. **SVE/SVE2 variant** under `src/quant/grouped_pq_block/sve.rs`.
   The first landing uses a vector-length-agnostic SVE accumulation helper
   over f32 LUT gathers. Report a width-specific label such as `sve2-128`
   only when measured at runtime.
4. **AVX2 variant** under `src/quant/grouped_pq_block/avx2.rs`.
   The first landing uses `_mm256_i32gather_ps` against the canonical f32 LUT
   rows; byte/shuffle repacks are follow-up-only if justified by measurement.
5. **`QuantCodec` registration** in IVF + DiskANN grouped-PQ
   impls. Width-based gating per ADR-076.
6. **Per-(AM × quant × ISA) measurement** on real10k / 50k /
   100k corpora for IVF + DiskANN grouped-PQ surfaces.
7. **Recall byte-equal at bench level** at every cell. Scalar
   block kernel asserts bit-equal vs the per-candidate
   reference; SIMD variants ULP-tolerant per ADR-076.
8. **Per-(AM × ISA) closeout matrix.**

### Out of scope

- New grouped-PQ AM coverage (e.g., grouped-PQ on HNSW or SPIRE
  if not present). Cover only IVF + DiskANN where grouped-PQ
  already exists.
- Codebook training optimization (Task 23 territory).
- AVX-512 variant.

## Acceptance criteria

1. `src/quant/grouped_pq_block/` module live with scalar + NEON +
   SVE + AVX2 per Task 92 convention.
2. IVF + DiskANN grouped-PQ scoring routes through `QuantCodec::
   <batch method selected by Task 91>` and dispatches to the kernel
   for batches ≥ 32.
3. Recall byte-equal at every cell.
4. ≥ 2× scoring-share latency on each ISA per AM. Per-ISA stop
   condition < 1.5× → document and continue.
5. End-to-end p50/p95/p99 measured; no regression beyond ULP noise.
6. Existing `pg_test` surfaces for grouped-PQ on IVF + DiskANN
   pass.
7. No new `unsafe` outside ISA module boundary; full safety doc
   on intrinsic-using modules.
8. Per-AM closeout matrix in `reviews/task-94/.../artifacts/`.

## Phases

### Phase A — Scalar block kernel + layout audit + scalar-baseline measurement

- Land `grouped_pq_block/scalar.rs` following the FAISS PQ4-FastScan
  algorithm shape. Per-group LUT lookup + group-wise accumulation
  across all 32 lanes.
- Audit IVF and DiskANN grouped-PQ code packing, group ordering,
  LUT signedness, and accumulation range before implementing SIMD.
  SIMD kernels must match the audited layout rather than assuming
  FAISS-compatible nibble packing.
- Route IVF + DiskANN grouped-PQ scoring through `QuantCodec`.
- Real10k baseline + kernel-on/off counters on both AMs.

### Phase B — NEON variant + Graviton ARM measurement

- Land `grouped_pq_block/neon.rs` using the approved first-pass f32 LUT
  gather / vector accumulate shape.
- Graviton 4 measurement with SVE disabled or the NEON dispatch path
  forced. A cheaper ARM host may be used only as supplemental sanity
  evidence.

### Phase C — SVE variant + Graviton 4 measurement

- Land `grouped_pq_block/sve.rs` using vector-length-agnostic SVE/SVE2
  accumulation. This is where Graviton 4's vector width may pay off most;
  record the measured vector length in artifacts.
- AWS Graviton 4 measurement; snapshot + destroy per memory.

### Phase D — AVX2 variant + Intel desktop measurement

- Land `grouped_pq_block/avx2.rs` using the approved first-pass f32 LUT
  gather path.
- Intel desktop measurement.

### Phase E — Per-(AM × ISA) closeout matrix + status flip

- Aggregate matrix.
- Scoring-share per ISA per AM with ≥2× gate call.
- End-to-end deltas with attribution.
- Status flip to `complete`.

## Per-AM validation gate

Same as Task 93. For each (AM × corpus) cell, kernel-on vs
kernel-off:

1. Recall byte-equal at bench level.
2. Scoring-share latency improves ≥ 2× per ISA.
3. End-to-end p50/p95/p99 improves or stays within ULP noise.
4. Storage unchanged.
5. `pg_test` surfaces pass.

## Stop conditions

- If per-ISA scoring-share < 1.5× on a touched AM: document, do
  not back out.
- If grouped-PQ algebra requires multiple group counts (current
  source has `group_count` parameter), parameterize the kernel
  for the most common group counts (typically 8, 16, 32). If a
  particular `group_count` configuration doesn't fit the kernel
  cleanly, fall back to scalar for that configuration and
  document.
- Recall byte-equality failure → BLOCK and triage.

## Coordination

- **Depends on Task 91 Phase 5** (DiskANN migration onto
  `QuantCodec`). IVF grouped-PQ trait registration may need Task
  91 Phase 2 IVF retouch first, depending on packet 008's
  grouped-PQ limit.
- **Depends on Task 92** infrastructure.
- **Parallel with Tasks 93, 95–98.** Highest-ROI Phase III task —
  prioritize coder assignment.
- **Consumed by Task 99.**

## References

- Task 15 (PqFastScan first-class)
- Task 23 (LSQ codebook refinement — grouped codebook prior art)
- Task 60 (DiskANN RaBitQ — grouped-PQ predecessor on DiskANN)
- ADR-030 (FastScan grouped subvector scoring)
- ADR-076 (universal block kernel pattern — Task 92)
- FAISS PQ4-FastScan paper (André et al., "Accelerated nearest
  neighbor search with quick ADC") — primary literature reference

## Estimated size

Medium-large. 5–8 weeks for one coder. The algorithm is the most
heavily documented kernel in the literature, but the AM-side
integration (grouped codebook model state ownership — Task 87
packet 008's known limit) needs to be cleanly handled at the
`QuantCodec` trait boundary first.
