# Verdict: UNSHELVE Task 172

## Decision

Task 172's shelving condition is satisfied.

Task 179 delivered the required physically hash-sharded owner lane and received
outside acceptance for its physical architecture, format discipline,
lifecycle/fault behavior, and exact/disjoint topology. The evidence excludes
the old replicated-serving control: physical owners load no full source/index
replica, no build-then-delete or tombstone-pruning step exists in the physical
lane, and topology is checked before downstream evidence is accepted.

The strongest acceptance chain is:

1. `reviews/task-179/031-real-physical-multicluster/` establishes the real
   physical fixture and fail-closed suite gate.
2. `reviews/task-172/002-physical-multinode-benchmark/` supplies
   outside-accepted exact/disjoint topology at 10k, 50k, and 100k.
3. `reviews/task-179/053-physical-publish-fault-windows/` supplies accepted real
   three-process publication-fault evidence.
4. Task 179 packets 002/005/006/034 supply the TC-050 format evidence.
5. `reviews/task-179/059-closeout/` accepts the aggregate Task 179 architecture
   and AC matrix.
6. `reviews/task-179/060-recovery-state-closeout/` closes the remaining
   conditions and confirms Task 179 done.

Task 179's implementation is therefore a valid prerequisite lane even though
the later Task 203 audit finds its BW=4/H=100 measurement regime suspect. That
regime finding affects which configuration Task 172 should characterize; it
does not resurrect the replicated-topology blocker that caused shelving.

## Recommendation

Change Task 172 from `SHELVED` to `READY` or `IN PROGRESS` by operator decision.
Start the reusable telemetry, concurrency, injected-RTT, and capacity-planning
suite work now.

Do not execute the final decision matrix until:

- Task 204 makes storage evidence per-arm and per-node;
- Task 208 makes NFR-021/NFR-022 admissibility mechanical;
- Task 205 dispositions owner-side threshold/limit pushdown; and
- Task 206 selects the distributed traversal regime to characterize.

Task 207 may proceed independently. If it changes the production head before
the Task 172 run is frozen, the final matrix must identify the exact head
implementation and commit.

## Non-blocking traceability defect

`spec/tests.md` still says TC-040, TC-042, and TC-050 are `Planned`. The status
labels should be reconciled to the accepted Task 179 packets, but the stale
labels do not outweigh the immutable artifacts and outside closeout decisions.

## Scope boundary

This verdict authorizes resuming Task 172. It does not:

- close or promote Task 172;
- promote the existing concurrency-one latency rows;
- treat Task 166 or a traversal replica as a valid decision control;
- waive Task 172's throughput, full telemetry, overhead, or 1m/10m modeling
  requirements; or
- authorize running the benchmark matrix before the sequencing gates above.
