# 30369 SPIRE Relation Replacement Publish Helper — feedback

## What landed

`publish_relation_replacement_epoch_from_object_placements` accepts already-
written replacement object placements, writes the new placement-directory
rows, builds the validated replacement epoch draft, retires the previous
epoch manifest, writes the new manifest bundle, and advances root/control.

## Correctness

- Retire-before-publish ordering is preserved: `previous_epoch_manifest` is
  required as an input and is checked equal to `*snapshot.epoch_manifest`
  before any writes (line 1634), so a caller that constructed the snapshot
  outside the publish lock cannot retire the wrong manifest.
- The shape mirrors the existing insert/vacuum replacement publish path
  exactly, so split/merge live execution gets the same crash semantics
  (placement entries durable, manifest write fails ⇒ orphan placements
  reclaimed by retention) without inventing new failure modes.

## Status

Lands cleanly. This is the relation-side terminator that 30382's scheduled
publisher composes on top of.
