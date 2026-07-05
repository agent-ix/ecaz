# 30376 SPIRE Scheduled Routing Rewrite — feedback

## What landed

`rewrite_scheduled_replacement_parent_routing` validates decision shape,
rejects parent-PID mismatch and replacement-child count mismatch, then
delegates to `rewrite_routing_partition_for_leaf_replacement` (the
existing structural rewriter from 30363).

## Correctness

- Wrapper-only logic: no new structural mutations, just decision-binding.
  The underlying rewriter still owns affected-child membership checks,
  PID collision rejection, dimension/finiteness checks, and Root vs
  Internal kind preservation.
- Parent-PID-vs-decision check (line 689-694) is the right early bail —
  catches the bug where a scheduler reloaded a different parent than the
  decision targeted.

## Status

Lands cleanly. Thin and well-typed.
