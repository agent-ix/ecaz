# Artifact Manifest

Packet: `30038-task28-ivf-heap-rerank-smoke`

Head SHA: `9b42a71`

## `pg18-anchor-exists.log`

- head SHA: `9b42a71`
- lane: Task 28 IVF initial tuning
- fixture: existing DBPedia-derived 10k x 1536 anchor table check
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --sql "SELECT to_regclass('task28_ivf_anchor10k1536_corpus') IS NOT NULL AS has_anchor;" --log-output review/30038-task28-ivf-heap-rerank-smoke/artifacts/pg18-anchor-exists.log`
- timestamp: 2026-04-26 America/Los_Angeles
- surface: shared scratch database; packet uses a fresh copied table for the measured index
- key lines: `t`

## `pg18-ivf-anchor10k1536-heap-rerank-smoke.sql`

- head SHA: `9b42a71`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 copy, `nlists = 32`, `nprobe = 32`
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: SQL input for the rerun below
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table with one IVF index
- key lines: see corresponding log

## `pg18-ivf-anchor10k1536-heap-rerank-smoke.log`

- head SHA: `9b42a71`
- lane: Task 28 IVF initial tuning
- fixture: same as rerun, but script selected zero queries because anchor query ids are not `0..19`
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30038-task28-ivf-heap-rerank-smoke/artifacts/pg18-ivf-anchor10k1536-heap-rerank-smoke.sql --log-output review/30038-task28-ivf-heap-rerank-smoke/artifacts/pg18-ivf-anchor10k1536-heap-rerank-smoke.log`
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table with one IVF index
- key lines: `SELECT 0`; not used for recall claims

## `pg18-ivf-anchor10k1536-heap-rerank-smoke-rerun.log`

- head SHA: `9b42a71`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 copy, first 20 ordered queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30038-task28-ivf-heap-rerank-smoke/artifacts/pg18-ivf-anchor10k1536-heap-rerank-smoke.sql --log-output review/30038-task28-ivf-heap-rerank-smoke/artifacts/pg18-ivf-anchor10k1536-heap-rerank-smoke-rerun.log`
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table with one IVF index
- key lines:
  - `SELECT 10000`
  - `1	1536`
  - `CREATE INDEX` / `Time: 25130.250 ms (00:25.130)`
  - `9379840	9160 kB	1048576	1024 kB`
  - `Execution Time: 706.722 ms`
  - `SELECT 200` / `Time: 88600.644 ms (01:28.601)` for exact top-10 materialization
  - `SELECT 200` / `Time: 14453.155 ms (00:14.453)` for IVF top-10 materialization
  - `200	200	1.0000`
  - `0	10	10`

## `pg18-ivf-anchor10k1536-heap-rerank-latency.sql`

- head SHA: `9b42a71`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 heap-rerank index from smoke rerun
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: SQL input for the latency loop below
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table with one IVF index
- key lines: see corresponding log

## `pg18-ivf-anchor10k1536-heap-rerank-latency.log`

- head SHA: `9b42a71`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 heap-rerank index, first 20 ordered queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30038-task28-ivf-heap-rerank-smoke/artifacts/pg18-ivf-anchor10k1536-heap-rerank-latency.sql --log-output review/30038-task28-ivf-heap-rerank-smoke/artifacts/pg18-ivf-anchor10k1536-heap-rerank-latency.log`
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table with one IVF index
- key lines:
  - `DO` / `Time: 13914.965 ms (00:13.915)`
  - `20	10	10	686.339	714.122	748.029	685.006`
  - slowest rows: `990000	748.029	10`, `990004	714.122	10`, `990007	711.357	10`
