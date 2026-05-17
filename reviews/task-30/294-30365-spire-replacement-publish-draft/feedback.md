# 30365 SPIRE Replacement Publish Draft — feedback

## What landed

`build_replacement_epoch_draft_from_object_placements` turns a planned
replacement placement directory plus durable placement-write evidence into
a `SpireReplacementEpochDraft`: epoch manifest, object manifest, validated
epoch snapshot, root/control state, and encoded publish bundle. Reuses the
existing publish coordinator's evidence-shape checks before live
split/merge relation publishing is wired.

## Correctness

- The draft's publish input flows through `publish_input()` so retire +
  manifest write + root/control advance use the same coordinator as the
  vacuum and insert replacement paths. No new publish-state machine.
- Replacement leaf-input validation is reused (single source of truth for
  "no delta-insert flags, no duplicate `vec_id`s, only visible-primary
  rows"), so the draft assembly cannot accept a leaf-input that the live
  scheduler should have already rejected.

## Status

Good. This is the integration point where every later scheduled-publish
helper terminates.
