# Task 61 HNSW Scan Frontier Overhead Artifacts

- head SHA: `7928649b0`
- task bucket: `reviews/task-61/002-hnsw-scan-frontier-overhead`
- timestamp: `2026-05-24T23:00:59-07:00`

| Artifact | Command | Result |
| --- | --- | --- |
| `cargo-test-no-run-unique-prefetch.log` | `script -q -e -c "cargo test -p ecaz --no-run unique_prefetch_blocks_keeps_first_block_order_and_skips_invalid_tids" reviews/task-61/002-hnsw-scan-frontier-overhead/artifacts/cargo-test-no-run-unique-prefetch.log` | compile-only validation passed; plain runtime `cargo test` is blocked in this pgrx crate by unresolved `pg_re_throw` |

The follow-up cloud benchmark evidence is tracked in
`benchmarks/task61-hnsw-scan-frontier-overhead/`.
