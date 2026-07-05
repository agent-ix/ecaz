# Task 86 TurboVec/TurboQuant Options Report

## Scope

This report compares TurboVec's TurboQuant implementation only against our TurboQuant implementation. It intentionally excludes RaBitQ, grouped PQ, PQ-FastScan, and FAISS except where needed to explain code paths.

## Evidence Status After Packet 008

Packet `reviews/task-86/008-spire-real-spread` is the production-facing benchmark evidence for the no-format-change SPIRE LUT option. It compares pre-LUT SPIRE TurboQuant against the current SPIRE TurboQuant LUT scorer on the normal real10k/real50k/real100k spread using `ecaz bench suite`, `storage_format=turboquant`, `ec_spire`, 200 queries, 1000 latency iterations, and identical nprobe sweeps.

Measured result: recall and storage were unchanged, and SQL latency improved by roughly 2.4% to 4.5% on mean latency across the nine sweep points. Pipeline query p50 improved by roughly 3.8% to 5.7%.

The TQ+ calibration and renorm sections below remain source/prototype findings, not accepted production improvements. They need the same real-corpus recall/latency/storage treatment before any implementation should be considered complete.

## Source Finding

TurboVec is a flat scan over compressed vector codes, with an optional ID map. It is not an HNSW, DiskANN, IVF, or SPIRE-style graph/partition index. Its interesting TQ-specific idea is not a different graph layout; it is a per-index calibrated coordinate space:

- Database vectors are normalized, rotated, shifted/scaled per coordinate, and quantized into packed 4-bit codes.
- Queries are rotated and inverse-calibrated into a lookup table plus bias.
- Candidate scoring remains a packed-code scan. It does not reconstruct full vectors at query time.

Our existing no-QJL 4-bit TurboQuant scorer also scans packed bytes directly. The distinction is that TurboVec calibrates the scalar quantization space and frames the query as LUT/bias in that calibrated space.

## Size

For 1536 dimensions and 4-bit no-QJL TQ:

- Current per-vector TQ payload: `4 + dim * 4 / 8 = 772` bytes (`gamma` plus 768 packed MSE bytes).
- TQ+ calibration-only prototype: same per-vector packed code byte count. It needs index-level calibration metadata: `2 * dim * sizeof(f32)` for shift/scale, about 12 KiB at 1536 dimensions.
- TQ+ with renorm: same packed bytes plus one extra `f32` per vector if persisted. The probe did not justify this scalar on normalized/IP data.
- Byte-pair query LUT: no storage change, but query memory grows from `dim * 16 * sizeof(f32)` to `(dim / 2) * 256 * sizeof(f32)`: about 96 KiB to 768 KiB at 1536 dimensions.

## Probe Results

### SPIRE LUT Scoring

Packet: `reviews/task-86/008-spire-real-spread`

This is the only production-backed performance improvement from Task 86 so far. SPIRE assignment scoring now prepares and uses the existing no-QJL 4-bit TQ LUT scorer when eligible. The benchmark used SPIRE with TurboQuant storage on real10k/50k/100k:

```text
SQL mean latency, pre-LUT -> LUT:
real10k:  nprobe 8  3.44 -> 3.30 ms, 24 8.02 -> 7.69 ms, 32 10.2 -> 9.74 ms
real50k:  nprobe 16 12.3 -> 12.0 ms, 48 33.9 -> 32.9 ms, 64 48.0 -> 46.1 ms
real100k: nprobe 32 25.3 -> 24.5 ms, 96 74.1 -> 71.7 ms, 128 95.3 -> 92.3 ms
```

Storage stayed identical: 8.2 MiB / 857.7 B per row at 10k, 39.8 MiB / 834.1 B per row at 50k, and 79.5 MiB / 833.9 B per row at 100k. Recall was identical at every sweep point.

### TQ+ Calibration

Packet: `reviews/task-86/002-tqplus-prototype`

Focused deterministic 1536-dimensional anisotropic probe:

```text
baseline_mae=0.02642813 tqplus_mae=0.00344425 baseline_rmse=0.03058423 tqplus_rmse=0.00453193 mae_delta_pct=-86.97 rmse_delta_pct=-85.18
```

Interpretation: TQ+ calibration is a quality candidate, not a benchmark-backed production improvement yet. It keeps packed code scanning and materially reduces error in the synthetic probe, but still needs real-corpus recall/latency/storage evidence.

### Renorm Isolation

Packet: `reviews/task-86/003-calibration-renorm-isolation`

```text
baseline_mae=0.02642813 tqplus_unrenorm_mae=0.00311462 tqplus_mae=0.00344425 baseline_rmse=0.03058423 tqplus_unrenorm_rmse=0.00400303 tqplus_rmse=0.00453193
```

Interpretation: calibration-only beat calibration-plus-renorm in this normalized/IP probe. The lower-storage option should be investigated first.

### Byte LUT Kernel

Packet: `reviews/task-86/004-byte-lut-kernel`

```text
direct_ns_per_score=9356.24 dim_lut_ns_per_score=4448.95 byte_lut_ns_per_score=5458.79 byte_lut_speedup_vs_direct=1.714 byte_lut_speedup_vs_dim_lut=0.815
```

Interpretation: byte-pair LUTs are not the first kernel to productionize. They are correct and faster than direct codebook multiply, but slower than our existing dim-LUT scorer while using substantially more query memory.

## SIMD/Kernels

Current state from inspected code:

- QJL-active TQ paths have AVX2/FMA and NEON checked paths.
- No-QJL 4-bit TQ uses scalar packed-byte scans.
- The direct no-QJL scorer does two codebook loads and multiplies per byte.
- The existing dim-LUT scorer does two LUT loads per byte and is materially faster in the probe.
- The byte-pair LUT uses one lookup per byte but loses to dim-LUT locally, likely due to larger query-side memory footprint.

Best next kernel work is not byte LUT. Better candidates are:

- Make the dim-LUT path default everywhere eligible, then benchmark index-level impact.
- Explore fused scan/top-k or score batching where index code can keep candidates in tight loops.
- Consider no-QJL 4-bit SIMD only after release-profile profiling shows scalar dim-LUT scoring remains the bottleneck; gather-heavy SIMD is not obviously favorable on Apple NEON.

## Index Fit

- HNSW: already has TurboQuant exact-score modes for exact, full LUT, tiled LUT, and int8 approximate. TQ+ would need metadata/page support plus scan prepared-query support.
- IVF: already auto-prepares no-QJL 4-bit TQ LUT queries. TQ+ would need IVF metadata to persist calibration and encode tuples in calibrated space.
- SPIRE: now uses the existing TQ LUT path when eligible. TQ+ would need leaf/assignment payload format versioning or a new quantizer profile that carries calibration metadata.
- DiskANN: the inspected `ec_diskann` build codec currently exposes grouped-PQ and RaBitQ search codecs, not the same direct TurboQuant search-code hook seen in HNSW/IVF/SPIRE. If a separate DiskANN TQ path exists, it should be mapped before a TQ+ design is claimed cross-index.

## Recommended Next Tasks

1. Promote or rerun the packet 008 SPIRE TurboQuant benchmark as the current local lane.
   The measured improvement is modest but real, recall-neutral, and storage-neutral. A second host-class run should decide whether it is lane-promotable.

2. Prototype calibration-only TQ+ as an explicit quantizer profile.
   Persist index-level shift/scale, keep per-vector packed bytes unchanged, and compare recall/latency/storage against our current TQ in one index first.

3. Extend TQ+ profile across indexes only after one index proves out.
   Start with IVF or SPIRE because their TQ scoring adapter surfaces are narrower than HNSW. HNSW can follow once metadata and scan prepared-query contracts are settled.

4. Deprioritize byte-pair LUTs.
   Revisit only if a cache-blocked release-profile benchmark contradicts the focused probe.

5. Defer per-vector renorm.
   Keep it as a separate experiment for non-normalized or norm-sensitive lanes, but do not add 4 bytes/vector to the first TQ+ storage design.
