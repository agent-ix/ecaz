# Task 43 Review Request: SPIRE Serialization Layout Closure

## Summary

This packet closes the remaining pure-subsystem breadth row for SPIRE
serialization/layout helpers.

Promoted to `miri_`:

- assignment row round-trip, prefix/tail decoding, borrowed ref decoding,
  visibility helper semantics, invalid flags, invalid payload format, and
  length mismatch rejection,
- delta partition object insert/delete round-trip and invalid header, invalid
  delta flags, invalid delete payload, and duplicate vec-id rejection,
- local vec-id round-trip, global vec-id payload preservation, and invalid
  vec-id shape rejection.

This is not a final completion packet. Remaining work is cargo-careful
mirroring/blocker documentation, mutation probes, aggregate final lanes, and
the final audit.

## Code Under Review

Code commit: `e555e3c43c2aee809a2a46d9f6885a507c833bd5`

Changed files:

- `src/am/ec_spire/storage/tests/assignment.rs`
- `src/am/ec_spire/storage/tests/delta.rs`
- `src/am/ec_spire/storage/tests/vec_and_routing.rs`

## Validation

Artifacts are packet-local under `artifacts/`; see
`artifacts/manifest.md` for commands and key result lines.

- `miri-spire-assignment-rows.log`: 7 passed; 0 failed.
- `miri-spire-delta-object.log`: 5 passed; 0 failed.
- `miri-spire-vec-id-invalid.log`: 1 passed; 0 failed.
- `miri-spire-local-vec-id.log`: 1 passed; 0 failed.
- `miri-spire-global-vec-id.log`: 1 passed; 0 failed.
- `cargo-fmt-check.log`: exit 0.
- `git-diff-check.log`: exit 0.

## Tracker Update

`reviews/task-43/001-coverage-survey-strategy/artifacts/campaign-tracker.md`
has been updated to mark the last SPIRE serialization/layout breadth row done
and to move the remaining work into packets 012-014.
