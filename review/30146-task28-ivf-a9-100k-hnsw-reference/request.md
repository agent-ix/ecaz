# Review Request: Task 28 A9 100k HNSW Reference Attempt

## Scope

This packet records a bounded attempt to fill the missing A9 100k HNSW
reference row.

The local PG18 database has the 100k HNSW corpus and query tables:

- `task28_a9_100k_hnsw_corpus`
- `task28_a9_100k_hnsw_queries`

The attempted index shape was:

- `USING ec_hnsw (embedding ecvector_ip_ops)`
- `m = 16`
- `ef_construction = 128`

## Result

The build was stopped after it exceeded the bounded comparison budget for this
slice. At the first activity check it had been active for `00:07:17.69136`.
PostgreSQL `pg_cancel_backend` and `pg_terminate_backend` both returned `true`,
but the backend stayed active, so the OS process was killed.

The PG18 server was healthy afterward, and no partial HNSW index remained:

| check | result |
|---|---|
| `SELECT 1 AS pg18_alive` | `1` |
| `to_regclass('task28_a9_100k_hnsw_idx')` | null |

## Interpretation

No HNSW performance claim is made from this attempt.

This confirms that even the 100k HNSW reference build can consume enough local
time to interrupt IVF momentum. The next A9 work should either:

- reuse an already-built HNSW reference surface, or
- run HNSW in a deliberately separate comparison slice, or
- choose a smaller HNSW reference shape with an explicit pre-set time budget.

The IVF-side 100k and 990k measurements remain the usable local A9 evidence in
the current branch.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/build_hnsw_100k.sql --raw --log-output review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/build_hnsw_100k.log`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "SELECT pid, state, now() - query_start AS age, left(query, 160) AS query FROM pg_stat_activity WHERE query ILIKE '%task28_a9_100k_hnsw_idx%' OR query ILIKE '%ec_hnsw%' ORDER BY query_start;" --raw --log-output review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/hnsw_build_activity_before_cancel.log`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "SELECT pg_cancel_backend(3058423) AS canceled;" --raw --log-output review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/hnsw_build_cancel.log`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "SELECT pg_terminate_backend(3058423) AS terminated;" --raw --log-output review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/hnsw_build_terminate.log`
- `kill 3058423`
- `kill -9 3058423`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "SELECT 1 AS pg18_alive, to_regclass('task28_a9_100k_hnsw_idx') AS hnsw_idx;" --raw --log-output review/30146-task28-ivf-a9-100k-hnsw-reference/artifacts/pg18_health_after_hnsw_kill.log`

## Artifacts

- `artifacts/build_hnsw_100k.sql`
- `artifacts/build_hnsw_100k.log`
- `artifacts/hnsw_build_activity_before_cancel.log`
- `artifacts/hnsw_build_cancel.log`
- `artifacts/hnsw_build_activity_after_cancel.log`
- `artifacts/hnsw_build_terminate.log`
- `artifacts/hnsw_build_activity_after_terminate.log`
- `artifacts/pg18_health_after_hnsw_kill.log`
- `artifacts/manifest.md`
