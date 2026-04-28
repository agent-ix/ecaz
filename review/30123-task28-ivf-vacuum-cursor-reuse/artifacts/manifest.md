# Artifacts Manifest

## ivf_sustained_churn_cursor_smoke.log

- head SHA: `377baf7d`
- packet/topic: `30123-task28-ivf-vacuum-cursor-reuse`
- lane: Task 28 IVF vacuum replacement/reuse smoke
- fixture: 50k 4D synthetic churn, isolated nlists=32 and nlists=64 tables/indexes, three delete/vacuum/refill cycles
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.sql --raw --log-output review/30123-task28-ivf-vacuum-cursor-reuse/artifacts/ivf_sustained_churn_cursor_smoke.log`
- timestamp: 2026-04-28 14:33-14:40 America/Los_Angeles
- isolation: isolated one-index-per-table surfaces
- key result lines:
  - `cycle0_build | task28_ivf_churn_n32_idx | 4464640 | 4360 kB`
  - `cycle0_build | task28_ivf_churn_n64_idx | 4472832 | 4368 kB`
  - `INSERT 0 25000` / `Time: 44891.922 ms (00:44.892)` for nlists=32 cycle1 refill
  - `INSERT 0 25000` / `Time: 36117.590 ms (00:36.118)` for nlists=64 cycle1 refill
  - `cycle1_refill | task28_ivf_churn_n64_idx | 4579328 | 4472 kB`
  - `INSERT 0 25000` / `Time: 45734.061 ms (00:45.734)` for nlists=32 cycle2 refill
  - `INSERT 0 25000` / `Time: 113753.512 ms (01:53.754)` for nlists=64 cycle2 refill
  - `cycle2_refill | task28_ivf_churn_n64_idx | 4751360 | 4640 kB`
  - `INSERT 0 25000` / `Time: 44860.814 ms (00:44.861)` for nlists=32 cycle3 refill
  - `INSERT 0 25000` / `Time: 134251.989 ms (02:14.252)` for nlists=64 cycle3 refill
  - `cycle3_refill | task28_ivf_churn_n32_idx | 4464640 | 4360 kB`
  - `cycle3_refill | task28_ivf_churn_n64_idx | 4964352 | 4848 kB`
