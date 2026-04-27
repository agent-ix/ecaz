# Artifact Manifest

Packet: `30040-task28-ivf-nprobe-rerank-width-grid`

Head SHA: `697f49e`

## `pg18-ivf-anchor10k1536-nprobe-rerank-width-grid.sql`

- head SHA: `697f49e`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 copied table from packet 30038
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: SQL input for the rerun below
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table, one rebuilt IVF index per grid point
- key lines: see corresponding rerun log

## `pg18-ivf-anchor10k1536-nprobe-rerank-width-grid.log`

- head SHA: `697f49e`
- lane: Task 28 IVF initial tuning
- fixture: same as rerun
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30040-task28-ivf-nprobe-rerank-width-grid/artifacts/pg18-ivf-anchor10k1536-nprobe-rerank-width-grid.sql --log-output review/30040-task28-ivf-nprobe-rerank-width-grid/artifacts/pg18-ivf-anchor10k1536-nprobe-rerank-width-grid.log`
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table
- key lines:
  - failed with `ERROR: record "q" is not assigned yet`
  - not used for measurement claims

## `pg18-ivf-anchor10k1536-nprobe-rerank-width-grid-rerun.log`

- head SHA: `697f49e`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 copied table, first 20 ordered queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30040-task28-ivf-nprobe-rerank-width-grid/artifacts/pg18-ivf-anchor10k1536-nprobe-rerank-width-grid.sql --log-output review/30040-task28-ivf-nprobe-rerank-width-grid/artifacts/pg18-ivf-anchor10k1536-nprobe-rerank-width-grid-rerun.log`
- timestamp: 2026-04-26 America/Los_Angeles
- surface: isolated copied table, one rebuilt IVF index per grid point
- key lines:
  - corpus check: `10000	1536`
  - exact table check: `200	20`
  - full DO block: `Time: 461014.228 ms (07:41.014)`
  - recall rows:
    - `4	25	24845.587	928.551	200	137	0.6850`
    - `4	50	24684.712	880.816	200	137	0.6850`
    - `4	100	24432.790	947.334	200	137	0.6850`
    - `4	200	24602.765	1040.134	200	137	0.6850`
    - `8	25	24430.881	1304.588	200	167	0.8350`
    - `8	50	25171.336	1278.358	200	167	0.8350`
    - `8	100	24273.582	1338.851	200	167	0.8350`
    - `8	200	24771.256	1785.309	200	167	0.8350`
    - `16	25	24797.927	2137.126	200	188	0.9400`
    - `16	50	24738.397	1976.563	200	188	0.9400`
    - `16	100	24789.034	2080.087	200	188	0.9400`
    - `16	200	24782.827	2170.724	200	188	0.9400`
    - `32	25	25082.440	3609.018	200	200	1.0000`
    - `32	50	24828.433	3442.137	200	200	1.0000`
    - `32	100	24409.817	3633.249	200	200	1.0000`
    - `32	200	24571.573	4216.870	200	200	1.0000`
  - latency rows:
    - `4	50	20	10	10	33.666	47.603	49.002	34.708`
    - `8	50	20	10	10	51.854	69.077	72.197	54.223`
    - `16	100	20	10	10	97.893	108.554	110.344	97.971`
    - `32	100	20	10	10	173.411	178.811	179.668	174.285`
