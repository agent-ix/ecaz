# Task 118 Packet 020 Artifact Manifest

- head SHA: `c64247c5d14dba08b0799f9df1ddcf5c79f1613c`
- task bucket: `reviews/task-118/020-score-sanity-runtime-validation`
- generated: `2026-06-21T17:03:28-07:00`
- lane / fixture / storage format / rerank mode: synthetic known-order HNSW
  score-correlation runtime fixture for TurboQuant, PqFastScan, and RaBitQ.
- isolated surface: one tiny table and one HNSW index per storage format inside
  the pg_test fixture.

## Artifacts

### `cargo-pgrx-test-pg18-score-sanity-rerun.log`

- command:
  `cargo pgrx test pg18 test_ech_score_correlation_synthetic_known_ordering`
- purpose: current-head retry of the packet 009 synthetic score-correlation
  runtime fixture.
- result: inconclusive. On this AMD host, the command remained at
  `Compiling ecaz v0.1.1` and was interrupted. Do not treat this as runtime
  validation.

## Handoff Update

Packet 020 updates the Intel closeout runbook and audit template to require:

`reviews/task-118/006-final-attribution-matrix/artifacts/cargo-pgrx-test-pg18-score-sanity-intel.log`

The final closeout should treat a failing score-sanity test as a blocker for
Task 118 score-correlation interpretation.
