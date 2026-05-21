# Review Request: IVF Debug Vacuum Owned Stats

Task: 50 unsafe burndown

Commit under review:

- `459817bf` - `Return owned IVF debug vacuum stats`

## Summary

This packet removes the IVF guardrail hit for a safe public helper returning PostgreSQL's raw `IndexBulkDeleteResult`.

- Adds `DebugEcIvfVacuumStats`, a test/debug owned scalar row containing the fields the pg tests assert.
- Changes `debug_ec_ivf_vacuum_stats` and `debug_ec_ivf_vacuum_remove_heap_tids` to return that owned row.
- Consolidates the stats pointer copy into one private helper, so the copied PostgreSQL stats pointer does not leak through the safe public test API.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count: `1345 -> 1344`.
- The broadened boundary-signature guard no longer reports `src/am/ec_ivf/vacuum.rs`.
- Remaining guard hits are HNSW-only:
  - `src/am/ec_hnsw/options.rs`
  - `src/am/ec_hnsw/shared.rs`

See `artifacts/unsafe-counts-and-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1344` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

## Reviewer Focus

- Confirm the owned debug row preserves the test-observed vacuum stats fields.
- Confirm both IVF debug vacuum helpers no longer expose raw PostgreSQL stats structs through safe public signatures.
