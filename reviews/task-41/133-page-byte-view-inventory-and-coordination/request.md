# Review Request: Task 41 Invariant #2 page byte-view inventory and coordination

Audit head: `ff4d7b960e3acbd799a3220b4722a68e7927cf4d`

## Summary

This packet starts Phase D by refreshing the current raw-slice inventory and
separating page/buffer byte views from surfaces already handled by earlier
invariant #2 packets.

The inventory confirms the remaining broad surface is page/DSM/message byte
views, not detoast, slot-Datum, palloc scan query, or C-string ownership. It
also identifies local next targets for Phase D without changing buffer release
semantics that overlap invariant #3.

## Classification

- Already handled by Phase C:
  - `src/am/ec_ivf/scan.rs` query and selected-list owner methods.
  - `src/am/ec_hnsw/scan.rs` / `scan_debug.rs` query owner methods.
  - `src/am/ec_hnsw/build_parallel.rs` DSM code/source callback scoping.
- Non-page SQL input buffers:
  - `src/lib.rs` input prefix/code/payload slices are synchronous function
    input views.
- Remaining Phase D page/buffer clusters:
  - `src/am/ec_ivf/page.rs`
  - `src/am/ec_hnsw/{insert,vacuum,graph,shared,scan,scan_debug}.rs`
  - `src/am/ec_diskann/{insert,routine,scan_state}.rs`
  - `src/am/ec_spire/page.rs`
- Remaining DSM/test/message clusters:
  - `src/am/ec_hnsw/build_parallel.rs` initialization/readback/test/message
    slices.

## Scope

- Audit/coordination packet only; no code change.
- Does not claim Phase D complete.
- Does not alter any buffer pin, lock, release, page layout, or DSM ownership
  behavior.

## Evidence

- `artifacts/page-slice-current-inventory.log` is the current raw slice
  inventory.
- `artifacts/page-slice-by-file.log` groups the inventory by file.
- `artifacts/git-status.log` records the audit worktree context.

## Validation

No tests were run because this is an inventory packet with no code change.

## Reviewer Focus

- Confirm the remaining Phase D clusters are correctly classified.
- Confirm the next code packets should stay local by AM/file and avoid changing
  invariant #3 buffer release semantics.
- Confirm Phase D is still open after this packet.
