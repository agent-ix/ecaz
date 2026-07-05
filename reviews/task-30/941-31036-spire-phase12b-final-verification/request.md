# Review Request: SPIRE Phase 12b Final Verification

## Code Under Review

- Code commit: `b11ae95a58c9959a222f26042ee82076c78267c2`
- Topic: Phase 12b cleanup closeout and PG18 verification stabilization.
- Packet-local manifest: `artifacts/manifest.md`

## Summary

This checkpoint closes the Phase 12b cleanup tracker after the oversized
`src/tests/mod.rs` split and the final PG18 verification pass.

The code changes are limited to verification fallout:

- Move prepared-xact intent marking out of pgrx `PreCommit` callbacks and
  mark `commit_local` before registering commit/abort callbacks.
- Make non-PG Rust unit-test calls to SPIRE session-option getters use the
  same defaults instead of reading PostgreSQL GUC state outside a backend.
- Update PG18 fixtures for current tuple transport and remote identity
  contracts.
- Stabilize the concurrent insert/VACUUM/scan fixture by retrying only the
  transient active-epoch mismatch in its scan worker.

## Verification

- `cargo test -p ecaz`
  - log: `artifacts/cargo-test-ecaz-rerun5.log`
  - result: `1714 passed; 0 failed; 4 ignored` for the pg_test block; all
    remaining Rust test targets and doc-tests also ended with `0 failed`.
- `cargo pgrx test pg18`
  - log: `artifacts/cargo-pgrx-test-pg18.log`
  - result: `1714 passed; 0 failed; 4 ignored` for the pg_test block; all
    remaining Rust test targets and doc-tests also ended with `0 failed`.
- Line-count audit:
  - `src/am/ec_spire/dml_frontdoor/mod.rs` is the largest SPIRE production
    file at 2,498 lines.
  - `src/tests/remote_search/contracts.rs` is the largest fixture file at
    2,864 lines.
  - `src/tests/mod.rs` is 2,799 lines.

## Notes For Review

The initial sandboxed full-suite run failed during pgrx install with a
read-only filesystem error under `/home/peter/.pgrx`, which poisoned the
pgrx test mutex and cascaded failures. The passing full-suite logs are
from escalated runs that can write to the configured PG18 pgrx install.
