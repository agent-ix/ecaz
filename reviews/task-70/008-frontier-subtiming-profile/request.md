# Task 70 / Packet 008: Frontier Sub-Timing Profile

## Packet Scope

- Code commit: `f1665fe5760f8eec569bb2560841ed91f270ed09`
- Review drivers:
  - `reviews/task-70/003-phase1-suite-config/feedback/2026-05-31-001-reviewer.md`
  - `reviews/task-70/004-frontier-neighbor-retention/feedback/2026-05-31-001-reviewer.md`
  - `reviews/task-70/005-retained-frontier-heap/feedback/2026-05-31-001-reviewer.md`
- Manifest: `artifacts/manifest.md`
- Summary: `artifacts/frontier-subtiming-summary.md`

This packet requests review for the requested frontier sub-timing instrumentation and packet-local measurement. It is a profiling packet, not a claimed performance win.

## Code Change

The scan profile NOTICE now includes frontier subfields:

- `frontier_candidate_heap_us`
- `frontier_visited_set_us`
- `frontier_neighbor_iter_us`
- `frontier_retained_insert_us`
- `frontier_candidate_heap_ops`
- `frontier_visited_set_ops`
- `frontier_neighbor_slots`
- `frontier_retained_inserts`

`src/am/ec_diskann/scan.rs` adds a `FrontierProfile` accumulator and profile-specific scan entrypoints. The default scan path still calls the existing `vamana_scan_with` path without constructing a profile object.

No new `unsafe` was introduced.

## Validation

Commands and logs:

- `cargo fmt --check`
- `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` -> `artifacts/cargo-test-diskann-scan.log`
- `cargo check --all-targets --no-default-features --features pg18` -> `artifacts/cargo-check-pg18.log`
- `./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file artifacts/install-ecaz-pg-test.log`
- `./target/debug/ecaz bench suite run --config artifacts/suite.json ...` -> `artifacts/suite-run.log`, `artifacts/suite-manifest.json`, `artifacts/results.jsonl`

The focused scan module passes 19 tests. PG18 cargo check finishes successfully. The suite run succeeded with packet-local artifacts.

## Measurement Results

Recall remains on target:

| lane | recall@k | recall mean q-time |
| --- | ---: | ---: |
| L64 | 0.9965 | 0.68 ms |
| L200 | 0.9975 | 0.96 ms |

The profiled latency step enables one NOTICE per scan, so it is diagnostic rather than a new baseline:

| lane | profiled mean | profiled p95 | profiled p99 |
| --- | ---: | ---: | ---: |
| L64 | 0.83 ms | 1.13 ms | 1.60 ms |
| L200 | 1.29 ms | 1.51 ms | 1.59 ms |

The pgvectorscale comparison step, without scan profile NOTICE overhead, measured:

| engine | recall@k | mean | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| ec_diskann L64 | 0.9965 | 0.69 ms | 0.82 ms | 1.01 ms |
| pgvectorscale L64 | 0.9960 | 0.65 ms | 0.84 ms | 1.00 ms |
| ec_diskann L200 | 0.9975 | 0.97 ms | 1.14 ms | 1.21 ms |
| pgvectorscale L200 | 1.0000 | 1.21 ms | 1.59 ms | 1.69 ms |

## Frontier Sub-Timing Signal

Each raw profile step emitted 200 NOTICE rows.

| field | L64 mean | L64 p95 | L200 mean | L200 p95 |
| --- | ---: | ---: | ---: | ---: |
| `frontier_us` | 401.60 us | 484 us | 920.04 us | 1113 us |
| `frontier_candidate_heap_us` | 4.13 us | 9 us | 10.96 us | 20 us |
| `frontier_visited_set_us` | 0.23 us | 0 us | 1.06 us | 8 us |
| `frontier_neighbor_iter_us` | 0.09 us | 0 us | 0.79 us | 6 us |
| `frontier_retained_insert_us` | 0.01 us | 0 us | 0.00 us | 0 us |

The micro-timers show explicit candidate-heap time is small relative to the full frontier residual. The other per-operation timers mostly quantize to zero at microsecond resolution, so the operation counts are the more useful signal:

| field | L64 mean | L64 p95 | L200 mean | L200 p95 |
| --- | ---: | ---: | ---: | ---: |
| `frontier_candidate_heap_ops` | 893.67 | 1117 | 1990.26 | 2530 |
| `frontier_visited_set_ops` | 1751.26 | 1973 | 5256.80 | 5704 |
| `frontier_neighbor_slots` | 1751.26 | 1973 | 5256.80 | 5704 |
| `frontier_retained_inserts` | 67.15 | 72 | 201.95 | 205 |

## Reviewer Notes

This closes the requested sub-timing pass, with one caveat: wall-clock micro-timing inside the fastest inner-loop pieces is too fine-grained to fully allocate the residual `frontier_us`. The counts show L200 scales primarily in visited-set and neighbor-slot volume, not candidate-heap time. Based on this packet, the next P0 slice should avoid another candidate-heap rewrite and should either reduce duplicated visited/neighbor work or first reframe profiling into coarser loop buckets.
