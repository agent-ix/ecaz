# Task 190: ec_distann Architecture Escalation Gate

Status: **decision authored — select coordinator traversal replica for Task
198; awaiting outside review** (2026-07-23). Priority: P3 decision task.

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

## Activation and scope

The operator activated this gate on 2026-07-23 after Task 194 completed the
missing nine-way traversal attribution and Tasks 195--197 landed the retained
production baseline and benchmark-integrity guard.

This is a **latency-only** architecture decision. Tasks 185, 186, 188, and 189
remain independent recall/head/codec work and are neither closed nor
superseded here. Their remaining candidates cannot remove the measured
sequential transport wait without changing the architecture premise. Task 190
therefore preserves their search/recall contracts and selects no recall
policy.

## Entry gate

The entry packet must summarize:

- retained recall/latency/storage Pareto points after Tasks 184--189;
- stage and query-level attribution of the remaining gap;
- all relevant negative results;
- 1m scaling evidence when the preceding task's trigger permits it;
- operational/deployment constraints; and
- why no narrower candidate can address the measured dominant cost.

If this case is not established, close without selecting an architecture.

The case is established for latency:

- accepted Task 195 release evidence retains 0.9990 / 0.9685 / 0.9625 recall
  and 20.90 / 20.90 / 19.90 ms warm mean at 10k / 50k / 100k;
- accepted Task 194 attribution records ten sequential traversal rounds,
  7.429 ms remote expansion, 2.259 ms owner service, and 5.013 ms transport
  wait per 100k scan; the lighter-observer run records 4.078 ms transport
  wait;
- connection, request encode, and receive/decode total only 0.071 ms/scan,
  while logical request/response volume is only 13.9 / 10.5 KiB; and
- the bounded BW8/H50 candidate cut rounds and transport wait but left warm
  mean flat and regressed p95, showing that more work per round is not an
  end-to-end answer.

No 1m run was triggered: Task 194's isolated candidate failed its usefulness
gate at 100k. The absence is a pre-registered conditional skip, not an
environment deferral.

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

## Decision rule

Choose one architecture only when it has a credible measured path to a material
end-to-end improvement that narrower work cannot provide, with bounded work and
an acceptable correctness/operations/storage cost. Record it in a new ADR or
explicit ADR-085 supersession and author a separate implementation task with
milestones and full 10k/50k/100k gates.

Otherwise record STOP and retain the current architecture. Do not select an
architecture merely to keep the program active.

## Decision

Compare only:

1. `ARCH-02` / `TRAV-28`: a fingerprint-bound, coordinator-resident traversal
   replica derived from the Published generation; and
2. `ARCH-07`: a dedicated binary traversal transport.

Select the coordinator traversal replica for a separately gated implementation
in Task 198. It is the only compared family with a direct path to removing the
measured per-round wait rather than merely changing serialization around it.
The replica is a rebuildable derived artifact, never an owner/source of truth,
and retains owner-side lazy payload materialization.

The first faithful implementation may cost up to one additional physical
generation per coordinator (2,496,626,688 bytes at 100k). A later compact
traversal image has a measured-byte lower envelope of about 1.445 GB at the
same corpus/dimension/degree, but that is a design estimate, not benchmark
evidence. Task 198 must measure actual bytes and may not claim the estimate.

Incremental graph mutation invalidates the replica and forces the existing
owner traversal path until a later task supplies a reviewed coherence
protocol. Missing, stale, partial, or digest-mismatched replicas also fall
back; no partial-result success is permitted.

ADR-086 records the decision. Task 198 owns prototype, lifecycle/failure
semantics, same-query identity, and 10k/50k/100k promotion gates. Task 190
implements no production behavior.

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
