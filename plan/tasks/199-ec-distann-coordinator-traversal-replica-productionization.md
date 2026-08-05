# Task 199: ec_distann Coordinator Traversal Replica Productionization

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **complete — outside-reviewed PROMOTE** (2026-07-26). Priority: P1
production promotion and operations.

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
REPEATABLE READ and SERIALIZABLE scans bypass the optional replica and retain
owner traversal. Writes at every PostgreSQL isolation level use the post-lock
index callback's fresh catalog snapshot: `RowExclusiveLock` conflicts with the
replica builder's `ShareRowExclusiveLock`, so a Ready image cannot commit
between that lookup and the final tuple mutation.

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

## Mandatory Task 198 review findings

Task 198's outside review accepted its measured closeout but found the
following production coherence and operations gaps. Every item is in scope
for this task; the numbered source of record is
`reviews/task-198/005-full-scale-decision/feedback/2026-07-24-01-reviewer.md`.

1. **F1 — real mutation wiring (P1).** Invoke invalidation before every
   published-generation mutation dispatch, including real insert/delete and
   participant tombstone paths. Prove the first retryable `40001` through
   those paths rather than by calling the guard directly.
2. **F2 — coherent construction (P1).** Fence the entire bounded owner copy
   against published-generation mutation. A copy assembled from independent
   snapshots must never self-validate and become Ready.
3. **F3 — bounded side transaction.** The invalidation control transaction
   takes a plain OID, performs only catalog/fence work, has a bounded
   `statement_timeout`, and cannot acquire a pgrx relation lock behind a
   queued `AccessExclusiveLock`.
4. **F4 — reachable retirement.** Ready, Stale, and superseded-epoch replicas
   can all enter Retiring and be reclaimed idempotently; epoch turnover cannot
   leak the previous replica relations.
5. **F5 — snapshot semantics.** Replica selection enforces the isolation level
   required for fresh Stale visibility; REPEATABLE READ/SERIALIZABLE cannot
   select an invalidated replica while owner reads use fresh snapshots.
6. **F6 — deployable authentication and ownership.** Invalidation does not
   depend on the invoking user's passwordless loopback authentication, and a
   caller with normal DML authority is not rejected when no eligible replica
   exists. Any fail-closed dependency has an explicit operator preflight and
   recovery path.
7. **F7 — visible, self-healing fallback.** Ready-image validation/search
   failures emit bounded production diagnostics, record the reason, durably
   demote the image, and then perform a full owner restart. Absence is not
   counted or logged as a replica failure.
8. **F8 — suite normalization.** Validation and expansion apply identical
   effective defaults for `beam_width` and `hop_rounds` in owner/replica
   pairs, so every accepted config remains pairable at runtime.

The review's nine P3 observations are also required closeout items:

1. fallback telemetry does not double-count failed replica work plus owner
   rerun;
2. the mid-scan drill cannot turn a snapshot/query error into
   `fallback_count=1`;
3. the operator API conforms to FR-084 for SECURITY DEFINER, fixed
   `search_path`, PUBLIC revocation, return types, and the stable
   `EC_REPLICA_INVALIDATED` token;
4. build/status/retire/reclaim explicitly reject non-authoritative or
   multi-coordinator use;
5. invalidation targets the exact active identity, not every Ready row for an
   index;
6. the relation-drop race falls back instead of aborting the scan;
7. heterogeneous-ISA ordered identity is tested and final ordering has a
   deterministic `vec_id` tie-break;
8. `benchmark_exact_neighbor` and replica traversal cannot silently select
   different algorithms;
9. normal bootstrap does not create unusable prototype-only replica catalogs.

## Phases

### Scope waiver (2026-07-25)

Task 199 promotion accepts a scoped cross-ISA limitation: the retained
Graviton4/aarch64 run verifies ordered identity and lifecycle/fault handling
within ARM, but does not compare one shared generation's `(distance, vec_id)`
sequence against x86. Cross-ISA final-order equivalence is deferred to a
dedicated shared-generation portability task; this task records the limitation
in its release packet rather than claiming that comparison was performed.

1. **Normal scan selection and feature isolation**
   - Remove `ec_distann.benchmark_traversal_replica` from the selection path.
   - A valid Ready replica is the normal preferred path; absence or any
     validation failure is owner fallback.
   - Keep fault injection, stage counters, and A/B selectors absent from
     normal builds. Remove any prototype-only normal SQL/catalog surface that
     is not part of the accepted operator API.
   - Close F5, F7, and P3 items 1, 6, 7, and 8.
2. **Operator API and authority**
   - Stabilize build/status/retire/reclaim entry points, ownership checks,
     privileges, diagnostics, and idempotency.
   - Require explicit construction; document coordinator disk/WAL headroom,
     build traffic, expected duration, cancellation, and rollback.
   - Reject a second active replica coordinator for the same logical index.
   - Close F3, F6, and P3 items 3, 4, 5, and 9.
3. **Lifecycle and recovery**
   - Cover coordinator restart, owner outage/partial copy, digest mismatch,
     activation and retirement races, disk exhaustion, manual relation loss,
     in-flight scan versus invalidation, and crash-after-control-commit.
   - Prove exactly one first-mutation `40001`, no owner mutation before the
     control commit, the actual post-invalidation retry posture, immutable
     pre-invalidation scan completion, and fail-safe restart recovery.
   - Close F1, F2, F4, and P3 item 2 with real DML, concurrent build/mutation,
     epoch-turnover, lock-queue, authentication, and error-path drills.
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
   - Close F8 in the checked-in suite runner before using the release matrix.

## Acceptance criteria

1. No benchmark selector is required or present in the normal build; a Ready
   replica is used automatically and a non-Ready replica cannot be selected.
2. Operator construction is explicit, privileged, idempotent, cancellable,
   and leaves no eligible partial image.
3. All listed restart/race/exhaustion/mutation drills pass on PG18, including
   genuine mid-scan full restart, exactly one retryable first mutation, zero
   owner mutation on that attempt, and the measured behavior of its retry.
4. Normal observability and documentation disclose the measured storage/WAL/
   build envelope, the single-authority/read-mostly restriction, owner fallback
   for stronger-isolation reads, lock-fenced invalidation for
   stronger-isolation writes, and any fail-closed mutation dependency.
5. Release 10k/50k/100k evidence preserves exact recall/results and confirms
   a material end-to-end benefit; otherwise owner traversal remains normal and
   the productionization stops.
6. Every F1--F8 and P3 item above has direct code/runtime evidence in the
   owning packet; no item is closed solely by documentation or a synthetic
   guard invocation.
7. The final PR receives outside review before any merge. Reviewer findings
   remain open until their fixes and evidence are accepted.

## Required review packets

1. `reviews/task-199/001-normal-selection-and-api/`;
2. `reviews/task-199/002-operations-lifecycle-and-isolation/`;
3. `reviews/task-199/003-release-matrix-and-decision/`.

## References

- Task 198 packets 001--005.
- ADR-085 and ADR-086.
- FR-078 through FR-084 and NFR-017 through NFR-020.
