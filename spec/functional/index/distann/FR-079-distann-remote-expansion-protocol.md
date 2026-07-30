---
id: FR-079
title: Distann Remote Expansion and Row Materialization Protocol
type: FR
status: PROPOSED
object: api_endpoint
relationships:
  - target: "ix://agent-ix/ecaz/FR-076"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-079: Distann Remote Expansion and Row Materialization Protocol

## Description

Each data node SHALL expose fingerprint-selected SQL endpoints for expanding
locally owned graph records and materializing their frozen source rows.

## Endpoint

SQL function on every data node, invoked by the coordinator once per owning
node per hop round over the pooled libpq transport:

`ec_distann_expand_nodes(index_regclass regclass, epoch_fingerprint bytea,
query real[], vec_ids bigint[], code_threshold real DEFAULT NULL,
candidate_limit integer DEFAULT NULL) RETURNS
TABLE (vec_id bigint, exact_dist real, is_tombstone bool, neighbor_vec_ids
bigint[], neighbor_code_dists real[])`

Final tuple materialization endpoint, invoked once per owner for each demanded
window of the proven result prefix:

`ec_distann_materialize_row_payloads(index_regclass regclass,
epoch_fingerprint bytea, vec_ids bigint[], projection_attnums smallint[],
expected_schema_fingerprint bytea) RETURNS TABLE (vec_id bigint,
is_tombstone bool, payload_nulls boolean[], payload_values bytea[])`

## Inputs

- `index_regclass` — the local ec_distann index
- `epoch_fingerprint` — the coordinator's active epoch identity
- `query` — full-precision query vector
- `vec_ids bigint[]` — locally-owned records to expand (≤ beam width)
- `code_threshold` — optional score floor; neighbors scoring below it MAY be
  omitted
- `candidate_limit` — optional owner-side limit `l`; after applying the score
  floor, the owner merges the candidates for the whole requested batch, sorts
  by `(code_distance, vec_id)`, and returns at most `l` candidates in total,
  redistributed across the response rows. `NULL` preserves the unbounded
  response.
- `projection_attnums` — non-dropped source attribute numbers required by the
  target list and coordinator-side quals, without duplicates
- `expected_schema_fingerprint` — the row-tier schema identity bound by the
  coordinator's scan epoch

## Outputs

Set of rows: `(vec_id, exact_dist, is_tombstone, neighbor_vec_ids bigint[],
neighbor_code_dists real[])`, one row per requested vec_id.

The materialization endpoint returns one `(vec_id, is_tombstone,
payload_nulls, payload_values)` row per requested vec_id in request order. Each
non-NULL `payload_values` element is the owning node's catalog-resolved
PostgreSQL binary representation for the corresponding `projection_attnums`
entry.

## Behavior

- Each endpoint SHALL resolve `epoch_fingerprint` to the exact retained
  Published generation before any record or row-tier read.
- If the fingerprint is unknown, Building, Ready, Retired-and-reclaimed, or
  inconsistent with the local manifest, then the endpoint SHALL raise a
  retriable epoch-mismatch error before reading data.
- Expansion SHALL perform exactly one logical graph-record read per requested
  vec_id. It SHALL perform exactly one co-located exact-vector row-tier read for
  each live requested record and MAY omit that read for a tombstoned record.
- Neighbor scoring SHALL use only the embedded neighbor codes from
  [FR-076](./FR-076-distann-graph-node-record-format.md).
  Both reads are node-local — the epoch row is co-placed by
  [FR-078](./FR-078-distann-hash-placement.md) — so the call remains one
  network round-trip per node per hop round with no separate rerank
  round-trip.
- Requested vec_ids SHALL resolve to exactly one of four defined outcomes:
  (a) present with its co-placed vector readable (row returned, `exact_dist`
  set, `is_tombstone` reflecting the flag); (b) not owned by this node under
  the epoch's placement → placement error (never an empty result); (c) record
  owned but absent → structural fault error; (d) record present but its
  co-placed vector missing or unreadable (`heap_tid` resolves nothing under
  the epoch) → structural fault error, distinct code from (c). Within a
  published epoch neither records nor the vector tier are physically
  reclaimed ([FR-082](./FR-082-distann-epoch-lifecycle.md)), so cases (c) and
  (d) always indicate corruption or co-placement drift, never a vacuum race
  ([NFR-020](../../../non-functional/NFR-020-distann-fault-behavior.md)); a
  mid-hop failure of either the record read or the vector read maps to the
  corresponding structural fault, non-retriable (distinct from the retriable
  epoch-mismatch of the first bullet).
- `heap_tid` SHALL be interpreted as the epoch-scoped handle to the vec_id's
  frozen co-placed source row ([FR-082](./FR-082-distann-epoch-lifecycle.md)).
- In a multi-owner epoch, `heap_tid` SHALL NOT be interpreted as a live
  source-table `ItemPointer`.
- In a one-owner `distributed_control=true` roster, `heap_tid` SHALL still
  reference the AM-owned frozen epoch row tier. Only the legacy
  `distributed_control=false` single-node lane MAY reference a local base-table
  tuple under the AM's tombstone/vacuum-consistency handling.
- Exact distances SHALL be computed against the node's co-placed
  full-precision vector (resolved via `heap_tid`,
  [FR-078](./FR-078-distann-hash-placement.md)) — not against any vector
  stored in the index record, which carries none — so the coordinator needs
  no separate rerank round-trip. This is exactly the `ec_diskann`
  coarse-search-then-heap-rerank split, executed node-locally; the
  rerank-fidelity source is the co-placed heap (the AM-owned frozen epoch heap
  for every distributed-control roster, or the base heap only for the legacy
  non-distributed single-node lane), exact in both cases (ADR-085 D11).
- For a tombstoned record, the expansion endpoint MAY omit the row-tier read
  and leave `exact_dist` unset (NULL).
- For a tombstoned record, the expansion endpoint SHALL return
  `is_tombstone = true` so its neighbor edges remain available for traversal.
- `code_threshold` and `candidate_limit` SHALL default to NULL (unbounded
  response). The production coordinator has a real retained-candidate heap
  parameter `L` (`ec_distann.candidate_heap_limit`). At each round it retains
  at most the best `L` live, unexpanded candidates, derives
  `t = peek_worst(H_C)` from the L-th retained candidate when that candidate
  exists, and passes the same constant `l = L` to the owner. The owner applies
  that limit once to the merged candidate set for the whole request, not once
  per requested vec_id; a multi-owner coordinator applies the same L bound to
  each owner share. Thus `l` is not a function of the remaining `BW × H`
  budget.
- Under this L-bounded derivation, the threshold is not an independent
  recall-risk heuristic: a neighbor below `t` is worse than every candidate
  retained in `H_C`, and per-owner top-`l` selection is the bounded response
  contract. The equivalence claim is specifically against the same L-bounded
  coordinator path with owner controls disabled; it does not claim identity
  with an unbounded frontier. Threshold ties are retained and resolved by the
  coordinator's `(distance, vec_id)` order; tombstones remain traversable and
  never contribute an `exact_dist` result.
- Equivalence requires a live threshold derived from the current retained heap,
  deterministic `(code_distance, vec_id)` ordering, and the stated `l` bound.
  A stale threshold, a different L, or an owner limit smaller than the
  declared per-response `l` is outside the guarantee and must be labeled in
  benchmark provenance. The cost bound is explicit: the coordinator stores
  O(L) live frontier candidates and each owner response returns at most `L`
  merged candidates before coordinator dedupe and re-retention.
- Gate benchmark runs use the coordinator-derived controls and must report the
  ordered-result identity against the same generation with the controls
  disabled as the reference path.
- The response schema of this function is a fixed wire contract independent
  of the record layout (ADR-085 D1): if neighbor codes move from embedded to
  piggybacked, the returned columns do not change.
- The expansion endpoint SHALL execute over the lifted async transport (connection
  pool, batched statements; operational/security posture per
  [NFR-014](../../../non-functional/NFR-014-spire-transport-security-and-operations.md))
  with one call per node per hop round.
- The materialization endpoint SHALL validate
  `expected_schema_fingerprint` against the selected epoch row tier before
  resolving any row.
- The materialization endpoint SHALL reject dropped, duplicate, zero, or
  out-of-range projection attnums as `EC_BAD_INPUT`.
- The materialization endpoint SHALL resolve each attribute's binary send
  function from the selected row-tier relation's PostgreSQL catalogs.
- The materialization endpoint SHALL NOT accept a caller-supplied function
  name or function OID.
- The materialization endpoint SHALL resolve every requested vec_id through
  the selected generation's owner-local directory.
- If an owned live record has no row-tier tuple, then the materialization
  endpoint SHALL raise `EC_VECTOR_MISSING` without returning a partial batch.
- If an owned vec_id is absent from the selected generation, then the
  materialization endpoint SHALL raise `EC_RECORD_MISSING` without returning a
  partial batch.
- If a requested vec_id is not owned by the participant, then the
  materialization endpoint SHALL raise `EC_PLACEMENT` without returning a
  partial batch.
- The coordinator SHALL reconstruct virtual tuples with its local catalog
  receive functions after validating the same schema fingerprint.
- Final payload work SHALL be executor-driven in deterministic windows of 10
  global-ranked slots. A request SHALL contain only pending remote vec_ids from
  the demanded window and SHALL end no later than the current proven prefix.
  The window size is fixed internal policy, not a reloption or production GUC.
- Search deepening SHALL preserve already-materialized slots in the stable
  ranked prefix. One remote vec_id SHALL NOT be requested twice in one scan
  solely because a deepened search rebuilt that prefix.
- The coordinator SHALL evaluate remaining SQL quals against reconstructed
  tuples before exposing rows to the executor. A qual that rejects a window
  SHALL advance demand to the next deterministic window, subject to the
  [NFR-019](../../../non-functional/NFR-019-distann-per-query-touch-bound.md)
  deepening ceiling.
- An error from any owner in any window, including a window requested after
  rows passed earlier quals, SHALL abort the query with zero completed result;
  the coordinator SHALL NOT reinterpret the already-seen prefix as a complete
  result.
- Every generated implementation function and normative SQL overload in the
  remote expansion/materialization endpoint class SHALL revoke `PUBLIC`
  execute, run as `SECURITY DEFINER`, and fix `search_path` to
  `pg_catalog`, the extension schema, and `pg_temp`. An unprivileged role SHALL
  not execute an implementation-signature sibling to bypass a secured wrapper.
- This protection is fail-closed across the extension-owned
  `ec_distann_*` SQL-function class, not a signature allowlist. New functions
  in that class SHALL inherit the same revoke/definer/search-path policy unless
  they are explicitly classified as a non-privileged public surface. The
  access-method handler, pure placement helper, and read-only epoch-status
  helper are the current public exceptions. Debug/test functions SHALL not be
  present in a production extension build.

## Error Conditions

| Code | Condition | Retriable | Partial rows allowed |
|------|-----------|-----------|----------------------|
| `EC_BAD_INPUT` | malformed fingerprint/query, invalid dimension, invalid projection attnum array, or request over its documented cap | no | no |
| `EC_EPOCH_MISMATCH` | unknown/non-readable fingerprint or local manifest disagreement | yes under FR-082 restart-once | no |
| `EC_EPOCH_FINGERPRINT_VERSION` | unknown fingerprint version | no | no |
| `EC_PLACEMENT` | requested vec_id hashes to another participant | no | no |
| `EC_RECORD_MISSING` | locally owned vec_id is absent from the selected directory | no | no |
| `EC_VECTOR_MISSING` | graph record exists but its epoch-row-tier tuple/vector is missing or unreadable | no | no |
| `EC_SCHEMA_MISMATCH` | expected materialization schema differs from the selected row tier | no | no |
| `EC_UNSUPPORTED_PROJECTION` | request references an unspecified system-column identity | no | no |
| `EC_REMOTE_INTERNAL` | local relation, catalog, decode, or storage failure not classified above | no | no |

Every endpoint SHALL raise the stable `EC_*` category with sanitized context and
zero returned rows when one request member fails.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-079-AC-1 | Response rows preserve request order and cover every requested vec_id | Test |
| FR-079-AC-2 | Stale epoch_fingerprint yields the retriable epoch-mismatch error, never data | Test |
| FR-079-AC-3 | Non-owned vec_id yields a placement error; owned-but-absent yields a structural fault error; tombstones return normally with the flag set | Test |
| FR-079-AC-4 | Neighbor code distances equal direct QuantCodec scoring of the same codes | Test |
| FR-079-AC-5 | `exact_dist` for each returned vec_id equals the full-precision distance between the query and the node's co-placed heap vector | Test |
| FR-079-AC-6 | Materialization returns one row per requested vec_id in request order and preserves NULLs and binary values for every requested attnum | Test (TC-040) |
| FR-079-AC-7 | Reconstructed tuples produce the same projection values and qual outcomes as the frozen build-snapshot source rows | Test (TC-040) |
| FR-079-AC-8 | Unknown generation, schema mismatch, invalid attnum, non-owner, missing record, and missing row-tier tuple each produce their documented error with zero partial rows | Test (TC-040, TC-042) |
| FR-079-AC-9 | Structural inspection proves the materialization request contains no caller-selected function name/OID and no raw conninfo | Test (TC-040) |
| FR-079-AC-10 | While an old epoch is retained, both old and new Published fingerprints resolve their own record and row-tier generations without cross-generation reads | Test (TC-042) |
| FR-079-AC-11 | With schema usage granted, an unprivileged role receives function-level permission denial from a real call to every protected extension-owned `ec_distann_*` overload enumerated from `pg_proc`; every protected overload is SECURITY DEFINER with the fixed safe search path, and production extension SQL contains no debug/test endpoint | Test (TC-040) |
| FR-079-AC-12 | Payload requests are deterministic global-ranked windows of at most 10 pending remote vec_ids, never cross the proven prefix, and do not re-request a materialized stable-prefix vec_id after deepening | Test (TC-040, TC-041) |
| FR-079-AC-13 | A later-window owner failure aborts the query instead of returning an earlier qualifying prefix as complete | Test (TC-042) |

## Dependencies

- **Upstream**: [FR-076](./FR-076-distann-graph-node-record-format.md),
  [FR-078](./FR-078-distann-hash-placement.md)
- **Downstream**: [FR-081](./FR-081-distann-query-orchestration.md),
  [FR-082](./FR-082-distann-epoch-lifecycle.md)
