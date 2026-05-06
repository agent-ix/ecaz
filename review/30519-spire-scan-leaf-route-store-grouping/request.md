# Review Request: SPIRE Scan Leaf Route Store Grouping

- Branch: `task30-spire-partition-object-spec`
- Code commit: `3d66fea4`
- Scope: Phase 4 scan grouping boundary for local multi-store fetch

## Summary

This checkpoint adds and then tightens the first scan-side grouping boundary
for future multi-store local fetch.

It:

- uses a single `SpireStoreObjectReadGroup` for leaf and delta read grouping;
- groups selected quantized scan leaf routes by `(node_id, local_store_id)`;
- preserves leaf route order inside each store group while processing stores
  in deterministic store-key order;
- feeds the grouped routes through the existing synchronous object-reader and
  candidate-scoring flow;
- records delta routes whose parent leaf was not selected through scan
  placement diagnostics instead of dropping them silently;
- adds a two-store write + scan-fetch fixture that builds a hash-routed
  two-store partitioned draft, reads through the multi-store object-reader set,
  and proves candidates are fetched from leaves placed in both local stores;
- leaves global candidate ranking and reranking unchanged.

This does not open auxiliary store relations, perform parallel reads, or make
any multi-NVMe performance claim. It only makes the scan path's selected PID to
store-group boundary explicit.

## Files

- `src/am/ec_spire/scan/snapshot.rs`
- `src/am/ec_spire/scan/types.rs`
- `src/am/ec_spire/scan/tests.rs`
- `src/am/ec_spire/scan/tests/candidates.rs`
- `src/am/ec_spire/scan/tests/diagnostics.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/lib.rs`

## Review Focus

- Whether grouping selected leaf routes before leaf/delta reads is the right
  first scan boundary for the Phase 4 store-grouped fetch design.
- Whether deterministic store-key order plus per-store route-order preservation
  is appropriate before true parallel fetch.
- Whether `dropped_unselected_delta_route_count` is the right SQL-visible
  diagnostics name for filtered delta route accounting.
- The real grouped/batch fetch consumer remains intentionally open; this
  packet should not be extended with more grouping primitives until that
  consumer exists.

## Reviewer Follow-Up: 2026-05-06

Addressed feedback from `feedback/2026-05-05-01-reviewer.md`:

- collapsed the duplicate `SpireStoreLeafRouteGroup` wrapper into
  `SpireStoreObjectReadGroup`;
- added a code comment that group ordering is store-keyed rather than
  phase-keyed;
- surfaced filtered delta routes through
  `dropped_unselected_delta_route_count` in the internal diagnostics row and
  `ec_spire_index_scan_placement_snapshot`;
- kept the two-store end-to-end write/fetch fixture as an explicit open gate
  rather than adding more inert grouping helpers.

Follow-up commit `3d66fea4` closes that in-memory end-to-end fixture gate. The
remaining store-fetch gap is now relation-backed auxiliary store creation/open
plus measured parallel fetch, not the logical write/read path through the
store set.

## Validation

- `cargo test group_leaf_and_delta_reads_by_local_store --lib`
- `cargo test collect_quantized_routed_probe_candidates_reads_hash_routed_two_store_build --lib`
- `cargo test collect_quantized_routed_probe_candidates --lib`
- `cargo test collect_scan_placement_diagnostics --lib`
- `cargo pgrx test pg18 test_ec_spire_scan_placement_snapshot_sql`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
