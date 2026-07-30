---
task: 172
packet: 004-unshelve-readiness
role: coder
status: review-requested
head: 2963756ab8701adc5c2afb200def2d7e3678efb0
date: 2026-07-29
---

# Review request: Task 172 unshelve readiness

## Requested decision

Please confirm the evidence-only verdict in `verdict.md`:

> **UNSHELVE Task 172.** Its sole shelving prerequisite is satisfied. Task 179's
> real physical owner lane exists, its TC-040/TC-042/TC-050 obligations have
> accepted implementation and validation evidence, and the suite-driven
> topology preflight is fail-closed and has decision-grade 10k/50k/100k proof.

This packet does not run a benchmark, change Task 172's status line, or claim
Task 172 complete. It decides only whether work may resume.

## Condition under review

`plan/tasks/172-ec-distann-real-multinode-benchmark-gate.md` shelves the task
until Task 179's physically sharded lane exists and passes TC-040, TC-042,
TC-050, and the fail-closed topology audit.

Task 179 now states all of the following:

- status `done`;
- its physical fixture and accepted matrix no longer block Task 172;
- the implementation-ready checkpoint unblocked Task 172; and
- packet 059's outside reviewer accepted the aggregate architecture and Task
  179 AC-13, while packet 060 closed the remaining mechanical conditions.

See `plan/tasks/179-ec-distann-physical-hash-shard-generations.md`.

## Evidence map

| Prerequisite | Evidence | Disposition |
| --- | --- | --- |
| Real physical lane exists | `reviews/task-179/031-real-physical-multicluster/feedback/2026-07-12-01-reviewer.md` approves a genuine three-instance fixture: participants load zero source rows, the physical lane has no pruning step, and the old replicated path is explicitly non-topology evidence | Met |
| TC-040 physical handoff/materialization | Task 179 packets 005 and 031 cover streamed transactional handoff, physical owner generations, remote materialization, and in-roster/out-of-roster shapes; packet 059's aggregate review marks the corresponding physical storage/read/topology ACs done | Met |
| TC-042 lifecycle/fault/retention | `reviews/task-179/053-physical-publish-fault-windows/feedback/2026-07-14-01-reviewer.md` accepts the genuine three-process partial-ack and post-ack/pre-swap drills; packets 060/064 harden and reverify the state gate | Met |
| TC-050 persisted/wire formats | `reviews/task-179/002-format-and-control/`, `005-streamed-handoff/`, `006-publication-and-retention/`, and `034-cancelled-generation-recovery/` contain the golden, independent-decode, endian/version, layout, and upgrade evidence; packet 059 explicitly marks AC-2 / TC-050 done | Met |
| Fail-closed topology audit | Packet 031 adds the suite gate so absence or failure prevents accepted downstream evidence; Task 179 requires invalidation for absent, incomplete, unpublished, or owner-disagreeing topology; Task 172 packet 002's outside review accepts exact/disjoint topology at 10k/50k/100k | Met |
| Aggregate acceptance | `reviews/task-179/059-closeout/feedback/2026-07-13-01-reviewer.md` accepts all 13 ACs subject to three mechanical conditions; `reviews/task-179/060-recovery-state-closeout/feedback/2026-07-13-01-reviewer.md` closes those conditions and confirms the done status | Met |

The aggregate PG18 evidence at packet 060 reports 238 passing DistANN tests,
zero failures, and 21/21 passing on-disk fixtures. The current physical matrix
source cited by Task 172 packet 003 reports three successful scales and all
twelve thresholds passing.

## Caveat

`spec/tests.md` still labels TC-040, TC-042, and TC-050 `Planned`. That conflicts
with the later immutable packet evidence, Task 179's reconciled done status, and
the outside closeout decisions. This is stale traceability metadata, not an
absent physical lane or a failed prerequisite.

The matrix labels should be reconciled under the active Task 203/208
program-integrity work before Task 172 closeout. They do not justify keeping
Task 172 shelved.

## Recommended sequencing

1. Unshelve Task 172 now and begin its missing suite capability work:
   throughput/concurrency, injected-RTT characterization, benchmark versus full
   metrics mode, bottleneck attribution, and capacity modeling.
2. Do not run or promote the final Task 172 matrix yet.
3. Require Task 204's per-arm/per-node storage fidelity and Task 208's
   NFR-021/NFR-022 mechanical gates before the final run.
4. Freeze the decision-bearing traversal configuration only after Task 205's
   Algorithm 1 pushdown and Task 206's wide-beam/few-round regime have been
   dispositioned. This prevents Task 172 from characterizing the already
   suspect BW=4/H=100 regime as the program target.
5. Use the owner-traversal lane as the decision-bearing distributed control.
   Single-instance AMs and any coordinator-resident replica may be context only.

## Validation policy

No tests or benchmarks were run. This packet changes documentation only and
audits already committed, outside-reviewed evidence. Running the matrix at this
checkpoint would violate the handoff and would predate the in-flight
measurement-integrity and traversal prerequisites above.
