# Task 173: BatANN Spec Authoring — State-Passing Coordination for ec_distann

Status: in progress (2026-07-09). Depends on: Task 161 (distann spec batch),
tasks 162–165 implementation context on branch `task-165-ec-distann-m3`.
Owner: spec lane, branch `task-173-batann-specs`. One coder, one branch.
Priority: P1 — plans the reopened ADR-085 D4 lane; no implementation.

## Why

The coordinator loop pays one per-node RTT per hop round by construction
(FR-081), and ADR-085 D4 deferred BatANN baton passing with a measured
reopen trigger. The BatANN paper (arXiv:2512.09331) shows 1.44–2.09×
DistributedANN throughput at 1B/10 servers by forwarding the full query
state to the node owning the next frontier candidate. Speccing the
mechanism as a query-time mode turns the D4 trigger from a projection into
a direct A/B.

## Goal

A reviewed, quire-clean spec batch: ADR-086 (reopens ADR-085 D4),
FR-084..FR-089, NFR-021..NFR-022, TC-045..TC-048 matrix rows, design doc
`plan/design/batann-state-passing-coordination.md`, milestone task files
174–178, and the Plan-002 bundle.

## Scope

- Design doc (normative home of milestones B0–B4).
- `/specify`: ADR-086 D1–D11; FR-084 mode selection; FR-085 relay-state
  wire format; FR-086 relay endpoint + local drain (Algorithm 2); FR-087
  stack return; FR-088 direct return (shmem mailbox); FR-089 depth budget +
  hybrid terminal resume; NFR-021 relay resource bounds; NFR-022 three-way
  mode bench gate.
- `/spec-matrix`: TC-045..TC-048 + coverage/permutation rows in
  `spec/tests.md`.
- `/spec-review`: seven dimensions (base + failure-domain, integrity,
  dependency, evidence, risk-complexity, scope-boundary) under
  `spec/reviews/batann/`, findings reconciled into the specs.
- Task files 174–178 + README index; `plan/Plan-002-batann-state-passing/`
  bundle via `/spec-to-plan`.

## Required Evidence

Quire validation clean over the batch; every AC mapped in the matrix;
all review findings dispositioned in the SR docs' Reconciliation sections;
packet `reviews/task-173/001-batann-spec-batch/`.

## Non-Goals

Any code change (`src/**`, `crates/**`); implementation starts at Task 174.

## Acceptance Criteria

1. Spec batch merged to the lane with review packet accepted.
2. All seven SR docs carry full disposition tables (no OPEN findings).
3. Milestone→task mapping 173–178 recorded in the design doc and README.

## References

- ADR-086, FR-084..089, NFR-021/022; ADR-085 D4
- `plan/design/batann-state-passing-coordination.md`
- BatANN paper arXiv:2512.09331; DistributedANN arXiv:2509.06046
