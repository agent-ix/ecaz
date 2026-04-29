# Artifact Manifest: 30146 Task 28 A9 100k HNSW Reference Attempt

## `build_hnsw_100k.sql`

- head SHA: `95d48633`
- packet/topic: `30146-task28-ivf-a9-100k-hnsw-reference`
- lane / fixture / storage format / rerank mode: A9 100k HNSW reference build attempt, `m=16`, `ef_construction=128`
- command: packet-local SQL input for `build_hnsw_100k.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated `task28_a9_100k_hnsw_*` surface
- key result lines: source SQL only

## `build_hnsw_100k.log`

- head SHA: `95d48633`
- packet/topic: `30146-task28-ivf-a9-100k-hnsw-reference`
- lane / fixture / storage format / rerank mode: A9 100k HNSW reference build attempt, `m=16`, `ef_construction=128`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/build_hnsw_100k.sql --raw --log-output review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/build_hnsw_100k.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated `task28_a9_100k_hnsw_*` surface
- key result lines:
  - build did not complete
  - `server closed the connection unexpectedly`
  - `connection to server was lost`

## `hnsw_build_activity_before_cancel.log`

- head SHA: `95d48633`
- packet/topic: `30146-task28-ivf-a9-100k-hnsw-reference`
- lane / fixture / storage format / rerank mode: A9 100k HNSW active-build check
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "SELECT pid, state, now() - query_start AS age, left(query, 160) AS query FROM pg_stat_activity WHERE query ILIKE '%task28_a9_100k_hnsw_idx%' OR query ILIKE '%ec_hnsw%' ORDER BY query_start;" --raw --log-output review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/hnsw_build_activity_before_cancel.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated `task28_a9_100k_hnsw_*` surface
- key result lines: backend `3058423`, `active`, age `00:07:17.69136`

## Cancellation Logs

- head SHA: `95d48633`
- packet/topic: `30146-task28-ivf-a9-100k-hnsw-reference`
- lane / fixture / storage format / rerank mode: A9 100k HNSW build cancellation
- commands:
  - `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "SELECT pg_cancel_backend(3058423) AS canceled;" --raw --log-output review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/hnsw_build_cancel.log`
  - `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "SELECT pg_terminate_backend(3058423) AS terminated;" --raw --log-output review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/hnsw_build_terminate.log`
  - `kill 3058423`
  - `kill -9 3058423`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated `task28_a9_100k_hnsw_*` surface
- key result lines:
  - `pg_cancel_backend` returned `t`
  - `pg_terminate_backend` returned `t`
  - backend stayed active after PostgreSQL cancel/terminate checks

## `pg18_health_after_hnsw_kill.log`

- head SHA: `95d48633`
- packet/topic: `30146-task28-ivf-a9-100k-hnsw-reference`
- lane / fixture / storage format / rerank mode: PG18 health check after killed HNSW build backend
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "SELECT 1 AS pg18_alive, to_regclass('task28_a9_100k_hnsw_idx') AS hnsw_idx;" --raw --log-output review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/pg18_health_after_hnsw_kill.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated `task28_a9_100k_hnsw_*` surface
- key result lines: `pg18_alive=1`, `hnsw_idx` null
