---
id: SR-004
type: SpecReview
analysis: dependency
scope: "StR-008; FR-075..FR-083 (spec/functional/index/distann); NFR-017..NFR-020; ADR-085; re-run against revision d25ea9e0c (inline vector → co-placed heap rerank, ADR-085 D11); reconciled at b19551e21"
review_set: all
title: "Dependency and Ordering Analysis: ec_distann Spec Batch"
---
# SR-004: Dependency and Ordering Analysis — ec_distann Spec Batch

## Summary

Dependency analysis of the ec_distann batch (StR-008, FR-075..FR-083,
NFR-017..NFR-020, ADR-085) against the planned milestone order
M0 (FR-075/076/080 single-node) → M1 (FR-077 stitch) → M2 (FR-078/079/081
two-node) → M3 (FR-082 lifecycle+faults) → M4 (bench gate) → M5 (FR-083
incremental insert).

This is a re-run after revision **d25ea9e0c**, which moved ec_distann from
inline full-precision vectors in the graph record to a **co-placed heap
rerank tier**: FR-076 records are now lean (coarse `search_code` + adjacency
+ neighbor codes, no inline vector); FR-078 co-places each record's
full-precision heap row on the same `hash(vec_id)`-owned node; FR-079 reads
`exact_dist` from that co-located local heap row; ADR-085 gains decision D11
(and D1 drops ~5.0× → ~4.0×); NFR-018 names the heap tier as the 1.0× ratio
denominator.

**Acyclicity: PASS (re-verified for the revision).** The declared
`depends_on` prerequisite graph is still a DAG. The revision did **not** add a
frontmatter `depends_on` edge FR-076 → FR-078: FR-076's frontmatter still
declares only FR-075, while FR-078 continues to `depends_on` FR-076 — so the
FR-076/FR-078 pair carries exactly one build-order edge and **no 2-node
cycle** exists (FND-013). The valid topological order is unchanged
(StR-008 → FR-075 → FR-076 → {FR-077, FR-078} → {FR-079, FR-080} → FR-081 →
FR-082 → FR-083).

**Revision introduced a new artifact — the co-placed heap tier — that is not
yet fully owned across the batch.** Five orphaned-assumption / broken-edge
findings result (FND-008..FND-012): FR-082's epoch lifecycle does not version,
freeze, fingerprint, or reclaim the heap tier (FND-008); FR-083's incremental
insert / write endpoint writes the new record but never co-places its heap row
(FND-009); the tests.md traceability matrix omits the three new
co-placement/no-inline-vector ACs (FND-010); FR-077's emitted-artifact
description does not acknowledge the heap-tier co-placement obligation FR-078
now hangs off it (FND-011); and NFR-018/NFR-019 gain load-bearing dependencies
on FR-078/D11 that their own Dependencies do not cite (FND-012).

**Disposition after the b19551e21 spec fixes: all dependency findings are now
resolved or addressed — none open.** The four milestone-ordering findings
(FND-001..FND-004) are ADDRESSED: the FR text is milestone-agnostic and the M0–M5
mapping is owned by the design doc's milestone table, which now carries the
degenerate single-shard head-sample case (FND-001), the M0 local-expansion slice
of FR-081 (FND-002), the M2 epoch-fingerprint subset of FR-082 (FND-003), and
FR-083's early-milestone delete/interim-insert slicing with the FR-083→FR-081
edge present (FND-004). FND-005/FND-006/FND-007 (edge symmetry, identity-citation
precision, lifted transport NFR-014) are RESOLVED — FR-075 now cites ADR-063 and
FR-081 now cites NFR-014. The five co-placed-heap-tier findings introduced by the
revision (FND-008..FND-012) are RESOLVED: FR-082 now versions/freezes/attests/
reclaims the co-placed vector tier (FND-008), FR-083 insert co-places the vector
(FND-009), the spec test matrix covers the new co-placement ACs and EC-024..027
(FND-010), FR-077 emits the full-precision rerank tier (FND-011), and
NFR-018/NFR-019 cite FR-078/D11 (FND-012). FND-013 remains informational
(no defect — the semantic-completeness edge is deliberately not a `depends_on`,
so no cycle).

## Classification

| Requirement | Class | Rationale |
|-------------|-------|-----------|
| StR-008 | Feature (stakeholder need) | Business-visible outcome: distributed search at single-instance economics |
| FR-075 | Enablement | AM registration, reloptions, GUCs, IndexAmRoutine scaffold — no standalone business behavior |
| FR-076 | Enablement | On-disk record format (now lean: coarse `search_code` + adjacency + neighbor codes, no inline vector) + global vec_id identity; schema-class work all other FRs consume. Exact-rerank completeness now depends on FR-078's co-placed heap tier (semantic, not build-order — FND-013) |
| FR-077 | Feature | Sharded build + stitch — the user-visible CREATE INDEX behavior and its recall-parity guarantee; its emitted artifact now feeds FR-078 record+heap co-placement (FND-011) |
| FR-078 | Enablement | Deterministic placement + directory metadata; now also co-places the full-precision heap row on the record's owning node (D11). Load-balance plumbing, invisible to results by design |
| FR-079 | Enablement | Remote expansion SQL endpoint; `exact_dist` now from the co-located local heap read, not an inline vector. Protocol surface consumed by FR-081, no user-visible behavior alone |
| FR-080 | Enablement | Coordinator-local head index; seeds FR-081, not independently observable |
| FR-081 | Feature | Top-k scan semantics — the query behavior StR-008 is satisfied by |
| FR-082 | Enablement | Epoch lifecycle/consistency machinery; governs FR-079/FR-081/FR-083 but does not yet own the co-placed heap tier (FND-008) |
| FR-083 | Feature | DML behavior (delete visibility, interim insert, incremental insert); insert path must now also co-place the new vector's heap row but does not (FND-009) |
| NFR-017 | Gate (constrains FR-081, StR-008) | M4 latency/recall bar |
| NFR-018 | Gate (constrains FR-076) | Space-amplification budget; the co-placed heap tier is now the 1.0× denominator (D11), a dependency on FR-078 its Dependencies omit (FND-012) |
| NFR-019 | Gate (constrains FR-081, StR-008) | BW×H touch bound; now the load-bearing justification for D11's affordability (reads==expansions==reranks==materialized), an edge D11 cites but NFR-019 does not record (FND-012) |
| NFR-020 | Gate (constrains FR-079/081/082) | Fault-behavior drills, M3 (+M5 insert cases) |

## Dependency Graph

```mermaid
graph TD
  StR008[StR-008: Distributed search economics]
  FR075[FR-075: AM surface]
  FR076[FR-076: Lean record + vec_id]
  FR077[FR-077: Sharded build + stitch]
  FR078[FR-078: Hash placement + heap co-placement]
  FR079[FR-079: Remote expansion protocol]
  FR080[FR-080: Coordinator head index]
  FR081[FR-081: Query orchestration]
  FR082[FR-082: Epoch lifecycle]
  FR083[FR-083: DML path]
  NFR017[NFR-017: Latency/recall gate]
  NFR018[NFR-018: Space budget]
  NFR019[NFR-019: Touch bound]
  NFR020[NFR-020: Fault behavior]

  StR008 --> FR075
  FR075 --> FR076
  FR076 --> FR077
  FR076 --> FR078
  FR076 --> FR079
  FR078 --> FR079
  FR077 --> FR080
  FR079 --> FR081
  FR080 --> FR081
  FR078 --> FR082
  FR079 --> FR082
  FR077 --> FR083
  FR082 --> FR083
  FR081 -.->|undeclared in FR-083 upstream, see FND-004| FR083
  FR082 -.->|fingerprint subset needed by M2, see FND-003| FR079
  FR077 -.->|head sample source, M0 gap, see FND-001| FR080

  FR078 -.->|co-placed heap tier: exact-rerank completeness, see FND-013| FR076
  FR077 -.->|must emit heap rows for co-placement, orphaned, see FND-011| FR078
  FR082 -.->|heap tier not epoch-versioned/frozen/reclaimed, see FND-008| FR078
  FR083 -.->|insert must co-place new heap row, orphaned, see FND-009| FR078
  FR078 -.->|1.0x denominator + affordability, uncited, see FND-012| NFR018
  NFR019 -.->|D11 affordability rests on BW×H bound, see FND-012| FR078

  FR076 --> NFR018
  FR081 --> NFR017
  FR081 --> NFR019
  FR079 --> NFR020
  FR081 --> NFR020
  FR082 --> NFR020
  StR008 --> NFR017
  StR008 --> NFR019
```

Solid edges are declared `depends_on`/downstream relationships; dotted edges
are undeclared or newly implied by the revision (findings). The
FR-078 ⇢ FR-076 dotted edge is a **semantic completeness** edge (the lean
record cannot supply an exact distance without its co-placed heap row), and is
deliberately *not* a `depends_on` — the only build-order edge in that pair is
FR-078 `depends_on` FR-076, so no cycle forms.

External contracts consumed (not part of this batch): ADR-068/ADR-063
source identity + FR-055 topology (FR-076 vec_id, FR-075 reloption);
ADR-056 eager-scan pattern (FR-081); ADR-067/post-142 SPIRE
CustomScan/transport pooling (FR-079/FR-081); SPIRE epoch-manifest
machinery (FR-082); `SpirePlacementDirectory` (FR-078); task-144
distance-ratio closure machinery (FR-077); Vamana core
`build_vamana_graph_with_stats`/`robust_prune` (FR-077/FR-080/FR-083);
`QuantCodec` (FR-076/FR-079). The co-placed heap tier reuses the
`ec_diskann` coarse-in-index / exact-from-heap split (ADR-085 D11).

## Topological Order (suggested implementation sequence)

1. FR-075, FR-076 — enablement (AM scaffold, then lean record format) [M0]
2. FR-077 — build + stitch (monolithic degenerate case first for M0, full stitch M1); must also emit/co-place heap rows per FR-078 (see FND-011)
3. FR-080 — head index [spec says M0; requires the FND-001 degenerate-case fix]
4. FR-081 local-expansion slice — single-node scan closing FR-075-AC-3/4 [M0, see FND-002]
5. FR-078 — placement + heap co-placement (enablement) [M2]
6. FR-082 publish/fingerprint subset — enablement for FR-079 validation; must also version/freeze the co-placed heap tier [see FND-003, FND-008]
7. FR-079 — expansion protocol (enablement), exact_dist from co-located heap read [M2]
8. FR-081 multinode — full orchestration [M2]
9. FR-082 full lifecycle + NFR-020 drills; heap-tier reclaim on retire [M3, see FND-008]
10. NFR-017/018/019 gate matrix [M4]
11. FR-083 incremental insert — must co-place the new vector's heap row (delete + interim posture land earlier per D5) [M5, see FND-009]

## Cycles

None detected in declared `depends_on` edges, re-verified for revision
d25ea9e0c. The revision introduces two body-level references worth checking:

- **FR-076 → FR-078/FR-079 (co-placement, exact rerank).** FR-076's prose and
  Layout now cite FR-078/FR-079 as the source of the exact-rerank vector, and
  FR-076's Downstream lists FR-078. These are *downstream* (consumers of
  FR-076) plus a *semantic completeness* reliance, **not** a build-order
  `depends_on`. FR-076's frontmatter still declares only FR-075. The revision
  correctly did **not** add a `depends_on` FR-076 → FR-078; had it, the
  existing FR-078 `depends_on` FR-076 would close a 2-node cycle. No cycle.
- The FR-079 ↔ FR-082 fingerprint mutual reference remains the same latent
  cross-milestone edge as before (FND-003); still not a declared cycle.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | medium | ADDRESSED — FR-080 states the single-shard monolithic degenerate case ("one medoid, one BFS sample"); the M0/M1 mapping is owned by the design doc's milestone table (FRs are milestone-agnostic). | FR-080, FR-077, ADR-085 D3 |
| FND-002 | medium | ADDRESSED — FR-081 states "The single-node local-expansion form of this loop is the first implementation slice (milestone M0)". | FR-075, FR-081 |
| FND-003 | medium | ADDRESSED — the design doc milestone table assigns the epoch-fingerprint-validation subset of FR-082 to M2 and full FR-082 to M3; the FR text is milestone-agnostic, so no cross-milestone cycle in the spec. | FR-079, FR-078, FR-082 |
| FND-004 | medium | ADDRESSED — FR-083's "Milestone slicing" note marks delete/tombstone + interim-insert as early-milestone prerequisites; TC-043 is tagged M3/M5; the FR-083→FR-081 edge is present. | FR-083, FR-081, ADR-085 D5, FR-075 |
| FND-005 | low | RESOLVED by d25ea9e0c. Prior one-sided edges are now bidirectional: FR-076's Downstream now lists FR-078 ("co-places the heap row"), matching FR-078's existing `depends_on` FR-076; and FR-078's `depends_on` FR-077 (pre-existing) matches FR-077's Downstream FR-078. Both FR-076↔FR-078 and FR-077↔FR-078 edges extract mechanically. No action. | FR-076, FR-077, FR-078 |
| FND-006 | low | RESOLVED (b19551e21) — FR-075 now cites ADR-063 for the identity contract (ADR-068 = topology); FR-076 already did. | FR-075, FR-076, ADR-068, ADR-063, FR-055 |
| FND-007 | low | RESOLVED — FR-079 already cited NFR-014; FR-081 now cites it on the pooled transport too. | FR-079, FR-081, NFR-014, ADR-085 |
| FND-008 | medium | RESOLVED — FR-082 assembly/publication tuple/D10 immutability/fingerprint/reclaim now all include the co-placed vector tier; +AC-5/AC-6. | FR-082, FR-078, ADR-085 D10, ADR-085 D11 |
| FND-009 | medium | RESOLVED — FR-083 insert + write endpoint co-place the vector; +AC-7. | FR-083, FR-078, FR-079, ADR-085 D11 |
| FND-010 | medium | RESOLVED — spec-matrix updated FR-076/078/079/082/083 AC coverage + EC-024..027. | spec/tests.md, FR-076, FR-078, FR-079 |
| FND-011 | low | RESOLVED — FR-077 now emits each vec_id's full-precision vector as the co-placed rerank tier. | FR-077, FR-078, ADR-085 D11 |
| FND-012 | low | RESOLVED — NFR-018 Dependencies now cite FR-078 + D11; NFR-019 records the D11 rerank-read equality. | NFR-018, NFR-019, FR-078, ADR-085 D1, ADR-085 D11 |
| FND-013 | low | **Cycle check (informational, no defect).** FR-076's body/Layout now cite FR-078/FR-079 for exact-rerank co-placement and FR-076's Downstream lists FR-078; the record's exact-rerank *completeness* is now provided by FR-078. The revision correctly kept this out of FR-076's frontmatter `depends_on` (still FR-075 only), so no FR-076↔FR-078 build-order cycle exists against FR-078's `depends_on` FR-076. This dependency graph represents that reliance as a dotted semantic-completeness edge (FR-078 ⇢ FR-076) distinct from the solid build-order edge, so the analysis stays faithful to the lean-record/co-placed-heap split without implying a cycle. | FR-076, FR-078, FR-079 |
