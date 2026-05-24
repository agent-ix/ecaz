# Review Request: HNSW Debug Vacuum Owned Stats

Task: 50 unsafe burndown

Commit under review:

- `d86da7d5` - `Return owned HNSW debug vacuum stats`

## Summary

This packet removes the HNSW guardrail hit for a safe public helper returning PostgreSQL's raw `IndexBulkDeleteResult`.

- Adds `DebugHnswVacuumStats`, a test/debug owned scalar row containing the fields the pg tests assert.
- Changes `debug_vacuum_stats` to return that owned row.
- Keeps the stats pointer copy inside a private helper, so the PostgreSQL stats struct no longer crosses the safe debug API boundary.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count remains `1344`; this is a signature/API cleanup.
- The broadened boundary-signature guard no longer reports `src/am/ec_hnsw/shared.rs`.
- Remaining guard hit:
  - `src/am/ec_hnsw/options.rs`

See `artifacts/unsafe-counts-and-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1344` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

## Reviewer Focus

- Confirm the owned debug row preserves the test-observed HNSW vacuum stats fields.
- Confirm the safe debug helper no longer exposes raw PostgreSQL stats structs.
