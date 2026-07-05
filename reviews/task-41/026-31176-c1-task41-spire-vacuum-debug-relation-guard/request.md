# Review Request: Task 41 SPIRE Vacuum Debug Relation Guard

Code commit: `441af70e1bdf16a8430907cfc8a0f80dfd491dba`

## Summary

This checkpoint wraps the pg_test-only SPIRE vacuum debug relation opens in
`src/am/ec_spire/vacuum/mod.rs`.

- Adds `ShareUpdateExclusiveIndexRelation` for the debug vacuum helpers.
- Keeps the index relation live across `ec_spire_ambulkdelete` and, for the
  remove helper, `ec_spire_amvacuumcleanup`.
- Deletes the two manual `index_open` / `index_close` pairs.

## Safety Delta

- Baseline entries: `4319` -> `4315`.
- `src/am/ec_spire/vacuum/mod.rs`: `39` -> `35`.
- Remaining vacuum entries are callback, stats, page, tuple, and manifest
  unsafe residuals rather than direct debug relation open/close cleanup.

## Reviewer Focus

- Confirm the ShareUpdateExclusive lock mode still matches the previous debug
  helper open/close behavior.
- Confirm `info.index` is only borrowed from the guard while the guard remains
  live across the vacuum calls.
- Confirm the new null-open behavior is acceptable for debug helpers: explicit
  `pgrx::error!` instead of passing a null relation into vacuum callbacks.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `git diff --check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Packet-local logs and baseline snapshots are in `artifacts/`; see
`artifacts/manifest.md`.
