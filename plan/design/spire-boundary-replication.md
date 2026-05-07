# SPIRE Boundary Replication Design

Status: Phase 5 design checkpoint for Task 30
Date: 2026-05-07
Scope: local boundary replicas over the existing SPIRE partition-object,
epoch, and local-store placement model

This note defines the first boundary-replication contract for SPIRE. It is
subordinate to ADR-049, the Phase 0 partition-object storage design, the Phase
2 update mechanics plan, the Phase 3 recursive hierarchy plan, and the Phase 4
local multi-store placement plan.

## Goals

Phase 5 turns the latent one-to-many assignment shape into controlled behavior:

```text
vec_id -> primary leaf PID plus zero or more boundary replica leaf PIDs
leaf PID -> local_store_id -> object placement
scan -> candidate scoring -> vec_id dedupe -> final top-k
```

The first implementation must:

- keep the default path primary-only;
- bound assignment fanout and storage growth;
- preserve deterministic scan results when a vector appears in multiple leaf
  PIDs;
- keep local multi-store placement hash-by-PID;
- expose enough diagnostics to measure recall/storage tradeoffs before making
  product claims.

Phase 5 does not change the epoch publication contract, add remote placement,
implement multi-store REINDEX, bind populated PQ-FastScan SPIRE scans, or
physically reclaim old epoch objects.

## Option Surface

Boundary replication is opt-in per index. The first reloption surface should
be:

```text
boundary_replica_count int  -- default 0, max 8
```

`0` preserves current behavior: one primary assignment row per vector, scan
dedupe mode `NoReplicaDedupeDisabled`, and no boundary-replica diagnostics
other than zero counts.

Values greater than zero enable at most that many secondary leaf assignments
per vector. The value is an upper bound, not a guarantee: fewer replicas may be
written when fewer eligible leaves exist or when threshold rules reject all
secondary leaves.

A session GUC is intentionally deferred for build-time fanout. Unlike
`ec_spire.nprobe` and `ec_spire.rerank_width`, boundary fanout changes durable
index contents and should not vary by backend session. Scan dedupe mode is
derived from active index metadata instead of a session override.

If a future study needs a distance-margin rule, add a second reloption:

```text
boundary_replica_min_margin real  -- optional, default disabled
```

Do not add it in the first code slice unless the recall study proves top-N
fanout alone is too blunt.

## Boundary Predicate

The first predicate is top-N nearby leaves by the same route ordering already
used for scan routing:

1. Score the vector against all level-1 child centroids using inner product.
2. The highest-ranked leaf is the primary assignment.
3. The next `boundary_replica_count` ranked leaves are eligible replicas.
4. Exclude duplicate child PIDs and malformed/missing children.
5. For recursive indexes, apply this predicate only at the final leaf-routing
   level after recursive descent chooses the parent routing object. Higher
   routing levels do not get replicas in the first Phase 5 slice.

Tie-breaks use the existing route order: higher inner product, lower centroid
ordinal, then lower child PID. This keeps primary/replica assignment stable
across rebuilds with identical centroids and input rows.

The top-N rule is deliberately conservative. It produces a clear storage
overhead target and can be evaluated before adding a margin-based predicate.
For `boundary_replica_count = R`, the planned maximum assignment growth is:

```text
total assignment rows <= primary rows * (1 + R)
```

## Assignment Rows

The existing leaf assignment schema already carries the required flags:

```text
PRIMARY
BOUNDARY_REPLICA
```

Phase 5 writes exactly one `PRIMARY` row for each live vector and up to
`boundary_replica_count` `BOUNDARY_REPLICA` rows with the same `vec_id`,
heap TID, payload format, gamma, and encoded payload. Replica rows differ by
their containing leaf PID and row flags.

Replica rows must never be delete-delta rows. Delete deltas continue to target
the vector identity through `vec_id`; scan-time deleted-`vec_id` suppression
must cover both primary and replica rows.

Build, insert-delta publication, and future split/merge replacement leaves all
use the same assignment planner so boundary behavior is not build-only.

## Scan Dedupe

When active metadata indicates boundary replicas are possible, scan plans must
use:

```text
SpireCandidateDedupeMode::VecIdDedupeEnabled
```

The current primary-only plan remains:

```text
SpireCandidateDedupeMode::NoReplicaDedupeDisabled
```

Dedupe happens before final bounded top-k selection. For duplicate `vec_id`
candidates, keep the candidate selected by the existing candidate ordering:
lower ORDER BY score, newer serving epoch, primary before boundary replica in
the same epoch, lower heap TID, lower PID, lower row index, then lower
`vec_id` bytes.

The primary-before-replica tie-break matters only when scores and epoch match.
It keeps result provenance stable while still allowing a replica with a better
score to win.

## Local Placement

Boundary replication does not introduce a new store-placement rule. Phase 4
placement remains authoritative:

```text
pid -> local_store_id -> relation/object location
```

Replica rows are stored in the leaf object for their replica PID. Store choice
therefore follows the existing PID placement planner. With hash-routed local
stores, replicas naturally spread by replica PID; with one store, they remain
in the root/control relation.

This preserves the Phase 4 scan grouping and prefetch boundary. Scan fetch
groups selected leaf and delta routes by `(node_id, local_store_id)` before
object reads, then dedupes candidate rows by `vec_id` after scoring.

## Diagnostics

Add SQL-visible accounting before publishing measured claims:

```text
primary_assignment_count
boundary_replica_assignment_count
total_assignment_count
boundary_replica_configured_count
observed_max_assignment_fanout
assignment_growth_ratio
dedupe_mode
duplicate_candidate_suppressed_count
```

The leaf diagnostics surface should expose per-leaf primary and replica counts.
The relation storage snapshot should continue to report physical object bytes;
the assignment diagnostics supply the logical overhead denominator.

Scan diagnostics should count duplicate candidates suppressed by vec-id dedupe
so operators can see whether boundary replicas are affecting the scan path.

## Measurement Gate

The first recall/storage packet should compare:

- `boundary_replica_count = 0`
- `boundary_replica_count = 1`
- optionally `boundary_replica_count = 2` if local storage overhead remains
  reasonable

Use the same corpus, nlists, nprobe, rerank width, storage format, and local
store configuration across lanes. Report recall delta, latency, primary rows,
replica rows, total assignment rows, and physical object bytes. Local results
are implementation evidence only; they are not product-scale or multi-machine
claims.

## Implementation Order

1. Add parsed reloption metadata and diagnostics that always report zero
   replicas on existing indexes.
2. Add a pure assignment planner that returns primary plus bounded secondary
   leaf PIDs without writing them.
3. Wire build and insert-delta publication to write replica assignment rows.
4. Switch scan plan resolution to `VecIdDedupeEnabled` when the active index
   can contain replicas.
5. Add logical storage-accounting diagnostics and a focused PG18 fixture.
6. Run the recall/storage comparison packet.
