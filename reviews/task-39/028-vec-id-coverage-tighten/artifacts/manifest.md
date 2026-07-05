# Artifact manifest

- Head SHA: `ca10cef215d9fa818b659def7dcbdc57e2c354d8`
- Task bucket: `reviews/task-39/028-vec-id-coverage-tighten`
- Lane: SpireVecId boundary / discriminator coverage
- Fixture / storage format / rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `vec-id-focused-tests.log`

- Command: `cargo test --manifest-path hardening/careful/Cargo.toml
  --lib careful_spire::storage::tests::miri`
- Timestamp: 2026-05-19
- Key result: 20 passed; 0 failed (filters down to the
  `miri_`-prefixed storage tests, which now includes
  `miri_global_vec_id_max_payload_is_accepted` and
  `miri_vec_id_local_sequence_is_none_for_global`).
