---
id: FR-075
title: ec_distann Access Method Surface
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/StR-008"
    type: "implements"
    cardinality: "N:1"
---
# FR-075: ec_distann Access Method Surface

## Description

The extension SHALL provide a fifth index access method `ec_distann`,
registered via `CREATE ACCESS METHOD ec_distann TYPE INDEX HANDLER
ec_distann_handler`, implementing a single global Vamana graph over all
indexed vectors with hash-placed node records. The AM SHALL expose the same
operator-class surface as `ec_diskann` (inner-product ordered scans over the
extension's vector type) so existing corpora and bench tooling apply
unchanged.

## Inputs

- `CREATE INDEX ... USING ec_distann (embedding) WITH (<reloptions>)`
- Reloptions: `graph_degree` (R), `build_list_size`, `alpha`,
  `neighbor_code_format` (QuantCodec selector), `closure_epsilon`
  (build-shard overlap band), `head_index_cap` (C), plus the shared
  `source_identity` reloption whose identity contract is ADR-063 (reached via
  ADR-068's distributed topology; ADR-068 owns roster/placement, ADR-063 owns
  identity), and `distributed_control` (boolean, default `false`).
- Session GUCs: `ec_distann.beam_width` (BW), `ec_distann.hop_rounds` (H),
  registered at `_PG_init` via the module `register_gucs()` convention.

## Outputs

- A valid index relation whose scans return top-k tuples in score order.
- EXPLAIN output reporting the per-query traversal counters required by
  [NFR-019](../../non-functional/NFR-019-distann-per-query-touch-bound.md).

## Behavior

- The handler SHALL populate the `IndexAmRoutine` with build, insert,
  bulkdelete, vacuumcleanup, costestimate, options, beginscan, rescan,
  gettuple, and endscan callbacks following the `ec_diskann` routine pattern.
- The AM SHALL validate reloptions at `amoptions` time and reject
  out-of-range values with actionable errors.
- When `distributed_control = false`, `ambuild` SHALL retain the existing
  single-node behavior and build a directly scannable local graph.
- When `distributed_control = true`, `ambuild` SHALL create only a logical
  control index and SHALL NOT copy source graph records into that control
  relation. The control index SHALL remain query-invisible until every required
  participant generation is Published under
  [FR-078](./build/FR-078-distann-hash-placement.md) and the coordinator activates the
  cluster-wide epoch under
  [FR-082](./lifecycle/FR-082-distann-epoch-lifecycle.md).
- A distributed-control source table and index SHALL be permanent WAL-logged
  relations.
- `ambuild` SHALL reject temporary or unlogged persistence before
  initializing control metadata: an unlogged init fork cannot preserve the
  never-reused logical-index UUID across crash recovery, and physical
  generation catalogs and acknowledged handoff batches require WAL durability.
- `distributed_control = false` SHALL continue to write the existing 97-byte
  metadata format v4 byte-for-byte.
- The opt-in control SHALL write metadata
  format v5: the same 97-byte prefix with flags bit 0 set, followed by one
  non-zero 16-byte RFC 4122 version-4 logical-index UUID at byte offset 97
  (113 bytes total).
- The logical-index UUID SHALL have version nibble `4` and RFC 4122 variant
  bits `10`.
- The v5 control SHALL encode zero or invalid values, as appropriate, for its
  entry point, dimensions, node count, all graph/codebook/directory heads,
  content/delta counts, active epoch, and in-flight count.
- The decoder SHALL reject unknown flag bits, malformed/zero UUID bytes, or any
  local graph state. There is no implicit v4→v5 migration; the transition is
  explicit and rebuild-only under NFR-016.
- `REINDEX` of a distributed-control index SHALL be an explicit destructive
  control rebuild: in the same transaction it SHALL remove every old physical
  generation/internal relation and index-scoped catalog row, write a fresh
  never-reused logical-index UUID, and return to unpublished fail-closed state.
  It SHALL NOT silently attach an old generation to the fresh control identity.
- `distributed_control` is a build-mode reloption. `ALTER INDEX ... SET/RESET
  (distributed_control ...)` SHALL NOT reinterpret the existing bytes; it takes
  effect only through an explicit later `REINDEX`. A mode-changing `REINDEX` is
  likewise destructive: it removes all generation/catalog state, invalidates
  the prior logical UUID, and builds the selected lane from scratch. Converting
  to control creates a fresh unpublished UUID; converting to legacy creates a
  v4 local graph with no logical UUID.
- `CREATE INDEX CONCURRENTLY` is unsupported for
  `distributed_control = true` in v1. The AM SHALL reject the concurrent build
  as `EC_GENERATION_MISSING` before producing a usable control index, including
  for an empty source, and PostgreSQL may leave its normal invalid-index
  artifact for the operator to drop. Non-concurrent creation is the required
  surface.
- A distributed-control index SHALL be scanned only through the coordinator
  CustomScan path.
- While a distributed-control index lacks a Published manifest, the AM SHALL
  reject a direct ordinary index scan instead of returning an empty or
  legacy-local result.
- While the index participates in a multinode deployment, scans SHALL execute
  through the coordinator orchestration path of
  [FR-081](./read/FR-081-distann-query-orchestration.md). The deployment mode is
  determined by the published epoch manifest's node roster: roster size > 1
  is multinode; no session state or GUC overrides it.
- While the deployment is single-node, the AM SHALL serve the same plan shape
  with local expansion.
- The Vamana core (build, prune, traversal) is shared with `ec_diskann`
  ([FR-034](../index/diskann/FR-034-diskann-build-and-storage.md) lineage), not
  forked.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-075-AC-1 | `CREATE ACCESS METHOD ec_distann` succeeds and `CREATE INDEX ... USING ec_distann` builds a queryable index | Test |
| FR-075-AC-2 | Invalid reloption values are rejected at index creation with a descriptive error | Test |
| FR-075-AC-3 | Ordered top-k scans return results in non-increasing score order | Test |
| FR-075-AC-4 | Single-node distinct_recall@10 at 10k is within 0.002 of `ec_diskann` at equivalent parameters | Test (bench A/B via `ecaz bench suite`) |
| FR-075-AC-5 | `distributed_control = true` creates no local graph records, rejects non-permanent persistence and reads before first publish, and becomes queryable through the coordinator path only after every participant generation is Published and the cluster-wide epoch is active | Test (TC-040, TC-042) |
| FR-075-AC-6 | Dropping a control index dependency-cleans every physical relation/catalog row, while `REINDEX` removes the old generation state, writes a fresh UUID, and remains unpublished | Test (TC-040, TC-042) |

## Dependencies

- **Upstream**: [StR-008](../../stakeholder/StR-008-distributed-search-single-instance-economics.md)
- **Downstream**: [FR-076](./storage/FR-076-distann-graph-node-record-format.md),
  [FR-081](./read/FR-081-distann-query-orchestration.md)
