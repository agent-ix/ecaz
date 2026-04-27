# Artifact Manifest

Packet: `30044-task28-ivf-anchor25k-candidate-check`

Head SHA: `81f5468`

## `pg18-ivf-anchor25k-candidate-check.sql`

- head SHA: `81f5468`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 25k x 1536 corpus, 100 ordered anchor queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: SQL input for the run below
- timestamp: 2026-04-27 America/Los_Angeles
- surface: isolated 25k copied corpus table, one rebuilt IVF index per candidate point
- key lines: see corresponding log

## `pg18-ivf-anchor25k-candidate-check.log`

- head SHA: `81f5468`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 25k x 1536 corpus, 100 ordered anchor queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30044-task28-ivf-anchor25k-candidate-check/artifacts/pg18-ivf-anchor25k-candidate-check.sql --log-output review/30044-task28-ivf-anchor25k-candidate-check/artifacts/pg18-ivf-anchor25k-candidate-check.log`
- timestamp: 2026-04-27 America/Los_Angeles
- surface: isolated 25k copied corpus table, one rebuilt IVF index per candidate point
- key lines:
  - table copy: `SELECT 25000` / `Time: 11456.016 ms (00:11.456)`
  - corpus check: `25000	1536`
  - query check: `100	1536`
  - exact materialization: `SELECT 1000` / `Time: 1126749.490 ms (18:46.749)`
  - exact count: `1000	100`
  - candidate summary:
    - `32	24	25	46137.984	33660.063	1000	976	0.9760	23027712	22 MB`
    - `32	32	25	45067.688	42650.677	1000	1000	1.0000	23027712	22 MB`
  - latency summary:
    - `32	24	25	100	10	10	331.674	371.690	407.329	329.108`
    - `32	32	25	100	10	10	434.858	456.380	521.759	429.340`

## `pg18-active-during-anchor25k.log`

- head SHA: `81f5468`
- lane: Task 28 IVF initial tuning
- fixture: in-progress DBPedia-derived 25k x 1536 corpus exact materialization
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --sql "SELECT pid, state, wait_event_type, wait_event, now() - query_start AS age, left(query, 160) FROM pg_stat_activity WHERE datname = 'postgres' AND state <> 'idle' ORDER BY query_start;" --log-output review/30044-task28-ivf-anchor25k-candidate-check/artifacts/pg18-active-during-anchor25k.log`
- timestamp: 2026-04-27 America/Los_Angeles
- surface: shared scratch database status check while packet query was running
- key lines:
  - active exact `CREATE TABLE` backend age `00:12:54.367519`
  - no wait event reported
