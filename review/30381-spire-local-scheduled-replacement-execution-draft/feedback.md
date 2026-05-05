# 30381 SPIRE Local Scheduled Replacement Execution Draft — feedback

## What landed

`build_local_scheduled_replacement_epoch_draft` is the local-store
dry-run end-to-end seam: validate execution input against publish plan
→ validate snapshot/decision consistency → write replacement objects →
re-validate placement output against PID plan → assemble publish draft.

## Correctness

- The "validate → write → re-validate placement output" sandwich is the
  right shape: the caller can't supply an invalid input, *and* a writer
  bug that produced misordered placements would be caught before the
  draft assembly commits.
- `SpireLocalScheduledReplacementExecutionInput` carries
  `placement_write_evidence: Vec<...>` so local dry-run callers can
  fabricate evidence in tests; the relation analogue (30382) generates
  the evidence from real `write_placement_entries_to_relation` output.
- This is dry-run only — no root/control advance, no relation writes —
  matches the packet's stated scope.

## Status

Lands cleanly. Will be the unit-test substrate that proves the relation
publisher behaves identically minus the FFI surface.
