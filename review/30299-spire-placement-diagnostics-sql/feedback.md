# 30299 SPIRE Placement Diagnostics SQL — review

Code commit `4decc7fe` (with follow-up `3126e992`). Read
`src/am/ec_spire/diagnostics.rs`, `src/am/ec_spire/mod.rs`, `src/lib.rs`,
`src/am/mod.rs`, and the task plan diff. Re-checked against the post-follow-up
state of `diagnostics.rs`.

## What landed

- New `collect_store_placement_diagnostics` in `diagnostics.rs:185-315` walks
  the placement directory, groups by `(node_id, local_store_id)` via a
  `BTreeMap` (so output is deterministically ordered), and reads each
  Available placement's object header to bucket bytes by kind.
- `index_placement_snapshot` in `mod.rs:347-395` is the unsafe entry that
  reads root control, returns empty when `active_epoch == 0`, otherwise loads
  the relation epoch manifests, opens a `SpireRelationObjectStore`, and
  converts diagnostic rows to `SpireIndexPlacementSnapshotRow`.
- `ec_spire_index_placement_snapshot(index_oid)` in `lib.rs:1116-1182` is the
  pgrx wrapper. Style matches the existing
  `ec_spire_index_active_snapshot_diagnostics` (`AccessShareLock`,
  open-then-close, `i64::try_from(...).expect(...)` per column).
- Plan and decision-record updates note the new surface and that scan-time
  candidate rows and scanned PID counts remain open.

## Correctness

- Stale / Unavailable / Skipped placements `continue` after incrementing the
  per-state counter, matching the existing `collect_snapshot_diagnostics`
  contract: `placement_object_bytes` covers all placements (incl. stale),
  `available_object_bytes` only covers Available, and the kind buckets sum to
  `available_object_bytes`. The unit test exercises that final invariant
  (`store.available_object_bytes == routing + leaf + delta`), but only when
  every placement is Available, so `placement_object_bytes >
  available_object_bytes` is never exercised.
- `Internal` kind is handled identically to `Root` (routing bytes +
  `routing_child_count`), which is correct, but no test ever produces an
  Internal-kind placement (single-level partitioned input has Root+Leaf only).
  Same gap exists in `collect_snapshot_diagnostics`, so this isn't a
  regression — just worth flagging for the multi-level path.
- Delta kind reads `read_delta_object` and adds `assignments.len()` to
  `assignment_count`. Leaf kind uses `header.assignment_count` (the cheaper
  header field) rather than reading the leaf object body. Both behaviors mirror
  the existing snapshot diagnostics; no extra body reads added.
- Empty-active-epoch path returns `Vec::new()` before opening the object
  store, which avoids a needless `for_index_relation` call. Confirmed by the
  PG18 test on `ec_spire_place_empty_idx`.
- Overflow handling is consistent: every byte and assignment accumulator uses
  `checked_add` with a kind-specific error message. `usize::try_from(header.assignment_count)`
  also bounded.

## Test coverage

- Unit test `store_placement_diagnostics_groups_available_objects_by_store`
  asserts a single store, 3 placements (1 root + 2 leaf), all Available,
  routing_child_count = 2, assignment_count = 2, and the bytes sum invariant.
- PG18 test `test_ec_spire_placement_snapshot_sql` covers empty-index (0 rows)
  and a populated single-store index (1 row, `placement_count = 3`,
  `assignment_count = 3`).
- Gaps:
  - No unit test exercises the Stale / Unavailable / Skipped state branches
    on the new function. The existing `collect_snapshot_diagnostics` has a
    "degraded_unavailable" test that flips a placement to Unavailable; an
    analogous test here would prove `placement_count` includes the unavailable
    row while `available_placement_count`, `object_count`, and the kind
    buckets exclude it. Worth adding before this surface gets relied on.
  - No coverage for Internal-kind (multi-level routing) or Delta-kind objects.
    Acceptable for this checkpoint since the single-level foundation doesn't
    produce them, but the diagnostic shape will go untested for those columns
    until multi-level / delta paths land.
  - Multi-store grouping (>1 row in output) is untested — the local
    single-store path can only produce one `(node_id=0, local_store_id=0)`
    row. The BTreeMap iteration order claim is therefore unverified. Local
    multi-store is in the explicit "remain open" list, so this matches scope.

## Style / minor

- `diagnostics.rs:255-261` and `:267-273` use a temporary `let
  routing_object_bytes = entry.routing_object_bytes.checked_add(...)?; entry.routing_object_bytes = routing_object_bytes;`
  pattern, while the Leaf/Delta branches in the same function and the entire
  `collect_snapshot_diagnostics` use direct assignment. Not a borrow-checker
  necessity (Leaf/Delta show direct assignment works on the same `&mut entry`).
  Worth collapsing for consistency.
- `node_id` and `local_store_id` are stored both in the `BTreeMap` key and as
  fields of `SpireStorePlacementDiagnostics`. Redundant but harmless; the
  current shape lets `into_values().collect()` produce the row vec without
  re-hydration.
- The two collector functions (`collect_snapshot_diagnostics` and
  `collect_store_placement_diagnostics`) duplicate ~80 lines of state-branch
  + kind-branch logic. They have to stay in sync — if a new placement state or
  object kind is added, both must be updated. Not blocking, but a candidate
  for a shared `for placement in ... { ... }` helper later.

## Follow-up status

Follow-up commit `3126e992` (`Cover SPIRE degraded placement diagnostics`)
already addressed the two main items above:

- New unit test
  `store_placement_diagnostics_counts_degraded_skipped_objects_without_reading_them`
  flips `SPIRE_FIRST_PID + 1` to `Unavailable` and `SPIRE_FIRST_PID + 2` to
  `Skipped` under a `Degraded` epoch manifest, then asserts:
  `placement_count = 3`, `available_placement_count = 1`,
  `unavailable_placement_count = 1`, `skipped_placement_count = 1`,
  `object_count = 1`, `placement_object_bytes > available_object_bytes`,
  `available_object_bytes == routing_object_bytes`. So the
  state-counter behavior is now end-to-end exercised, including the key
  invariant that non-Available bytes accumulate in `placement_object_bytes`
  but are excluded from per-kind buckets and `available_object_bytes`.
- The Stale state path remains unexercised in this test, but the packet
  notes (correctly) that a validated published Degraded snapshot rejects
  Stale at construction time, so the `Stale` arm of
  `collect_store_placement_diagnostics` is unreachable through this entry
  point. That makes the `assert_eq!(stale_placement_count, 0)` check the
  only meaningful coverage available, which is fine.
- The temp `let routing_object_bytes = entry...; entry.routing_object_bytes
  = routing_object_bytes;` pattern in the Root/Internal arms was collapsed
  to direct assignment, and the duplicated `"ec_spire placement
  diagnostics routing byte count overflow"` literal was extracted into a
  shared `placement_routing_byte_count_overflow()` helper. Style nit
  closed.

What's still open:

- Internal-kind coverage. Single-level partitioned input doesn't produce
  Internal placements; this gap will only be closable when multi-level
  routing arrives.
- Delta-kind coverage in this surface. None of the placement-snapshot
  tests build an epoch with delta objects, so the Delta arm of the byte
  and assignment accumulators is still dead code as far as these tests
  are concerned. The scan-placement diagnostic does cover delta semantics
  (in 30300 + its follow-up), so this is a per-surface gap, not a
  system-wide one.
- Multi-store grouping (>1 row in the BTreeMap iteration) — still
  blocked by local single-store constraints, matches scope.
- Code-level duplication with `collect_snapshot_diagnostics` is unchanged.
  Not blocking, but two collectors still walk the same placement
  directory with the same state/kind branching.

## Status

Lands cleanly post-follow-up. The state-counter behavior I flagged as
unexercised is now covered. Remaining gaps (Internal kind, Delta kind in
this specific surface, multi-store grouping) are bounded by what the
single-level local single-store path can actually produce; they will
close as those features land.
