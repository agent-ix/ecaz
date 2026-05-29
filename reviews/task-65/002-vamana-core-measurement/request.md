# Task 65 Measurement: Vamana Core Build Performance

## Summary

This packet records the small-corpus validation after the DiskANN Vamana build
overhaul and the follow-up visibility-safe loader fix.

Code checkpoints:

- `351987249` - `Optimize DiskANN Vamana build core`
- `da2807c0e` - `Bound DiskANN greedy frontier with heaps`
- `4bd460081` - `Fix DiskANN build validation edges`
- `a8b0b8789` - `Fix DiskANN build visibility handling`
- `de2ef72e4` - `Trim DiskANN Vamana build hot path`
- `8e860324c` - `Add Vamana build dhat profiler`

## Results

Performance gate:

- release real10k `pq_fastscan`, `graph_degree=32`, `build_list_size=100`,
  `alpha=1.2`: build `7.62s`
- release real10k `pq_fastscan`, `graph_degree=32`, `build_list_size=200`,
  `alpha=1.2`: build `14.92s`
- full fresh fixed-loader prefix completion: `24.95s`
- Task 65 target: index build at or below `16s` on real10k, or explain
  remaining out-of-scope phases
- Task 29c floor: `70.678s`

The fixed-loader total is higher than the older update-in-place loader total
because the loader now stages corpus rows and inserts encoded `ecvector NOT
NULL` rows directly. That is deliberate: it avoids dead pre-update NULL heap
tuples while preserving PostgreSQL's index-build visibility contract.

Functional gate:

- `cargo check -p ecaz --lib --no-default-features --features pg18`: passed
- `cargo check -p ecaz-cli`: passed
- `cargo check -p ecaz-cli --bin ecaz`: passed
- `cargo fmt --check`: passed
- `cargo test -p ecaz --features pg18 ec_diskann`: passed; `182 passed; 0 failed`
- macOS dyld status: fixed for the direct Rust/pg_test path; the suite ran
  instead of aborting on `_BufferBlocks`.

Behavioral recall:

| fixture | L values | Task 65 recall@10 | Baseline |
|---|---:|---:|---:|
| real10k fixed-loader R32/L100 | 64 / 128 / 200 | 0.9965 / 0.9970 / 0.9975 | Task 29d: 0.9965 / 0.9965 / 0.9970 |
| real10k fixed-loader R32/L200 | 64 / 128 / 200 | 0.9975 / 0.9975 / 0.9975 | Task 29d: 0.9965 / 0.9965 / 0.9970 |
| synth10k R32/L200 | 64 / 200 / 800 | 0.1610 / 0.2625 / 0.3270 | Task 29 synth smoke: 0.1650 / 0.2665 / 0.3260 |

The real10k recall gate holds. Raising build-time `build_list_size` to `200`
also brings the synthetic L=200 point within the task's 0.5pp behavioral gate
without changing the task spec: `0.2625` vs old `0.2665` is `-0.40pp`.

Memory/code audit:

- Build-time exact-vector heap tuple dedup is removed.
- Runtime insert overflow remains covered by existing insert/overflow tests.
- Build greedy search uses `SearchScratch` with `Vec<u64>` bitsets and bounded
  heaps.
- Build-path greedy search no longer constructs the sorted top-`L` frontier
  vector that only scan/tests need.
- No build hot-loop `vec![false; n]`, linear `min_by` frontier search, or
  repeated `frontier.sort()/truncate()` remains.
- `dhat_vamana_build` profiles the Vamana hot loop on the first 1,000 real10k
  rows at R32/L200. The profile starts after TSV parsing and medoid selection,
  and the attached JSON records only the Vamana graph build call.

Reviewer feedback resolution:

- Packet 001 B1/B2: addressed by `a8b0b8789`. DiskANN ambuild no longer skips
  `tuple_is_alive = false`; the non-chunked loader no longer creates dead NULL
  indexed tuples before `CREATE INDEX`.
- Packet 001 non-blocking Vamana clone/counter nit: addressed by `de2ef72e4`.
- Packet 002 synth10k L=200: resolved by R32/L200 measurement within the
  0.5pp gate.
- Packet 002 memory profiler evidence: resolved by `dhat_vamana_build` 1k
  smoke plus the static hot-loop audit.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/validation-summary.md`
- `artifacts/hot-loop-static-audit.md`
- `artifacts/install-ecaz-pg-test-after-loader-fix.log`
- `artifacts/install-ecaz-pg-test-after-hotpath-trim.log`
- `artifacts/load-real10k-diskann-pq-fastscan-loaderfix-r32-l100-short.log`
- `artifacts/recall-real10k-diskann-pq-fastscan-loaderfix-r32-l100.log`
- `artifacts/load-real10k-diskann-pq-fastscan-loaderfix-r32-l200.log`
- `artifacts/recall-real10k-diskann-pq-fastscan-loaderfix-r32-l200.log`
- `artifacts/load-real10k-diskann-pq-fastscan-release-r32-l100.log`
- `artifacts/recall-real10k-diskann-pq-fastscan-release-r32-l100.log`
- `artifacts/load-synth10k-diskann-pq-fastscan-release-r32-l100.log`
- `artifacts/recall-synth10k-diskann-pq-fastscan-release-r32-l100-l64-200-800.log`
- `artifacts/load-synth10k-diskann-pq-fastscan-release-r32-l200.log`
- `artifacts/recall-synth10k-diskann-pq-fastscan-release-r32-l200-l64-200-800.log`
- `artifacts/dhat-vamana-build-real1k-r32-l200-summary.md`
- `artifacts/dhat-vamana-build-real1k-r32-l200.json`
