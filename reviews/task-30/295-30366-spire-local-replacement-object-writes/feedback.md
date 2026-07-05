# 30366 SPIRE Local Replacement Object Writes — feedback

## What landed

`write_local_replacement_objects` (and the shared
`write_replacement_objects_with_writer`) writes the rewritten parent
routing object and replacement V2 leaf objects through
`SpireLocalObjectStore`, returning placements ordered by replacement
routing children.

## Correctness

- `replacement_parent.header.kind` is checked to be Root or Internal (line
  1887) before any writes, so a misrouted leaf object cannot be promoted.
- Leaf placements are produced strictly in
  `replacement_children`-iteration order via the
  `inputs_by_pid` HashMap lookup per child; the returned vec ordering is
  therefore the same as the parent's child ordering, which is what 30364's
  placement directory and 30380's PID-plan-output validator both rely on.
- `write_replacement_leaf_object_v2_from_rows` carries
  `replacement_parent.header.pid` as `parent_pid`, so leaf objects are
  durably backref'd to the rewritten parent's PID (not the old parent).
- `epoch == 0` and `leaf_object_version == 0` rejection are early.

## Status

Lands cleanly. The shared writer trait used here is the same one the
relation store implements in 30367, so the local + relation paths cannot
drift.
