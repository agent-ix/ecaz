# Task 111h / 005 Artifacts Manifest

Head SHA: `6edd28d1449a0148e676b8181f5dc1bcbf362d77`

Task bucket: `reviews/task-111h/005-packed-rerank-group-integration/`

Timestamp: `2026-06-20T04:48:57Z`

Scope: packed index-side rerank group integration, scan lookup integration, live
insert/vacuum integration, and IVF format v5 metadata/fixture updates.

Storage surface: `rerank_placement = 'index'`, compact persisted payloads,
packed rerank group layout with `0x2B` group headers and `0x2C` payload
continuation segments.

Formats covered by validation: f16, RaBitQ-4, RaBitQ-8, TurboQuant through
focused Rust unit tests and PG18 `pg_test` coverage.

Suite surface: no benchmark suite. These are correctness/static validation logs,
not latency/recall/storage measurement evidence. No benchmark tables were
created; no isolated one-index-per-table or shared-table benchmark surfaces were
used.

## Artifacts

- `cargo-check-pg18.log`
  - Command: `script -q -c "cargo check --no-default-features --features pg18" reviews/task-111h/005-packed-rerank-group-integration/artifacts/cargo-check-pg18.log`
  - Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.15s`

- `cargo-test-rerank-group.log`
  - Command: `script -q -c "cargo test --no-default-features --features pg18 rerank_group --lib" reviews/task-111h/005-packed-rerank-group-integration/artifacts/cargo-test-rerank-group.log`
  - Key result: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2196 filtered out`

- `cargo-test-data-page-chain.log`
  - Command: `script -q -c "cargo test --no-default-features --features pg18 data_page_chain_ivf_tuple_roundtrips --lib" reviews/task-111h/005-packed-rerank-group-integration/artifacts/cargo-test-data-page-chain.log`
  - Key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2198 filtered out`

- `cargo-test-on-disk-ivf-metadata.log`
  - Command: `script -q -c "cargo test --no-default-features --features pg18 --test on_disk_fixtures ivf_metadata" reviews/task-111h/005-packed-rerank-group-integration/artifacts/cargo-test-on-disk-ivf-metadata.log`
  - Key result: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 45 filtered out`

- `cargo-test-size-of-assertions.log`
  - Command: `script -q -c "cargo test --no-default-features --features pg18 --test size_of_assertions" reviews/task-111h/005-packed-rerank-group-integration/artifacts/cargo-test-size-of-assertions.log`
  - Key result: `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

- `cargo-test-upgrade-matrix.log`
  - Command: `script -q -c "cargo test --no-default-features --features pg18 --test upgrade_matrix" reviews/task-111h/005-packed-rerank-group-integration/artifacts/cargo-test-upgrade-matrix.log`
  - Key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

- `cargo-test-index-quant-formats.log`
  - Command: `script -q -c "cargo test --no-default-features --features pg18 index_quant_formats_top_neighbor --lib" reviews/task-111h/005-packed-rerank-group-integration/artifacts/cargo-test-index-quant-formats.log`
  - Key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2198 filtered out`

- `cargo-test-index-placement.log`
  - Command: `script -q -c "cargo test --no-default-features --features pg18 index_placement --lib" reviews/task-111h/005-packed-rerank-group-integration/artifacts/cargo-test-index-placement.log`
  - Key result: `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 2189 filtered out`

- `cargo-test-coarse-rerank.log`
  - Command: `script -q -c "cargo test --no-default-features --features pg18 coarse_rerank --lib" reviews/task-111h/005-packed-rerank-group-integration/artifacts/cargo-test-coarse-rerank.log`
  - Key result: `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 2176 filtered out`
