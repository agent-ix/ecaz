---
topic: spire-prepared-transaction-capacity-hint
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30921
stage: phase-12.4
status: open
---

# Review Request: SPIRE Prepared Transaction Capacity Hint

## Scope

Please review commit `c201d20c` (`Hint SPIRE prepared transaction capacity
failures`).

This is a narrow Phase 12.4 `max_prepared_transactions` readiness slice. It
does not yet add descriptor-registration preflight probing; that tracker row
remains open.

## What Changed

- Documented the remote `max_prepared_transactions` requirement in ADR-069 and
  `docs/SPIRE_DIAGNOSTICS.md`.
- Added a shared remote `PREPARE TRANSACTION` error wrapper for coordinator
  INSERT and DELETE prepare paths.
- The wrapper appends a SPIRE-specific remediation hint when PostgreSQL reports
  prepared transactions disabled, prepared-transaction capacity exhaustion, or
  a `max_prepared_transactions` capacity message.
- Added a Rust unit test for the classifier across expected SQLSTATE/message
  combinations and unrelated false positives.
- Phase 12.4 marks the documentation and error-hint rows complete while
  leaving descriptor-registration check/warn open.

## Evidence

See `artifacts/manifest.md`.

Validation run against `c201d20cc237ea6fe41379fc3c159bf0c1e6a0af`:

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo test --features pg18 --no-default-features prepare_transaction_capacity_classifier_matches_postgres_errors`

## Review Focus

- Confirm the capacity classifier is broad enough for PostgreSQL's disabled and
  exhausted prepared-transaction failures without catching unrelated resource
  errors.
- Confirm sharing the wrapper between coordinator INSERT and DELETE remote
  prepare paths is appropriate.
- Confirm the tracker correctly leaves live descriptor-registration
  check/warn work open.
