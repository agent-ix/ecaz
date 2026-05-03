# SPIRE Placement Diagnostics SQL

## Checkpoint

- Code commit: `4decc7fe` (`Expose SPIRE placement diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: per-local-store placement/object diagnostics for active relation-backed
  `ec_spire` snapshots

## Summary

This checkpoint adds the first placement-map diagnostic surface for SPIRE:

- Added `ec_spire_index_placement_snapshot(index_oid)` as a stable, strict SQL
  table function for `ec_spire` indexes.
- The function validates the supplied OID as an `ec_spire` index, loads the
  active epoch manifests, and returns one row per active
  `(node_id, local_store_id)` placement group.
- Empty indexes with no active epoch return no placement rows.
- Each row reports active epoch, node/store identity, total placements,
  placement-state counts, object-kind counts, assignment count, routing-child
  count, and object-byte buckets.
- Available placements are decoded through the relation object store so root,
  internal, leaf, and delta object counts reflect the currently readable active
  snapshot. Stale, unavailable, and skipped placements are counted but not read.
- The current local single-store path reports one default store row for a
  populated single-level build.
- The Task 30 plan now records placement snapshot coverage while keeping
  scan-time candidate rows, scanned PID counts, parallel local fetch, physical
  cleanup, and remote placement work open.

This does not implement local multi-store placement, replica reads, scan-time
placement counters, physical old-epoch cleanup, real SQL `VACUUM` end-to-end
validation, recall/latency summary evidence, or PQ-FastScan scorer binding.

## Changed Files

- `src/am/ec_spire/diagnostics.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_placement_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1085 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `205 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a measurement or recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- Real SQL `VACUUM` end-to-end validation remains open; psql access to the
  local test sockets is blocked in the current sandbox without escalation.
