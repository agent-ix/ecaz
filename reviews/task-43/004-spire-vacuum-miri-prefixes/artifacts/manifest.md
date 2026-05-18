# Artifact Manifest: Task 43 Packet 004

- Head SHA: `f1c00e824fd53f1789d01723982ad9de746ee4a1`
- Code checkpoint SHA: `f1c00e82`
- Task bucket: `reviews/task-43/004-spire-vacuum-miri-prefixes`
- Timestamp: `2026-05-18T09:43:31-07:00`
- Lane: targeted Miri execution for promoted pure `miri_` tests
- Fixture / storage format / rerank mode: not applicable
- Index surface: not applicable; pure Rust unit tests only

## Artifacts

### `miri-diskann-vacuum-repair.log`

- Command: `cargo +nightly miri test --lib miri_vc_006_repair_neighbors_compacts_and_pads`
- Key result: `test am::ec_diskann::vacuum::tests::miri_vc_006_repair_neighbors_compacts_and_pads ... ok`
- Exit: `0`

### `miri-diskann-vacuum-encoded-length.log`

- Command: `cargo +nightly miri test --lib miri_vc_009_repair_preserves_encoded_length`
- Key result: `test am::ec_diskann::vacuum::tests::miri_vc_009_repair_preserves_encoded_length ... ok`
- Exit: `0`

### `miri-spire-bounded-dedupe.log`

- Command: `cargo +nightly miri test --lib miri_rank_routed_leaf_rows_by_ip_keeps_bounded_best_deduped_candidates`
- Key result: `test am::ec_spire::scan::tests::miri_rank_routed_leaf_rows_by_ip_keeps_bounded_best_deduped_candidates ... ok`
- Exit: `0`

### `miri-spire-candidate-cursor.log`

- Command: `cargo +nightly miri test --lib miri_scan_candidate_cursor_emits_ranked_candidates_once`
- Key result: `test am::ec_spire::scan::tests::miri_scan_candidate_cursor_emits_ranked_candidates_once ... ok`
- Exit: `0`

