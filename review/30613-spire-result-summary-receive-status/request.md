# SPIRE result summary receive status

## Scope

This packet threads the remote-search libpq receive state into the final
coordinator result summary.

Code checkpoint: `21d70122` (`Thread SPIRE receive status into result summary`)

## Changes

- Adds `libpq_receive_count` and `libpq_receive_status` to
  `ec_spire_remote_search_coordinator_result_summary(...)`.
- Passes the receive fields through from the coordinator gate row into the
  final result-summary row.
- Extends PG18 coverage for local-ready, degraded-ready, and descriptor-blocked
  result summaries so the final row now proves both ready/no-receive and
  blocked/descriptor-receive states.
- Updates the Phase 5 task note so the public result-summary surface includes
  receive status in its advertised contract.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_search_local_heap_resolution_plan`
- `cargo pgrx test pg18 test_ec_spire_remote_search_local_heap_degraded_skip_status`
- `cargo pgrx test pg18 test_ec_spire_remote_heap_resolution_summary_blocks_remote`
- `git diff --check`
