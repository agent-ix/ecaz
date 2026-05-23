# Review Request: DML Test Analyzed Query Helper Removal

Task: 50 unsafe burndown

Commit under review:

- `74fe15dd` - `Remove raw analyzed query test helper`

## Summary

This packet removes the remaining test-local raw `Query*` analysis helper used by DML frontdoor tests.

- Changes `src/tests/dml_frontdoor.rs` to pass SQL text into the scoped analyzed-query view helper instead of constructing raw analyzed `pg_sys::Query` pointers in the test fixture.
- Deletes `analyzed_query` from `src/tests/mod.rs`, removing direct parser/analyzer/list pointer unsafes from the shared test module.
- Removes the now-unused test re-export of the raw `with_dml_frontdoor_query_view` boundary alias from `src/am/mod.rs` and `src/am/ec_spire/mod.rs`.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1344` to `1339`.
- `analyzed_query(` no longer appears under `src/tests`.
- The broadened boundary-signature guard still has one remaining hit:
  - `src/am/ec_hnsw/options.rs`

See `artifacts/unsafe-counts-and-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1339` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`; this slice removes only the stale raw-query-view re-export made unused by this change.

## Reviewer Focus

- Confirm the DML test fixture now uses the scoped analyzed-query helper rather than exposing raw `Query*` values to each test.
- Confirm deleting the shared `analyzed_query` helper does not remove coverage or change the SQL shapes being exercised.
