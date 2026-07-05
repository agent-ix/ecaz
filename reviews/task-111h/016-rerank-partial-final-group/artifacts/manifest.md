# Task 111h / 016 Rerank Partial Final Group

- Head SHA: `8c9c9ad5f9acc70eede9fd4c6b22812e2c73010f`
- Branch: `bench-ivf-111g-115-attribution`
- Packet: `reviews/task-111h/016-rerank-partial-final-group`
- Timestamp: 2026-06-20 00:08 PDT
- Scope: code-review packet for scan-level partial final packed rerank group coverage.
- Benchmark lane / fixture / corpus: not applicable.
- Storage format / rerank mode: index-side f16 packed rerank group, `rerank_width = 8`, three valid rows.
- Isolated one-index-per-table vs shared-table surface: not applicable for the compile check; the PG18 fixture uses pgrx test-local table/index setup.

## Artifacts

### `artifacts/cargo-check-pg18.log`

- Command:
  `script -q -e -c "cargo check --no-default-features --features pg18" reviews/task-111h/016-rerank-partial-final-group/artifacts/cargo-check-pg18.log`
- Result:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`

### `artifacts/cargo-pgrx-test-pg18-partial-final-group.log`

- Command:
  `script -q -e -c "cargo pgrx test pg18 test_ec_ivf_index_placement_partial_final_group" reviews/task-111h/016-rerank-partial-final-group/artifacts/cargo-pgrx-test-pg18-partial-final-group.log`
- Result:
  `test tests::pg_test_ec_ivf_index_placement_partial_final_group ... ok`
- Summary:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2204 filtered out; finished in 61.81s`
