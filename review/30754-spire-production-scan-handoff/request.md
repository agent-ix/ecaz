# 30754 - SPIRE Production Scan Handoff

## Summary

This packet reviews commit `38beeb6fbc33dec4e84295a82451d67960040f1b`
(`Add SPIRE production scan handoff`).

The slice adds the first production AM-scan handoff surface:
`ec_spire_remote_search_production_scan_handoff_summary(...)`. It derives
selected leaf PIDs through the same scan router used by `amrescan`, fans those
PIDs into the production remote executor, runs live compact-candidate receive,
merges only ready compact batches, and reports
`requires_remote_heap_resolution` instead of pretending remote SQL rows are
ready.

Two design fixes are included because the handoff exposed them:

- Coordinator fanout now loads manifests through
  `load_relation_epoch_manifests_for_coordinator_fanout(...)`, so remote node
  placements are valid for planning.
- Scan planning has a routing-only leaf count,
  `count_scan_plan_routable_leaf_pids(...)`, so the coordinator can size
  `nprobe` without reading remote leaf payloads locally.

This is still Stage C / C5 candidate handoff work. It does not claim final
tuple production, remote heap visibility, remote locator resolution, or
AWS/product-scale performance. Stage D remains the blocker before
`amrescan`/`amgettuple` can return coordinator-visible remote rows.

## Key Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/scan/routing.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/design/spire-production-coordinator-executor.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check -- <changed code/docs>`
- `cargo pgrx install --test -c /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features "pg18 pg_test"`
- Focused PG18 SQL wrapper:
  - reset packet database
  - `CREATE EXTENSION ecaz CASCADE`
  - `SELECT tests.test_ec_spire_prod_scan_handoff_receive()`

The focused wrapper creates a loopback remote index and coordinator index with
matching deterministic `rabitq` shape, rewrites one coordinator leaf placement
to node 2, registers the remote descriptor, and verifies:

```text
effective_nprobe = 2
selected_pid_count = 2
local_pid_count = 1
remote_pid_count = 1
dispatch_count = 1
candidate_receive_ready_dispatch_count = 1
candidate_row_count = 1
merged_candidate_count = 1
final_heap_fetch_status = requires_remote_heap_resolution
next_blocker = remote_heap_resolution
status = requires_remote_heap_resolution
```

Broad `cargo pgrx test` was not run for this packet. The focused SQL-wrapper
path was used because the recent direct `cargo pgrx test` path is still blocked
by the existing standalone test-binary loader issue tracked in packet `30753`
(`undefined symbol: SPI_finish`).

## Review Focus

- Is the summary/proof surface the right boundary for C5 while Stage D remote
  heap resolution is still absent?
- Does the routing-only leaf-count helper preserve the coordinator/remote
  ownership boundary cleanly enough for deeper multi-layer SPIRE routing?
- Is the production executor handoff status model clear enough for the future
  move into `amrescan`/`amgettuple`, or should the final tuple path get a
  narrower internal state object before Stage D lands?
