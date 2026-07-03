# Review Request: Task 137 — Distributed Result Deduplication via Source-Identity Provider

Status: evidence complete; review requested. The fix meets every
pre-registered criterion at 10k/50k/100k.

## Summary

Task 131 packet 027 proved the distributed SPIRE production read returns the
same corpus row multiple times inside one top-k result (183/200
duplicate-containing top-10s at 10k, 1000/1000 at 50k). This packet lands the
Task 137 fix and its A/B evidence.

## Confirmed mechanism (task file asked to confirm, not assume)

The suspected replica/vec_id mechanism is confirmed, with one refinement: the
final merge is **not** using the wrong key — there is no right key to use on
the current surface.

- The local multi-instance fixture exports overlapping per-node row slices
  from the coordinator's leaf assignments (`boundary_replica_count=4` places
  the same corpus row's leaves on up to all 3 remote nodes), then each remote
  builds an **independent** index over its slice
  (`write_leaf_owned_local_plan` / `remote_load_args`,
  `crates/ecaz-cli/src/commands/dev/spire_multicluster.rs`).
- Each remote index allocates node-local `0x01` vec_ids from its own root
  control sequence. Copies of the same corpus row on different nodes carry
  unrelated vec_ids; different rows on different nodes can share one vec_id.
- The merge dedupe key (`remote_search_candidate_dedupe_key`,
  `src/am/ec_spire/coordinator/remote_candidates/pipeline.rs`) therefore
  scopes local vec_ids by node — per ADR-063, which already states such
  indexes "cannot make cross-node replica dedupe claims". Merging bare local
  vec_ids across nodes would collapse *different* rows: strictly worse.
- The duplicate runs of 2-3 identical ids at adjacent ranks in the packet 027
  identity artifacts match one row served by 2-3 of the 3 nodes at identical
  distance.

## The fix (ADR-083)

The correct global row identity is the ADR-055/063 source-identity payload.
The merge already dedupes global `0x02` vec_ids across nodes (unit-covered:
`remote_candidate_merge_dedupes_global_vec_ids_across_nodes`,
`remote_heap_candidate_result_merge_dedupes_global_vec_ids_across_nodes`,
boundary-replica global-id write coverage in `src/tests/insert.rs`). What was
missing is any way for the distributed load path to engage the provider:

- `ecaz corpus load --reloption source_identity=include` (ec_spire only) now
  creates the corpus table with a stored generated 16-byte identity column
  (`sha256(int8send(id))[..16]`, the same derivation as the loader's static
  shard routing) and builds the index with `INCLUDE (source_identity)` — the
  ADR-063 v1 DDL shape. Fails closed for non-ec_spire profiles and chunked
  loads.
- The reloption flows through the existing suite-config plumbing to the
  coordinator and all remote multinode loads; no fixture code change.
- No AM code change. The `requires_global_vec_id` boundary-replica identity
  diagnostic already covers the operator surface.
- ADR-083 records the decision: distributed SPIRE read surfaces must run with
  the provider on; no heuristic cross-node local-id dedupe.

## Evidence (artifacts/, manifest.md)

A/B per change at 10k/50k/100k, n128/b4, nprobe=96, k=10, 200 queries per
cell: `identity-off` (pre-fix surface) vs `identity-on` (fix engaged).

Key results (full table + decision-rule readout in `artifacts/manifest.md`):

- **Zero duplicate returned IDs in all 600 identity-on queries** across
  10k/50k/100k; every identity-off arm reproduces the defect, with the 10k
  off arm matching Task 131 packet 027 exactly (183/200, distinct 0.5195).
- **True recall roughly doubles**: distinct_recall@10 goes 0.5195→0.9855
  (10k), 0.4115→0.9730 (50k), 0.4265→0.9810 (100k). The tolerant and
  distinct metrics converge exactly in every on arm.
- **Latency neutral**: p50 +0.4% / -1.7% / -1.4%; off arms track historical
  baselines.
- **Storage accounted**: per-vector index bytes +10.4-11.0% (16-byte identity
  x 5 assignment copies at b4 + wider global vec_id); 100k coordinator build
  +2.6%.
- Strict/degraded semantics unchanged (no AM code change; fault drills not
  rerun — the read path is byte-identical code).

The old published recall for this surface (0.9985-1.0000) is now quantified
as duplicate inflation; the honest distinct recall at nprobe=96 is
~0.97-0.99, the baseline Task 139 works against.

Pre-fix failing artifact: Task 131 packet 027 identity JSONL (cited in the
task file) plus this packet's fresh `identity-off` arms.

## Metric hardening note

Task 138 owns the duplicate-tolerant metric fix; this packet's distinct
numbers use the Task 138 `distinct_recall@k` runner surface
(`ecaz bench rescore-identity` and the extended query-metrics table). Older
Task 123/131 distributed recall numbers predate the fix and are
duplicate-tolerant; the benchmark readout note the task file requires lives
in `reviews/task-138/001-distinct-recall-rescore/` (corrected table) and
ADR-083's consequences section.
