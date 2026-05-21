# Task 50 Review Request: SPIRE Remote Node Helper Boundary

## Summary

This slice narrows SPIRE remote-node snapshot and capability helper boundaries now that live relation construction is safe.

Code commit: `e2ffd4c0d4922f57ba0dac2efec6130c49a3caa2`

## Changes

- Converted `remote_node_snapshot`, `remote_node_capability_plan`, and `remote_node_capability_summary` to safe APIs.
- Switched the remote-node snapshot/capability SQL wrappers to `with_live_index_relation_safe!`.
- Removed caller unsafe blocks in remote-search operator diagnostics and target readiness fanout.

The remaining unsafe in these modules is still tied to lower-level relcache, timeout symbol, transaction ID, or heap/materialization boundaries.

## Validation

- `git diff --check HEAD~1..HEAD`
- `make unsafe-block-count`
- `make UNSAFE_LEDGER=reviews/task-50/123-spire-remote-node-helper-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/123-spire-remote-node-helper-boundary unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/123-spire-remote-node-helper-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Key results:

- Direct unsafe blocks dropped to `1635` across `122` files.
- Ledger check passed: `ledger covers 1635 current unsafe rows`.
- PG18 bench-feature compile check passed with the known existing SPIRE DML unused-import warning in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the packet-local artifact index.
