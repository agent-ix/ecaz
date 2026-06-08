# Task 86 Requirements Re-Audit

This audit re-checks the original Task 86 request after the real SPIRE TurboQuant benchmark packet.

## 1. Query Encoding / Comparison Path

TurboVec source finding: their TurboQuant variant normalizes vectors, rotates into a calibrated per-coordinate space, applies learned shift/scale, stores packed 4-bit codes, and prepares queries as a calibrated LUT plus bias. That is a real same-space query formulation, but it is not a new index type and not decompression-free in the sense of avoiding code decoding entirely. It still scans packed codes and decodes nibbles or bytes into score lookups.

Our measured implementation change: packet 008 does not adopt TQ+ calibration. It measures SPIRE using our existing no-QJL 4-bit TurboQuant dim-LUT scorer. This avoids full vector reconstruction at query time and improves scoring, but it is not the TurboVec calibrated encoding.

Status: covered. Calibration-only TQ+ remains a follow-up candidate, not an accepted improvement.

## 2. Vector Size

Current no-QJL 4-bit TurboQuant payload formula is `4 + dim / 2` bytes: 4 bytes of gamma plus packed 4-bit MSE codes. In the no-QJL 4-bit exact score, gamma is ignored by construction, but it is still present in the current payload contract.

| Dim | Current TQ payload | TQ+ calibration-only payload | TQ+ index metadata |
| ---: | ---: | ---: | ---: |
| 768 | 388 B/vector | 388 B/vector | 6 KiB shift/scale |
| 1536 | 772 B/vector | 772 B/vector | 12 KiB shift/scale |
| 3072 | 1540 B/vector | 1540 B/vector | 24 KiB shift/scale |

Optional per-vector renorm would add 4 B/vector. Packet 003 did not justify that cost for normalized/IP data.

Measured packet 008 storage: SPIRE LUT scoring changed no storage. The SPIRE index stayed 8.2 MiB at 10k, 39.8 MiB at 50k, and 79.5 MiB at 100k.

Status: covered with both formula-level and benchmark-level evidence.

## 3. SIMD / Kernel Comparison

Current code state:

- QJL-active TQ has checked AVX2/FMA and NEON paths.
- No-QJL 4-bit TQ is scalar packed-code scoring.
- The existing dim-LUT path is faster than direct codebook multiply in the focused probe.
- Byte-pair LUT lost to dim-LUT in packet 004 while using much more query-side memory.
- Packet 008 proves the dim-LUT path is useful at SPIRE index level.

Practical conclusion: make the existing dim-LUT path the default everywhere eligible before chasing a byte-pair LUT or gather-heavy SIMD. A no-QJL SIMD kernel should be driven by release-profile profiling after the scalar dim-LUT path is universally used.

Status: covered. SIMD remains an optimization candidate, not yet benchmark-backed as a production change.

## 4. Index Type

TurboVec is a flat compressed-code scan with optional ID mapping. It is not HNSW, DiskANN, IVF, or SPIRE.

Mesh with our indexes:

- HNSW: already has several TurboQuant exact-score modes, including LUT variants.
- IVF: already prepares no-QJL 4-bit TQ LUT queries.
- SPIRE: packet 008 validates the missing no-QJL 4-bit LUT use at index level.
- DiskANN: inspected adapters did not expose the same direct TurboQuant search-code path; map this before claiming TQ+ cross-index support.

Status: covered. TurboVec's source ideas are quantizer/scoring ideas, not index-structure ideas.

## Final Reconsideration

The task now has one measured production improvement: SPIRE TurboQuant LUT scoring. It is recall-neutral, storage-neutral, and consistently faster across 10k/50k/100k.

The remaining options are properly scoped as future investigations:

1. Calibration-only TQ+ profile with persisted shift/scale metadata.
2. Cross-index metadata plumbing after one index proves TQ+ recall/latency/storage.
3. No-QJL 4-bit SIMD only after profiling shows scalar dim-LUT scoring remains the bottleneck.
4. Byte-pair LUT only if a cache-blocked release-profile benchmark reverses packet 004.
