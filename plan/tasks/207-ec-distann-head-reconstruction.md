# Task 207: ec_distann Head Reconstruction (Paper §2.2/§3)

Status: **ready** (2026-07-29). Priority: P0 recall.

Entry gate: none. Independent of Tasks 204--206 and may run in parallel, but
`k_head` widening is only meaningful once Task 206 establishes a wide beam.

**Boundary against Task 185** (set by the Task 203 audit correction
2026-07-29-02). Task 185 is **not** superseded and is not re-scoped away; it owns
a different lever, and the two are independent given this split:

- **207 (this task) owns the pool, the search path, and sharding** —
  per-partition union construction (§3), restoring the persisted Vamana graph in
  place of the 4,096-point exact scan, and distributing the head across the
  roster (§2.2).
- **185 owns the selection objective** — which landmarks are chosen from the
  pool, and which are returned as seeds. For the promoted
  `training_landmarks_exact` policy the pool is already the whole corpus
  (`head_sample.rs:462-467`), so the objective is the operative lever there, and
  185 is gated on Task 206 because its diversity arm cannot pay off at BW=4.

They must not run **concurrently** — two lanes mutating the head destroys
attribution — but they may run in either order without re-baselining each other.

**Task 186 is likely superseded by this task.** 186's arms are head-capacity
growth (cap 8,192/16,384) and a two-level hierarchy; Task 203 found capacity is
not the measured bottleneck and construction is. Confirm 186's disposition with
the operator before starting, so it is not left as a stale `proposed` task
inviting conflicting head work.

## Why

Tasks 181 and 185 independently established the controlling fact: **head
membership, not head search, bounds recall.** Task 181's table shows exact
scoring of the cap-4096 sample returning the same 0.9275 recall as graph search,
while the same-graph owner oracle reaches 0.9970
(`plan/tasks/181-...md:18-25`, `:27-30`):

> "**Exact scoring cannot select useful entry nodes that are absent from the
> persisted sample.**"

Task 185 then found three different 4,096-row selection objectives produced "the
same ordered top-32 seeds and the same 0.9625 recall"
(`plan/tasks/185-...md:20-23`).

`DISTRIBUTEDANN` §3 names the cause:

> "In order to ensure that the entire graph is reachable, we build the head index
> from the union of the top layers of **each partition's** graph, **rather than
> the top layers of the stitched-together graph**."

ecaz builds from the stitched global graph: `shard_build.rs:587-589` returns a
global-space graph and `ambuild.rs:122-169` samples it from a single global
medoid. Default `build_shards = 1` (`mod.rs:247`), so the common case has no
partitions at all. The paper explicitly warns this construction does not ensure
reachability — which is the untested explanation for the measured membership
bound.

Three further divergences:

- **Not sharded.** §2.2 specifies "a conventional **sharded** in-memory ANN
  index"; `DistannPhysicalHeadIndex` appears in no owner path. §4.1's fix when the
  head went CPU-bound was "increase the number of head index replicas"; ecaz has
  no head replica concept, only a thread-local 2-entry backend cache
  (`generation_read.rs:261-277`).
- **The ANN index is bypassed.** A Vamana graph over the sample is built,
  persisted, digested, validated, and loaded — then never traversed. The promoted
  `training_landmarks_exact` policy brute-forces 4,096 full-precision inner
  products plus a full 4,096-element sort, per query, single-threaded on the
  coordinator, to yield 32 seeds (`head_sample.rs:1048-1050`, `:1130-1165`).
- **The promoted policy was defined as diagnostic.** `plan/tasks/181-...md:108-110`
  calls `training_landmarks` "a **diagnostic** policy" that frequency-ranks nodes
  over 200 disjoint *training queries* (`head_sample.rs:452-497`). Task 182
  promoted it to production.

The paper's two structural remedies are logged as `HEAD-11` "unmeasured" and
`HEAD-12` "deferred", and are outside the scope of both Task 185 and Task 186.

## Goal

Rebuild head construction to the paper's design and measure whether it moves the
membership bound. The pre-registered hypothesis: **recall is bounded by entry
coverage, and per-partition union construction is the mechanism that fixes it.**

## Phases

1. **Per-partition union construction (the primary candidate).** Build the head
   from the union of per-owner / per-partition top layers rather than the stitched
   global graph, so every region is represented. Measure owner-oracle seed
   membership and overlap@k against the current head on the same generation, at
   fixed BW/H and fixed cap 4,096, so construction is the only variable.
2. **Restore the ANN search path.** Use the persisted Vamana graph instead of the
   4,096-point exact scan, or record why the exact scan is retained. Note §4.1's
   finding that the head becomes CPU-bound: a full sort of 4,096 candidates to
   extract 32 is strictly worse than a bounded heap.
3. **Shard the head.** Distribute head storage/search across the roster per §2.2.
   Required for NFR-021 if the cap ever becomes a function of N; optional while
   the cap is constant, so this phase is measured on its own merits (coordinator
   CPU, not storage).
4. **Requalify seed width.** Re-test `k_head` toward the paper's 200 at the beam
   width Task 206 selects. `NEG-01` was measured at BW=4 where extra seeds are
   structurally unusable.

## NFR-021 constraint

Cap `C` must remain **constant in N**, or the head must be sharded. A head that
grows as a fraction of the corpus on one node is inadmissible regardless of its
recall effect. State the admissibility verdict at pre-registration.

## Benchmark gate

10k/50k/100k recall + latency + storage via `ecaz bench suite`, owner arm as
control, one phase at a time — construction, search path, sharding, and seed
width are four separate A/Bs and must not be stacked. Report owner-oracle seed
membership and overlap@k, not only end-to-end recall, so the mechanism is visible
even when recall does not move.

## Required review packets

1. `reviews/task-207/001-construction-contract/` — FR-080 reconciliation and the
   pre-registered hypothesis.
2. `reviews/task-207/002-union-construction/` — the primary candidate A/B.
3. `reviews/task-207/003-search-and-sharding/` — phases 2 and 3.
4. `reviews/task-207/004-full-scale-decision/` — 10k/50k/100k and disposition.

## Spec reconciliation (in scope)

- `FR-080:22-27` already specifies per-shard-medoid BFS with hop radius and
  per-shard union — **no code implements it.** Either implement it or correct the
  FR; do not leave them divergent.
- `FR-080:44-52` specifies a 2-entry LRU keyed on
  `(index_oid, logical_index_uuid, build_id, epoch_fingerprint)`;
  `head_cache.rs:75-106` is an unbounded `HashMap` keyed on `index_oid` alone.

## Non-goals

- Graph degree, neighbor codec, or traversal budget changes (Tasks 205/206).
- Growing the cap as the primary lever. Task 203 found capacity growth is not the
  measured bottleneck; construction is.
- Reviving the traversal replica.

## References

- `DISTRIBUTEDANN` §2.2, §3, §4.1.
- `reviews/task-203/001-decision-reaudit/` Defect 3.
- Tasks 180, 181, 182, 183, 185, 186; ledger `HEAD-11`, `HEAD-12`, `HEAD-33`,
  `NEG-01`, `NEG-06`.
- `FR-080`, `NFR-021`, `NFR-022`.
