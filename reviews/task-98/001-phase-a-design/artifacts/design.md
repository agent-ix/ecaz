# Task 98 Phase A Design: TiledLut + Int8Approx Block Kernels

## Surfaces (verified against source)

- Mode selection: `resolve_turboquant_exact_score_mode` (`ec_hnsw/scan.rs`).
  Was env-var-only — the new `ec_hnsw.turboquant_exact_score_mode` GUC
  (commit `ed0fe1a23`) makes per-session bench cells possible; env var kept
  as fallback at GUC default. This resolves the Task 87 packet 020/022-024
  zero-counter instrumentation gap before any kernel lands.
- `TiledLut`: `score_ip_from_split_parts_tiled_lut_no_qjl_4bit`
  (`prod.rs:636`) — per-dim 16-entry f32 LUT sums, iterated in tiles of
  `prepared.tile_size` dims for cache locality. Same algebra as FullLut.
- `Int8Approx`: `score_ip_from_split_parts_int8_approx_no_qjl_4bit`
  (`prod.rs:571`) — `sum_i32(codebook_i8[nibble] * rotated_i8[dim])`
  scaled once by `score_scale`. Pure integer accumulation.

## Parity contracts

- `int8_approx32`: i32 sums are order-independent — **integer-exact strict
  equality on every backend** (hamming32 contract, no tolerance framing).
  The single f32 multiply at the end is exact for |sum| < 2^24 · scale
  granularity; the final score compares with `to_bits()` equality because
  both paths perform the identical single conversion.
- `tiled_lut32`: f32 sums — forced-scalar strict anchor mirroring the
  production op order (tile-outer, dim-inner), SIMD backends under the
  established envelope, recall byte-equal binding (Task 93 packet 003
  contract).

## Kernel shapes

- `src/quant/tiled_lut32/{mod,scalar,neon,sve,avx2}.rs`: block32 + partial
  entry points per the partial-width convention (HNSW frontier batches are
  structurally sub-32; Task 93 packet 004 finding). Scalar first; NEON
  reuses the FullLut gather strategy per tile. SVE routes to NEON; AVX2
  placeholder pending Intel lane (same policies as rabitq32/hamming32).
- `src/quant/int8_approx32/{mod,scalar,neon,sve,avx2}.rs`: scalar nibble
  loop; NEON via 16-lane i8 multiplies with widening accumulation (SDOT
  where available later — first slice uses vmull_s8/vpadal for exactness
  clarity). Same dispatch policies.

## Routing

HNSW `CandidateScoreDispatch::Exact` arms for `TiledLut` / `Int8Approx`
accumulate like the FullLut batch (same `ec_hnsw.candidate_batch_scoring`
gate) and flush through new wrappers
`score_turboquant_tiled_lut_batch_for` / `score_turboquant_int8_batch_for`
under `QuantCodecKind::TurboQuant` (mode disclosed in the packet, not the
counter key — counter-kind explosion deferred to Task 99's taxonomy).
Wrappers record `record_flush_width` (infra commit `fb7083c78`), giving
acceptance criterion 4's histogram directly from bench cells.

## Phase A decision data

real10k/50k/100k HNSW cells per mode via the GUC; the width histogram's
`width_ge32` share decides Phase C (SVE cloud) per the task's <20% stop
condition. Note: with partial-width dispatch the kernels still help sub-32
batches, so the stop condition gates only the *cloud measurement spend*,
not the kernels themselves.
