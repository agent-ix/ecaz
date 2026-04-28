# Artifact Manifest

## ivf_sustained_churn_smoke.sql

- head SHA: `10b141b4`
- packet/topic: `30120-task28-ivf-vacuum-sustained-churn`
- lane: Task 28 IVF A3/F2 sustained churn smoke
- fixture: synthetic 50k 4D, isolated one-index-per-table surfaces
- storage format: `turboquant`
- nlists: `32,64`
- nprobe: `8`
- rerank mode: `heap_f32`
- timestamp: `2026-04-28T13:32:12-07:00`
- surface: isolated one-index-per-table surfaces
- command:
  `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.sql --raw --log-output review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.log`
- key results:
  - `n32 cycle0_build 4464640 bytes`
  - `n32 cycle1_refill 4464640 bytes`
  - `n32 cycle2_refill 4464640 bytes`
  - `n32 cycle3_refill 4464640 bytes`
  - `n64 cycle0_build 4472832 bytes`
  - `n64 cycle1_refill 4579328 bytes`
  - `n64 cycle2_refill 4751360 bytes`
  - `n64 cycle3_refill 4980736 bytes`
  - `n64 refill insert times rose from 42473.263 ms to 156093.247 ms`
