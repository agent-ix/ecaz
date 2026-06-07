# TurboVec TurboQuant Analysis

Scope: compare TurboVec's TurboQuant implementation to our TurboQuant
implementation only. This report intentionally does not evaluate other
quantizers or external leaderboards.

TurboVec source inspected: `/Users/peter/dev_bak/turbovec` at
`efe29a184986cbf562a9847c2ac52a2990bfaca2`.

Our source inspected: `/Users/peter/dev/tqvector` at
`39d72f3fdc5bd11114e8cc5269e7b62584f670a5`.

## Executive Answer

TurboVec does use an index type, but it is a flat positional compressed-vector
index. `TurboQuantIndex` stores packed codes, one per-vector scale, optional
TQ+ calibration arrays, and lazy search caches. `IdMapIndex` adds stable
external IDs over the same positional index.

It is not a graph, disk-neighbor, partitioned, or recursive index. Search scans
the compressed code blocks directly, with optional bitmask filtering that skips
entire 32-vector blocks when no slot is allowed.

The README phrase about searching in the same space is accurate in the
asymmetric-scoring sense: database vectors are normalized, rotated, calibrated,
quantized, and packed; queries are rotated and inverse-calibrated, then used to
build lookup tables. The query is not packed into the same database code format.

## TurboVec Database Encoding

Relevant source:

- `/Users/peter/dev_bak/turbovec/turbovec/src/lib.rs:96` defines
  `TurboQuantIndex` with `packed_codes`, `scales`, TQ+ shift/scale arrays, and
  lazy blocked-code caches.
- `/Users/peter/dev_bak/turbovec/turbovec/src/lib.rs:250` through `:283`
  initializes rotation/codebook state and calls `encode::encode`.
- `/Users/peter/dev_bak/turbovec/turbovec/src/encode.rs:1` through `:27`
  documents normalize, rotate, TQ+ calibrate, quantize, pack, and scale.
- `/Users/peter/dev_bak/turbovec/turbovec/src/encode.rs:65` through `:133`
  implements normalize, dense rotation, calibration, packed-code allocation,
  and per-row fused quantize/scale/pack.
- `/Users/peter/dev_bak/turbovec/turbovec/src/encode.rs:245` through `:352`
  explains and implements the per-vector renormalization scalar.

Encoding shape:

1. Normalize each source vector and keep the norm.
2. Rotate normalized vectors with a deterministic dense orthogonal matrix.
3. Fit or reuse per-coordinate TQ+ calibration:
   `u_calibrated[d] = (u_rot[d] + shift[d]) * scale_tq[d]`.
4. Quantize calibrated coordinates with a Lloyd-Max scalar codebook.
5. Pack coordinate codes into bit planes.
6. Store one per-vector scalar:
   `norm / dot(rotated_unit, reconstructed_centroid_in_original_calibrated_space)`.

The calibration is frozen after the first non-empty add. Subsequent adds reuse
the same calibration, so all vectors live in one calibrated coordinate system.

## TurboVec Query Preparation

Relevant source:

- `/Users/peter/dev_bak/turbovec/turbovec/src/search.rs:1466` through `:1530`
  implements full query search setup.
- `/Users/peter/dev_bak/turbovec/turbovec/src/search.rs:1492` through `:1508`
  rotates all queries with a batched GEMM.
- `/Users/peter/dev_bak/turbovec/turbovec/src/search.rs:1510` through `:1518`
  applies inverse TQ+ calibration.
- `/Users/peter/dev_bak/turbovec/turbovec/src/search.rs:1419` through `:1455`
  defines `q_calib[d] = q_rot[d] / scale_tq[d]` and bias
  `-sum(q_rot[d] * shift[d])`.
- `/Users/peter/dev_bak/turbovec/turbovec/src/search.rs:1165` through `:1280`
  builds per-query `u8` nibble lookup tables with per-subtable mins, one shared
  scale, and a bias.

This is not symmetric code-to-code scoring. The database side is packed integer
codes. The query side is transformed floating-point values collapsed into
per-query LUTs. That is still useful because database vectors are not
decompressed into dense float vectors at query time.

## TurboVec Search And Index Shape

Relevant source:

- `/Users/peter/dev_bak/turbovec/turbovec/src/lib.rs:357` through `:475`
  exposes `search` and `search_with_mask`, materializes a blocked-code cache,
  packs optional filters into a slot bitset, and calls `search::search`.
- `/Users/peter/dev_bak/turbovec/turbovec/src/pack.rs:1` through `:60`
  repacks per-vector bit-plane codes into SIMD-blocked layout.
- `/Users/peter/dev_bak/turbovec/turbovec/src/pack.rs:62` through `:88`
  uses an x86 FAISS-style 32-vector block layout with split hi/lo nibbles.
- `/Users/peter/dev_bak/turbovec/turbovec/src/pack.rs:90` through `:114`
  uses a sequential 32-vector block layout on non-x86.
- `/Users/peter/dev_bak/turbovec/turbovec/src/search.rs:1293` through `:1335`
  implements block and block-pair filter skips.
- `/Users/peter/dev_bak/turbovec/turbovec/src/search.rs:1337` through `:1417`
  shows scalar scoring over all blocks and lanes with fused heap maintenance.
- `/Users/peter/dev_bak/turbovec/turbovec/src/search.rs:1532` onward dispatches
  platform-specific scoring and top-k.

There is no routing layer in the inspected implementation:

- no neighbor graph;
- no posting lists;
- no coarse centroid assignment;
- no DiskANN/Vamana beam expansion;
- no recursive or partition-object routing.

The core search complexity is therefore flat scan over all compressed vector
slots, minus optional allowlist/mask block skips.

Transfer consequence: kernel ideas can transfer, but end-to-end index claims
must be remeasured inside our AMs because our bottleneck often includes
candidate generation, graph traversal, page layout, heap rerank, or tuple I/O.

## Stored Bytes

TurboVec file layout source:

- `/Users/peter/dev_bak/turbovec/turbovec/src/io.rs:178` through `:213` writes
  bit width, dimension, vector count, packed codes, per-vector scales, and TQ+
  calibration.
- `/Users/peter/dev_bak/turbovec/turbovec/src/io.rs:262` through `:276` reads
  `packed_bytes = (dim / 8) * bit_width * n_vectors` and `n_vectors` f32
  scales.

TurboVec per-vector storage:

```text
packed code bytes = dim * bits / 8
scale bytes       = 4
per-vector total  = dim * bits / 8 + 4
```

TurboVec also stores index-level TQ+ metadata:

```text
n_calib bytes       = 4
shift bytes         = 4 * dim
scale_tq bytes      = 4 * dim
index-level total   = 4 + 8 * dim
```

For `dim=1536,bits=4`, TurboVec stores `768 + 4 = 772` bytes per vector, plus
`12,292` bytes of index-level calibration. For `dim=1536,bits=2`, it stores
`384 + 4 = 388` bytes per vector, plus the same calibration overhead.

Our byte layout source:

- `/Users/peter/dev/tqvector/src/quant/prod.rs:14` through `:19` defines
  `EncodedTq { gamma, mse_packed, qjl_packed }`.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:1466` through `:1497` defines
  when QJL is enabled, MSE bits, MSE byte length, and QJL byte length.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:1563` through `:1565` defines
  payload length as `4 + mse_code_len + qjl_code_len_for_bits`.
- `/Users/peter/dev/tqvector/src/quant/rotation.rs:16` through `:25` shows
  the 1536-dim tiled-FWHT compatibility path that disables QJL for `bits=4`.

Our per-vector payload:

```text
payload bytes = 4 + mse_code_len(dim,bits) + qjl_code_len_for_bits(dim,bits)
```

When QJL is disabled, `mse_bits == bits`, so our hot payload matches
TurboVec's per-vector byte count: `dim * bits / 8 + 4`.

When QJL is enabled, `mse_bits == bits - 1` and QJL adds one bit per coordinate.
That keeps the same nominal bit budget for the code bytes, but spends one bit
on the QJL sidecar instead of full scalar MSE. The important investigation is
not nominal bytes; it is whether TurboVec's full scalar TQ plus calibration and
renormalization beats our MSE/QJL split at the same payload size.

## SIMD And Kernel Differences

TurboVec:

- Packs candidates into 32-vector blocks before search.
- Builds per-query `u8` nibble LUTs with local subtable mins, a shared scale,
  and a bias.
- Scores compressed database codes by LUT lookup and integer accumulation, then
  converts through the LUT scale and per-vector scale.
- Has block-level filter skipping.
- Has 4-query fused scoring paths that reuse code loads across query batches.
- Keeps top-k maintenance fused with scan for scalar/fallback paths and partly
  fused in SIMD paths.

Our current TurboQuant:

- Uses SRHT/FWHT-style rotation rather than TurboVec's dense rotation.
- Prepares queries with rotated floats, optional f32 LUTs, optional QJL
  projection, and a QJL scale.
- For the 1536-dim 4-bit no-QJL lane, disables the default f32 LUT in the exact
  path and scores directly from packed MSE codes.
- Includes explicit no-QJL 4-bit f32 LUT, tiled f32 LUT, int8 approximate query,
  and binary-sign query prep surfaces for experiments.
- SIMD kernels primarily score one candidate payload at a time in the shared
  quantizer API, rather than scanning 32-vector blocked slabs with fused top-k.

Relevant source:

- `/Users/peter/dev/tqvector/src/quant/prod.rs:196` through `:230` prepares
  the current query state.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:493` through `:523` dispatches
  scoring.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:597` through `:655` implements
  explicit and tiled f32 LUT no-QJL 4-bit scoring.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:760` through `:967` shows
  code-to-code SIMD for 3-bit MSE codes.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:1021` onward shows AVX2/NEON
  query-to-code paths when QJL is enabled.
- `/Users/peter/dev/tqvector/src/quant/simd.rs:1` through `:213` shows runtime
  SIMD dispatch.

The largest structural difference is batching. TurboVec's kernels own the flat
scan loop and can arrange memory as blocked slabs. Our shared quantizer scorer
is currently shaped around scoring one encoded payload at a time, which is
portable across AMs but leaves less room for block-level reuse.

## Options To Investigate

1. TQ+ calibration for our TurboQuant.
   - Store or derive per-coordinate shift/scale metadata.
   - Apply inverse calibration in query preparation and fold bias into scoring.
   - First benchmark in a quantizer microbench or IVF-style contiguous lane.

2. Full scalar TQ at the same payload bytes.
   - Compare our active MSE/QJL split against full scalar bits plus TQ+
     calibration and per-vector renormalization.
   - This is especially important outside the 1536-dim 4-bit no-QJL lane.

3. Renormalization scalar semantics.
   - TurboVec stores `norm / dot(rotated_unit, reconstructed_centroid)`.
   - Our `gamma` is residual magnitude for the QJL correction, not the same
     estimator-bias correction.
   - Test whether TurboVec-style renormalization helps no-QJL or full-scalar
     lanes without extra per-vector bytes.

4. `u8` nibble LUT scoring.
   - Convert our f32 LUT rows to TurboVec-style per-subtable-min `u8` LUTs with
     shared scale/bias.
   - Compare against existing f32 LUT, tiled f32 LUT, direct no-QJL scoring,
     and int8 approximate query prep.

5. 32-vector blocked code slabs.
   - Add a benchmark-only blocked scoring path for contiguous candidates before
     touching graph AM traversal.
   - IVF or a quantizer microbench is the right first surface.

6. Fused scoring plus top-k.
   - Useful for flat or posting-list scans.
   - Less directly useful for graph AMs unless the graph scan can hand over
     candidate batches without broad API churn.

7. Four-query fused scoring.
   - Potentially useful for batch query workloads and benchmark lanes.
   - Not likely to help single-query graph traversal.

8. Dense rotation is a lower-priority candidate.
   - TurboVec's dense rotation makes the statistical calibration story simple
     but is expensive relative to our SRHT/FWHT path.
   - Treat it as a quality diagnostic, not the first production path.

## Recommended First Prototype

Start with TQ+ calibration plus query-side inverse calibration in a benchmark
or internal switch.

Reasoning:

- It directly targets quality at unchanged per-vector code bytes.
- It does not require adopting TurboVec's flat index shape.
- It composes with our current query preparation model.
- It gives a clean answer on whether TurboVec's claimed quality path improves
  our TurboQuant before we touch block layout or AM scan APIs.

Second prototype should be `u8` nibble LUT scoring for a contiguous-candidate
lane, because that is the most plausible query-time speed transfer from
TurboVec's flat scanner.
