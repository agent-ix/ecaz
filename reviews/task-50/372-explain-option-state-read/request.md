# Task 50 Review Request: EXPLAIN Option State Read

## Summary

This slice advances Task 50 shared AM/common residual minimization.

Code commit: `58dbb7e340e8fe912594988cc24844e1b4291847`

Changed `src/am/common/explain.rs` so the PG18 EXPLAIN extension-state lookup
and bool dereference live in one explicit boundary block inside
`explain_option_enabled`, instead of splitting the raw state pointer lookup and
typed dereference into separate direct unsafe blocks.

## Unsafe Counts

- `src/am/common/explain.rs`: `12 -> 11`
- `src/` total direct unsafe blocks: `1184 -> 1183`

See `artifacts/unsafe-counts.log`.

## Plan Coverage

- Program: P1/P2, FFI callback and PostgreSQL handle boundary cleanup.
- Wave/tranche: Wave 4, AM common residual minimization.
- Disposition: coalesced an explain-option state raw pointer read into the
  existing unsafe callback boundary helper.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed; it reports the known pre-existing `src/am/mod.rs` unused SPIRE DML
  re-export warning.
- `git diff --check` passed.
- `rustfmt --check src/am/common/explain.rs` passed.
- Raw-boundary guard produced no matches.
- Generated unsafe ledger covers all `1183` current `src` unsafe rows.

Artifacts are under `artifacts/`; see `artifacts/manifest.md`.
