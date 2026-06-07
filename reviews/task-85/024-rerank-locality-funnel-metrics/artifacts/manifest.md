# Task 85 Packet 024 Artifact Manifest

- Head SHA under review:
  `a9a938771dde55ac2ed02984071c2360949c2ec0`
- Task bucket:
  `reviews/task-85/024-rerank-locality-funnel-metrics/`
- Lane:
  local instrumentation validation
- Fixture:
  focused Rust unit tests; no AWS benchmark in this packet
- Storage format:
  unchanged
- Rerank mode:
  unchanged; diagnostic-only rerank-prefix heap locality measurement
- Timestamp:
  2026-06-07

## Artifacts

### `cargo-fmt-check.log`

- Command:
  `cargo fmt --check`
- Result:
  passed
- Key result:
  exited `0`; rustfmt emitted existing stable-toolchain warnings for
  nightly-only import grouping options.

### `ecaz-rerank-locality-test.log`

- Command:
  `CARGO_DISABLE_GIT_DISCOVERY=1 cargo test -p ecaz --lib --locked --offline rerank_prefix_heap_locality_counts_prefix_block_scatter -- --nocapture`
- Result:
  passed
- Key result:
  `test am::ec_spire::scan::tests::rerank_prefix_heap_locality_counts_prefix_block_scatter ... ok`
  and `1 passed; 0 failed; 1974 filtered out`.

### `ecaz-cli-funnel-record-test.log`

- Command:
  `CARGO_DISABLE_GIT_DISCOVERY=1 cargo test -p ecaz-cli --locked --offline funnel_record_carries_task85_read_and_score_breakdown -- --nocapture`
- Result:
  passed
- Key result:
  `test commands::bench::spire_pipeline::tests::funnel_record_carries_task85_read_and_score_breakdown ... ok`
  and `1 passed; 0 failed; 405 filtered out`.

## Evidence Status

This is a harness/instrumentation checkpoint only. It enables the required AWS
1M/q500 rerank-locality measurement, but it does not accept or reject the
candidate-set-preserving rerank-locality workstream by itself.
