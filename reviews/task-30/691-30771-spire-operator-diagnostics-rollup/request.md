# 30771 - SPIRE Operator Diagnostics Rollup

## Summary

This packet reviews commit `7c3522e059e70742216a6334104abcd94941c7c8`
(`Expose SPIRE operator diagnostics rollup`).

The slice adds the Phase 11.6 packet-friendly operator diagnostic path before
the local one-coordinator / two-remote fault fixture work.

Changes:

- Adds `ec_spire_remote_search_operator_diagnostics()`, a single-row SQL
  rollup over the existing production scan and remote readiness surfaces.
- Reports remote readiness counts from remote node snapshots, excluding the
  local node from `ready_remote_node_count` / `blocked_remote_node_count`.
- Reports remote last-served epoch range, effective nprobe, selected/local/
  remote/skipped PIDs, remote fanout, candidate batches, candidate row count,
  heap-resolution counts, result source, merge status, AM delivery status, and
  next blocker.
- Preserves the AM-boundary blocker: remote-origin heap outputs are surfaced as
  `requires_remote_row_materialization` until the materialized-row provider
  lands.
- Registers the new diagnostic function in the operator entrypoint contract.
- Updates Phase 11 to mark the packet-friendly operator diagnostic rollup done
  while leaving explicit replica-manifest freshness fixture evidence open.

## Key Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/root/types.rs src/am/ec_spire/root/remote_candidates.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo check --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `cargo pgrx test pg18 test_ec_spire_prod_scan_heap_resolution`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`

All commands passed. The first pgrx run initially caught that the diagnostic
was using capability-summary ready counts, which include the local node; the
implementation now computes ready/blocked remote-node counts directly from
remote snapshots.

## Review Focus

- Does this expose the right one-row operator view for readiness, served epoch,
  fanout, candidate batches, heap resolution, merge/result source, and AM
  delivery blocker state?
- Is excluding the local node from ready/blocked remote-node counts correct?
- Is it clear that this is a diagnostic rollup over existing production
  surfaces, not the multi-instance fault fixture itself?
- Is the Phase 11 task update scoped correctly, with replica-manifest freshness
  fixture evidence still open?
