# Task 29 DiskANN Early-Stop Scan Latency

## Request

Review the follow-up persisted scan optimization that replaces active-frontier
eviction with a pgvectorscale-style visited top-L early stop.

Measured commit: `27bb6af8a037b29918f13ca894cc1c1a466c834d`

## Change

`src/am/ec_diskann/scan.rs` now keeps:

- one min-heap of discovered candidates;
- an inserted set via `VisitedState::in_frontier`;
- a sorted `visited_best` list;
- an early stop when the next active candidate cannot beat the current
  top-`list_size` visited candidate.

This removes the second "worst active candidate" heap and active-frontier
eviction bookkeeping from `11097`. It still caches decoded neighbor lists for
discovered entries and preserves the binary-sidecar recall behavior.

## Result

On local PG18, same `task29_diskann_real10k` prefix and same truth cache:

| list_size | 11096 mean | 11097 mean | 11098 mean | recall@10 |
|---:|---:|---:|---:|---:|
| 64 | 52.87 ms | 52.95 ms | 50.36 ms | 0.9955 |
| 128 | 56.50 ms | 55.86 ms | 48.80 ms | 0.9960 |
| 200 | 67.65 ms | 64.81 ms | 53.15 ms | 0.9970 |
| 400 | 109.07 ms | 79.49 ms | 58.89 ms | 0.9970 |
| 800 | 247.34 ms | 108.25 ms | 68.90 ms | 0.9975 |

Percentile pass:

| list_size | p50 | p95 | p99 | HWM |
|---:|---:|---:|---:|---:|
| 64 | 47.8 ms | 54.1 ms | 57.0 ms | 65024 KiB |
| 200 | 55.9 ms | 75.0 ms | 90.1 ms | 64544 KiB |
| 800 | 66.7 ms | 76.9 ms | 80.0 ms | 66640 KiB |

Compared with packet `11097`, L=800 p50/p95/p99 improved from
`110.8/137.9/149.1 ms` to `66.7/76.9/80.0 ms`.

## Validation

- `cargo test --lib am::ec_diskann::scan -- --nocapture`
- `cargo check --all-targets --no-default-features --features pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx test pg18 test_ec_diskann_sql_ordered_index_scan_executes`
- `git diff --check`

After the pg_test run, the normal PG18 extension build was reinstalled and the
local PG18 server was restarted before measurement.

## Recommendation

Keep the early-stop visited frontier implementation. At this point Task 29 has
both the recall fix (`11096`) and the major scan-latency fix (`11097`/`11098`).
The next useful landing check is a final local packet that reruns the callback
smoke set and summarizes the branch as ready for outside review, unless a
reviewer asks for another targeted change.

Raw logs are under `artifacts/`; see `artifacts/manifest.md`.
