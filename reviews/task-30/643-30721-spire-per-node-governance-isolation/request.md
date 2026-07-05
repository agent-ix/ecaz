# Review Request: SPIRE Per-Node Governance Isolation

Code checkpoint: `0ff350cf` (`Cover SPIRE per-node governance isolation`)

## Summary

This slice closes reviewer 30717 P3 by proving the per-node governance cap is
scoped to each remote `node_id`: saturating node 2's advisory slot does not
cause node 3 to report `remote_executor_governance`.

## Scope

- Adds a test-only helper for per-node governance advisory-lock keys.
- Adds a multi-placement test rewrite helper so a fixture can assign two leaf
  PIDs to two remote nodes in one manifest rewrite.
- Adds PG18 coverage:
  - `ec_spire.remote_search_max_concurrent_dispatches_per_node = 1`;
  - a separate backend holds node 2's per-node slot;
  - receive attempts over node 2 and node 3 run in degraded mode;
  - node 2 reports `remote_executor_overload` /
    `remote_executor_governance`;
  - node 3 proceeds independently to `requires_conninfo_secret_resolution`.
- Updates the Phase 11 task file with the per-node isolation coverage.

## Validation

Packet-local logs live under `artifacts/` and are indexed in
`artifacts/manifest.md`.

- `cargo check --no-default-features --features pg18`
  - `Finished dev profile ... target(s) in 0.12s`
- `cargo fmt --check`
  - exited `0`; existing stable-rustfmt warnings for unstable options remain
- `cargo pgrx test pg18 test_ec_spire_libpq_executor_per_node_governance_isolated`
  - `test tests::pg_test_ec_spire_libpq_executor_per_node_governance_isolated ... ok`
  - `1 passed; 0 failed; 1524 filtered out`
- `git diff --check`
  - exited `0`

## Review Questions

- Does this fixture prove the intended per-node-vs-global isolation invariant?
- Is the multi-placement rewrite helper an acceptable test-only utility for
  future multi-remote diagnostics fixtures?
