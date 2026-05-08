# SPIRE Merge Semantics Closeout

## Summary

This packet marks the Phase 7 merge semantics checklist item complete. The
task note had lagged behind the landed receive, merge, heap-resolution, local
heap-candidate, and coordinator result-summary surfaces.

Changes:

- Marks **Merge semantics** complete in
  `plan/tasks/30-spire-ivf-foundation.md`.
- Records that the production merge helper globally ranks compact candidates,
  dedupes by stable `vec_id`, preserves primary-before-boundary tie behavior,
  validates candidate envelopes, and caps after dedupe.
- Records that `ec_spire_remote_search_merge_order_contract()` exposes the
  comparator order.
- Records that coordinator-local heap resolution and local heap candidates now
  carry ranked candidates through local heap block/offset decoding.
- Records that `ec_spire_remote_search_coordinator_result_summary(...)`
  composes the final local-ready, degraded-ready, and remote-blocked result
  gate.

## Files

- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `f41161b0`

- Tests not run for this docs/status-only checkpoint.

Previously cited coverage for the completed behavior:

- `review/30568-spire-remote-receive-merge-contracts/`
- `review/30569-spire-remote-finalization-contracts/`
- `review/30580-spire-heap-resolution-summary/`
- `review/30582-spire-local-heap-candidates/`
- `review/30584-spire-coordinator-result-summary/`

## Notes

This closeout does not claim remote-origin heap fetch is implemented. Remote
heap resolution stays deferred under the coordinator transport item until libpq
execution can return real remote candidate batches.
