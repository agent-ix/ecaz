# Task 198: ec_distann Coordinator Traversal Replica

Status: **proposed by Task 190 / ADR-086** (2026-07-23). Priority: P2
architecture implementation and measurement.

## Why

Accepted Task 194 attribution isolates ten sequential traversal rounds and
4.078--5.013 ms/scan of transport wait at 100k. The retained Task 195
production point is 0.9625 recall / 19.90 ms warm mean. BW8/H50 reduced rounds
but did not improve end-to-end latency usefully and regressed p95. Task 190
therefore selected one invasive direction: execute the unchanged traversal
against a fingerprint-bound coordinator replica, while leaving final payload
materialization owner-side.

## Goal

Implement and measure an optional, rebuildable coordinator traversal replica
for one Published generation. Advance it only if it preserves exact production
semantics and produces a material end-to-end Pareto improvement after its
storage, lifecycle, failure, cache, and build costs are included.

This task does not promote a default merely because the prototype works.

## Frozen control

- current production `training_landmarks_exact`, cap 4,096, exact head score,
  ordered 32 seeds;
- graph degree 32, BW4/H100, RaBitQ stored neighbor values, exact final
  distance, convergence and work caps;
- production owner-schema cache and identity-keyed lazy10 payload reuse;
- hash ownership, active-epoch identity, scan fencing, failure semantics, and
  final owner payload path; and
- Task 195 query identities and exact/disjoint three-owner topology.

## Phase 1: contract and faithful prototype

1. Define replica catalog/state keyed by logical index UUID, build id, epoch
   fingerprint, descriptor digest, and deterministic content digest.
2. Stream the traversal-required graph and exact-vector state from all owners
   into a coordinator-local derived relation/image. Do not copy arbitrary
   payload columns.
3. Validate cardinality, every owner partition, vec_id uniqueness, dimensions,
   codec/degree/options, digests, and complete coverage before `Ready`.
4. Atomically select only a `Ready` replica after the scan pins and revalidates
   the active epoch.
5. Execute the same traversal core and exact ordering locally. Final lazy10
   payload reads still use owners.
6. Missing/stale/partial/corrupt/retiring replicas use the existing remote
   traversal path. A replica error cannot return a partial prefix.

The faithful prototype is one candidate. Do not stack compact packing, sparse
replication, a new codec, changed BW/H, or payload replication into its A/B.

## Phase 2: lifecycle and mutation safety

- Build/activate/retire/reclaim are crash-safe and idempotent.
- Replica scan pins participate in retirement fencing.
- New-epoch publication cannot bind an older fingerprint's replica.
- Tombstone, insert, update, and back-edge amendment invalidate the replica
  before traversal-visible mutation and force remote fallback.
- Coordinator restart, owner outage during build, partial copy, digest
  mismatch, activation race, retirement race, disk exhaustion, and manual
  replica removal have explicit drills.
- Multi-coordinator deployment treats each replica independently and reports
  amplification per coordinator.

Mutation propagation/coherence is a separate future decision. This task owns
safe invalidation and fallback only.

## Phase 3: isolated 100k A/B

Use one fresh generation and a checked-in `ecaz bench suite` with:

- remote production traversal control versus replica traversal candidate;
- same-query ordered result and seed identity;
- 200 held-out queries / 2,000 top-10 trials;
- 50 warm concurrency-1 latency samples after 10 warmups;
- complete Task 194 traversal reconciliation, including remote wait removed,
  local graph/vector work, remaining payload transport, and fallback count;
- physical replica bytes, WAL if any, build time, peak memory, bytes copied,
  cache residency, and source generation bytes;
- exact/disjoint topology, remote payload engagement, release profile, and
  unanimous binary provenance; and
- fault/rollback results from Phase 2.

The candidate is useful only if end-to-end mean and tails improve materially
with identical recall/result semantics and an explicitly accepted storage and
build envelope. A stage-local movement is not enough.

## Phase 4: conditional full-scale and decision

Only a useful 100k candidate proceeds to matched 10k/50k/100k recall, latency,
storage, cache, build, topology, provenance, and mutation-fallback evidence.
Add 1m only if the 100k result is useful and host/runtime capacity makes the
projection decision-relevant.

Output either:

- PROMOTE to a separately reviewed production-default/operations task, with
  actual format, upgrade, monitoring, capacity, and rebuild policy; or
- STOP and retain ADR-085 owner traversal unchanged.

## Acceptance criteria

1. Same-query ordered result identity and recall parity across normal, fallback,
   tombstone, qual/projection/null/toast, and owner-failure cases.
2. No scan can select a replica for a different or mutation-stale fingerprint.
3. Work remains corpus-independent and no payload or owner scan becomes
   unbounded.
4. 100k paired A/B causally reconciles the expected round-trip movement.
5. A useful candidate has complete 10k/50k/100k evidence before any promotion.
6. Rollback is demonstrated by invalidating/removing the replica and serving
   through the unchanged owner traversal path.

## Required review packets

1. `reviews/task-198/001-contract-and-format/`;
2. `reviews/task-198/002-faithful-prototype/`;
3. `reviews/task-198/003-lifecycle-and-faults/`;
4. `reviews/task-198/004-isolated-100k/`;
5. `reviews/task-198/005-full-scale-decision/`.

## Non-goals

- Sparse bridge-only replication, compact packing, dedicated binary RPC, or
  changed placement in the same causal A/B.
- Owner payload replication.
- Replica mutation propagation/coherence.
- Task 167's full physical incremental-DML implementation.
- More than one coordinator replica candidate.

## References

- ADR-085 and ADR-086.
- Tasks 184, 187, 191, 194, 195, and 197.
- FR-078 through FR-083 and NFR-017 through NFR-020.
- `plan/design/ec-distann-recall-latency-roadmap.md`.
