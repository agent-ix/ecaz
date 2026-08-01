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
- Reloptions: `graph_degree` (R, default 32), `build_list_size` (default 100),
  `alpha` (default 1.2), `neighbor_code_format` (QuantCodec selector, default
  `rabitq`), `closure_epsilon` (build-shard overlap band, default 0.3),
  `head_index_cap` (C, default 4096), `build_shards` (default 1: `0` selects
  the automatic shard-count policy, `1` the monolithic build, `>= 2` the
  sharded closure-overlap build; the automatic policy and shard semantics are
  owned by [FR-077](./build/FR-077-distann-sharded-build-and-stitch.md)),
  plus the shared `source_identity` reloption whose identity contract is
  ADR-063 (reached via ADR-068's distributed topology; ADR-068 owns
  roster/placement, ADR-063 owns identity), and `distributed_control`
  (boolean, default `false`).
- Scan-shaping session GUCs (userset), registered at `_PG_init` via the
  module `register_gucs()` convention: `ec_distann.beam_width` (BW, default
  4), `ec_distann.hop_rounds` (H, default 100),
  `ec_distann.candidate_heap_limit` (retained-frontier bound L, default 32;
  semantics in [FR-079](./read/FR-079-distann-remote-expansion-protocol.md)
  and [FR-081](./read/FR-081-distann-query-orchestration.md)), and
  `ec_distann.top_k` (default 10) — the FR-081 convergence early-exit bar k.
  `ec_distann.top_k` SHALL be a performance hint, not a correctness knob:
  when a consumer reads past the proven top-k prefix (for example a SQL
  `LIMIT` above k), the scan SHALL transparently re-run with a doubled bar
  (iterative deepening).
- Head-topology session GUCs (Task 210): `ec_distann.shard_head_storage`
  (default on), `ec_distann.sharded_head_search` (default on),
  `ec_distann.head_replica_count` (default 0), and
  `ec_distann.gateway_copy_capacity` (default 0). Their normative semantics
  are owned by [FR-080](./read/FR-080-distann-coordinator-head-index.md) and
  [FR-086](./read/FR-086-distann-gateway-copies.md); FR-075 only registers
  the surface. Specced but not yet implemented (join this registry when
  their tasks land): `ec_distann.crown_capacity` (default 0,
  [FR-089](./read/FR-089-distann-crown-cache.md)) and the
  [FR-088](./read/FR-088-distann-head-scaling-law.md) sizing reloptions
  `head_sampling_rate` (default 0), `head_cap_floor` (default 4096), and
  `head_cap_ceiling` (default 1,048,576).
- `ec_distann.allow_nonconforming_replica` (default off): explicit opt-in
  for the non-conforming coordinator traversal replica of
  [FR-084](./read/FR-084-distann-coordinator-traversal-replica.md) under
  [NFR-021](../../non-functional/NFR-021-distann-distribution-invariant.md)
  clause 4.
- Operational session GUCs: `ec_distann.remote_connect_timeout_ms` (default
  5000) and `ec_distann.remote_statement_timeout_ms` (default 120000) bound
  remote participant RPCs; `ec_distann.physical_epoch_cache` (default on) is
  the kill switch for the per-backend cache of validated immutable epoch
  descriptors and head graphs; `ec_distann.scan_profile_notice` (default
  off) emits the per-query
  [NFR-019](../../non-functional/NFR-019-distann-per-query-touch-bound.md)
  traversal counters as NOTICE; `ec_distann.replica_control_password_file`
  (string, SIGHUP context) names a server-local mode-0600 credential file
  for the dedicated replica control connection.
- Postmaster-start capacity GUCs: `ec_distann.max_scan_pins` and
  `ec_distann.max_retire_fences` size the shared scan-token/fence registry
  of [FR-082](./lifecycle/FR-082-distann-epoch-lifecycle.md); zero disables
  allocation and makes distributed registration fail closed.
- Fixture/bootstrap roster GUCs (userset): `ec_distann.roster`
  (placement-ordered `node_id@conninfo` entries separated by `;`),
  `ec_distann.local_node_id` (default 0), and `ec_distann.epoch` (default 0)
  configure the legacy lane only; their scope and boundaries are defined
  under Behavior (deployment mode and lanes).
- Debug fault-injection GUC class: eight off-by-default
  `ec_distann.debug_*` GUCs (`debug_fail_hop_round`,
  `debug_missing_node_record`, `debug_fail_insert`,
  `debug_fail_handoff_after_prepare`, `debug_fail_recover_after_publish_ack`,
  `debug_crash_after_replica_control_commit`, `debug_source_capture_fault`,
  `debug_fail_tombstone_write`) inject the
  [NFR-020](../../non-functional/NFR-020-distann-fault-behavior.md)
  fault-drill boundaries; they SHALL be inert at their defaults.
- Benchmark-only GUC family: the `ec_distann.benchmark_*` GUCs are
  compile-gated behind the `distann-head-attribution-benchmark` cargo
  feature and are absent from normal production builds.

## Outputs

- A valid index relation whose scans return top-k tuples in score order.
- EXPLAIN output reporting the per-query traversal counters required by
  [NFR-019](../../non-functional/NFR-019-distann-per-query-touch-bound.md).
  **Implementation gap (Task 214 audit, 2026-08-01):** this output does not
  ship — the coordinator CustomScan registers no `ExplainCustomScan`
  callback, so no EXPLAIN counter surface exists. The counters are currently
  observable only through the off-by-default
  `ec_distann.scan_profile_notice` NOTICE GUC. The requirement is retained
  as an open obligation (cross-ref
  [NFR-019](../../non-functional/NFR-019-distann-per-query-touch-bound.md)).

## Behavior

- The handler SHALL populate the `IndexAmRoutine` with build, insert,
  bulkdelete, vacuumcleanup, costestimate, options, beginscan, rescan,
  gettuple, and endscan callbacks following the `ec_diskann` routine pattern.
- The AM SHALL validate integer, real, and boolean reloptions at `amoptions`
  time and reject out-of-range values with actionable errors.
- The string reloptions (`neighbor_code_format`, `source_identity`) are
  registered without amoptions-time validators; their values are parsed when
  a build consumes them. Consequently `CREATE INDEX` SHALL fail immediately
  on an invalid string value (`ambuild` parses before any storage work), but
  `ALTER INDEX ... SET (neighbor_code_format = <invalid>)` is accepted at
  ALTER time and fails at next use.
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
- Deployment mode and lanes. The AM ships two build lanes. The
  `distributed_control = true` physical lane is the design surface (the
  [FR-085](./FR-085-distann-domain-model.md) Bounded Context): its deployment
  mode is determined by the published epoch manifest's node roster — roster
  size > 1 is multinode — and on this lane no session state or GUC overrides
  it. The `distributed_control = false` legacy lane is fixture/bootstrap
  substrate, not part of the distributed domain model: it derives
  multinode-ness and the node set from the session GUCs `ec_distann.roster`
  and `ec_distann.local_node_id`, and its scan epoch falls back to the
  `ec_distann.epoch` GUC only while the index metadata carries no Published
  epoch (a Published index always reads the persisted `active_epoch`). The
  roster GUCs SHALL have no effect on the physical lane's roster or active
  epoch.
- While the index participates in a multinode deployment (under either
  lane's mode derivation), scans SHALL execute through the coordinator
  orchestration path of
  [FR-081](./read/FR-081-distann-query-orchestration.md).
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
| FR-075-AC-7 | Every production session GUC listed in Inputs is registered at `_PG_init` with its documented default (`beam_width`=4, `hop_rounds`=100, `candidate_heap_limit`=32, `top_k`=10, `shard_head_storage`=on, `sharded_head_search`=on, `head_replica_count`=0, `gateway_copy_capacity`=0, `allow_nonconforming_replica`=off, `physical_epoch_cache`=on, `scan_profile_notice`=off, `remote_connect_timeout_ms`=5000, `remote_statement_timeout_ms`=120000) | Test |
| FR-075-AC-8 | `CREATE INDEX` with an invalid `neighbor_code_format` or `source_identity` string value fails with a descriptive error before any index storage is published | Test |
| FR-075-AC-9 | On the physical lane, setting `ec_distann.roster` or `ec_distann.epoch` in a session changes neither the scan's node set nor its active epoch | Test |

## Dependencies

- **Upstream**: [StR-008](../../stakeholder/StR-008-distributed-search-single-instance-economics.md)
- **Downstream**: [FR-076](./storage/FR-076-distann-graph-node-record-format.md),
  [FR-081](./read/FR-081-distann-query-orchestration.md)
