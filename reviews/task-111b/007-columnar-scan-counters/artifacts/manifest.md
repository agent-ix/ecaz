# Task 111b Packet 007 Artifact Manifest: Columnar Scan Counters

- Head SHA: `643928e947bb3dfb42b8074427bd3052c5e0179a`
- Task bucket: `reviews/task-111b`
- Packet path: `reviews/task-111b/007-columnar-scan-counters`
- Timestamp: `2026-06-17T19:47:46Z`
- Storage format / lane: columnar counter unit coverage plus PG18 gated `columnar_frozen_lists = 1` fixture.
- Index/table surface: PG18 columnar test uses one isolated table/index surface; EXPLAIN counter tests are Rust unit tests.

## Artifacts

### `cargo-test-ivf-explain-counters.log`

- Command: `cargo test -q ivf_explain --lib`
- Purpose: unit coverage for the new IVF EXPLAIN counter fields and property rendering.
- Key result: `2 passed; 0 failed; 0 ignored; 0 measured; 2125 filtered out`

### `cargo-test-columnar-counters.log`

- Command: `cargo test -q test_ec_ivf_columnar_frozen_lists_scan_insert_vacuum --lib`
- Purpose: PG18 end-to-end validation that columnar scans populate dedicated columnar counters and no longer charge columnar postings to dense posting counters.
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 2126 filtered out`
