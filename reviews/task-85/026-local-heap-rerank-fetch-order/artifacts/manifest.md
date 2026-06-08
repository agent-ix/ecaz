# Task 85 Packet 026 Artifact Manifest

- Head SHA under review:
  `4f92108ed7d043eb8920a544f15ae27d2a05825f`
- Task bucket:
  `reviews/task-85/026-local-heap-rerank-fetch-order/`
- Lane:
  local implementation validation
- Fixture:
  focused Rust unit test
- Storage format:
  unchanged
- Rerank mode:
  retained `rerank_width=25`; candidate set unchanged
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

### `ecaz-local-heap-fetch-order-test.log`

- Command:
  `CARGO_DISABLE_GIT_DISCOVERY=1 cargo test -p ecaz --lib --locked --offline local_heap_fetch_order_sorts_candidates_by_heap_tid -- --nocapture`
- Result:
  passed
- Key result:
  `test am::ec_spire::tests::local_heap_fetch_order_sorts_candidates_by_heap_tid ... ok`
  and `1 passed; 0 failed; 1975 filtered out`.

## Evidence Status

This is a local implementation checkpoint only. The next required packet must
measure AWS 1M/q500 before accepting or rejecting the locality lever.
