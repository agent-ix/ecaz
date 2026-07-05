# 30382 SPIRE Relation Scheduled Replacement Publish — feedback

## What landed

`publish_relation_scheduled_replacement_epoch` is the relation-side
end-to-end publisher: writes relation-backed replacement objects,
validates placements against PID plan, writes the placement directory,
builds the scheduled replacement epoch draft, and publishes through
`publish_replacement_epoch_to_relation`.

## Correctness

- Sequencing is the canonical order:
  validate-input → snapshot-check → write-objects → validate-pid-plan-output
  → build-placement-directory → write-placement-rows → build-draft →
  retire+publish manifest. Identical to the existing
  `publish_relation_replacement_epoch_from_object_placements` shape, so
  scheduled split/merge inherits the existing publish concurrency
  contract.
- `previous_epoch_manifest` is required and checked against
  `*snapshot.epoch_manifest` (line 1562-1567) before any writes — same
  guard as the non-scheduled relation publisher.
- The unsafe boundary is correctly scoped — only the relation writes
  live inside `unsafe { ... }` blocks; pure helpers stay outside.

## Status

Lands cleanly. This is the live execution wire that the eventual
scheduler entry point will call.

## Cross-cutting concern (see 30388)

The execution-input validator does not check that the rewritten
`replacement_parent` actually contains the replacement child PIDs in its
routing table. A caller that forgets to call
`rewrite_scheduled_replacement_parent_routing` would publish an
unrewritten parent. Worth tightening before live scheduler wiring.
