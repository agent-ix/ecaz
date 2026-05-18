# 30766 - SPIRE Governance pg_test Isolation

## Summary

This packet reviews commit `d06a7c793423619dcffc785f37d6b4257dcebf10`
(`Isolate SPIRE governance pg tests`).

After fixing the standalone loader stubs in packet `30764`, the broader
production pgrx filter became runnable. The first run of
`cargo pgrx test pg18 test_ec_spire_prod_` passed 19 tests and failed
`test_ec_spire_prod_receive_local_cancel_remote_cancel`: the test observed
`remote_executor_overload` instead of `local_query_cancelled`.

The cause was test interference, not the receive path itself. Multiple
production governance/cancel tests use cluster-wide advisory locks with the
same production keys, and pgrx runs the filtered test set in parallel. This
slice adds a pg_test-only GUC:

`ec_spire.remote_search_governance_test_namespace`

The GUC is compiled only for test / `pg_test` builds. When set, it offsets the
advisory-lock class range so tests can keep exercising the real governance
permit code without sharing lock keys. Production builds keep namespace `0`
and the existing advisory-lock keys.

The Phase 11 task file now records the broader production pgrx pass as complete
for the `test_ec_spire_prod_` surface.

## Key Files

- `src/am/ec_spire/options.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/lib.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

- `git diff --check -- <changed code/docs>`
- `cargo fmt --check`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_prod_`
  - before fix: 19 passed, 1 failed with `remote_executor_overload` vs
    `local_query_cancelled`;
  - after fix: 20 passed.

No distributed fixture or performance run was started for this packet.

## Review Focus

- Is a pg_test-only namespace GUC the right way to isolate advisory-lock tests
  without changing production lock behavior?
- Does the namespace offset preserve the production governance path being
  tested?
- Is the Phase 11 validation claim scoped correctly to the production pgrx
  filtered set, not the full remote-search suite?
