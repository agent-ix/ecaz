# 30388 SPIRE Local Scheduled Publish Plan Validation — feedback

## What landed

`validate_local_scheduled_replacement_execution_publish_plan` is now
required at `build_local_scheduled_replacement_epoch_draft` entry (line
1513). Mirrors the relation-side validator from 30386. Local + relation
publish-plan validation paths are now symmetric.

## Correctness

- The local dry-run draft now refuses to write objects if the execution
  input has drifted from the publish plan — same closed-loop guarantee
  as the relation publisher.
- Three independent enforcement seams for consistency mode (manifest →
  publish plan → execution input) now match in both local and relation
  paths.

## Status

Lands cleanly.

---

# Phase 2 Helper Arc Cross-Cutting Review (covers 30362–30388)

## Architectural assessment

Coder is on track. The Phase 2 helper progression is composing cleanly:

```
choose_decision (30370) → recheck (30372) → plan_pids (30371) →
plan_publish (30383) → build_routing_children (30375) →
rewrite_parent (30376) → build_leaf_inputs (30373/30374) →
build_execution_input_from_plan (30385/30387) →
validate_against_plan (30386/30388) → write_objects (30378) →
validate_pid_plan_output (30380) → build_draft (30377/30379/30384) →
publish_relation (30382)
```

Each step is pure where possible, decision-bound, and reuses shared
validators (`validate_leaf_replacement_schedule_decision_shape`,
`validate_replacement_leaf_object_inputs`,
`validate_scheduled_replacement_execution_publish_plan_parts`) so the
local + relation paths cannot drift.

## Strengths

1. **Defense in depth without redundancy**: validators run at every seam
   that could be entered independently. Successor-epoch and consistency-
   mode constraints are each enforced at ≥3 places, but the duplication
   is at *seams*, not in the same code path twice.
2. **Allocator-cursor discipline**: cursor advances are commit-on-success
   in `plan_leaf_replacement_pids`, and `plan_scheduled_replacement_publish_epoch`
   re-checks all PIDs against `root_control.next_pid` and
   `pid_plan.next_pid`. Concurrent PID-allocation regressions are
   structurally rejected.
3. **Trait-based local/relation parity**: `SpireReplacementObjectWriter`
   means there's exactly one writer code path; the local + relation
   wrappers are typed-thin.
4. **Test coverage per slice is complete**: every helper has positive +
   stale + drift + count-mismatch + cursor-regression rejection coverage.

## Concerns

### Cross-cutting: replacement_parent contents are not re-validated

The execution-input validators (`validate_*_scheduled_replacement_execution_publish_plan`
and the shared `_parts` validator at 1411-1477) check
`replacement_parent.header.pid` against the decision's parent PID, and
they check that `replacement_children`'s PIDs match the PID plan, but
they do **not** check that the `replacement_parent` routing object's
`children()` list actually contains those replacement child PIDs (and
excludes the affected leaf PIDs).

Today, that property is only enforced by
`rewrite_routing_partition_for_leaf_replacement` →
`validate_replacement_routing_children` *at the moment the rewritten
parent is constructed*. If a future caller (live scheduler) constructed
`SpireRelationScheduledReplacementExecutionParts` with the
unrewritten parent — or with a parent rewritten for a different decision
— the validators would not catch it, and the writer would publish an
unrewritten parent that still references the affected leaves. Scans on
the new epoch would then route to the (about to be removed) old
leaves.

Suggested fix: in
`validate_scheduled_replacement_execution_publish_plan_parts`, after
checking `replacement_parent.header.pid`, walk `replacement_parent.children()`
and assert:
- every `replacement_children[i].child_pid` is present in the parent's
  child list
- no `decision.affected_leaf_pids[j]` is present in the parent's child
  list
- (optional) `replacement_parent.dimensions == parent's expected
  dimensions`

This is cheap (children list is small for the single-level root) and
makes the execution-input self-validating rather than relying on
"caller-must-have-called-the-rewriter" discipline. Worth tightening
before the live scheduler entry point lands.

### Minor

- `recheck_leaf_replacement_schedule_decision` (30372) implicitly assumes
  `choose_leaf_replacement_schedule` is *deterministic and stable* across
  selector tweaks. A comment binding the two functions would prevent
  future drift.
- `build_merge_replacement_leaf_object_input` and
  `build_split_replacement_leaf_object_inputs` pass empty centroids to
  `validate_replacement_leaf_object_inputs`; the validator doesn't
  inspect centroid contents but a one-line comment on the empty-centroid
  pass-through would help readers.
- `validate_relation_…_publish_plan` and `validate_local_…_publish_plan`
  duplicate three identical leading checks (epoch, consistency_mode,
  next_local_vec_seq). Could fold into a generic, but the duplication is
  short.

## What's still open in Phase 2

- Live scheduler entry point (manual SQL, VACUUM hook, or background
  worker — design doc keeps this open).
- Centroid training for split / merge — currently a live-scheduler
  responsibility, no helpers cover it yet.
- Concurrency stress beyond the existing same-leaf-insert harness:
  delete overlap and longer mixed insert/delete/scan/split/merge.

## Status

The helper arc is in good shape. The only flag worth acting on before
live scheduler wiring is the **replacement_parent contents
re-validation** above; everything else is polish.
