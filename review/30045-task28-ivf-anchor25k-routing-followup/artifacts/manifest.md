# Artifact Manifest

Packet: `30045-task28-ivf-anchor25k-routing-followup`

Head SHA: `b84c278`

## `pg18-ivf-anchor25k-routing-followup.sql`

- head SHA: `b84c278`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 25k x 1536 corpus, 100 ordered anchor queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: SQL input for the run below
- timestamp: 2026-04-27 America/Los_Angeles
- surface: isolated 25k copied corpus table, one rebuilt IVF index per candidate point
- key lines: see corresponding log

## `pg18-ivf-anchor25k-routing-followup.log`

- head SHA: `b84c278`
- lane: Task 28 IVF initial tuning
- fixture: DBPedia-derived 25k x 1536 corpus, 100 ordered anchor queries
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- dev sql --socket-dir /home/peter/.pgrx --port 28818 --database postgres --file review/30045-task28-ivf-anchor25k-routing-followup/artifacts/pg18-ivf-anchor25k-routing-followup.sql --log-output review/30045-task28-ivf-anchor25k-routing-followup/artifacts/pg18-ivf-anchor25k-routing-followup.log`
- timestamp: 2026-04-27 America/Los_Angeles
- surface: isolated 25k copied corpus table, one rebuilt IVF index per candidate point
- key lines:
  - exact table check: `1000	100`
  - full DO block: `Time: 434774.141 ms (07:14.774)`
  - candidate summary:
    - `32	28	25	46086.404	38764.622	1000	983	0.9830	23027712	22 MB`
    - `64	32	25	74052.730	23880.588	1000	984	0.9840	23289856	22 MB`
    - `64	48	25	74443.659	43213.066	1000	1000	1.0000	23289856	22 MB`
  - latency summary:
    - `32	28	25	100	10	10	382.821	414.706	438.567	368.566`
    - `64	32	25	100	10	10	433.881	453.544	509.243	418.803`
    - `64	48	25	100	10	10	433.318	452.825	458.398	425.895`
