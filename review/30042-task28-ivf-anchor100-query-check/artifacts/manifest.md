# Artifact Manifest

Packet: `30042-task28-ivf-anchor100-query-check`

Head SHA: `a9d70f3`

## `pg18-ivf-anchor10k1536-anchor100-query-check.sql`

- head SHA: `a9d70f3`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 corpus, first 100 ordered anchor queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: SQL input for the rerun below
- timestamp: 2026-04-27 America/Los_Angeles
- surface: isolated copied corpus table, one rebuilt IVF index per candidate point
- key lines: see corresponding rerun log

## `pg18-ivf-anchor10k1536-anchor100-query-check.log`

- head SHA: `a9d70f3`
- lane: Task 28 IVF initial tuning
- fixture: accidental 20-query local table reuse
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30042-task28-ivf-anchor100-query-check/artifacts/pg18-ivf-anchor10k1536-anchor100-query-check.sql --log-output review/30042-task28-ivf-anchor100-query-check/artifacts/pg18-ivf-anchor10k1536-anchor100-query-check.log`
- timestamp: 2026-04-27 America/Los_Angeles
- surface: isolated copied corpus table
- key lines:
  - `SELECT 200`
  - `200	20`
  - not used for measurement claims because the fixture only contained 20 queries and divided recall by 1000

## `pg18-ivf-anchor10k1536-anchor100-query-check-rerun.log`

- head SHA: `a9d70f3`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 corpus, first 100 ordered anchor queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30042-task28-ivf-anchor100-query-check/artifacts/pg18-ivf-anchor10k1536-anchor100-query-check.sql --log-output review/30042-task28-ivf-anchor100-query-check/artifacts/pg18-ivf-anchor10k1536-anchor100-query-check-rerun.log`
- timestamp: 2026-04-27 America/Los_Angeles
- surface: isolated copied corpus table, one rebuilt IVF index per candidate point
- key lines:
  - corpus check: `10000	1536`
  - query check: `100	1536`
  - exact materialization: `SELECT 1000` / `Time: 441785.070 ms (07:21.785)`
  - exact count: `1000	100`
  - candidate summary:
    - `32	16	50	24418.410	10082.994	1000	980	0.9800`
    - `32	32	25	24328.649	18876.778	1000	1000	1.0000`
    - `64	16	25	42214.022	10879.598	1000	980	0.9800`
  - latency summary:
    - `32	16	50	100	10	10	100.013	120.946	123.989	101.617`
    - `32	32	25	100	10	10	177.806	202.947	215.933	171.247`
    - `64	16	25	100	10	10	108.563	131.297	135.519	110.231`
