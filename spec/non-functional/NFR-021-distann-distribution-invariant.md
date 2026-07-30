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

The ec_distann index SHALL shard every structure whose size is O(N) in the
number of indexed vectors across the serving roster, without retaining a
complete corpus-proportional copy on a coordinator or non-owner.

This bound applies to **resident state**, not per-query work, and it applies to
**derived and optional structures** — caches, replicas, samples, summaries, and
performance objects — exactly as it applies to the authoritative index. A
structure does not escape this requirement by being rebuildable, optional,
off by default, filtered, tombstoned, or stored in a relation that is not
literally a graph-node shard.

The following are bounded and therefore permitted to be coordinator-resident or
replicated:

- the FR-080 head index at its fixed capacity `C` (ADR-085 D3), provided `C` is
  a constant independent of `N`; if `C` is ever made a function of `N`, the
  ec_distann index SHALL shard the head across the roster;
- per-scan result and candidate heaps, bounded by `k` and `L`;
- codec artifacts, codebooks, and rotation matrices, bounded by dimension;
- row-schema, endpoint, and prepared-plan caches, bounded by relation and
  projection count;
- a bounded number of immutable epoch entries retained for cache identity.

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
| head index capacity `C` | constant in N | constant, or sharded across the roster | build manifest inspection |
| coordinator-owned graph records when the coordinator is outside the serving roster | 0 | 0 | topology audit |

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

A candidate that cannot satisfy this requirement is inadmissible under
[NFR-022](./NFR-022-distann-control-validity.md) and SHALL NOT be advanced to a
latency or recall A/B, regardless of its measured effect.

## Dependencies

- **Upstream**: [StR-008](../stakeholder/StR-008-distributed-search-single-instance-economics.md)
- **Constrains**: [FR-078](../functional/index/distann/FR-078-distann-hash-placement.md),
  [FR-080](../functional/index/distann/FR-080-distann-coordinator-head-index.md),
  [FR-084](../functional/index/distann/FR-084-distann-coordinator-traversal-replica.md)
- **Related**: [NFR-018](./NFR-018-distann-space-amplification.md) (summed
  storage budget; NFR-021 supplies the per-node term),
  [NFR-019](./NFR-019-distann-per-query-touch-bound.md) (per-query work bound),
  [NFR-022](./NFR-022-distann-control-validity.md) (admissibility gate)
