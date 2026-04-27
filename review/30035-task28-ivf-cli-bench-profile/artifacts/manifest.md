# Artifact Manifest

Packet: `review/30035-task28-ivf-cli-bench-profile`

Head SHA before this packet's code changes: `566e78630fca5af6b570544979adf8525f587dbd`

Local machine:

- OS: WSL2 Linux `6.6.87.2-microsoft-standard-WSL2`
- CPU: Intel Core i9-10900K, 20 logical CPUs
- Memory: 62 GiB total, 48 GiB available at capture
- PostgreSQL: 18.3, x86_64, gcc 11.4.0
- Storage/cache state: normal local scratch cluster; cache not explicitly
  dropped, so results should be treated as warm/local smoke data only.

## Artifacts

### `pg18-version-smoke.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql 'select version();' --raw --log-output review/30035-task28-ivf-cli-bench-profile/artifacts/pg18-version-smoke.log`
- timestamp: 2026-04-26 local
- purpose: verifies PG18 server version.
- key result: PostgreSQL 18.3.

### `pg18-corpus-inspect.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql 'select relname, relkind, reltuples::bigint ...' --raw --log-output review/30035-task28-ivf-cli-bench-profile/artifacts/pg18-corpus-inspect.log`
- timestamp: 2026-04-26 local
- purpose: identifies reusable local corpus tables.
- key result lines: 10k source table has 10,000 rows; 990k anchor table has
  990,000 rows; 50k table has 50,000 rows.

### `pg18-corpus-schema.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql 'select table_name, column_name, data_type, udt_name ...' --raw --log-output review/30035-task28-ivf-cli-bench-profile/artifacts/pg18-corpus-schema.log`
- timestamp: 2026-04-26 local
- purpose: records why the smoke copy derives `embedding` from `source` for
  the 10k table.
- key result: `ec_hnsw_parallel_concurrent_dsm_recall_corpus` has `id` and
  `source`, but no `embedding`.

### `pg18-extension-am-check.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql 'select extname, extversion ...; select amname ...' --raw --log-output review/30035-task28-ivf-cli-bench-profile/artifacts/pg18-extension-am-check.log`
- timestamp: 2026-04-26 local
- purpose: records scratch-catalog state before manual IVF catalog bootstrap.
- key result: extension version was `0.1.1`; only `ec_hnsw` existed in
  `pg_am`.

### `pg18-install-ivf-catalog.sql`

- command artifact: packet-local SQL to install the missing `ec_ivf` handler,
  access method, and opclasses into the existing scratch database catalog.
- note: needed only because the scratch database already had extension version
  `0.1.1`, so installing the rebuilt library did not replay bootstrap SQL.

### `pg18-install-ivf-catalog.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30035-task28-ivf-cli-bench-profile/artifacts/pg18-install-ivf-catalog.sql --raw --log-output review/30035-task28-ivf-cli-bench-profile/artifacts/pg18-install-ivf-catalog.log`
- timestamp: 2026-04-26 local
- purpose: raw catalog bootstrap log.
- key result: `pg_am` listed both `ec_hnsw` and `ec_ivf`.

### `pg18-ivf-10k-smoke.sql`

- command artifact: packet-local smoke SQL.
- lane / fixture / storage / rerank: IVF, 10k copied real corpus, `turboquant`,
  `rerank = off`.
- isolated surface: uses one copied smoke table with one IVF index; no shared
  HNSW/IVF table-index selection.

### `pg18-ivf-10k-smoke.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30035-task28-ivf-cli-bench-profile/artifacts/pg18-ivf-10k-smoke.sql --raw --log-output review/30035-task28-ivf-cli-bench-profile/artifacts/pg18-ivf-10k-smoke.log`
- timestamp: 2026-04-26 local
- lane / fixture / storage / rerank: IVF, copied 10k x 64 fixture, `turboquant`,
  `rerank = off`.
- key result lines cited by `request.md`:
  - build time: `Time: 5822.967 ms (00:05.823)`
  - index size: `1236992`
  - full-probe EXPLAIN execution time: `38.212 ms`
  - candidates scored: `10000`
  - recall rows: nprobe `1/4/16/64` all returned `200`, exact hits `142`,
    recall@10 `0.7100`.
