# Task 111h / 001 Placement Semantics Artifacts

- Head SHA: `a3b6eda24e897484b6ef26bc4f4b463f6bc42d46`
- Task bucket: `reviews/task-111h/001-placement-semantics/`
- Timestamp: `2026-06-19T20:33:24-07:00`
- Scope: placement semantics only; no benchmark matrix in this packet.
- Storage surface: code-level reloption/admin/scan semantics; no corpus load.
- Isolated one-index-per-table vs shared-table: focused unit/pg_test fixtures create
  their own test relations under the pgrx test harness; no benchmark tables.

## Artifacts

### `cargo-test-coarse-rerank.log`

- Command:
  `script -q -c "cargo test --no-default-features --features pg18 coarse_rerank --lib" reviews/task-111h/001-placement-semantics/artifacts/cargo-test-coarse-rerank.log`
- Purpose: focused PG18 validation for coarse_rerank option resolution and pg_test
  admin/scan fixtures after renaming source/table placement semantics.
- Key result:
  `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 2171 filtered out; finished in 23.60s`
- Notable covered fixtures:
  - `coarse_rerank_preset_resolves_dense_rabitq1_heap_f32`
  - `coarse_rerank_rejects_table_placement_until_real_table_payloads_exist`
  - `coarse_rerank_rejects_source_placement_with_compact_format`
  - `coarse_rerank_accepts_source_diagnostic_with_compact_format`
  - `pg_test_ec_ivf_coarse_rerank_contract_admin_snapshot`
  - `pg_test_ec_ivf_coarse_rerank_f16_rabitq4_admin_snapshot`
