# Artifact Manifest

Packet: `30043-task28-ivf-anchor100-midprobe`

Head SHA: `47b302a`

## `pg18-ivf-anchor10k1536-anchor100-midprobe.sql`

- head SHA: `47b302a`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 corpus, 100 ordered anchor queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: SQL input for the run below
- timestamp: 2026-04-27 America/Los_Angeles
- surface: isolated copied corpus table, one rebuilt IVF index per candidate point
- key lines: see corresponding log

## `pg18-ivf-anchor10k1536-anchor100-midprobe.log`

- head SHA: `47b302a`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 10k x 1536 corpus, 100 ordered anchor queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30043-task28-ivf-anchor100-midprobe/artifacts/pg18-ivf-anchor10k1536-anchor100-midprobe.sql --log-output review/30043-task28-ivf-anchor100-midprobe/artifacts/pg18-ivf-anchor10k1536-anchor100-midprobe.log`
- timestamp: 2026-04-27 America/Los_Angeles
- surface: isolated copied corpus table, one rebuilt IVF index per candidate point
- key lines:
  - exact table check: `1000	100`
  - full DO block: `Time: 254831.130 ms (04:14.831)`
  - candidate summary:
    - `32	24	25	24340.187	14616.073	1000	998	0.9980`
    - `32	24	50	24467.272	14829.800	1000	998	0.9980`
    - `64	24	25	42328.069	14672.893	1000	998	0.9980`
    - `64	24	50	41558.253	15704.177	1000	998	0.9980`
  - latency summary:
    - `32	24	25	100	10	10	135.073	146.331	154.755	127.040`
    - `32	24	50	100	10	10	137.423	149.456	151.791	128.767`
    - `64	24	25	100	10	10	143.789	153.870	160.552	144.614`
    - `64	24	50	100	10	10	146.575	169.468	184.034	139.907`
