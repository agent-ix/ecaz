# Task 111h / 011 Rerank Stage Timing Counters

- Head SHA: `7a58a882565523e13538aab526221fb104a0234f`
- Branch: `bench-ivf-111g-115-attribution`
- Packet: `reviews/task-111h/011-rerank-stage-timing-counters`
- Timestamp: 2026-06-19 23:19 PDT
- Scope: code-review packet for EXPLAIN/debug counter coverage only.
- Benchmark lane / fixture / corpus: not applicable.
- Storage format / rerank mode: source f32 and index-side packed compact rerank paths are instrumented; focused runtime fixture uses the existing index-placement rerank byte fixture.
- Isolated one-index-per-table vs shared-table surface: not applicable for unit/check runs; the PG18 fixture uses its own pgrx test-local table/index setup.

## Artifacts

### `artifacts/cargo-test-ivf-explain.log`

- Command:
  `script -q -e -c "cargo test --no-default-features --features pg18 ivf_explain --lib" reviews/task-111h/011-rerank-stage-timing-counters/artifacts/cargo-test-ivf-explain.log`
- Result:
  `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2201 filtered out; finished in 0.00s`
- Key tests:
  - `am::common::explain::tests::ivf_explain_properties_render_the_current_counter_values ... ok`
  - `am::common::explain::tests::ivf_explain_counters_record_each_staged_statistic ... ok`

### `artifacts/cargo-check-pg18.log`

- Command:
  `script -q -e -c "cargo check --no-default-features --features pg18" reviews/task-111h/011-rerank-stage-timing-counters/artifacts/cargo-check-pg18.log`
- Result:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 11.96s`

### `artifacts/cargo-pgrx-test-pg18-index-placement-rerank-bytes.log`

- Command:
  `script -q -e -c "cargo pgrx test pg18 test_ec_ivf_index_placement_fewer_rerank_bytes" reviews/task-111h/011-rerank-stage-timing-counters/artifacts/cargo-pgrx-test-pg18-index-placement-rerank-bytes.log`
- Result:
  `test tests::pg_test_ec_ivf_index_placement_fewer_rerank_bytes ... ok`
- Summary:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2202 filtered out; finished in 54.47s`
