# Task 210: ec_distann Distribution Restoration

Status: **review pending — implementation and zero-byte gate evidence complete**
(2026-08-08). Priority: **P0 — top priority.**

Entry gate: none. This task does not wait on Tasks 205, 206, or 207, and they do
not wait on it.

Implementation and measurement evidence is present in review packets
`reviews/task-210/001-conformance-emitter/`, `002-default-path/`,
`003a-head-sharding/`, `004-gateway-copies/`, `005-default-gate/`, and
`006-zero-byte-head/`. Packet 006 establishes the round-2
zero-byte membership-head gate and has a Codex review recorded in
`reviews/task-210/006-zero-byte-head/feedback/2026-08-08-01-reviewer.md`;
independent external review remains outstanding.

Merge state (verified 2026-08-08): the implementation commits — including
`35c7f3c3b` (membership-only head as a bounded state-row blob) and `4070ff6cb`
(shard ordinal derived from members) — are **on `origin/main`**. The sharded
head is the shipped default. The implementation and evidence are complete, but
the packet remains review-open pending an independent external reviewer.

**No phase in this task is conditional, optional, or gated on a measured win.
Sharding is a property this task delivers, not a candidate it screens.** A phase
closes when the property holds in the shipped default configuration and the
conformance row proves it. "Measured and not worth it" is not an available
outcome for any phase here.

## Scope

This task changes **where index state lives** and **which path serves the default
query**. Nothing else.

Out of scope by construction, owned elsewhere, and not blocked by this task:
Algorithm 1 pushdown (205), traversal regime (206), head *construction* method
and seed selection (207/185), degraded completion (209).

## Why

The audit that began at Task 201 was triggered by one discovery: ec_distann had
stopped being sharded. Tasks 198/199 promoted a coordinator traversal replica
holding every owner's graph record and full-precision vector on one node
(1,659,518,976 bytes at 100k, linear in N), and it became the program's latency
control, so forward work was measured against a single-node index.

That finding reached the **requirements**: `StR-008`'s conformance precondition
("a configuration that meets the p50 criterion by ceasing to distribute the index
... abandons it"), `NFR-021`, `NFR-022`, and the roadmap ledger marking `TRAV-30`
**ACTIVE** as "the NFR-021-conforming direction ... reinstated as the successor
direction to the withdrawn replica."

It did not reach the **tasks**. Every latency and recall lever received an owning
task, a sequence position, and a benchmark gate. Every sharding item became a
non-goal, an optional phase, or an orphaned ledger row:

| Sharding work | Where it landed before this task |
| --- | --- |
| `TRAV-30` bounded gateway copies | ACTIVE in the ledger, imported by no task; Tasks 187/194 point at Task 190, which the audit closed **INVALID** |
| FR-084 / replica disposition | non-goal in 203 and 204; "reviving the traversal replica" a non-goal in 205, 206, 207, 209; no owner anywhere |
| Shard the head (§2.2) | Task 207 Phase 3, "**optional** while the cap is constant" |
| Replicate the head (§4.1) | not tasked |
| Coordinator-resident bytes in the gate | not emitted — `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:5381` hardcodes the coordinator row to `graph_bytes=0 directory_bytes=0 control_bytes=0` |

The mechanism is structural, and this task exists to defeat it. The ledger
advances "at most one candidate" that shows a measured end-to-end win. Sharding
cannot win on that metric: the replica is *faster* (15.3/16.4/16.2 ms against the
sharded owner arm's 18.3/20.4/19.9 ms at 10k/50k/100k), and sharding the head
costs coordinator CPU while buying no latency. So sharding was correctly excluded
from a ledger that only admits winners — and then picked up nowhere else.

In shipped code today, `src/am/ec_distann/generation_read.rs:2523` opens
`ReadyTraversalReplica` on the normal read path and prefers it whenever a Ready
image exists, with no GUC gating it. Building the image is an explicit call
(`ec_distann_build_traversal_replica`), so a deployment that never invokes it is
sharded — but the preference is automatic and unlabeled, and it is what Task 199
promoted as the normal path.

## Goal

The sharded configuration is complete and is the **default**. The non-sharded
path cannot be entered without an explicit, labeled opt-in, and can never be a
decision control.

**The replica code is retained. This task deletes no code.**

## The definition of distribution this task lands in NFR-021

A configuration is *distributed* if and only if, at every measured scale:

1. **Every O(N) structure is partitioned across the serving roster**, each node
   holding only its own partition — graph adjacency, embedded neighbor codes,
   full-precision vectors, the row payload tier, directories, and every derived,
   optional, or default-off relation. Non-owner records: 0.
2. **No node outside the roster holds O(N) index state.** Coordinator-resident
   state is bounded by `k`, `L`, dimension, roster size, and relation count —
   never by N.
3. **Structures the paper distributes are distributed here even when they are
   small.** The head is sharded across the roster (§2.2) and replicated for
   capacity (§4.1) regardless of its constant cap. Smallness is not an exemption;
   this clause replaces NFR-021's current constant-`C` carve-out.
4. **No read path silently substitutes a non-distributed structure for a
   distributed one.**
5. **It holds in the shipped default configuration**, not only in a benchmark
   arm. A property that requires a non-default flag to be true is not delivered.

## Phases

All four land. The order is execution order, **not** a gate — no phase may be
closed by citing another phase's result.

### P0 — Make it provable

- `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:5369-5390`: replace
  the hardcoded-zero coordinator `physical_benchmark_storage_node` row with a
  real per-relation measurement — head sample, head graph, head cache, every
  coordinator-resident index or index-derived relation, plus the existing replica
  relation. Reuse the owner-row shape so the two are directly comparable. The
  head's `head_sample_bytes=25280512` / `head_graph_bytes≈614095` currently reach
  only the separate `physical_benchmark_head` row, which the conformance check
  does not consume.
- `crates/ecaz-cli/src/commands/bench/suite.rs`: the Task 208
  `physical_benchmark_nfr_021_conformance` derivation consumes the coordinator
  row. An unclassified coordinator-resident structure is `unavailable`, never a
  pass.
- `spec/non-functional/NFR-021-distann-distribution-invariant.md`: land the
  five-clause definition above; **remove the constant-`C` coordinator-resident
  head exemption**; add a metric row for coordinator-resident bytes by relation.

Evidence: a 10k/50k/100k run whose coordinator row itemises every resident
relation, with the head visible as coordinator-resident and the verdict
reflecting it.

### P1 — The sharded path is the default

- `src/am/ec_distann/generation_read.rs:2523`: the read path does not open
  `ReadyTraversalReplica` unless explicitly opted in. Sharded owner traversal is
  the default, with no image check in the hot path.
- `src/am/ec_distann/options.rs`: new `ec_distann.allow_nonconforming_replica`
  (Userset, default off), documented as a non-conforming accelerator.
- A scan that uses the replica emits a conformance label reaching
  `results.jsonl`; `suite.rs` config validation rejects a labeled arm as control
  or candidate under NFR-022, keyed on the label as well as the storage row.
- `FR-084` amended: explicitly opted-in, labeled, non-conforming accelerator;
  never the default, never a decision control. `ADR-086` gains the NFR-021
  Consequences note it currently omits.

Evidence: default-config 10k/50k/100k run with the replica never opened; an
opt-in run showing the label present and the arm rejected as a control.

### P2 — Shard the head across the roster

The head is the only structure the paper distributes that ec_distann keeps
central: 4,096 full-precision f32 landmarks
(`DistannHeadSampleEntry { vec_id, vector: Vec<f32> }`, 25,280,512 bytes,
constant across 10k/50k/100k) plus a ~614 KB Vamana graph, single copy on the
coordinator.

- **P2a Sharding (§2.2).** Head storage and head search distribute across the
  roster. `DistannPhysicalHeadIndex` exists at `src/am/ec_distann/head_sample.rs:1000`
  and appears in no owner path; give it an owner endpoint alongside
  `ec_distann_expand_physical_nodes` in `remote_transport.rs` /
  `generation_read.rs`, with the coordinator merging bounded per-owner head
  results. Both policies must work sharded: the default `current_sample_graph`
  traverses the persisted Vamana graph (`head_sample.rs` `search`), the opt-in
  `training_landmarks_exact` exact-scans.
- **P2b Replication (§4.1).** A head replica count, so a CPU-bound head is
  answered the way §4.1 answers it rather than by the 2-entry thread-local
  backend cache at `generation_read.rs:261-277`.

**Implementation contract (established in code, 2026-07-30).** Two findings from
the landed slices fix how P2a is built; neither was known when the phase was
written.

1. **Shard graphs are per-shard, not slices of the stitched head graph.** A
   subgraph of the global head is not a navigable index over the shard. This is
   the same property that makes `DISTRIBUTEDANN` §3 build the head from
   per-partition top layers. `shard_head_sample()` does this.
   For the promoted exact policy, sharding is **result-identical** to the
   unsharded head — proven by
   `sharded_exact_head_search_is_identical_to_the_unsharded_head` — so P2a
   carries no recall risk for the shipped policy.
2. **Head vectors never need to move.** A landmark's full-precision vector
   already lives on the owner its FR-078 placement hash selects, because the
   co-placed row tier uses the identical hash (ADR-085 D11). An owner therefore
   materialises its own shard from **local reads** given only the bounded
   membership list. `head_shard_members()` / `build_owner_head_shard()` do this,
   and `owner_built_shards_match_coordinator_side_partitioning` proves the two
   agree.

Consequently the remaining P2a wiring is:

- **Persistence.** `persist_head_sample` / `load_head_sample` keep the head
  *membership* (vec_ids) and the state row on the coordinator — bounded by `C`,
  permitted by clause 2 — and stop storing landmark vectors there. This is a
  format change, which is free (research index, rebuild not migrate).
- **Owner endpoint.** `ec_distann_head_search_physical(index_regclass,
  epoch_fingerprint, query, member_vec_ids, search_width, seed_count)` →
  `(vec_id, dist)`. It resolves its members' vectors through the existing
  local row-tier read used by `exact_distance`, builds or reuses its shard, and
  returns at most `seed_count` seeds. Both policies must work sharded.
- **Owner shard cache.** Keyed like `CachedPhysicalEpoch`
  (`index_oid, logical_index_uuid, build_id, fingerprint`), same 2-entry bound,
  invalidated by the existing `invalidate_generation_caches` hook.
- **Coordinator fan-out.** Replace the local `head_index` in
  `DistannPhysicalScanState` with a per-owner fan-out that reuses the
  `PhysicalMultiOwnerExpander` routing shape and merges with
  `merge_head_seeds()`. Coordinator retains `seed_count` seeds — clause 2
  bounded state — and zero landmark vectors.

Gate: P2a and P2b each get their own 10k/50k/100k recall + latency + storage A/B
against the owner arm — separately, never stacked — with the P0 conformance row
`conforming` in every arm, `outstanding_distribution_gap=none`, and coordinator
head bytes reaching zero.

### P3 — `TRAV-30`: bounded gateway copies

The ledger's reinstated conforming direction, and the sharding-preserving answer
to the latency question the replica answered by abandoning sharding. Bounded
gateway / top-layer copies whose capacity is a stated constant independent of N,
proven so by the P0 emitter at every scale rather than by argument.

**Implementation finding (established in code, 2026-07-30).** A gateway copy can
answer the *candidate* half of Algorithm 1 locally — neighbour ids and neighbour
code scores, which is pure routing — but **not** the *result* half. A node's
`exact_dist` requires its full-precision vector, and holding those at the
coordinator is precisely the FR-084 trap this candidate exists to avoid. So a
gateway copy reduces response payload and owner scoring work; it does **not**
remove the owner round trip. `DistannGatewayCopySet` therefore stores routing
payload only, and the P3 A/B must be judged on bytes and owner work, not on
eliminated hops. A design that eliminated the hop would be holding vectors and
would fail NFR-021.

Gate: 10k/50k/100k against the owner arm, reported as response bytes and owner
service time rather than round-trip count.

### P4 — Ownership, so this cannot be re-deferred

- `plan/tasks/207-ec-distann-head-reconstruction.md`: Phase 3 (shard the head)
  moves here and its "optional while the cap is constant" clause is deleted. 207
  retains head *construction* (per-partition union), the ANN search path, and
  `k_head` requalification.
- `plan/tasks/README.md`: this task's status line and 207's revised scope.
- `plan/design/ec-distann-recall-latency-roadmap.md`: `TRAV-30` assigned to 210
  P3; `TRAV-28` disposition recorded; an explicit carve-out that conformance work
  is delivered against the invariant and is never screened against latency.

## Benchmark gate

`ecaz bench suite` at **10k/50k/100k**, recall + latency + storage, with the
**owner-traversal arm as control** (NFR-022) and NFR-021 admissibility recorded
at pre-registration. Run directories live under `~/.ecaz/clusters/`, never in the
repo and never in `target/`.

Every arm in every run carries a `physical_benchmark_nfr_021_conformance` verdict
of `conforming`; `unavailable` fails the run.

The final acceptance run uses the **shipped default configuration** — no arm-only
flags — and shows: coordinator resident index bytes bounded and itemised,
`non_owned=0` and `orphans=0` on every node at every phase, head bytes attributed
to owners, and the replica never opened.

A phase whose A/B shows a latency cost still lands. Report the cost; do not
withhold the property. Where a cost is material, the follow-up is a *conforming*
optimization (P3 is the first one), never a return to the non-conforming path.

## Required review packets

1. `reviews/task-210/001-conformance-emitter/`
2. `reviews/task-210/002-default-path/`
3. `reviews/task-210/003a-head-sharding/`
4. `reviews/task-210/004-gateway-copies/`
5. `reviews/task-210/005-default-gate/`
6. `reviews/task-210/006-zero-byte-head/` (round-2 head-sharding and
   replication closeout)

Each carries `artifacts/manifest.md`, the `ecaz bench suite` config,
`results.jsonl`, and the NFR-021 conformance row for every arm.

## Non-goals

- Deleting the replica, `FR-084`, `traversal_replica.rs`, or `ADR-086`.
- Beam width, hop rounds, `L`, `top_k`, or seed count changes (205/206).
- Head construction method or selection objective (207/185).
- Degraded completion (209).

## References

- `DISTRIBUTEDANN` (arXiv:2509.06046) §2.2, §3, §4.1; verbatim citations in
  `reviews/task-205/003-ab/artifacts/paper-algorithm-citations.md`.
- `reviews/task-203/001-decision-reaudit/` Defect 4 and its
  `feedback/2026-07-30-01-reviewer.md`.
- `reviews/task-201/001-control-validity-supersession/feedback/2026-07-29-01-reviewer.md`.
- `StR-008`, `NFR-021`, `NFR-022`, `NFR-018`, `FR-078`, `FR-080`, `FR-084`,
  `ADR-067:47-51`, `ADR-086`.
