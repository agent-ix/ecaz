# 30760 - SPIRE Remote Row Materialization Constant

## Summary

This packet reviews commit `65c98d3eb5ce51b92a7310750895dc531011d571`
(`Reuse SPIRE remote row materialization constant`).

The slice closes packet `30758` reviewer P2. The AM remote-placement gate no
longer hard-codes `remote_row_materialization`; it references the shared
executor-step constant used by the production scan delivery classifier. That
keeps the blocker vocabulary single-sourced before AM cursor wiring depends on
the symbol.

No behavior changes beyond constant reuse.

## Key Files

- `src/am/ec_spire/scan/relation.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `git diff --check -- <changed code/docs>`

No PostgreSQL server was started for this packet. This is a compile-time
constant reuse fix.

## Review Focus

- Does referencing `super::SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_ROW_MATERIALIZATION`
  from the AM scan module preserve the intended single source of truth?
- Is this sufficient to close the `30758` P2 before AM cursor wiring lands?
