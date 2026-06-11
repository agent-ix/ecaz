# Task 103 Packet 001 Artifact Manifest

- head SHA: `0fb22f3f1` (task files only since `2f99971c5`; extension binary
  unchanged from the Task 102 packet 002 install)
- task bucket: `reviews/task-103/`
- packet path: `reviews/task-103/001-intel-baseline-matrix/`
- timestamp: 2026-06-10
- lane: local PG18 / Intel AVX2
- fixtures: `task87_phase6_real10k_hnsw` (real 10k, `storage_format=turboquant`,
  exact-mode GUC sweep) and `task67_local_fullq_50k_diskann` (real 50k,
  default binary-sidecar prefilter)
- surface isolation: existing one-index-per-table fixtures; nothing created
- backend provenance: `build-profile-probe.log` → `release`; suite preflight in
  `suite-manifest.json`: `build_profile=release`,
  SHA `3df91dd8733bb4b4d1fbbf8b08b0cab36b3dc6f1e6d5e57c919879cd280f80eb`
  (same install as Task 102 packet 002, `install-ecaz-pg18.log` there; no
  cargo test ran between install and these suites)

## Suites

- `task103-intel-baseline-matrix-suite.json` (9 steps): HNSW exact-mode
  recall+latency batch-on for `full_lut` / `tiled_lut` / `int8_approx` at
  `ef_search=80,160` with counters, plus `int8_approx` batch-off (scalar
  anchor), plus DiskANN binary-sidecar latency batch-on/off.
  `suite-audit.log` (passed: 9), `suite-run.log`, `suite-status.log`
  (`completed=9 failed=0`), `results.jsonl`, `results-report.jsonl`.
- `task103-binary-warm-recheck-suite.json` (2 steps): the DiskANN binary
  cells re-run warm, **off before on**, after the first run's kernel-on
  `list_size=64` cell absorbed the 50k fixture's cold-cache warmup
  (p50 11.4 ms, p95 21.2 ms — ordering artifact, not a batching cost).
  `warm-suite-run.log`, `warm-suite-manifest.json`, `warm-results.jsonl`.

## Key result lines

### tiled_lut vs full_lut (AC2 disposition input)

| Mode (batch-on) | ef=80 p50 | ef=160 p50 | per-candidate | recall@k / ndcg |
| --- | ---: | ---: | ---: | --- |
| full_lut (AVX2 kernel) | 4.52 ms | 6.76 ms | 492–546 ns/c multi-lane + 1,389 ns/c single-lane scalar | 0.6240 / 0.9321 |
| tiled_lut (scalar walk) | 6.66 ms | 10.00 ms | 2,994–3,001 ns/c | 0.6240 / 0.9321 |

tiled_lut is 47–48% slower end-to-end at byte-identical recall on the
canonical 1536-dim lane.

### int8_approx (AC1 baseline)

| Cell | ef=80 p50 | ef=160 p50 | per-candidate |
| --- | ---: | ---: | ---: |
| batch-on (scalar kernel only) | 4.52 ms | 6.62 ms | 918.7–923.0 ns/c (`quant=turboquant_int8 isa=scalar`) |
| batch-off | 4.19 ms | 6.13 ms | — |

recall@k 0.6230 / ndcg 0.9319 (approximate mode; its parity contract is
integer-exact against its own scalar reference, not byte-equal to full_lut).
Batch-on currently loses to batch-off for lack of a kernel; the scalar rate
already beats lut32's scalar rate (919 vs 1,389 ns/c), so an AVX2 kernel
makes this the presumptive fastest exact mode.

### hamming32 / binary sidecar (AC3 decision input)

- Scalar hardware-POPCNT rate: **11.5–11.8 ns/candidate**
  (`quant=binary isa=scalar`, 154k–250k candidates per cell).
- Warm-order end-to-end: batch-on 3.89 / 4.62 ms vs batch-off 4.00 / 4.51 ms
  (−2.8% / +2.4% — within noise).
- First-run kernel-on `list_size=64` cell (11.4 ms) is a cold-cache ordering
  artifact; superseded by the warm recheck.
