# Artifact Manifest

Packet: `30041-task28-ivf-nlists-routing-grid`

Head SHA: `cc80443`

## `pg18-ivf-anchor10k1536-nlists-routing-grid.sql`

- head SHA: `cc80443`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 copied table from packet 30038
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: SQL input for the run below
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table, one rebuilt IVF index per grid point
- key lines: see corresponding log

## `pg18-ivf-anchor10k1536-nlists-routing-grid.log`

- head SHA: `cc80443`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 copied table, first 20 ordered queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30041-task28-ivf-nlists-routing-grid/artifacts/pg18-ivf-anchor10k1536-nlists-routing-grid.sql --log-output review/30041-task28-ivf-nlists-routing-grid/artifacts/pg18-ivf-anchor10k1536-nlists-routing-grid.log`
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table, one rebuilt IVF index per grid point
- key lines:
  - corpus check: `10000	1536`
  - exact table check: `200	20`
  - full DO block: `Time: 984541.914 ms (16:24.542)`
  - best full-recall row by p95: `32	32	25	1.0000	174.180	179.704`
  - selected recall/build rows:
    - `32	32	25	1.0000	24519.101	3935.809	200	200	1.0000`
    - `64	32	50	0.5000	42282.024	3829.256	200	200	1.0000`
    - `128	32	25	0.2500	77218.319	3862.082	200	200	1.0000`
  - selected latency rows:
    - `32	16	50	20	10	10	99.576	112.549	114.396	100.319`
    - `32	32	25	20	10	10	174.180	179.704	182.104	175.199`
    - `64	16	25	20	10	10	107.256	117.004	123.139	108.135`
    - `128	32	25	20	10	10	181.354	184.835	188.849	181.976`

## `pg18-active-during-nlists-grid.log`

- head SHA: `cc80443`
- lane: Task 28 IVF initial tuning
- fixture: same in-progress nlists grid
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --sql "SELECT pid, state, wait_event_type, wait_event, now() - query_start AS age, left(query, 120) FROM pg_stat_activity WHERE datname = 'postgres' AND state <> 'idle' ORDER BY query_start;" --log-output review/30041-task28-ivf-nlists-routing-grid/artifacts/pg18-active-during-nlists-grid.log`
- timestamp: 2026-04-26 America/Los_Angeles
- surface: shared scratch database status check while packet query was running
- key lines:
  - active `DO` backend age `00:12:06.938783`
  - no wait event reported
