# Artifact Manifest: 30698 SPIRE Stable Source Identity Contract

Head SHA: `942efcae1becb14ffcfccb4e38504e206645829e`
Packet: `review/30698-spire-stable-source-identity-contract`
Timestamp: `2026-05-09T18:33:28-07:00`

## Scope

- Lane: Task 30 Phase 11.2 writer-side global vector identity.
- Fixture: local Rust and PG18 pgrx test harnesses.
- Storage format: assignment-layer source identity contract; Leaf V2 global
  storage from packet 30697 is the consumer.
- Rerank mode: not a rerank measurement packet.
- Surface: assignment allocator/source-identity APIs and SQL-visible vector
  identity contract.
- Index isolation: unit/pgrx tests only; no shared-table or multi-index
  benchmark surface.

## Validation Commands

| Command | Result |
| --- | --- |
| `cargo test fixed_global_source_identity --lib` | Passed; 2 passed, 0 failed |
| `cargo test assign --lib` | Passed; 61 passed, 0 failed |
| `cargo test remote_search_final_contract --lib` | Passed; `pg_test_ec_spire_remote_search_final_contract` ok |
| `cargo fmt --check` | Passed; existing stable rustfmt warnings about unstable import-grouping settings |
| `git diff --check` | Passed |

## Key Result Lines Cited By Request

- `test am::ec_spire::assign::tests::allocator_uses_fixed_global_source_identity_without_advancing_local_sequence ... ok`
- `test am::ec_spire::assign::tests::fixed_global_source_identity_rejects_wrong_width ... ok`
- `test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 1442 filtered out`
- `test tests::pg_test_ec_spire_remote_search_final_contract ... ok`
