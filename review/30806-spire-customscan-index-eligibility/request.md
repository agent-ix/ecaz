# Review Request: SPIRE CustomScan Index Eligibility Surface

Second code slice for the ADR-067 CustomScan pivot. This adds an index-level
eligibility surface that identifies active remote placements without using the
superseded AM-local materialization path.

## Scope

- Adds `ec_spire_custom_scan_index_eligibility(index_oid regclass)`.
- Reports:
  - active epoch;
  - local placement count;
  - remote node count;
  - remote placement count;
  - available remote placement count;
  - `eligible_for_custom_scan`;
  - status and next step.
- Uses `load_relation_epoch_manifests_for_coordinator_fanout(...)` and the
  placement directory directly. This avoids
  `ec_spire_index_placement_snapshot(...)`, which is still tied to the legacy
  AM-local materialization gate and errors when remote placements are present.
- Adds PG18 coverage for local-only status and for rewriting one leaf placement
  to a remote node, expecting `customscan_candidate`.
- Updates Phase 11 tracking for the planner-eligibility diagnostic sub-slice.

## Explicit Non-Scope

- Still no CustomPath generation from the planner hook.
- Still no query-shape detection for
  `ORDER BY <vector-distance-op> LIMIT k`.
- Still no path keys, costing, EXPLAIN output, tuple payload decode, or
  `SpireRemoteFanoutExecutor` tuple execution.

## Files

- `src/am/ec_spire/custom_scan.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `review/30806-spire-customscan-index-eligibility/artifacts/manifest.md`

## Validation

- `cargo test custom_scan --lib`
- `git diff --check`

The focused test command covered:

- Rust unit status shape for the provider scaffold.
- Rust unit shape for the eligibility row.
- PG18 proof that `ec_spire_custom_scan_status()` reports provider/hook
  registration.
- PG18 proof that `ec_spire_custom_scan_index_eligibility(...)` reports
  `local_only` before a remote placement rewrite and `customscan_candidate`
  after one active placement is moved to `node_id = 2`.

## Reviewer Focus

- Confirm the eligibility surface is using the right source of truth for
  CustomScan path generation.
- Confirm bypassing the AM-local materialization-gated placement snapshot is
  appropriate under ADR-067.
- Confirm `customscan_candidate` is correctly limited to indexes with an active
  epoch and at least one available remote placement.
