# SPIRE Scan Placement Diagnostics SQL

## Checkpoint

- Code commit: `dc98fa28` (`Expose SPIRE scan placement diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: query-specific per-store scan placement diagnostics for active
  relation-backed `ec_spire` snapshots

## Summary

This checkpoint extends the placement diagnostics surface from static active
snapshot shape to query-specific scan work:

- Added `ec_spire_index_scan_placement_snapshot(index_oid, query)` as a stable,
  strict SQL table function for `ec_spire` indexes.
- The function validates the supplied OID as an `ec_spire` index, validates the
  `real[]` query shape, resolves the same active scan plan used by the current
  single-level scan path, and returns one row per scan-touched
  `(node_id, local_store_id)`.
- Each row reports active epoch, effective `nprobe` and rerank-width source
  labels, scanned PID count, routed leaf PID count, delta PID count,
  candidate-row count, leaf/delta candidate-row split, and delete-delta row
  count.
- The helper-level diagnostics count visible candidate rows after routed
  delete-delta suppression, so a routed delete delta can explain why a scanned
  leaf contributes no live base candidates while a delta insert does.
- Empty indexes with no active epoch return no scan placement rows.
- The Task 30 plan now records that scan-time candidate rows and scanned PID
  counts are exposed for the local single-store path.

This does not implement local multi-store placement, parallel local fetch,
replica reads, physical old-epoch cleanup, real SQL `VACUUM` end-to-end
validation, recall/latency summary evidence, or PQ-FastScan scorer binding.

## Changed Files

- `src/am/ec_spire/scan.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib collect_scan_placement_diagnostics_counts_routed_store_rows --no-default-features --features pg18`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out`
- `cargo test --lib test_ec_spire_scan_placement_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `207 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a measurement or recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- The scan placement counts are query-specific diagnostic rows, not a persistent
  scan telemetry store.

## Reviewer Follow-up

- Follow-up code commit: `6c6ce94a` (`Share SPIRE scan diagnostics walker`)
- Addressed the main drift-risk feedback by replacing the diagnostic-only
  manifest/visibility walkers with observer hooks on the real quantized routed
  scan path.
- `collect_validated_quantized_routed_probe_candidates` now accepts a routed
  scan observer; normal scans use a no-op observer, while scan placement
  diagnostics collect scanned leaf/delta PID counts, delete-delta rows, and
  visible candidate-row counts from the same skip, header validation,
  delete-delta, visibility, payload, scoring, and candidate-gating loops used
  by the actual scan.
- Removed the old `collect_delta_scan_diagnostics_for_base_pid`,
  `collect_leaf_scan_candidate_diagnostics_for_pid`, and
  `store_scan_diagnostics_entry` diagnostic-only walkers.
- Added coverage for the `nprobe == 0` empty-diagnostics path, stale
  `scan_plan.leaf_count` rejection, and degraded-mode `Unavailable` leaf
  skipping.

Follow-up validation:

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib collect_scan_placement_diagnostics --no-default-features --features pg18`
  - `2 passed; 0 failed; 0 ignored; 0 measured; 1089 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `210 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## SQL Delta Follow-up

- Follow-up code commit: `65f7dad0` (`Cover SPIRE scan placement SQL deltas`)
- Extended `test_ec_spire_scan_placement_snapshot_sql` to insert into a built
  `ec_spire` index and verify that
  `ec_spire_index_scan_placement_snapshot` reports one scanned leaf PID plus
  one scanned delta PID for the routed query.
- The SQL test now asserts the post-insert split:
  `scanned_pid_count = 2`, `leaf_pid_count = 1`, `delta_pid_count = 1`,
  `candidate_row_count = 2`, `leaf_candidate_row_count = 1`,
  `delta_candidate_row_count = 1`, and `delete_delta_row_count = 0`.

SQL delta follow-up validation:

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_scan_placement_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1090 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `210 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

Remaining scope note:

- SQL-level delete-delta scan-placement coverage remains open. The current
  public SQL lifecycle can create insert deltas directly; the available debug
  vacuum helper compacts delete-delta state during cleanup, so it does not
  leave a stable SQL-visible delete-delta placement snapshot to assert here.
