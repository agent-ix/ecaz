# Artifact manifest

- Head SHA: `6afcc6911427e4f84437b452a23edfbe65b669df`
- Task bucket: `reviews/task-39/031-local-store-set-non-leaf`
- Lane: `SpireLocalObjectStoreSet` non-leaf object kind round-trip
- Fixture / storage format / rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `store-set-focused-tests.log`

- Command: `cargo test --manifest-path hardening/careful/Cargo.toml
  --lib local_object_store_set`
- Timestamp: 2026-05-19
- Key result: 4 passed; 0 failed (one more than prior — the new test
  is `local_object_store_set_round_trips_non_leaf_object_kinds`).
