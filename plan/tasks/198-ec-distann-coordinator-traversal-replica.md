# Task 198: ec_distann Coordinator Traversal Replica

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **complete — PROMOTE to Task 199 productionization** (2026-07-23).
Priority: P2 architecture implementation and measurement.

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

If Tasks 185/186/188/189 change a production default before Phase 3 or 4,
re-pin this control and its query/work identities to the then-current
production point before interpreting a replica delta.

## Phase 1: contract and faithful prototype

1. Define replica catalog/state at the authoritative control coordinator,
   keyed by logical index UUID, build id, epoch fingerprint, descriptor
   digest, and deterministic content digest. Scans observe that state during
   the existing active-epoch pin/revalidation, with no additional owner RPC.
2. Stream the traversal-required graph and exact-vector state from all owners
   into a coordinator-local derived relation/image. Payload columns are not
   copied. "Faithful" means identical traversal inputs/results, not a physical
   copy of the full row tier.
3. Validate cardinality, every owner partition, vec_id uniqueness, dimensions,
   codec/degree/options, digests, and complete coverage before `Ready`.
4. Atomically select only a `Ready` replica after the scan pins and revalidates
   the active epoch.
5. Execute the same traversal core and exact ordering locally. Final lazy10
   payload reads still use owners.
6. Missing/stale/partial/corrupt/retiring replicas use the existing remote
   traversal path. A replica failure after traversal starts discards all
   replica frontier/hit state and restarts from the beginning on the owner
   path under the same pinned epoch; it cannot return or reuse a partial
   prefix.

The faithful prototype is one candidate. Do not stack compact packing, sparse
replication, a new codec, changed BW/H, or payload replication into its A/B.

## Phase 2: lifecycle and mutation safety

- Build/activate/retire/reclaim are crash-safe and idempotent.
- Replica scan pins participate in retirement fencing.
- New-epoch publication cannot bind an older fingerprint's replica.
- Replica eligibility has one durable authority at the control coordinator.
  The currently supported topology is one authoritative coordinator per
  logical index; multiple independent query-coordinator replicas are rejected
  until a shared invalidation protocol is separately accepted.
- Tombstone, insert, update, and back-edge amendment cannot dispatch while a
  replica is Ready. The first attempt durably transitions Ready to Stale and
  returns a stable retryable error without owner mutation. A dedicated
  coordinator control transaction commits that transition independently of
  the failing user DML transaction; the error is returned only after the
  control commit succeeds. A retry observes Stale and uses the owner path. A
  crash after invalidation is fail-safe.
- A scan that pinned a Ready replica before invalidation completes against
  that pinned immutable image. New scans observe Stale. This is the permitted
  pre-mutation view under FR-082's concurrent-mutation model.
- Coordinator restart, owner outage during build, partial copy, digest
  mismatch, activation race, retirement race, disk exhaustion, manual replica
  removal, mid-scan replica failure with full owner restart, and an
  in-flight-scan/invalidation/mutation race have explicit drills.
- The mutation drill asserts that exactly one mutation attempt returns the
  retryable invalidation error before its retry succeeds through the owner
  path, including rollback and crash-after-control-commit variants.
- Capacity evidence reports supported single-coordinator measured bytes and
  projects linear per-coordinator amplification for a possible future
  shared-authority design. This projection does not permit multiple active
  replica coordinators.

Mutation propagation/coherence is a separate future decision. This task owns
safe invalidation and fallback only.

## Phase 3: isolated 100k A/B

Use one fresh generation and a checked-in `ecaz bench suite` with:

- remote production traversal control versus replica traversal candidate;
- same-query ordered result and seed identity;
- 200 held-out queries / 2,000 top-10 trials;
- 50 warm concurrency-1 latency samples after 10 warmups;
- complete Task 194 traversal reconciliation, including remote wait removed,
  local graph/vector work, remaining payload transport, fallback count,
  per-round wait, round count, and straggler spread;
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
  actual format, upgrade, monitoring, capacity, and rebuild policy, and with
  the first mutation against a Ready replica disclosed as a user-visible
  retryable failure; or
- STOP and retain ADR-085 owner traversal unchanged.

## Measured outcome

The feature-gated faithful replica preserved exact owner-arm recall at
`0.9990 / 0.9685 / 0.9625` for 10k / 50k / 100k. Warm mean latency improved
from `19.50 / 20.70 / 20.60` ms to `16.40 / 17.80 / 17.10` ms
(`15.9% / 14.0% / 17.0%`), and p95 improved from
`23.10 / 24.00 / 23.80` ms to `20.10 / 20.10 / 20.00` ms. The 100k
attribution reduced traversal from `7.866` to `3.617` ms, replaced
`6.405` ms of remote expansion with `3.386` ms of local graph/vector reads
plus `0.140` ms of RaBitQ scoring, and left final owner payload work in place.

The operating cost is substantial and explicit. Replica relations occupy
`158,326,784 / 823,705,600 / 1,659,518,976` bytes, or roughly
`65.2% / 66.3% / 66.5%` of the physical generation. Builds copy
`131,520,000 / 657,600,000 / 1,315,200,000` bytes and take
`5.208 / 24.736 / 51.995` seconds; 100k emits 1.926 GB WAL. Peak copy-batch
memory stays bounded at 3,366,912 bytes. Exact/digest identity, owner-outage
rollback, corrupt-image fallback, genuine second-batch full restart,
retryable `Ready -> Stale` invalidation, owner fallback, and idempotent
retire/reclaim pass.

Decision: **PROMOTE to Task 199**, not directly to a production default.
Task 199 owns normal-build feature isolation, operator API/privilege and
capacity policy, explicit-build/read-mostly rollout, removal of benchmark
selection controls, production lifecycle drills, and a fresh release-profile
10k/50k/100k gate. Until that task is accepted, production owner traversal is
unchanged and this implementation remains feature-gated.

## Acceptance criteria

1. Same-query ordered result identity and recall parity across normal, fallback,
   tombstone, qual/projection/null/toast, and owner-failure cases.
2. No scan can select a replica for a different or mutation-stale fingerprint.
   A scan already pinned before invalidation may complete on that immutable
   image and is covered by the concurrent-mutation identity drill.
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
- Multiple query coordinators before a single shared invalidation authority
  exists.
- Task 167's full physical incremental-DML implementation.
- More than one coordinator replica candidate.

## References

- ADR-085 and ADR-086.
- Tasks 184, 187, 191, 194, 195, and 197.
- FR-078 through FR-084 and NFR-017 through NFR-020.
- `plan/design/ec-distann-recall-latency-roadmap.md`.
