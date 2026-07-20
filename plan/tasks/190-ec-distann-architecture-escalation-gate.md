# Task 190: ec_distann Architecture Escalation Gate

Status: **proposed, dormant until Tasks 184--189 report** (2026-07-19).
Priority: P3 decision task.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidate
families `ARCH-01` through `ARCH-15` and deferred `TRAV-28` through `TRAV-30`.

## Why

The roadmap includes valuable but invasive options: a replicated routing layer,
boundary-node replication, alternate placement, packed row tier, binary RPC,
accelerators, coordinator routing, and workload-aware payload replication.
They affect formats, lifecycle, failure semantics, deployment, or the core
FR-078/ADR-085 architecture. They should not leak into narrow optimization
tasks or be implemented merely because smaller work is difficult.

## Goal

Given complete Tasks 184--189 evidence, decide whether a material remaining gap
justifies one architectural direction. Produce an accepted ADR/design and a
separately numbered implementation task, or STOP without implementation.

This task is a decision gate, not a bundled architecture rewrite.

## Entry gate

The entry packet must summarize:

- retained recall/latency/storage Pareto points after Tasks 184--189;
- stage and query-level attribution of the remaining gap;
- all relevant negative results;
- 1m scaling evidence when the preceding task's trigger permits it;
- operational/deployment constraints; and
- why no narrower candidate can address the measured dominant cost.

If this case is not established, close without selecting an architecture.

## Candidate narrowing

Select at most two architecture families for design comparison, from:

1. replicated/central routing layer (`ARCH-01`, `ARCH-02`, `TRAV-28`--`TRAV-30`);
2. alternate placement or boundary replication (`ARCH-03`, `ARCH-04`);
3. row/payload storage redesign (`ARCH-05`, `ARCH-06`, `ARCH-15`);
4. dedicated transport (`ARCH-07`, `ARCH-08`);
5. accelerator/query batching (`ARCH-09`--`ARCH-11`); or
6. coordinator/operational routing (`ARCH-12`--`ARCH-14`).

The comparison must quantify the expected ceiling from measured counters,
storage/build/network amplification, DML impact, lifecycle/recovery changes,
failure model, deployment constraints, compatibility, and rollback.

Small feature-gated prototypes are permitted only when a disputed feasibility
or performance premise cannot be decided from existing evidence. Any matrix
uses `ecaz bench suite`; prototypes never become production defaults here.

## Decision

Choose one architecture only when it has a credible measured path to a material
end-to-end improvement that narrower work cannot provide, with bounded work and
an acceptable correctness/operations/storage cost. Record it in a new ADR or
explicit ADR-085 supersession and author a separate implementation task with
milestones and full 10k/50k/100k gates.

Otherwise record STOP and retain the current architecture. Do not select an
architecture merely to keep the program active.

## Required review packets

1. `reviews/task-190/001-entry-and-residual-case/`;
2. `reviews/task-190/002-architecture-comparison/`;
3. `reviews/task-190/003-adr-and-task-decision/`.

## Non-goals

- Implementing more than a narrow feasibility prototype.
- Combining replication, placement, transport, storage, and accelerators.
- Reopening candidates rejected unchanged by Tasks 180--189.
- Task 167 physical DML or Task 172 general capacity work.

## References

- Tasks 179--189 and their accepted evidence.
- ADR-085 and FR-075 through FR-083.
- `plan/design/ec-distann-recall-latency-roadmap.md`.
