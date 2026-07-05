# Task 111h / 014 Rerank Byte Counter Semantics

- Head SHA: `17cb6f51a813f5f55b6d1448d3408d02ccedc502`
- Branch: `bench-ivf-111g-115-attribution`
- Packet: `reviews/task-111h/014-rerank-byte-counter-semantics`
- Timestamp: 2026-06-19 23:48 PDT
- Scope: code-review packet for IVF rerank byte-counter semantics.
- Benchmark lane / fixture / corpus: not applicable.
- Storage format / rerank mode: source f32 and index-side packed compact rerank counters.
- Isolated one-index-per-table vs shared-table surface: not applicable for the compile check; the PG18 fixtures use pgrx test-local table/index setup.

## Artifacts

### `artifacts/cargo-check-pg18.log`

- Command:
  `script -q -e -c "cargo check --no-default-features --features pg18" reviews/task-111h/014-rerank-byte-counter-semantics/artifacts/cargo-check-pg18.log`
- Result:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 6.94s`

### `artifacts/cargo-pgrx-test-pg18-index-placement.log`

- Command:
  `script -q -e -c "cargo pgrx test pg18 test_ec_ivf_index_placement" reviews/task-111h/014-rerank-byte-counter-semantics/artifacts/cargo-pgrx-test-pg18-index-placement.log`
- Result:
  `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2198 filtered out; finished in 57.45s`
- Key tests:
  - `test_ec_ivf_index_placement_mixed_fallback_chain ... ok`
  - `test_ec_ivf_index_placement_vacuum_tombstones_packed_group_slot ... ok`
  - `test_ec_ivf_index_placement_insert_maintains_packed_group ... ok`
  - `test_ec_ivf_index_placement_fewer_rerank_bytes ... ok`
  - `test_ec_ivf_index_placement_compact_admin_snapshot ... ok`
