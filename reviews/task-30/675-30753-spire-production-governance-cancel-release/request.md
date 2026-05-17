# 30753 - SPIRE Production Governance Cancel Release

## Summary

This packet reviews commit `8f3c10cc8fb9f9b13cb1cc366c4eb840987d1ed0`
(`Add SPIRE production governance cancellation release`).

The production async transport and compact-candidate receive adapters now use
the same global/per-node advisory governance permit as the blocking diagnostic
executor. The permit is acquired before conninfo parsing or remote socket open,
so saturated governance reports `remote_executor_overload` without exposing raw
conninfo or raw advisory-lock text. The permit is held for the remote attempt
and released through RAII on every return, including local cancellation.

The strict/degraded production fault matrix now includes
`remote_executor_overload` at the `remote_executor_governance` step. Strict
mode fails closed; degraded mode skips the affected node.

## Validation

- `cargo fmt --check`
  - Artifact: `artifacts/cargo-fmt-check.log`
  - Passes with only the existing stable-rustfmt warnings.

- `cargo check --no-default-features --features pg18`
  - Artifact: `artifacts/cargo-check-pg18.log`
  - Passes.

- `git diff --check -- <changed code/docs>`
  - Artifact: `artifacts/git-diff-check-code.log`
  - Passes.

- Focused PG18 wrapper execution through pgrx-installed SQL functions
  - Artifacts:
    - `artifacts/cargo-pgrx-install-pg18-pg-test.log`
    - `artifacts/cargo-pgrx-start-pg18.log`
    - `artifacts/pg18-focused-tests.sql`
    - `artifacts/pg18-focused-tests.log`
    - `artifacts/cargo-pgrx-stop-pg18.log`
  - Passed wrappers:
    - `tests.test_ec_spire_production_fault_matrix_contract()`
    - `tests.test_ec_spire_prod_transport_governance_overload()`
    - `tests.test_ec_spire_prod_receive_governance_overload()`
    - `tests.test_ec_spire_prod_transport_local_cancel_remote_cancel()`
    - `tests.test_ec_spire_prod_receive_local_cancel_remote_cancel()`

`cargo pgrx test pg18 production_fault_matrix_contract` was also attempted
outside the sandbox, but the standalone Rust test binary failed before running
the pgrx test body with `undefined symbol: SPI_finish`. That negative artifact
is retained as `artifacts/cargo-pgrx-test-pg18-production-fault-matrix-contract-escalated.log`.
The SQL-wrapper path above executes the same focused `#[pg_test]` wrappers
inside PostgreSQL.

## Review Focus

- Is acquiring governance permits inside the production async adapter, before
  conninfo parsing and socket open, the right Stage C ownership boundary?
- Are `remote_executor_overload` strict/degraded actions correct for C5 to
  consume from the production fault matrix?
- Do the local-cancel tests prove the relevant release property for global and
  per-node advisory permits before C5 routes the AM scan path through this
  adapter?
