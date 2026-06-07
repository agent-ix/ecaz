# TurboVec TurboQuant Analysis

Scope: compare TurboVec's TurboQuant implementation to our TurboQuant
implementation only. This report intentionally does not evaluate other
quantizers or external leaderboards.

TurboVec source inspected: `/Users/peter/dev_bak/turbovec` at
`efe29a184986cbf562a9847c2ac52a2990bfaca2`.

Our source inspected: `/Users/peter/dev/tqvector` at
`71e16fcdced96714e7db1dd98f396cd68941180e`.
Citation refresh for our TurboQuant source: current Task 86 branch head
`d6462c594210e60e15fd9bb6b46f1f82508ee82f`.

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

Representative TurboVec per-vector bytes:

| Dim | 2-bit vector bytes | 4-bit vector bytes | Index-level TQ+ metadata |
| ---: | ---: | ---: | ---: |
| 768 | 196 B | 388 B | 6,148 B |
| 1536 | 388 B | 772 B | 12,292 B |
| 3072 | 772 B | 1,540 B | 24,580 B |

Our byte layout source:

- `/Users/peter/dev/tqvector/src/quant/prod.rs:14` through `:19` defines
  `EncodedTq { gamma, mse_packed, qjl_packed }`.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:1745` through `:1775` defines
  when QJL is enabled, MSE bits, MSE byte length, and QJL byte length.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:1906` through `:1908` defines
  payload length as `4 + mse_code_len + qjl_code_len_for_bits`.
- `/Users/peter/dev/tqvector/src/quant/rotation.rs:9` through `:25` shows
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

Representative current TurboQuant payload bytes:

| Dim | 2-bit payload | 4-bit payload | Notes |
| ---: | ---: | ---: | --- |
| 768 | 196 B | 388 B | QJL-enabled: 1 MSE bit + 1 QJL bit at 2-bit; 3 MSE bits + 1 QJL bit at 4-bit. |
| 1536 | 388 B | 772 B | 4-bit uses tiled-FWHT no-QJL, so all 4 code bits are MSE bits. |
| 3072 | 772 B | 1,540 B | QJL-enabled with the same nominal code byte count as full scalar TQ. |

This table matches TurboVec's nominal per-vector byte count for these dimensions
and bit budgets. The storage difference is semantic, not size-first: TurboVec
spends the full code budget on scalar centroids plus index-level shift/scale;
our QJL-enabled lanes spend one bit per coordinate on residual sign correction.

## Renormalization Semantics

TurboVec's stored scalar is:

```text
s = norm(x) / dot(u_rot, recon_uncalibrated)
```

where `u = x / norm(x)`, `u_rot` is the rotated unit vector, and
`recon_uncalibrated` is the decoded scalar-code reconstruction mapped back out
of the TQ+ calibrated coordinate system.

At query time, after rotating and inverse-calibrating the query, the compressed
inner product estimate is effectively:

```text
score(q, x) ~= s * dot(q_rot, recon_uncalibrated)
```

This scalar forces the encoded vector to have the right self-dot under the
approximation:

```text
score(x, x) ~= norm(x) / dot(u_rot, recon_uncalibrated)
              * dot(norm(x) * u_rot, recon_uncalibrated)
           = norm(x)^2
```

It is therefore a norm/scale correction for the scalar-code reconstruction.
That makes it useful when scalar quantization shrinks or stretches the encoded
direction, but it does not by itself prove an unbiased estimator for arbitrary
queries. It is not the same object as our `gamma`: our `gamma` multiplies the
QJL residual-sign term after MSE scoring, while TurboVec's scalar rescales the
whole scalar-code estimate. Packet 003 tested this distinction in isolation and
did not find a normalized-IP quality win for the renormalization scalar.

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

Current-line equivalents after the Task 86 branch evolved:

- `/Users/peter/dev/tqvector/src/quant/prod.rs:223` through `:257` prepares
  the standard query state.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:346` through `:362` prepares the
  explicit no-QJL 4-bit dimension LUT.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:710` through `:722` dispatches
  explicit no-QJL 4-bit LUT scoring.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:863` through `:890` implements
  the no-QJL 4-bit dimension-LUT hot loop.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:938` through `:1001` implements
  the scalar reference scorer.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:1300` through `:1644` contains
  the AVX2 and NEON checked scoring paths.
- `/Users/peter/dev/tqvector/src/quant/prod.rs:1805` through `:1832` contains
  the benchmark-only byte-pair LUT builder that packet 004 rejected.

The largest structural difference is batching. TurboVec's kernels own the flat
scan loop and can arrange memory as blocked slabs. Our shared quantizer scorer
is currently shaped around scoring one encoded payload at a time, which is
portable across AMs but leaves less room for block-level reuse.

## Measurement Methodology

Prototype packets for this task should use `ecaz bench suite` and record, in
packet-local artifacts, the source commit, storage format, AM, dimensions, bit
budget, payload bytes, sidecar or metadata bytes, recall, p50/p95/p99 latency,
storage bytes, and the scalar/SIMD or LUT variant under test.

The benchmark lane established later in packet 008 is the current production
evidence shape for an accepted Task 86 code slice: isolated PG18 SPIRE
TurboQuant indexes on real10k, real50k, and real100k DBPedia corpora, each with
a low/medium/high probe spread, comparing pre-change and post-change source
commits through `ecaz bench suite`. Query-prep and scorer-only timings remain
future work unless the suite runner is extended to expose them for the AM path;
no packet should infer those timings from SQL latency alone.

## Not Learnable From Analysis Alone

- Flat-scanner LUT and blocked-code economics may not survive page-bounded AM
  scans. HNSW, DiskANN, IVF, and SPIRE all add candidate-generation or page
  traversal costs that can dominate a per-code scorer win.
- TurboVec freezes calibration after the first non-empty add. Streaming inserts
  may drift away from that first-batch coordinate distribution, so TQ+
  calibration needs an insertion-order and retraining policy before any durable
  format change.
- Dense rotation quality cannot be A/B tested against our SRHT/FWHT path from
  source inspection alone. It needs a separate quality probe because it changes
  both statistical shape and query-prep cost.
- Query-prep cache behavior and per-query LUT construction overhead must be
  measured directly. TurboVec's query work is not free just because database
  vectors remain packed.
- DiskANN direct-TurboQuant transfer is not proven by TurboVec's flat index.
  Any graph or disk-resident AM must show that candidate locality and page I/O
  still leave the TurboQuant scorer on the critical path.

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
