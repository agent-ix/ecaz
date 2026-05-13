# Artifact Manifest: SPIRE Tuple Transport Measurement

- head SHA at run time: `560c71b0cbb31051d035b345e156e45285613b52`
- packet/topic: `30975-spire-tuple-transport-measurement`
- timestamp: `2026-05-13T07:11:33Z`
- fixture: local PG18 pgrx database `spire_phase12_measure`, loopback remote
  descriptor `spire/remote/loopback`, coordinator table
  `phase12_tuple_measure_corpus`, remote table
  `phase12_tuple_measure_remote`, coordinator index
  `phase12_tuple_measure_coord_idx`, remote index
  `phase12_tuple_measure_remote_idx`.
- surface shape: isolated two-index loopback fixture, with one remote
  table/index and one coordinator table/index. This is not the old
  shared-table remote surface.
- storage format / rerank mode: `rabitq`, `nprobe=8`, default rerank settings,
  `k=10`, scalar tuple payload projection `title,body`.

## Setup Artifacts

### `install-ecaz-pg18-pg-test.log`

- command:
  `target/debug/ecaz dev install ecaz-pg-test --pg 18 --log-file review/30975-spire-tuple-transport-measurement/artifacts/install-ecaz-pg18-pg-test.log`
- purpose: install current PG18 pg_test extension before recreating the
  fixture.
- key result: backend artifact assertion passed and installed
  `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`.

### `create-measure-db-scalar.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database tqvector_bench --sql 'DROP DATABASE IF EXISTS spire_phase12_measure; CREATE DATABASE spire_phase12_measure' --log-output review/30975-spire-tuple-transport-measurement/artifacts/create-measure-db-scalar.log`
- purpose: recreate a clean measurement database.

### `create-scalar-twoindex-fixture.sql.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_measure --file review/30975-spire-tuple-transport-measurement/artifacts/create-scalar-twoindex-fixture.sql --log-output review/30975-spire-tuple-transport-measurement/artifacts/create-scalar-twoindex-fixture.sql.log`
- purpose: create extension, load 2,000 remote/corpus rows, load 40 query
  vectors, and build separate remote/coordinator `ec_spire` indexes.
- key result lines:
  - `INSERT 0 2000`
  - `SELECT 2000`
  - `SELECT 40`
  - `CREATE INDEX`
  - `CREATE INDEX`

### `fixture-summary.sql` and `fixture-summary.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_measure --file review/30975-spire-tuple-transport-measurement/artifacts/fixture-summary.sql --log-output review/30975-spire-tuple-transport-measurement/artifacts/fixture-summary.log`
- purpose: capture packet-local fixture counts, index reloptions, remote
  snapshot, and endpoint tuple transport readiness after setup.
- key result lines:
  - `remote_rows 2000`
  - `coordinator_rows 2000`
  - `query_rows 40`
  - `phase12_tuple_measure_coord_idx {nlists=16,nprobe=8,rerank_width=16,recursive_fanout=2,nprobe_per_level=2,storage_format=rabitq}`
  - `phase12_tuple_measure_remote_idx {nlists=16,nprobe=8,rerank_width=16,recursive_fanout=2,nprobe_per_level=2,storage_format=rabitq}`
  - remote snapshot rows show local node `0` with 3 placements and remote
    node `2` with 16 placements.
  - `pg_binary_attr_v1 ready {pg_binary_attr_v1}`

### `register-loopback-scalar.sql.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_measure --file review/30975-spire-tuple-transport-measurement/artifacts/register-loopback-scalar.sql --log-output review/30975-spire-tuple-transport-measurement/artifacts/register-loopback-scalar.sql.log`
- purpose: rewrite coordinator leaf placements to remote node 2 and register
  the loopback remote descriptor.
- key result lines:
  - `t`
  - `16`
  - `0 ready 3`
  - `2 ready 16`
  - `ready pg_binary_attr_v1 ready`

### `customscan-json-smoke-scalar.sql.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_measure --file review/30975-spire-tuple-transport-measurement/artifacts/customscan-json-smoke-scalar.sql --log-output review/30975-spire-tuple-transport-measurement/artifacts/customscan-json-smoke-scalar.sql.log`
- purpose: smoke a scalar tuple-payload CustomScan read with
  `json_tuple_payload_v1`.
- key result lines:
  - `1 title-1 120`
  - `711 title-711 144`
  - `1421 title-1421 156`

## Measurement Artifacts

### `bench-json-simple.sql`

- command source for the JSON fallback run.
- lane / fixture / storage / rerank: local PG18 loopback, scalar
  `title,body` payload, `rabitq`, `nprobe=8`, default rerank.
- mode: `SET ec_spire.remote_tuple_transport = 'json_tuple_payload_v1'`.
- shape: 20 dynamic SQL coordinator KNN queries, each returning `k=10` rows
  and materializing `id,title,body`.

### `bench-json-simple.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_measure --file review/30975-spire-tuple-transport-measurement/artifacts/bench-json-simple.sql --log-output review/30975-spire-tuple-transport-measurement/artifacts/bench-json-simple.log`
- key result:
  `json_tuple_payload_v1 20 200 31510 30.231 29.753 31.829 35.525 33.079`
- columns:
  `transport query_count rows_returned payload_bytes avg_ms p50_ms p95_ms p99_ms queries_per_second`.

### `bench-json-simple-warm2.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_measure --file review/30975-spire-tuple-transport-measurement/artifacts/bench-json-simple.sql --log-output review/30975-spire-tuple-transport-measurement/artifacts/bench-json-simple-warm2.log`
- key result:
  `json_tuple_payload_v1 20 200 31510 29.342 28.988 30.459 35.103 34.081`

### `bench-typed-simple.sql`

- command source for the typed transport run.
- lane / fixture / storage / rerank: same as JSON run.
- mode: `SET ec_spire.remote_tuple_transport = 'pg_binary_attr_v1'`.

### `bench-typed-simple.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_measure --file review/30975-spire-tuple-transport-measurement/artifacts/bench-typed-simple.sql --log-output review/30975-spire-tuple-transport-measurement/artifacts/bench-typed-simple.log`
- key result:
  `pg_binary_attr_v1 20 200 31510 32.672 32.199 33.942 39.107 30.607`

### `bench-typed-simple-warm2.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_measure --file review/30975-spire-tuple-transport-measurement/artifacts/bench-typed-simple.sql --log-output review/30975-spire-tuple-transport-measurement/artifacts/bench-typed-simple-warm2.log`
- key result:
  `pg_binary_attr_v1 20 200 31510 34.293 33.892 35.179 41.313 29.160`

## Notes

- The benchmark uses packet-local SQL through `target/debug/ecaz dev sql`
  because the Rust `tokio-postgres` query-metrics path hit the v1 DML
  frontdoor fail-closed guard on this remote-placement KNN shape, while psql
  dynamic SQL exercised the CustomScan read path successfully.
- The measurement validates transport selection and endpoint typed readiness
  for the tuple payload path. On this small local scalar loopback fixture,
  typed transport was slower than JSON fallback; this packet does not claim a
  typed speedup.
