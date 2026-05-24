# Task 50 Review Request: HNSW Remove Forwarding Unsafe Wrappers

## Summary

Removed redundant unsafe forwarding wrappers from `src/am/ec_hnsw/mod.rs`.

The module now re-exports the underlying crate-visible unsafe functions from
`shared` and `scan` directly:

- `index_cost_snapshot`
- `index_admin_snapshot`
- `planner_integration_snapshot`
- `explain_counters_from_index_scan_state`

This preserves the existing unsafe call contract at the SQL/EXPLAIN boundary
while deleting wrapper functions that only repeated the same contract and
forwarded immediately.

## Unsafe Burndown

- `src/am/ec_hnsw/mod.rs` unsafe grep count: `8 -> 0`
- repository `src` unsafe grep count: `2440 -> 2432`

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_hnsw/mod.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Review Focus

Please verify the re-exports preserve the public crate API expected by
`src/am/mod.rs` and tests while avoiding the extra unsafe wrapper layer.
