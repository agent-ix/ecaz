# Task 85 Packet 029 Artifact Manifest

- head SHA: `94fef559c`
- task bucket: `reviews/task-85/029-local-heap-prefetch/`
- lane: local implementation checkpoint for SPIRE retained-recall rerank locality
- fixture: local unit validation only
- storage format: not applicable locally
- rerank mode: preserve candidate order; prefetch decoded local heap blocks
  before exact local heap source-vector scoring
- timestamp: 2026-06-07
- isolation: no benchmark run; AWS acceptance is required next

## Artifacts

### `cargo-fmt-check.log`

- command:
  `script -q -e -c 'cargo fmt --check' reviews/task-85/029-local-heap-prefetch/artifacts/cargo-fmt-check.log`
- result: passed
- note: stable-rustfmt emitted existing warnings for unstable config keys
  `imports_granularity` and `group_imports`.

### `local-heap-prefetch-focused-test.log`

- command:
  `script -q -e -c 'CARGO_DISABLE_GIT_DISCOVERY=1 cargo test -p ecaz --lib --locked --offline local_heap_prefetch_blocks_dedupes_without_reordering_candidates -- --nocapture' reviews/task-85/029-local-heap-prefetch/artifacts/local-heap-prefetch-focused-test.log`
- result: passed
- key line:
  `test am::ec_spire::tests::local_heap_prefetch_blocks_dedupes_without_reordering_candidates ... ok`

## Decision

This packet is an implementation checkpoint only. It advances the rerank
locality workstream to AWS measurement. It does not claim an accepted latency
win until AWS 1M/q500 proves retained-recall p50/p95/p99 improvement.
