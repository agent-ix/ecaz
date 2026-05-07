# 30360 SPIRE Phase 1 Validation Checklist

## Request

Review the Task 30 checklist update that closes the Phase 1 behavior-validation
item while leaving the landing review/measurement packet open.

## Scope

- Marked Phase 1 `Validation` complete.
- Kept the final `Review packet` item open for packet-local logs and
  recall/latency sanity evidence.
- Clarified that physical page reclamation and old-epoch cleanup remain
  separate follow-ups.

## Rationale

The Phase 1 behavior-validation matrix now has focused PG18 or unit coverage
for:

- empty and populated build/scan
- TurboQuant and RaBitQ scannable paths
- empty PQ-FastScan no-row safety and populated PQ-FastScan deferral
- insert-after-build and empty-index insert bootstrap
- delete deltas and SQL VACUUM compaction
- diagnostics surfaces for active, options, placement, scan placement, root
  routing, hierarchy, object, epoch, leaf, delta, allocator, health, and
  insert debt
- same-leaf insert concurrency and heterogeneous insert/VACUUM/scan
  concurrency
- retired/bundle residue diagnostics and scan root-control cache refresh

The final landing review packet remains open because it needs packet-local
validation logs and a small recall/latency sanity row.

## Validation

- `git diff --check`

Docs-only change; no tests were run.
