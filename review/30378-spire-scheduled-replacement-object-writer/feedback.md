# 30378 SPIRE Scheduled Replacement Object Writer — feedback

## What landed

`write_local_scheduled_replacement_objects` and
`write_relation_scheduled_replacement_objects` are decision-bound
wrappers around `write_replacement_objects_with_writer`. Both validate
decision shape, immediate-successor epoch, parent-PID, and replacement-
child count *before* delegating to the shared writer.

## Correctness

- Epoch must be exactly `decision.active_epoch + 1`
  (`write_scheduled_replacement_objects_with_writer` line 1841-1849).
  Same successor invariant the publish-draft path enforces in 30379.
- Parent-PID and child-count checks here are redundant with the upstream
  rewrite helper's checks (30376) but cheap and provide defense-in-depth
  for callers that constructed the rewritten parent independently of the
  scheduled rewrite wrapper.
- Local + relation wrappers share `write_scheduled_replacement_objects_with_writer`
  via the `SpireReplacementObjectWriter` trait, so drift between
  unit-tested local and relation publication is structurally impossible.

## Status

Lands cleanly. Trait-based reuse is the right pattern.
