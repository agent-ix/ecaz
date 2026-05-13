# Review Request: SPIRE Tuple Transport Measurement

- coder: coder1
- code/evidence commit: `4770bc8df9a986725dbb1feeea5cf27134ba4d42`
- tracker row: Phase 12.2 tuple-heavy read throughput measurement

## Scope

This packet closes the Phase 12.2 row to measure tuple-heavy read throughput
before and after typed transport. It does not retire the JSON endpoint and does
not claim a typed speedup.

The fixture is a local PG18 pgrx loopback setup with separate remote and
coordinator `ec_spire` indexes:

- remote table/index:
  `phase12_tuple_measure_remote` / `phase12_tuple_measure_remote_idx`
- coordinator table/index:
  `phase12_tuple_measure_corpus` / `phase12_tuple_measure_coord_idx`
- storage format: `rabitq`
- query shape: 20 coordinator KNN queries, `nprobe=8`, `k=10`
- projected payload: `id,title,body`
- payload materialized by each run: 200 rows and 31,510 text bytes

## Evidence

Packet-local artifact metadata is in
`review/30975-spire-tuple-transport-measurement/artifacts/manifest.md`.

Warm-pass summary lines:

- JSON fallback:
  `json_tuple_payload_v1 20 200 31510 29.342 28.988 30.459 35.103 34.081`
- typed transport:
  `pg_binary_attr_v1 20 200 31510 34.293 33.892 35.179 41.313 29.160`

Columns are:
`transport query_count rows_returned payload_bytes avg_ms p50_ms p95_ms p99_ms queries_per_second`.

The fixture summary confirms:

- `remote_rows 2000`
- `coordinator_rows 2000`
- `query_rows 40`
- both indexes use `storage_format=rabitq`
- endpoint identity reports `pg_binary_attr_v1 ready {pg_binary_attr_v1}`
- remote snapshot has local node `0` with 3 placements and remote node `2`
  with 16 placements

## Notes For Review

The `ecaz bench spire-pipeline` Rust client path was not used as final evidence
because the v1 DML frontdoor fail-closed guard fired for this remote-placement
KNN shape under `tokio-postgres`. The packet uses `target/debug/ecaz dev sql`
and psql dynamic SQL instead; the JSON and typed runs use the same SQL driver
shape, differing only by `ec_spire.remote_tuple_transport`.

Reviewer focus:

- Is the packet-local evidence enough to close the measurement row?
- Is the no-speedup conclusion stated narrowly enough for this small scalar
  loopback fixture?
- Are the remaining open Phase 12 rows still correctly scoped to JSON
  retirement, cost calibration, full artifact capture, and final readiness
  bundle?
