# Artifact Manifest

Packet: `review/30104-task28-ivf-vacuum-churn-smoke`

## ivf_vacuum_churn_smoke.sql

- Head SHA: `c45e22c5`
- Lane: Task 28 IVF vacuum churn diagnostic
- Fixture: synthetic 4D `ecvector`, 5,000 initial rows per table
- Surfaces: `nlists in {8, 32, 64}`, one index per table
- Storage format: default `auto`
- Rerank mode: `heap_f32`
- Command input SQL: `artifacts/ivf_vacuum_churn_smoke.sql`

## ivf_vacuum_churn_smoke.log

- Head SHA: `c45e22c5`
- Timestamp: `2026-04-28T10:23:03-07:00`
- Command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30104-task28-ivf-vacuum-churn-smoke/artifacts/ivf_vacuum_churn_smoke.sql --raw --log-output review/30104-task28-ivf-vacuum-churn-smoke/artifacts/ivf_vacuum_churn_smoke.log`
- Surface isolation: isolated one-index-per-table
- Cache state: not controlled; local smoke
- Key result lines:
  - after build: all three indexes `448 kB`
  - VACUUM wall times: `16.109 ms`, `20.567 ms`, `45.931 ms`
  - after delete vacuum: all three indexes `448 kB`
  - after refill: nlists=8 `648 kB`, nlists=32 `648 kB`, nlists=64 `632 kB`
  - final live rows: all three surfaces `5000`
