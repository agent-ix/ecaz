# 30363 SPIRE Replacement Routing Rewrite — feedback

## What landed

`rewrite_routing_partition_for_leaf_replacement` produces a new parent
routing object: removes affected child PIDs, inserts replacement children at
the first affected slot, preserves unaffected child order, reassigns
sequential `centroid_index`, preserves Root vs Internal kind, parent_pid,
level, and dimensions, and rejects replacement PIDs that collide with
unaffected children.

## Correctness

- `validate_replacement_routing_children` (1987-2056) enforces the key
  invariants up front: parent contains every affected PID, replacement PIDs
  are unique and non-zero, replacement PIDs do not collide with non-affected
  children, centroid dimension matches `parent.dimensions`, all centroid
  components are finite. Rebalance's PID-reuse case is allowed because
  `affected.contains(replacement.child_pid)` short-circuits the collision
  check (line 2030).
- "did not find any affected child pid" defensive check at line 907 catches
  the edge case where `validate_replacement_routing_children` passed (no
  duplicates) but the parent's iterator didn't actually visit any affected
  PID — defensive belt-and-braces given the validator already enforces
  presence at 2010.
- `centroid_index` is reassigned strictly by output position via
  `u32::try_from(children.len())`, with overflow rejection.

## Status

Solid. This helper does the real structural work; later scheduled-rewrite
wrappers (30376) just bind the decision shape to it.
