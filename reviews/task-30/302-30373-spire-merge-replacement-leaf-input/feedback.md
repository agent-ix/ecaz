# 30373 SPIRE Merge Replacement Leaf Input — feedback

## What landed

`build_merge_replacement_leaf_object_input` validates merge decision
shape, requires exactly one fresh replacement PID, combines folded rows
from all selected affected leaves in decision order, and rejects missing,
duplicate, or unselected base PID row groups. Output runs through
`validate_replacement_leaf_object_inputs`.

## Correctness

- "Unselected base pid" rejection (line 467) catches the bug where the
  caller routed rows from a leaf the decision didn't actually pick up.
- "Missing rows for base pid" rejection (line 487) catches the dual case
  where a decision-listed leaf has no rows supplied — including the
  legitimate empty-leaf case, which is rejected here. That's correct: a
  merge of an empty leaf is meaningless and the scheduler should not pick
  empty leaves.
- Output-order preservation: rows are appended in `decision.affected_leaf_pids`
  order (line 485-491). The replacement-input validator does not depend
  on inter-leaf ordering, but downstream determinism (e.g., for stable
  test output) benefits.

## Note

Centroid is `Vec::new()` in the synthetic `SpireRoutingReplacementChild`
passed to the validator (line 500). Intentional —
`validate_replacement_leaf_object_inputs` does not inspect centroid
contents; centroid training is a separate live-scheduler concern. A
one-line comment on the empty-centroid pass-through would help readers.

## Status

Solid.
