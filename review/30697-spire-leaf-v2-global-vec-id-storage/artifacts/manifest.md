# Artifact Manifest: 30697 SPIRE Leaf V2 Global Vector ID Storage

Head SHA: `597c8f998542b49964433f2bf98967311c1dbbf1`
Packet: `review/30697-spire-leaf-v2-global-vec-id-storage`
Timestamp: `2026-05-09T18:25:23-07:00`

## Scope

- Lane: Task 30 Phase 11.2 writer-side global vector identity.
- Fixture: local Rust and PG18 pgrx test harnesses.
- Storage format: Leaf V2 `LocalU64` and fixed-width `GlobalBytes` vector-ID
  columns.
- Rerank mode: not a rerank measurement packet.
- Surface: local object-store and relation-backed Leaf V2 codecs, scan
  candidate collection, and SQL-visible vector identity contract.
- Index isolation: unit/pgrx tests only; no shared-table or multi-index
  benchmark surface.

## Validation Commands

| Command | Result |
| --- | --- |
| `cargo test leaf_v2 --lib` | Passed; `pg_test_ec_spire_relation_leaf_v2_roundtrip` ok |
| `cargo test leaf_partition_object_v2 --lib` | Passed; 4 passed, 0 failed |
| `cargo test global --lib` | Passed; 14 passed, 0 failed |
| `cargo test collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer --lib` | Passed; 1 passed, 0 failed |
| `cargo test remote_search_final_contract --lib` | Passed; `pg_test_ec_spire_remote_search_final_contract` ok |
| `cargo fmt --check` | Passed; existing stable rustfmt warnings about unstable import-grouping settings |
| `git diff --check` | Passed |

## Key Result Lines Cited By Request

- `test tests::pg_test_ec_spire_relation_leaf_v2_roundtrip ... ok`
- `test am::ec_spire::storage::tests::leaf_partition_object_v2_store_preserves_fixed_width_global_vec_ids ... ok`
- `test am::ec_spire::storage::tests::leaf_partition_object_v2_rejects_mixed_payload_or_vec_id_layout ... ok`
- `test am::ec_spire::tests::remote_candidate_merge_dedupes_global_vec_ids_across_nodes ... ok`
- `test tests::pg_test_ec_spire_remote_search_final_contract ... ok`
