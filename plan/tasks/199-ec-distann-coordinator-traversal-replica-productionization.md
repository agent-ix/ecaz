# Task 199: ec_distann Coordinator Traversal Replica Productionization

Status: **proposed by Task 198** (2026-07-23). Priority: P1 production
promotion and operations.

## Why

Task 198's feature-gated faithful replica preserved exact production recall at
10k/50k/100k and improved warm mean latency by 15.9%/14.0%/17.0%, with
similar p95 improvements and causally reconciled removal of remote traversal.
The benefit is real, but so is the cost: replica storage is about 65–66% of
the physical generation (1.660 GB at 100k), 100k emits 1.926 GB WAL, and the
100k build takes 52.0 seconds. The first mutation while a replica is Ready
must also return one retryable `40001` after durable invalidation.

Task 198 deliberately does not change normal production behavior. This task
owns the separately reviewed operator and release decision required by
ADR-086.

## Goal

Productionize the replica as an **explicit-build, single-authority,
read-mostly capability**. When an operator has built and activated a valid
Ready replica for the pinned generation, normal scans prefer it. Missing,
partial, stale, corrupt, retiring, or removed replicas use unchanged owner
traversal. Replica construction is not automatic in this task.

## Frozen behavior

- Keep Task 198's catalog identity, canonical content digest, graph/exact-
  vector representation, bounded streaming build, traversal core, and full
  owner restart semantics.
- Keep the Task 195/196 production point: trained cap-4,096 exact head,
  ordered 32 seeds, degree 32, BW4/H100, RaBitQ neighbor values, exact final
  scoring, identity-keyed lazy10 owner payload materialization.
- Keep exactly one authoritative coordinator per logical index.
- Do not add payload replication, mutation propagation, multi-coordinator
  invalidation, compact packing, sparse replication, or a new codec.

## Phases

1. **Normal scan selection and feature isolation**
   - Remove `ec_distann.benchmark_traversal_replica` from the selection path.
   - A valid Ready replica is the normal preferred path; absence or any
     validation failure is owner fallback.
   - Keep fault injection, stage counters, and A/B selectors absent from
     normal builds. Remove any prototype-only normal SQL/catalog surface that
     is not part of the accepted operator API.
2. **Operator API and authority**
   - Stabilize build/status/retire/reclaim entry points, ownership checks,
     privileges, diagnostics, and idempotency.
   - Require explicit construction; document coordinator disk/WAL headroom,
     build traffic, expected duration, cancellation, and rollback.
   - Reject a second active replica coordinator for the same logical index.
3. **Lifecycle and recovery**
   - Cover coordinator restart, owner outage/partial copy, digest mismatch,
     activation and retirement races, disk exhaustion, manual relation loss,
     in-flight scan versus invalidation, and crash-after-control-commit.
   - Prove exactly one first-mutation `40001`, no owner mutation before the
     control commit, retry through owners, immutable pre-invalidation scan
     completion, and fail-safe restart recovery.
4. **Production observability**
   - Expose bounded status for identity/state, bytes, build time, last error,
     Ready/Stale reason, pins, and reclaim eligibility without benchmark
     instrumentation.
   - Add operator documentation for read-mostly rollout and removal.
5. **Release promotion gate**
   - Build/install a normal PG18 release binary and prove feature isolation.
   - Run a checked-in `ecaz bench suite` before/after matrix at
     10k/50k/100k: 200 recall queries / 2,000 trials, 10 warmups / 50 timed
     samples, storage, build/WAL/cache, topology, ordered identity, fallback,
     mutation, and operator lifecycle evidence.
   - Promote only if the normal path reproduces Task 198's recall identity and
     material latency improvement within the disclosed capacity envelope.

## Acceptance criteria

1. No benchmark selector is required or present in the normal build; a Ready
   replica is used automatically and a non-Ready replica cannot be selected.
2. Operator construction is explicit, privileged, idempotent, cancellable,
   and leaves no eligible partial image.
3. All listed restart/race/exhaustion/mutation drills pass on PG18, including
   genuine mid-scan full restart and exactly one retryable first mutation.
4. Normal observability and documentation disclose the measured storage/WAL/
   build envelope and the single-authority/read-mostly restriction.
5. Release 10k/50k/100k evidence preserves exact recall/results and confirms
   a material end-to-end benefit; otherwise owner traversal remains normal and
   the productionization stops.

## Required review packets

1. `reviews/task-199/001-normal-selection-and-api/`;
2. `reviews/task-199/002-operations-lifecycle-and-isolation/`;
3. `reviews/task-199/003-release-matrix-and-decision/`.

## References

- Task 198 packets 001--005.
- ADR-085 and ADR-086.
- FR-078 through FR-084 and NFR-017 through NFR-020.
