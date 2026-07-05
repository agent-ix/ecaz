# Task 102 Packet 001 Artifact Manifest

- head SHA (code under review): `fc8db79af` (`Repack lut32 AVX2 kernel via byte transpose; add octet-granular tails`), on top of `3915c11a3` (`Add lut32 SIMD block kernels for AVX2, NEON, and SVE`)
- baseline SHA (comparison anchor): `50a86029c` (pre-Task-102 tree; lut32 backends are scalar-delegating stubs)
- task bucket: `reviews/task-102/`
- packet path: `reviews/task-102/001-lut32-simd-block-kernels/`
- timestamp: 2026-06-10
- lane: local PG18 / Intel AVX2 / TurboQuant no-QJL 4-bit, dim 1536
- fixtures / storage: real DBpedia 10k, `storage_format=turboquant`
  - HNSW: `task87_phase6_real10k_hnsw` (m=16, ef_construction=128), exact-mode `full_lut`, binary prefilter disabled
  - SPIRE: `task87_phase6_real10k_spire`
- surface isolation: existing one-index-per-table Task 87 phase-6 fixtures; no fixtures were created, dropped, or reloaded
- rerank mode: fixture defaults; sweeps are `ef_search=80,160` (HNSW) and `nprobe=32,64` (SPIRE)
- AWS / Graviton: not run; NEON/SVE2 kernels are local-compile-gated and deferred to the G4 evidence pass

## Method: two release-backend runs of one suite shape

The same 8-step suite (recall on/off + latency on/off, both AMs) ran twice,
once per head, each on a freshly installed, SHA-asserted release backend
with a PG restart and an `ecaz_build_profile()` probe in between. The
suite preflight recorded the backend into each suite manifest. No cargo
test or pg_test ran between install and bench in either pass (see the
debug-backend root cause in `reviews/task-94/027-latency-width-rerun/feedback/`).

### Baseline pass (stub kernels, scalar block path)

- checkout: `git checkout 50a86029c` (detached)
- `install-baseline-50a86029c.log`: `ecaz dev install ecaz-pg-test --pg 18`, backend SHA `dc9b8141751dd3db0d58a10e1bd4d9681e03cf58dabac439305387f1f1cb6646` (byte-identical to the Task 94 packet 028 backend — reproducible build of the same tree)
- `restart-pg18-baseline.log`: `ecaz dev scratch restart --pg 18`
- `baseline-build-profile-probe.log`: `SELECT ecaz_build_profile()` → `release`
- `baseline-suite-audit.log`: audit passed: 8 steps
- run: `bench suite run --config artifacts/task102-lut32-baseline-suite.json` (full command in `baseline-suite-run.log`)
- `baseline-suite-status.log`: `completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- `baseline-suite-manifest.json` preflight: `backend.build_profile=release`, SHA `dc9b8141…`

### Current pass (Task 102 kernels)

- checkout: branch head `fc8db79af`
- `install-current.log`: backend SHA `da57183e00ff3c4a404a3eb67b7b5e7fbb7b172e116fa7221cd040c6a6cea961`
- `restart-pg18-current.log`, `current-build-profile-probe.log`: `release`
- `current-suite-audit.log`: audit passed: 8 steps
- run: `bench suite run --config artifacts/task102-lut32-current-suite.json` (full command in `current-suite-run.log`)
- `current-suite-status.log`: `completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- `current-suite-manifest.json` preflight: `backend.build_profile=release`, SHA `da57183e…`

Both suite configs are checked in (`task102-lut32-baseline-suite.json`,
`task102-lut32-current-suite.json`); they differ only in step/label/log
naming. `truth-cache/` is shared between passes (ground truth is
head-independent).

## Key result lines

All rows below are from `baseline-results.jsonl` / `current-results.jsonl`
(`block_kernel_counters`, `latency`, and `recall` metrics).

### Recall — byte-equal at every cell

| Cell | baseline | current |
| --- | --- | --- |
| HNSW real10k full_lut kernel-on, ef=80 | recall@k 0.6240 / ndcg 0.9321 | identical |
| HNSW real10k full_lut kernel-off, ef=80 | recall@k 0.6240 / ndcg 0.9321 | identical |
| SPIRE real10k kernel-on, nprobe=32 | recall@k 1.0000 / ndcg 1.0000 | identical |
| SPIRE real10k kernel-off, nprobe=32 | recall@k 1.0000 / ndcg 1.0000 | identical |

### Kernel rate ladder (per-candidate, `quant=turboquant` counter rows)

| Lane | baseline scalar-block | current AVX2 | ratio |
| --- | ---: | ---: | ---: |
| SPIRE full blocks (w≥32 = 9,600 flushes, 3.0M candidates) | 1,054–1,062 ns/c | 235–237 ns/c | **4.5×** |
| Same-head unbatched scalar anchor (kernel-off rows) | — | 1,309–1,318 ns/c | 5.6× vs unbatched |
| HNSW exact-mode multi-lane flushes (octet tails + blocks) | — | 509–530 ns/live-candidate | 2.8× vs same-head scalar |

### End-to-end latency p50

| Cell | sweep | baseline | current | delta |
| --- | --- | ---: | ---: | ---: |
| HNSW full_lut kernel-on | ef=80 | 16.5 ms | 4.65 ms | **−71.8%** |
| HNSW full_lut kernel-on | ef=160 | 27.1 ms | 6.91 ms | **−74.5%** |
| HNSW full_lut kernel-off | ef=80 | 4.96 ms | 5.22 ms | +5.2% |
| HNSW full_lut kernel-off | ef=160 | 7.37 ms | 7.75 ms | +5.2% |
| SPIRE kernel-on | nprobe=32 | 17.3 ms | 8.54 ms | **−50.6%** |
| SPIRE kernel-on | nprobe=64 | 17.2 ms | 8.57 ms | **−50.2%** |
| SPIRE kernel-off | nprobe=32 | 19.5 ms | 19.5 ms | +0.0% |
| SPIRE kernel-off | nprobe=64 | 19.4 ms | 19.0 ms | −2.1% |

Kernel-off cells run unchanged code; their small deltas are run noise /
code-layout shift (the same `.so` carries the new kernel text either way).
HNSW kernel-on now beats kernel-off at both sweeps (4.65 vs 5.22 ms;
6.91 vs 7.75 ms) — at baseline it lost by 3.3×.

### Width histogram and attribution (current head, HNSW ef=80)

- `isa=avx2` rows: 206,262 kernel candidates, `w<8=42,921 w8-15=4,347 w16-31=1,284 w≥32=87`
- `isa=scalar` rows: 62,877 single-lane flushes through the scalar fast path (`w<8=62,877`)
- SPIRE: all 9,600 flushes `w≥32`, `scalar_candidates=300` (one single-lane tail per iteration)

## Interim v1 measurement (superseded, recorded for the record)

The first kernel shape (commit `3915c11a3`, per-dim scalar nibble
extraction into a stack array) measured **1,371 ns/candidate on the SPIRE
full-block lane — 0.77× of the scalar block kernel** — bottlenecked on
store-to-load forwarding between the scalar index writes and the vector
loads. That measurement motivated the `fc8db79af` shuffle-repack rewrite;
its raw artifacts were superseded in place by the current pass.
