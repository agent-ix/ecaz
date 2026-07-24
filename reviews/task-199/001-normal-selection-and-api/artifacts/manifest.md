# Artifact manifest

- Head SHA: `241579dfb1bb4cd491c38b77759b74ac8e2ed7ab`
- Task bucket: `reviews/task-199`
- Packet: `reviews/task-199/001-normal-selection-and-api`
- Timestamp: `2026-07-24T16:15:40-07:00`
- Lane: local WSL, managed PG18 `18.3`
- Fixture / storage / rerank mode: not applicable; this packet contains
  compilation and focused correctness evidence, not benchmark measurements.
- Isolation surface: not applicable; no corpus was loaded and no shared-table
  benchmark was run.

The commands ran against the source state committed as the head SHA above.

| Artifact | SHA-256 | Command | Key result |
|---|---|---|---|
| `cargo-clippy-pg18.log` | `8e6b8e26a3b909d737766786342a603a2a23eacdf31b4db82f1d141bdda2d0d6` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` | exit 0; dev profile finished |
| `cargo-test-scan-ordering.log` | `3a051dff3bc5b2ae24b98bc451091c94c9042f083a36ff9415a36661a744aec0` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --lib --no-default-features --features pg18 am::ec_distann::scan::tests` | 11 passed, 0 failed |
| `cargo-test-stage-counters.log` | `f0f6296c2ea1bc292eec577a5082d8d9760ba9e0916eac301fbf1220351f022c` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --lib --no-default-features --features pg18,distann-head-attribution-benchmark am::ec_distann::stage_counters::tests` | 2 passed, 0 failed |
| `cargo-test-suite-normalization.log` | `d0d235225039ec04dafe15eae472ecaf29c1ecf3a048311a43e30f7e380cd5d6` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test -p ecaz-cli distann_traversal_pair_with_implicit_search_shape_stays_pairable` | 1 passed, 0 failed |
| `pg18-limit-deepening.log` | `2d84c1dd9ef2ff0ab9ce9c05214c6647f6e70fe284df585c82bb83bdc26c2842` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx test pg18 test_ec_distann_limit_beyond_top_k_deepens_correctly --no-default-features --features pg18` | PG18 test passed; 1 passed, 0 failed, 2,525 filtered |

