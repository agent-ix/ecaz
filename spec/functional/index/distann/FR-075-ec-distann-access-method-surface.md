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
  identity).
- Session GUCs: `ec_distann.beam_width` (BW), `ec_distann.hop_rounds` (H),
  registered at `_PG_init` via the module `register_gucs()` convention.

## Outputs

- A valid index relation whose scans return top-k tuples in score order.
- EXPLAIN output reporting the per-query traversal counters required by
  [NFR-019](../../../non-functional/NFR-019-distann-per-query-touch-bound.md).

## Behavior

- The handler SHALL populate the `IndexAmRoutine` with build, insert,
  bulkdelete, vacuumcleanup, costestimate, options, beginscan, rescan,
  gettuple, and endscan callbacks following the `ec_diskann` routine pattern.
- The AM SHALL validate reloptions at `amoptions` time and reject
  out-of-range values with actionable errors.
- While the index participates in a multinode deployment, scans SHALL execute
  through the coordinator orchestration path of
  [FR-081](./FR-081-distann-query-orchestration.md). The deployment mode is
  determined by the published epoch manifest's node roster: roster size > 1
  is multinode; no session state or GUC overrides it.
- While the deployment is single-node, the AM SHALL serve the same plan shape
  with local expansion.
- The Vamana core (build, prune, traversal) is shared with `ec_diskann`
  ([FR-034](../diskann/FR-034-diskann-build-and-storage.md) lineage), not
  forked.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-075-AC-1 | `CREATE ACCESS METHOD ec_distann` succeeds and `CREATE INDEX ... USING ec_distann` builds a queryable index | Test |
| FR-075-AC-2 | Invalid reloption values are rejected at index creation with a descriptive error | Test |
| FR-075-AC-3 | Ordered top-k scans return results in non-increasing score order | Test |
| FR-075-AC-4 | Single-node distinct_recall@10 at 10k is within 0.002 of `ec_diskann` at equivalent parameters | Test (bench A/B via `ecaz bench suite`) |

## Dependencies

- **Upstream**: [StR-008](../../../stakeholder/StR-008-distributed-search-single-instance-economics.md)
- **Downstream**: [FR-076](./FR-076-distann-graph-node-record-format.md),
  [FR-081](./FR-081-distann-query-orchestration.md)
