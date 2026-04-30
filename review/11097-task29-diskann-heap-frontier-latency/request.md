# Task 29 DiskANN Heap Frontier Latency

## Request

Review the persisted scan frontier optimization and the local PG18 latency
measurement.

Measured commit: `6bd9c5ec42bdd5dcc182c1b3d8efcac72b1819d5`

## Change

`src/am/ec_diskann/scan.rs` now uses heap-backed frontier bookkeeping during
persisted greedy descent:

- `BinaryHeap<Reverse<ScanCandidate>>` selects the next best active candidate.
- `BinaryHeap<ScanCandidate>` evicts the worst active candidate when the
  frontier exceeds `list_size`.
- Active frontier entries cache the decoded neighbor list, so expanding a
  picked node does not re-read and re-decode the same index tuple.

This keeps the output frontier sorted by the same `ScanCandidate` ordering and
preserves the binary-sidecar recall behavior from packet `11096`.

## Result

On local PG18, same `task29_diskann_real10k` prefix and same truth cache as
`11096`:

| list_size | 11096 mean q-time | 11097 mean q-time | recall@10 |
|---:|---:|---:|---:|
| 64 | 52.87 ms | 52.95 ms | 0.9955 |
| 128 | 56.50 ms | 55.86 ms | 0.9960 |
| 200 | 67.65 ms | 64.81 ms | 0.9970 |
| 400 | 109.07 ms | 79.49 ms | 0.9970 |
| 800 | 247.34 ms | 108.25 ms | 0.9975 |

The optimization mainly matters once `list_size` is large enough for the old
linear pick + per-visit sort/truncate to dominate. At L=800, mean query time
improved by ~56% while recall stayed unchanged.

Percentile pass:

| list_size | p50 | p95 | p99 | HWM |
|---:|---:|---:|---:|---:|
| 64 | 50.9 ms | 61.7 ms | 71.0 ms | 63740 KiB |
| 200 | 62.7 ms | 70.4 ms | 73.4 ms | 64860 KiB |
| 800 | 110.8 ms | 137.9 ms | 149.1 ms | 65596 KiB |

For comparison, packet `11096` measured L=800 at p50 `249.9 ms`,
p95 `278.6 ms`, p99 `305.1 ms`, HWM `70948 KiB`.

## Validation

- `cargo test --lib am::ec_diskann::scan -- --nocapture`
- `cargo check --all-targets --no-default-features --features pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx test pg18 test_ec_diskann_sql_ordered_index_scan_executes`
- `git diff --check`

After the pg_test run, the normal PG18 extension build was reinstalled and the
local PG18 server was restarted.

## Recommendation

Keep this optimization. The next latency follow-up should be the remaining
scan-path read reduction and then proper Vamana early-stop if profiling still
shows traversal work at high `list_size`.

Raw logs are under `artifacts/`; see `artifacts/manifest.md`.
