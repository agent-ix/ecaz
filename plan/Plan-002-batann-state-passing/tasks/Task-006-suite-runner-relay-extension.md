---
id: Task-006
title: "Suite-runner relay extension — coordination-mode axis + counter emission"
type: Task
status: not_started
track: B
priority: P0
relationships:
  - target: ix://agent-ix/ecaz/FR-084
    type: references
  - target: ix://agent-ix/ecaz/NFR-022
    type: references
  - target: ix://agent-ix/ecaz/TC-048
    type: verifies
---
# Task-006: suite-runner relay extension

## Scope

Extend `crates/ecaz-cli/src/commands/bench/suite.rs` with the
coordination-mode axis and FR-084 relay-counter emission into results.jsonl
(pre-registered field schema), against the multi-instance distributed step
kind (task-172 lineage — the `distann-pipeline` step kind cited by
NFR-017/TC-044 does not exist yet and this task provides the concrete
step). Lands as its own commit per the FR-038 suite rule.

## Subtasks

- [ ] Mode axis (GUC set per step) + release-guard whitelist entry.
- [ ] results.jsonl relay fields: relay_hops, relay_depth_max,
      relay_depth_histogram, state_bytes_max/total, drains_executed,
      head_descents, handoffs_per_node, fallback_resumed, relay_journeys.
- [ ] Dry-run/audit/resume compatibility.

## Deliverables

- Suite extension commit (own SHA, recorded in the B4 packet manifest).

## Notes

- Track B; independent of Track A until its merge deadline (before
  Task-005 starts).
