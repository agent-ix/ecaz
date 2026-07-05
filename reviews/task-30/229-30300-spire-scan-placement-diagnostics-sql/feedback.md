# 30300 SPIRE Scan Placement Diagnostics SQL — review

Code commits `dc98fa28` (initial) plus follow-ups `6c6ce94a`
(`Share SPIRE scan diagnostics walker`), `65f7dad0`
(`Cover SPIRE scan placement SQL deltas`), and `a533eff5`
(`Update SPIRE scan diagnostics validation status`). Re-read against the
post-follow-up state of `scan.rs` and the SQL test in `lib.rs`.

## What landed

- `collect_single_level_scan_placement_diagnostics` (`scan.rs:540-558`) is the
  primary entry: validates the snapshot, loads the root routing object,
  resolves a `SpireSingleLevelScanPlan` from `EcSpireOptions`, and dispatches
  to the validated helper.
- `collect_single_level_scan_plan_placement_diagnostics` (`scan.rs:560-585`)
  takes a pre-resolved `scan_plan`, defensively re-checks `leaf_count` against
  the snapshot's actual root child count, and dispatches to the same helper.
- `collect_validated_single_level_scan_placement_diagnostics`
  (`scan.rs:587-628`) routes to leaf PIDs via
  `route_root_object_to_leaf_pids(query, nprobe)`, walks deltas first
  (`collect_delta_scan_diagnostics_for_base_pid`) to build the deleted-vec-id
  set, then walks the leaf rows
  (`collect_leaf_scan_candidate_diagnostics_for_pid`) using the same set.
- `nprobe == 0` returns an empty `stores` Vec without doing any walking.
- `index_scan_placement_snapshot` in `mod.rs:413-462` is the unsafe FFI entry
  that reads root control, returns empty when `active_epoch == 0`, otherwise
  loads epoch manifests + relation object store and threads the
  `EcSpireOptions` from `options::relation_options(index_relation)`.
- `ec_spire_index_scan_placement_snapshot(index_oid, query)` in
  `lib.rs:1184-1234` is the pgrx wrapper. Same `AccessShareLock` /
  open-then-close / `i64::try_from(...).expect(...)` shape as the active
  snapshot helpers. `effective_nprobe` and `effective_rerank_width_source`
  are duplicated on every per-store row.

## Correctness

- The two-pass walk (deltas first to seed `deleted_vec_ids`, then leaves
  with that set) is required because a single delta object can contain both
  inserts and delete-deltas, and a delete-delta can suppress an insert from
  any other delta object including itself. The implementation buffers
  `delta_object.assignments` into `delta_assignments` after the first pass so
  the visibility re-walk doesn't re-read the delta from disk; clones
  `vec_id` once for the deleted set. Acceptable cost for a diagnostic.
- `should_skip_placement(consistency_mode, state)` is reused, so the
  diagnostic correctly errors out under Strict mode when a placement is
  Stale/Unavailable/Skipped, and silently skips them under Degraded — same
  semantics the real scan uses.
- `read_leaf_scan_rows`, `is_visible_primary_assignment`, and
  `is_delete_delta_assignment` are reused. So the leaf-row body parsing and
  the visibility-bit checks are shared with the real scan.
- `leaf_count` mismatch in `collect_single_level_scan_plan_placement_diagnostics`
  rejects callers that pass a stale `scan_plan`. Good defensive check.
- `expect("delta diagnostics entry should exist")` (`scan.rs:800`) holds
  because the entry is always inserted in the previous loop iteration before
  the assignment vec was pushed. Safe but undocumented; a brief invariant
  comment would help future readers.
- Per-row `effective_nprobe` / `effective_rerank_width` carry duplicated
  scan-plan info. For single-store output this is one row; for multi-store
  output it'll repeat. Fine for a diagnostic surface.

## Drift risk: visibility logic is reimplemented, not instrumented

The big concern. `collect_delta_scan_diagnostics_for_base_pid` and
`collect_leaf_scan_candidate_diagnostics_for_pid` reproduce the manifest
walk + visibility/deletion/dedup gating that the actual quantized routed
scan path runs in
`append_quantized_delta_candidates_for_pid` /
`collect_delta_delete_vec_ids_for_base_pid` /
`append_quantized_v1_leaf_candidates`. Specifically:

- The delta diagnostic re-runs the same `manifest walk → state skip → kind
  filter → parent_pid filter → delete-delta accumulate → visible-primary
  filter` gating chain that the scan does in two separate functions.
- The leaf diagnostic re-runs the same `state skip → kind/parent check →
  visible-primary filter → deleted_vec_ids check` gating chain.

These are not instrumented from inside the scan path; they are a parallel
re-implementation. If the real scan ever adds (for example) a new flag-bit
visibility check, an additional skip class for a new placement state, or
changes the dedupe interaction, the diagnostic counters will silently
disagree with what a real scan would actually count. The diagnostic claims
"visible candidate rows after routed delete-delta suppression" — that claim
is only true while the two implementations stay in sync.

Two possible follow-ups:

1. Refactor the actual scan to a callback-driven walker (`for each visible
   delta candidate { ... }`), then have both the scoring path and the
   diagnostic path consume the same walker.
2. Have the actual scan optionally accumulate counters into a side struct;
   the diagnostic surface then just runs a real scan with the diagnostic
   counter mode.

Worth flagging now even if not addressed in this checkpoint.

## Test coverage

- Unit test `collect_scan_placement_diagnostics_counts_routed_store_rows`
  is well-constructed: builds a base epoch with 2 leaves, then a delta epoch
  containing 1 insert + 1 delete-delta targeting one base row. Asserts:
  - `scanned_pid_count = 2`, `leaf_pid_count = 1`, `delta_pid_count = 1`
  - `delete_delta_row_count = 1`
  - `candidate_row_count = 1`, `leaf_candidate_row_count = 0`,
    `delta_candidate_row_count = 1`
  - so the delete-delta is observed to suppress the matched base leaf row
    while leaving the delta insert visible. This is exactly the property
    the request claims about delete-delta suppression and it's verified.
- PG18 test `test_ec_spire_scan_placement_snapshot_sql` exercises the SQL
  surface end-to-end with a fresh build (no deltas): `nprobe=1` resolves,
  one store row, `leaf_pid_count = 1`, `delta_pid_count = 0`,
  `candidate_row_count > 0`. The query string formatting and `regclass`
  resolution all work.
- Gaps:
  - `nprobe == 0` early-return is unexercised. Trivial to add but currently
    a documentation-only assertion.
  - `leaf_count` mismatch path in
    `collect_single_level_scan_plan_placement_diagnostics` is unexercised.
  - `should_skip_placement` Degraded-mode path in the diagnostic is
    unexercised. Same gap as the static placement snapshot — no unit test
    flips a placement state.
  - No SQL test with deltas + delete-deltas. The unit test is the only
    place the visibility-suppression semantics are end-to-end verified;
    the SQL surface only sees the no-delta case. Worth adding before
    relying on these counters at psql level.
  - Multi-store rows (>1 row in the output) are unverified by either
    layer; only one local store exists today.

## Minor

- The `scan_plan` is carried in the per-store row output (via the pgrx
  wrapper), so all rows from one query share `effective_nprobe`,
  `effective_nprobe_source`, `effective_rerank_width`, and
  `effective_rerank_width_source`. Acceptable, but worth knowing for anyone
  joining these rows in downstream queries.
- `delta_assignments.push((node_id, local_store_id, delta_object.assignments))`
  carries the per-store key alongside the assignments only because the
  borrow on `by_store` was released between the two phases. Fine, but a
  one-line comment explaining the reason would help.

## Follow-up status

Follow-up `6c6ce94a` (`Share SPIRE scan diagnostics walker`) directly
addresses the drift-risk concern above. The change:

- Introduces a `SpireRoutedScanObserver` trait with five hooks:
  `scanned_leaf`, `scanned_delta`, `delete_delta_row`,
  `visible_leaf_candidate`, `visible_delta_candidate`. All hooks have
  empty default impls.
- A `SpireNoopRoutedScanObserver` (zero-cost; empty-impl trait) is used by
  the actual scan path. A `SpireScanPlacementDiagnosticsObserver` owns the
  per-store `BTreeMap` and fans observer events into it.
- `collect_quantized_routed_probe_candidates` now delegates to a new
  `collect_validated_quantized_routed_probe_candidates(observer: &mut
  impl SpireRoutedScanObserver)`. The diagnostic surface
  (`collect_validated_single_level_scan_placement_diagnostics`) now
  invokes that same function with the diagnostics observer, then discards
  the candidate vec — so the diagnostic counts come from the same
  manifest walk, the same `should_skip_placement`, the same
  `is_visible_primary_assignment` / `is_delete_delta_assignment`
  filtering, and the same V1/V2 read fallback that the real scan uses.
- The duplicated walkers (`collect_delta_scan_diagnostics_for_base_pid`,
  `collect_leaf_scan_candidate_diagnostics_for_pid`,
  `store_scan_diagnostics_entry`) were removed.
- Observer hooks fire at three semantic points in
  `append_quantized_leaf_candidates_for_pid` (`scanned_leaf` after the
  parent-pid validation, `visible_leaf_candidate` only after the visibility
  + deleted-vec-id gate in both V1 and V2 columnar paths) and in
  `append_quantized_delta_candidates_for_base_pid` /
  `collect_delta_delete_vec_ids_for_base_pid` (`scanned_delta` once per
  delta object, `delete_delta_row` per delete-delta assignment,
  `visible_delta_candidate` per surviving insert). So the diagnostic
  counters now mechanically track exactly what the scan path scored.

Plus follow-up `6c6ce94a` and `65f7dad0` together close every test gap I
flagged:

- `nprobe == 0` early-return: now exercised at the bottom of
  `collect_scan_placement_diagnostics_counts_routed_store_rows`, asserts
  `stores.is_empty()`.
- `leaf_count` mismatch error: now exercised in the same test
  (`stale_leaf_count_plan` with `leaf_count = 3`, asserts the error
  message contains `"does not match snapshot leaf_count 2"`).
- Degraded-mode skip: new
  `collect_scan_placement_diagnostics_skips_degraded_unavailable_leaf`
  flips `SPIRE_FIRST_PID + 1` to `Unavailable` under
  `Degraded`, sets `nprobe = 2` to cover both leaves, and asserts only
  one leaf was scanned (`scanned_pid_count = 1`, `leaf_pid_count = 1`).
- SQL-level delta scenario: `test_ec_spire_scan_placement_snapshot_sql`
  now does an extra INSERT after build (publishing a delta epoch) and
  asserts `scanned_pid_count = 2`, `leaf_pid_count = 1`,
  `delta_pid_count = 1`, `candidate_row_count = 2`,
  `leaf_candidate_row_count = 1`, `delta_candidate_row_count = 1`,
  `delete_delta_row_count = 0`. So the delete-delta=0 case at SQL level
  is covered. Delete-delta-at-SQL is still open per the packet's own
  scope note (no public SQL surface produces a stable delete-delta
  placement to assert against).

What's still open:

- Multi-store grouping is still untestable (single local store only). No
  surprise.
- `effective_nprobe`/`effective_rerank_width` are still per-row in the
  pgrx output. Documenting only.

## Status

Strong post-follow-up state. The walker is now genuinely shared, so
diagnostic counters mechanically track the real scan; the previously
unexercised branches (`nprobe == 0`, `leaf_count` mismatch, degraded
skip, post-build delta) all have focused tests. This is now the right
shape to extend into multi-store and remote placement when those land.
