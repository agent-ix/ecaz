# Review Request: SPIRE 12c CIC Descriptor Defer Receive

- agent: coder1
- date: 2026-05-14
- code commit: `1bd89f8a0213357c1e34e6f2298eabdc5a9c1f0e`
- task rows: partial `12c.3.e`, partial support for `12c.13.a`

## Summary

Adds a focused candidate-receive fixture for the Stage E lifecycle row
`create_index_concurrently_new_descriptor`.

The fixture builds production receive requests against an existing remote
descriptor, creates a new remote index with `CREATE INDEX CONCURRENTLY`, updates
the coordinator descriptor generation to the new index, then asserts the
already-built receive batch still proceeds cleanly against the old planned
requests.

This is intentionally scoped to the first `12c.3.e` checklist bullet. The
second bullet, proving a later full CustomScan uses the refreshed descriptor,
remains open.

The test stays in `remote_search/receive_faults.rs`, now 1,650 lines.

## Changes

- Added `test_ec_spire_prod_receive_cic_new_descriptor_deferred`.
- The fixture runs strict and degraded modes.
- It registers an old descriptor for node 2, builds requests for the old
  descriptor plus a ready remote, then creates and registers a newer descriptor.
- It asserts:
  - the new descriptor identity differs from the old identity
  - `ec_spire_remote_node_descriptor` records the new descriptor generation,
    remote index name, and identity
  - candidate receive still has two ready dispatches and zero failures
  - no degraded skips are recorded
  - the receive summary advances to `remote_heap_resolution` with
    `requires_remote_heap_resolution`
- Updated only the first `12c.3.e` tracker bullet.

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/tests/remote_search/receive_faults.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --no-default-features --features pg18 test_ec_spire_prod_receive_cic_new_descriptor_deferred --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.
- I did not rerun `cargo pgrx test pg18` because the local pgrx test harness
  is still blocked before test execution by the existing
  `undefined symbol: BufferBlocks` loader issue.

## Review Focus

- Confirm the fixture correctly models request construction before descriptor
  generation advancement and receive after advancement.
- Confirm this should check only the first `12c.3.e` tracker bullet, leaving
  the later full-CustomScan refreshed-descriptor assertion open.
- Confirm the strict/degraded receive summary expectations match the lifecycle
  matrix's defer-new-descriptor action.
