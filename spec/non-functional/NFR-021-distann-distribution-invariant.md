---
id: NFR-021
title: Distann Distribution Invariant
type: NFR
status: PROPOSED
quality_attribute: scalability
relationships:
  - target: "ix://agent-ix/ecaz/StR-008"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-080"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-084"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-021: Distann Distribution Invariant

## Statement

An ec_distann configuration is **distributed** if and only if all five clauses
below hold, at every measured scale. The ec_distann index SHALL be distributed.

1. **Every O(N) structure is partitioned across the serving roster**, each node
   holding only its own partition. This covers graph adjacency, embedded
   neighbor codes, full-precision vectors, the row payload tier, directories,
   and every derived, optional, or default-off relation. Non-owner records: 0.
2. **No node outside the serving roster SHALL hold O(N) index state.**
   Coordinator-resident state is bounded by `k`, `L`, dimension, roster size,
   and relation or projection count — never by `N`.
3. **A structure the reference design distributes SHALL be distributed here even
   when it is small.** The FR-080 head index SHALL be sharded across the roster
   (§2.2) and servable with replication for capacity (§4.1) regardless of its
   capacity `C`. The shard owner counts as each shard's first serving node;
   `ec_distann.head_replica_count` (default 0) adds attested replicas beyond
   the owner, so the shipped default — every shard served by its owner —
   satisfies this clause under clause 5. Smallness, constancy in `N`, and a
   measured absence of storage pressure are **not** exemptions.
4. **No read path SHALL silently substitute a non-distributed structure for a
   distributed one.** A non-conforming accelerator is reachable only through an
   explicit opt-in, labels every result and every emitted row it produces, and
   is inadmissible as a decision control under NFR-022.
5. **These properties SHALL hold in the shipped default configuration**, not
   only in a benchmark arm. A property that requires a non-default flag to be
   true is not delivered.

This bound applies to **resident state**, not per-query work, and it applies to
**derived and optional structures** — caches, replicas, samples, summaries, and
performance objects — exactly as it applies to the authoritative index. A
structure does not escape this requirement by being rebuildable, optional,
off by default, filtered, tombstoned, or stored in a relation that is not
literally a graph-node shard.

The following are bounded and therefore permitted to be coordinator-resident or
replicated:

- per-scan result and candidate heaps, bounded by `k` and `L`;
- codec artifacts, codebooks, and rotation matrices, bounded by dimension;
- row-schema, endpoint, and prepared-plan caches, bounded by relation and
  projection count;
- a bounded number of immutable epoch entries retained for cache identity.

**The head index is not on that list.** A prior revision of this requirement
permitted a coordinator-resident head while its capacity `C` was constant in
`N`. That exemption is removed by clause 3: the reference design's head is a
"conventional **sharded** in-memory ANN index" (§2.2) whose replica count is the
stated remedy for head CPU pressure (§4.1), and neither property is conditioned
on its size. The exemption is removed because it was load-bearing in exactly the
wrong direction — it made the only unsharded structure in the system the one
structure the requirement blessed.

## Scope

- Applies to: every ec_distann node in any role — coordinator, participant, or
  both — at every scale, in every build profile, whether or not the structure is
  enabled by default.
- Measured on published-epoch state. Transient build state is reported
  separately under NFR-018 and is not covered here.
- The co-placed epoch row tier (ADR-085 D11, FR-078) is disjoint across owners
  and is the once-stored copy of corpus vectors; it satisfies this requirement
  by construction and is not a violation.
- A structure that is O(N) but genuinely sharded — each node holding only its
  own partition — satisfies this requirement. With a fixed roster, each
  owner's resident bytes are expected to grow approximately linearly with `N`;
  that growth is not replication. Replication of a *bounded* structure across
  nodes also satisfies this requirement.

## Rationale

This is the architectural property ec_distann exists to deliver, stated as a
requirement rather than as a rejection rationale.

The reference system (`DISTRIBUTEDANN`, arXiv:2509.06046) treats it as its
defining constraint. §2.2 declines to centralize even the compressed vectors —
"for a sufficiently large index, the array of compressed vectors will not be
able to fit in a single machine" — and accepts roughly 10× space amplification
distributed across the key-value store rather than create a single-machine
array. §2.3 places scoring on each storage host so only scores cross the
network. §2.4 keeps the orchestration service at heap-sized state precisely so
"it can be hosted on many machines with low overhead". Even the bounded head
index is "a conventional **sharded** in-memory ANN index".

This project has already paid for learning this once. ADR-067 rejected the
SPIRE CustomScan design because "**Storage does not scale out.** The
coordinator's local heap must hold a mirror of every row whose remote-shard
candidate may ever be returned. Aggregate dataset size is bounded by the
coordinator's single-machine storage capacity. The 'distributed' property is
limited to compute parallelism on a shared dataset, not storage scale-out" —
calling that "the most important architectural property of a distributed vector
search system". That rationale was never lifted into a requirement, and a
coordinator-resident full-graph structure was subsequently built, benchmarked,
and promoted without any gate rejecting it.

The existing bounds do not catch this class. StR-008, NFR-019, and ADR-085 all
bound *per-query work*; a full local copy violates none of them. NFR-018 bounds
storage but only as a **sum across nodes**, so a single node holding the entire
index is arithmetically invisible to it. NFR-021 supplies the missing per-node
term as a first-class requirement.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| max owner graph-side bytes per owned graph record, as a function of N across 10k/50k/100k | stable or sublinear | normalized growth ratio `(100k bytes / 100k owned records) ÷ (10k bytes / 10k owned records) ≤ 2.0` for every owner role | per-node storage and ownership audit, every gate run |
| non-owner graph-node records resident on any node (any relation, including derived) | 0 | 0 | topology audit |
| non-owner full-precision vectors resident on any node outside its own row tier | 0 | 0 | topology audit |
| unsharded O(N) derived-relation bytes resident on a coordinator or non-owner | 0 | 0 | relation-classified per-node storage audit |
| coordinator-resident index and index-derived bytes, itemised by relation | bounded structures only | every relation classified, and each one bounded by `k`, `L`, dimension, roster size, or relation count | per-node storage audit; an unclassified coordinator-resident relation makes the verdict `unavailable`, never a pass |
| head index bytes resident on a node outside the serving roster | 0 | 0 | per-node storage audit (head sample, head graph, and head cache attributed to their holding node) |
| head shard serving nodes | ≥ 1 per roster shard (the owner); + `head_replica_count` attested replicas | sharded across the roster; capacity `C` is not a factor | build manifest inspection (not yet mechanized — see Verification) |
| coordinator-owned graph records when the coordinator is outside the serving roster | 0 | 0 | topology audit |
| non-conforming accelerator reachable without an explicit opt-in | 0 | 0 | default-configuration run: the accelerator is never opened and no arm carries its label |

### Storage-class vocabulary

Every `physical_benchmark_storage_relation` row carries an `nfr_021_class`
tag. The class vocabulary is normative:

- `coordinator_resident_unsharded` — a corpus-derived, unsharded structure
  resident on the coordinator. Always a distribution gap: any non-zero bytes
  in this class are a hard violation (the known-gap allowlist is empty by
  design).
- `bounded` — a structure bounded by a stated parameter that is never a
  function of `N`. Admitted bounding parameters: `k`, `L`, dimension, roster
  size, relation/projection count (the permitted list in the Statement),
  head capacity `C` and its replica multiple (owner- or replica-resident
  head-shard state per FR-080), and the stated capacity GUCs
  (`ec_distann.gateway_copy_capacity`, `ec_distann.crown_capacity`). Within
  this class, the **bounded codes-only subclass** (FR-086 gateway copies,
  FR-089 crown) additionally holds only identifiers and quantized codes —
  never full-precision vectors — and is the only `bounded` form permitted
  to be coordinator-resident and corpus-derived. This vocabulary is the
  single normative definition; FR-086/FR-087/FR-089 cite it and SHALL NOT
  restate their own. The conformance reader skips `bounded` rows in the
  derived-bytes check, so the tag is load-bearing: **no emitter currently
  produces it**, and a producer claiming `bounded` SHALL guarantee the
  bound by construction and name the bounding parameter in the emitting
  code and the packet manifest. An unbounded structure tagged `bounded` is
  a conformance-machinery defect, not a pass.
- `control` — control-plane metadata: digests, counts, and the
  membership-only head's bounded id blob (roster-like state, not
  corpus-derived).

Any other or absent class on a coordinator-resident relation is
**unclassified** and falls under the unclassified-relation verdict rule in
the table above.

The suite's `outstanding_distribution_gap`/`unowned` reporting scaffolding is
dead machinery left over from the deleted known-gap allowlist; it has no spec
counterpart and sanctions nothing.

## Verification

Every ec_distann gate benchmark emits per-node resident bytes for all index and
index-derived relations, at every scale in the run, into `results.jsonl` — not
into a log-only sidecar. The suite joins resident bytes to owned-record counts,
computes the cross-scale bytes-per-owned-record ratio for each owner role, and
fails the run on breach. Raw fixed-roster byte growth is reported but is not a
conformance threshold because a valid O(N) shard necessarily grows with `N`
when roster size is held constant.

The topology audit enumerates every relation on every node, classifies each
derived relation as bounded or O(N), and asserts zero non-owner graph records,
zero non-owner vectors, and zero unsharded O(N) derived-relation bytes. Missing
ownership counts, relation classification, or scale endpoints make the
conformance verdict unavailable and fail a decision-bearing suite closed.

**Implementation gap — unclassified-relation verdict shape (audited
2026-08-01, candidate code fix).** The requirement is that an unclassified
coordinator-resident relation makes the verdict `unavailable`, never a pass.
The shipped reader instead folds unclassified rows into the derived-bytes
hard violation, producing a `nonconforming` verdict. This matters because
pre-registration matching treats `nonconforming` as a legitimate expected
outcome for a context lane: an arm pre-registered `nonconforming` matches the
verdict and passes, silently absorbing unclassified relations, whereas
`unavailable` matches no pre-registration and always fails a decision-bearing
suite closed. The spec text stands; the reader should be changed.

**Implementation gap — head-row verification (audited 2026-08-01).** What is
mechanically checked for the head today is `head_capacity_constant` across
scales plus zero-byte coordinator-resident head relations — indirect evidence
of shardedness. Build-manifest inspection of head shardedness and per-shard
replica count is not yet mechanized and remains an open obligation; until it
lands, packets satisfy the head replica-count row by manual manifest
inspection stated in the packet.

The per-node storage audit itemises **coordinator-resident** relations on the
same footing as owner relations. A coordinator row reporting zero without
enumerating what it holds does not satisfy this requirement; the head sample,
head graph, head cache, and every index-derived relation are attributed to the
node that holds them.

The final acceptance evidence for any change to placement or the read path is a
run in the **shipped default configuration**, with no arm-only flags, showing
every clause of the Statement holding.

A candidate that cannot satisfy this requirement is inadmissible under
[NFR-022](./NFR-022-distann-control-validity.md) and SHALL NOT be advanced to a
latency or recall A/B, regardless of its measured effect.

Conversely, work whose purpose is to *establish* conformance is not screened on
latency: it is delivered against this invariant, a measured latency cost is
reported rather than used as grounds to withhold the property, and any remedy is
a conforming optimization rather than a return to a non-conforming path.

## Dependencies

- **Upstream**: [StR-008](../stakeholder/StR-008-distributed-search-single-instance-economics.md)
- **Constrains**: [FR-078](../functional/distann/build/FR-078-distann-hash-placement.md),
  [FR-080](../functional/distann/read/FR-080-distann-coordinator-head-index.md),
  [FR-084](../functional/distann/read/FR-084-distann-coordinator-traversal-replica.md)
- **Related**: [NFR-018](./NFR-018-distann-space-amplification.md) (summed
  storage budget; NFR-021 supplies the per-node term),
  [NFR-019](./NFR-019-distann-per-query-touch-bound.md) (per-query work bound),
  [NFR-022](./NFR-022-distann-control-validity.md) (admissibility gate)
