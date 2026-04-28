# Artifacts Manifest

## ivf_sustained_churn_hintset_smoke.log

- head SHA: `4323499a` plus uncommitted free-block-set experiment
- packet/topic: `30122-task28-ivf-vacuum-free-block-set`
- lane: Task 28 IVF vacuum replacement/reuse smoke
- fixture: 50k 4D synthetic churn, isolated nlists=32 and nlists=64 tables/indexes, three delete/vacuum/refill cycles
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.sql --raw --log-output review/30122-task28-ivf-vacuum-free-block-set/artifacts/ivf_sustained_churn_hintset_smoke.log`
- timestamp: 2026-04-28 14:22-14:29 America/Los_Angeles
- isolation: isolated one-index-per-table surfaces
- key result lines:
  - `cycle0_build | task28_ivf_churn_n32_idx | 4464640 | 4360 kB`
  - `cycle0_build | task28_ivf_churn_n64_idx | 4472832 | 4368 kB`
  - `INSERT 0 25000` / `Time: 45490.278 ms (00:45.490)` for nlists=32 cycle1 refill
  - `INSERT 0 25000` / `Time: 36130.262 ms (00:36.130)` for nlists=64 cycle1 refill
  - `cycle1_refill | task28_ivf_churn_n64_idx | 4579328 | 4472 kB`
  - `INSERT 0 25000` / `Time: 45835.687 ms (00:45.836)` for nlists=32 cycle2 refill
  - `INSERT 0 25000` / `Time: 114442.767 ms (01:54.443)` for nlists=64 cycle2 refill
  - `cycle2_refill | task28_ivf_churn_n64_idx | 4767744 | 4656 kB`
  - `INSERT 0 25000` / `Time: 45859.850 ms (00:45.860)` for nlists=32 cycle3 refill
  - `INSERT 0 25000` / `Time: 162264.328 ms (02:42.264)` for nlists=64 cycle3 refill
  - `cycle3_refill | task28_ivf_churn_n32_idx | 4464640 | 4360 kB`
  - `cycle3_refill | task28_ivf_churn_n64_idx | 5062656 | 4944 kB`
