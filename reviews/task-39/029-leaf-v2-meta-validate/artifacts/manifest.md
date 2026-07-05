# Artifact manifest

- Head SHA: `4c2f13b11ce094fb211bc7d08edfeb6d7cbf9d4a`
- Task bucket: `reviews/task-39/029-leaf-v2-meta-validate`
- Lane: `SpireLeafPartitionObjectV2Meta::validate` error-branch coverage
- Fixture / storage format / rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `leaf-v2-focused-tests.log`

- Command: `cargo test --manifest-path hardening/careful/Cargo.toml
  --lib careful_spire::storage::tests::miri`
- Timestamp: 2026-05-19
- Key result: 21 passed; 0 failed (one more than packet 028 — the
  new test is `miri_leaf_v2_meta_rejects_invalid_validate_inputs`).
