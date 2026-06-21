# Task 118 Packet 009 Artifact Manifest

- head SHA: `3d0caf862e0fa4b35876dbcccb244b3c5410e010`
- task bucket: `reviews/task-118/009-score-sanity-fixture`
- generated: `2026-06-21`
- lane / fixture / storage format / rerank mode: local synthetic HNSW score-correlation scorer fixture for TurboQuant, PqFastScan, and RaBitQ using source-backed build/rerank columns.
- isolated surface: one tiny table and one HNSW index per storage format inside the pg_test fixture.

## Artifacts

### `cargo-check-pg18-pgtest-score-sanity.log`

- command:
  `cargo check --features 'pg18 pg_test' --no-default-features`
- purpose: compile the new `pg18 pg_test` scorer-sanity fixture and the existing pg_test diagnostic exports without invoking the long-running pgrx test harness.
- key result:
  `Finished dev profile [unoptimized + debuginfo]`

### `cargo-pgrx-test-pg18-score-sanity.log`

- command:
  `cargo pgrx test pg18 test_ech_score_correlation_synthetic_known_ordering`
- purpose: attempted runtime execution of the new synthetic score-correlation fixture.
- result: inconclusive. On this AMD sandbox session the command remained at
  `Compiling ecaz v0.1.1` for several minutes and was interrupted. Do not treat
  this artifact as a passing runtime test.

## Validation Notes

I also attempted direct execution with:

- `cargo pgrx test pg18 test_ech_score_correlation_synthetic_known_ordering`
- `cargo test --features 'pg18 pg_test' --no-default-features test_ech_score_correlation_synthetic_known_ordering -- --nocapture`

The logged pgrx attempt and the earlier plain `cargo test` attempt were both
interrupted before producing a pass/fail result in this AMD sandbox session.
The fixture is committed because it closes a Task 118 scorer-sanity coverage gap
and compiles under the PG18 pg_test feature set; runtime execution should be
rerun on a normal PG18 test host.
