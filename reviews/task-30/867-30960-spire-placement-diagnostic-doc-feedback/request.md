# Review Request: SPIRE Placement Diagnostic Doc Feedback

## Summary

Accepts reviewer follow-up notes from packets 30956, 30957, and 30958.

This docs-only slice clarifies:

- `selection_ordinal` in
  `ec_spire_index_selected_pid_placement_snapshot(...)` is the 1-indexed input
  array position;
- selected-PID placement answers where a PID would be served from, not whether
  end-to-end reads are currently possible;
- fewer selected-PID rows than requested should send operators toward
  object-version mismatch checks;
- boundary-replica identity `node_id` and `local_store_id` are placement
  identity, not the local metadata read source; and
- freshness next actions `persist_remote_epoch_manifest` and
  `refresh_remote_epoch_manifest` are performed with
  `ec_spire_persist_remote_epoch_manifest(index_oid)`.

## Files

- `docs/SPIRE_DIAGNOSTICS.md`

## Validation

Packet-local logs are in `artifacts/` and indexed by
`artifacts/manifest.md`.

- `git diff --check dd2e0faf^ dd2e0faf`

No runtime tests were run; this is operator documentation only.

## Reviewer Focus

- Confirm the added wording addresses the actionable doc notes from 30956,
  30957, and 30958 without broadening the claimed diagnostic guarantees.
