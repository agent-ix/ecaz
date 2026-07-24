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
| `cargo-clippy-pg18.log` | `6fbce086c3d4f2e88be71ed3dcd9ce0d0676c8670368b25b8a9e04d18b98fe3b` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` | exit 0; dev profile finished |
| `cargo-test-scan-ordering.log` | `21fbfb35f2a22f7bd95036b3ea3a7a8bd15f6331eb5655d88769277b42955e6e` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --lib --no-default-features --features pg18 am::ec_distann::scan::tests` | 11 passed, 0 failed |
| `cargo-test-stage-counters.log` | `a58bed155b3db0db9ee2919924d289ddb6ed64bb17a6cb4c564b2958f0b87161` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --lib --no-default-features --features pg18,distann-head-attribution-benchmark am::ec_distann::stage_counters::tests` | 2 passed, 0 failed |
| `cargo-test-suite-normalization.log` | `7244e14c946f5d0d4325c7b917697457b573fe12c881a105c1b4d04e7feda4b6` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test -p ecaz-cli distann_traversal_pair_with_implicit_search_shape_stays_pairable` | 1 passed, 0 failed |
| `pg18-limit-deepening.log` | `f84476bc933dfb589366d3ca22cecae4ad8d198f2b70c836609269a706516849` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx test pg18 test_ec_distann_limit_beyond_top_k_deepens_correctly --no-default-features --features pg18` | PG18 test passed; 1 passed, 0 failed, 2,525 filtered |
