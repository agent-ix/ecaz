# Task 118 Review Request: Synthetic Score-Correlation Sanity Fixture

## Scope

This checkpoint adds a small deterministic `pg_test` fixture for Task 118 score-correlation diagnostics.

The fixture:

- creates 16-dimensional synthetic corpus/query tables with a known exact inner-product ordering;
- builds one `ec_hnsw` index for each required Task 118 storage format: TurboQuant, PqFastScan, and RaBitQ;
- calls `graph_scan_score_correlation_for_context`;
- asserts every emitted result has a comparison score;
- checks the known exact-best row remains first;
- verifies the debug comparison scores match the hand-authored source-vector scores for the emitted top rows;
- asserts bounded score delta, rank shift, and strong Spearman correlation.

This is not benchmark evidence. It is scorer-sanity coverage for the diagnostic path required by Phase 4, so wrong-sign, missing-comparison, or badly misordered synthetic cases can be caught before interpreting large-scale evidence.

## Validation

- `artifacts/cargo-check-pg18-pgtest-score-sanity.log`
  - command: `cargo check --features 'pg18 pg_test' --no-default-features`
  - result: passed

- `artifacts/cargo-pgrx-test-pg18-score-sanity.log`
  - command: `cargo pgrx test pg18 test_ech_score_correlation_synthetic_known_ordering`
  - result: inconclusive. The command remained at the compile phase in this AMD sandbox session and was interrupted.

The fixture should be rerun on a normal PG18 test host before treating it as runtime-validated.

## Remaining Task 118 Closeout Work

Final Task 118 closeout still requires the Intel 50k/100k source-vs-compressed suite evidence across TurboQuant, PqFastScan, and RaBitQ: recall, latency, storage, frontier containment, rerank counters, and score-correlation rows.
