# Artifact Manifest

Packet: `30039-task28-ivf-rerank-width-sweep`

Head SHA: `4d894bd`

## `pg18-ivf-anchor10k1536-rerank-width-sweep.sql`

- head SHA: `4d894bd`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 copied table from packet 30038
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: SQL input for the sweep below
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table, one rebuilt IVF index at a time
- key lines: see corresponding log

## `pg18-ivf-anchor10k1536-rerank-width-sweep.log`

- head SHA: `4d894bd`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 copied table, first 20 ordered queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30039-task28-ivf-rerank-width-sweep/artifacts/pg18-ivf-anchor10k1536-rerank-width-sweep.sql --log-output review/30039-task28-ivf-rerank-width-sweep/artifacts/pg18-ivf-anchor10k1536-rerank-width-sweep.log`
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table, one rebuilt IVF index at a time
- key lines:
  - exact table check: `200	20`
  - width 50 build: `CREATE INDEX` / `Time: 25007.474 ms (00:25.007)`
  - width 50 materialization: `SELECT 200` / `Time: 3704.448 ms (00:03.704)`
  - width 50 latency: `50	20	10	10	180.608	183.371	183.742	180.817`
  - width 200 build: `CREATE INDEX` / `Time: 24712.487 ms (00:24.712)`
  - width 200 materialization: `SELECT 200` / `Time: 3860.064 ms (00:03.860)`
  - width 200 latency: `200	20	10	10	189.890	194.932	195.676	190.830`
  - width 1000 build: `CREATE INDEX` / `Time: 24618.373 ms (00:24.618)`
  - width 1000 materialization: `SELECT 200` / `Time: 4753.514 ms (00:04.754)`
  - width 1000 latency: `1000	20	10	10	232.622	235.727	236.945	233.192`
  - width 0/full build: `CREATE INDEX` / `Time: 24677.250 ms (00:24.677)`
  - width 0/full materialization: `SELECT 200` / `Time: 14106.544 ms (00:14.107)`
  - width 0/full latency: `0	20	10	10	702.929	715.203	715.447	701.786`
  - recall summary:
    - `50	200	200	1.0000	0`
    - `200	200	200	1.0000	0`
    - `1000	200	200	1.0000	0`
    - `0	200	200	1.0000	0`
