# Task 65 Measurement: Vamana Core Build Performance

## Summary

This packet records the small-corpus validation after the DiskANN Vamana build
overhaul and the follow-up visibility-safe loader fix.

Code checkpoints:

- `351987249` - `Optimize DiskANN Vamana build core`
- `da2807c0e` - `Bound DiskANN greedy frontier with heaps`
- `4bd460081` - `Fix DiskANN build validation edges`
- `a8b0b8789` - `Fix DiskANN build visibility handling`

## Results

Performance gate:

- release real10k `pq_fastscan`, `graph_degree=32`, `build_list_size=100`,
  `alpha=1.2`: build `7.62s`
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
| synth10k R32/L100 | 64 / 200 / 800 | 0.1605 / 0.2500 / 0.3315 | Task 29 synth smoke: 0.1650 / 0.2665 / 0.3260 |

The real10k recall gate holds. The synthetic smoke fixture remains in the
known low-recall envelope documented in Task 29, but L=200 remains `1.65pp`
below the old smoke number. This packet records that as an explicit residual
behavioral-waiver item rather than claiming a clean synth gate.

Memory/code audit:

- Build-time exact-vector heap tuple dedup is removed.
- Runtime insert overflow remains covered by existing insert/overflow tests.
- Build greedy search uses `SearchScratch` with `Vec<u64>` bitsets and bounded
  heaps.
- No build hot-loop `vec![false; n]`, linear `min_by` frontier search, or
  repeated `frontier.sort()/truncate()` remains.
- `heaptrack` / standalone `dhat` were unavailable on this host, so the packet
  includes a static hot-loop allocation audit instead of runtime heap output.

Reviewer feedback resolution:

- Packet 001 B1/B2: addressed by `a8b0b8789`. DiskANN ambuild no longer skips
  `tuple_is_alive = false`; the non-chunked loader no longer creates dead NULL
  indexed tuples before `CREATE INDEX`.
- Packet 002 synth10k L=200: still documented as a residual sign-off item.
- Packet 002 memory profiler evidence: still static-only on this host.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/validation-summary.md`
- `artifacts/hot-loop-static-audit.md`
- `artifacts/install-ecaz-pg-test-after-loader-fix.log`
- `artifacts/load-real10k-diskann-pq-fastscan-loaderfix-r32-l100-short.log`
- `artifacts/recall-real10k-diskann-pq-fastscan-loaderfix-r32-l100.log`
- `artifacts/load-real10k-diskann-pq-fastscan-release-r32-l100.log`
- `artifacts/recall-real10k-diskann-pq-fastscan-release-r32-l100.log`
- `artifacts/load-synth10k-diskann-pq-fastscan-release-r32-l100.log`
- `artifacts/recall-synth10k-diskann-pq-fastscan-release-r32-l100-l64-200-800.log`
