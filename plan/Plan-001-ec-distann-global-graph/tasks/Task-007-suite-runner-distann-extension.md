---
id: Task-007
title: "Suite-runner extension: distann-pipeline step kind + multinode storage summation"
type: Task
status: not_started
track: B
priority: P1
relationships:
  - target: ix://agent-ix/ecaz/NFR-017
    type: references
  - target: ix://agent-ix/ecaz/NFR-018
    type: references
  - target: ix://agent-ix/ecaz/TC-044
    type: verifies
---
# Task-007: Suite-runner extension for the M4 gate

## Scope

Extend `ecaz bench suite` (`crates/ecaz-cli/src/commands/bench/suite.rs`)
with what the M4 gate needs, as its own commit(s) BEFORE Task-005 begins
(FR-038 discipline: extend the runner, never fork into scripts).

## Subtasks

- [ ] **`distann-pipeline` step kind** (mirror of `spire-pipeline`) driving
      the multinode query path with per-node connection targets.
- [ ] **Release-guard whitelist.** Add the new kind to
      `manifest_has_release_guarded_steps` (currently only
      `"latency" | "recall"`, `suite.rs:3959-3961`) — the debug-build trap
      invalidated months of SPIRE numbers; do not repeat it.
- [ ] **Multinode storage summation** for NFR-018 ratio rows (design-doc
      open item).
- [ ] Dry-run/resume/report coverage + unit tests for the new step kind.

## Deliverables

- Runner extension landed on its own branch/commit with focused tests;
  usable from Task-005's SuiteConfig.

## Notes

- Track B: parallel to Track A after M0 fixes the multinode surface shape
  (coordinate the step-kind schema with Task-003's fixture); hard merge
  deadline: before Task-005 starts (Task-005 carries the depends_on edge).
