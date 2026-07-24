---
type: ADR
id: ADR-086
title: "ec_distann: Fingerprint-Bound Coordinator Traversal Replica"
status: ACCEPTED
impact: Adds an optional derived traversal copy at the coordinator to remove serial owner expansion round trips while retaining hash-owned rows, owner-authoritative generations, lazy payload materialization, and the existing remote traversal fallback.
date: 2026-07-23
---
# ADR-086: ec_distann — Fingerprint-Bound Coordinator Traversal Replica

## Context

ADR-085 deliberately chose one global graph with hash-placed records and
coordinator-driven hop rounds. It made query work corpus-independent, but its
known latency floor is the number of sequential rounds multiplied by per-round
transport cost.

That floor is now measured. Task 194's accepted, reconciled 100k attribution
records ten hop rounds, 40 requested/returned nodes, zero repeated nodes,
7.429 ms/scan of remote expansion, 2.259 ms of owner service, and 5.013 ms of
transport wait. A lighter-observer release run records 4.078 ms of transport
wait. Connection readiness, request encoding, and coordinator receive/decode
together cost only 0.071 ms/scan, and logical traffic is only 13.9 KiB request
plus 10.5 KiB response per scan. This is serial remote/backend-boundary
pressure, not bandwidth or codec pressure. On the same-host benchmark lane,
backend scheduling and IPC dominate wire RTT; a genuinely remote deployment
adds network latency to the same boundary.

The retained post-Task-195 production point is 0.9990 / 0.9685 / 0.9625
distinct recall and 20.90 / 20.90 / 19.90 ms warm mean at 10k / 50k / 100k.
The fixed-work BW8/H50 experiment reduced rounds from 10.0 to 5.88 and
transport wait by 0.744 ms, but increased nodes and straggler spread, left
end-to-end mean effectively flat, and regressed p95. Narrower tuning cannot
simply buy away the round-trip floor with more work per round.

Task 190 compared two architecture families:

1. a coordinator-resident traversal replica (`ARCH-02`, informed by
   `TRAV-28`); and
2. dedicated binary traversal transport (`ARCH-07`).

## Decision

Build a **fingerprint-bound, coordinator-resident traversal replica** as an
optional, rebuildable derivative of one Published physical generation.

The replica contains exactly the graph and exact-vector data needed to execute
the existing production traversal and exact ranking locally. It does not copy
payload columns and does not become the generation or row-tier source of
truth. "Faithful" means identical logical traversal inputs and results, not a
physical copy of the full row tier. Final executor-driven lazy10 payload
materialization remains routed to hash owners under the existing epoch/failure
contract.

The implementation must preserve the current algorithm byte-for-byte in its
logical inputs and outputs:

- same persisted trained head and ordered 32 seeds;
- same BW/H, degree, RaBitQ neighbor values, exact-distance semantics,
  tombstone handling, convergence, and global result ordering;
- same active build id, epoch fingerprint, descriptor and schema fingerprints;
  and
- same owner-side ranked payload windows, projection, qual, snapshot, outage,
  and error behavior.

The replica is eligible only in a `Ready` state after cardinality, per-owner
coverage, descriptor identity, and a deterministic content digest have all
verified. Activation is atomic by `(logical_index_uuid, build_id,
epoch_fingerprint)`. A scan pins the active epoch before selecting a replica.
Missing, partial, stale, retiring, digest-mismatched, or otherwise unusable
replicas take the existing owner traversal path. A failure discovered after
replica traversal starts discards the entire local frontier/result state and
restarts from the beginning on the owner path under the same pinned epoch; it
never reuses or returns a partial prefix.

The first implementation copies the current graph and exact vectors but no
payload columns. Compact packing is not a prerequisite and cannot be combined
post-hoc with the initial causal A/B. At 100k the current physical generation
is 2,496,626,688 bytes; that is the hard per-coordinator storage ceiling, not
permission to copy the full payload-bearing row tier. The current owner graph
relations total 826,925,056 bytes. Adding one raw f32 vector per
1,536-dimensional row and the current directory gives a lower-envelope
estimate around 1.445 GB (about 57.9% of the generation), but PostgreSQL/layout
overhead and a real implementation decide the actual number. Task 198 must
report measured bytes, build time, peak memory, bytes copied, and cache
residency; the estimate is not a gate result.

Published-generation mutation remains owner-authoritative. Replica state has
one durable authority in the control coordinator's epoch catalog and is read
during the scan's existing active-epoch pin/revalidation. The supported
topology is one authoritative coordinator per logical index. Multi-coordinator
replicas are rejected until a shared invalidation authority is separately
designed.

A tombstone, insert, update, or back-edge amendment cannot dispatch while the
replica is Ready. Its first attempt durably changes Ready to Stale and returns
a stable retryable error without sending an owner mutation. The transition is
committed by a dedicated coordinator control transaction, independently of the
failing user DML transaction; the error is returned only after that commit
succeeds. A retry observes Stale and uses the owner path. This avoids a
distributed-commit requirement and prevents rollback of the invalidation from
causing a retry livelock. A crash after invalidation is fail-safe. A scan that
pinned Ready before the transition completes on the pinned immutable image,
representing the permitted pre-mutation view; new scans observe Stale. Task 198
may implement this invalidation/fallback contract but may not add replica
mutation propagation. A new epoch builds a new replica and cannot reuse one
across fingerprints.

## Expected ceiling and gates

The directly removable component is the measured 4.078--5.013 ms/scan
transport wait, 20--25% of the retained 19.90 ms 100k warm mean. Owner service
will be replaced by local graph/vector work rather than disappearing, so it is
not counted as a guaranteed saving. The design is credible only if a same-
generation paired 100k prototype materially improves end-to-end mean and
tails, not merely traversal timers.

Task 198 starts feature-gated and advances only after:

1. same-query ordered result identity, recall, work-bound, and fallback parity;
2. reconciled traversal counters showing that serial remote wait moved without
   hidden work;
3. atomic build/activate/retire/reclaim and mutation-invalidation fault drills;
4. measured storage/build/network/cache costs within an explicitly accepted
   operating envelope; and
5. a checked-in `ecaz bench suite` A/B at 100k, followed only on a useful
   result by 10k/50k/100k recall, latency, storage, build, topology, and
   provenance evidence.

Production remains unchanged until a later promotion decision accepts those
results. Rollback is removal/invalidation of the derived replica and immediate
use of the existing owner traversal path.

## Consequences

- The design can remove serial traversal RPCs without moving payload
  ownership, changing hash placement, or making partial success acceptable.
- It adds substantial per-coordinator storage, build traffic, cache pressure,
  and publication work. Capacity reporting also projects linear
  per-coordinator amplification for a possible future shared-authority design;
  multi-coordinator replica serving is not supported by this ADR.
- Read-mostly epochs can benefit first. Mutation-heavy epochs fall back until
  coherence is separately designed and accepted.
- Replica correctness joins epoch lifecycle and scan fencing; a stale copy is
  a correctness fault, not a performance miss.
- The existing transport remains mandatory as fallback and for payloads, so
  this is not a transport deletion.

## Rejected alternatives

### Dedicated binary traversal RPC (`ARCH-07`)

Rejected for this escalation. The measured serialization-adjacent work
(connection, request encode, receive/decode) is only 0.071 ms/scan, and traffic
is small. Replacing row/array encoding alone has negligible ceiling. An
always-on binary service might reduce backend scheduling overhead, but still
crosses ten sequential remote/backend boundaries and introduces a new service,
authentication boundary, snapshot/epoch protocol, cancellation path,
backpressure model, deployment surface, and rollback mode without an
attributed estimate of the portion it removes. It may be reconsidered only
after a separate feasibility measurement isolates service/protocol overhead
from network RTT. Task 198 retains per-round wait, round-count, and straggler
decomposition so that premise remains testable.

### Shared-memory or Unix-domain transport (`ARCH-08`)

Rejected for this escalation. It targets the same-host benchmark topology and
could reduce local IPC/service overhead, but it neither serves genuinely
remote owners nor removes the sequential owner boundaries. Selecting it would
optimize a lane-specific deployment artifact rather than the distributed
architecture. Reopen only for an explicitly same-host product topology.

### Sparse top-layer or bridge replication (`TRAV-28`--`TRAV-30`)

Not selected as the first implementation. Task 194 observed zero repeated
nodes and did not identify a stable hot-node subset; fixed-cap gateway and
hierarchy work also does not prove that a small copy eliminates the serial
lower-graph rounds. Task 198 may measure replica hit/coverage as a diagnostic,
but it may not silently substitute a sparse design for the full traversal
identity A/B.

### Placement, payload, accelerator, and coordinator routing families

Not selected. Hash placement is not causing owner imbalance large enough to
dominate the measured wait; payload work has already been reduced and cached
by Tasks 191/195; the scan is latency-at-concurrency-1 rather than a
cross-query throughput case; and moving the coordinator near one owner does
not remove requests to the other owners.

## Evidence

- `reviews/task-194/007-fixed-work-candidate/`
- `reviews/task-194/008-nine-way-completion-audit/`
- `reviews/task-194/008-nine-way-completion-audit/feedback/2026-07-22-01-reviewer.md`
- `reviews/task-195/002-release-matrix/`
- `reviews/task-195/002-release-matrix/feedback/2026-07-22-01-reviewer.md`
- `reviews/task-190/001-entry-and-residual-case/`
- `reviews/task-190/002-architecture-comparison/`
