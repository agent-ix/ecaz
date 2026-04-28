# Artifacts Manifest

## ivf_same_slice_churn_adjacent_smoke.log

- head SHA: `4ed20913`
- packet/topic: `30125-task28-ivf-adjacent-page-reuse`
- lane: Task 28 IVF vacuum replacement/reuse smoke
- fixture: 50k 4D synthetic churn, isolated nlists=32 and nlists=64 tables/indexes, repeated same first-half delete/refill slice
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30124-task28-ivf-vacuum-same-slice-churn/artifacts/ivf_same_slice_churn_smoke.sql --raw --log-output review/30125-task28-ivf-adjacent-page-reuse/artifacts/ivf_same_slice_churn_adjacent_smoke.log`
- timestamp: 2026-04-28 15:04-15:08 America/Los_Angeles
- isolation: isolated one-index-per-table surfaces
- key result lines:
  - `cycle0_build | task28_ivf_same_slice_n32_idx | 4464640 | 4360 kB`
  - `cycle0_build | task28_ivf_same_slice_n64_idx | 4472832 | 4368 kB`
  - `INSERT 0 25000` / `Time: 47932.344 ms (00:47.932)` for nlists=32 cycle1 refill
  - `INSERT 0 25000` / `Time: 31710.069 ms (00:31.710)` for nlists=64 cycle1 refill
  - `cycle1_refill | task28_ivf_same_slice_n64_idx | 4472832 | 4368 kB`
  - `INSERT 0 25000` / `Time: 46746.694 ms (00:46.747)` for nlists=32 cycle2 refill
  - `INSERT 0 25000` / `Time: 33247.982 ms (00:33.248)` for nlists=64 cycle2 refill
  - `cycle2_refill | task28_ivf_same_slice_n64_idx | 4489216 | 4384 kB`
  - `INSERT 0 25000` / `Time: 47312.721 ms (00:47.313)` for nlists=32 cycle3 refill
  - `INSERT 0 25000` / `Time: 43756.698 ms (00:43.757)` for nlists=64 cycle3 refill
  - `cycle3_refill | task28_ivf_same_slice_n32_idx | 4464640 | 4360 kB`
  - `cycle3_refill | task28_ivf_same_slice_n64_idx | 4538368 | 4432 kB`

## ivf_sustained_churn_adjacent_smoke.log

- head SHA: `4ed20913`
- packet/topic: `30125-task28-ivf-adjacent-page-reuse`
- lane: Task 28 IVF vacuum replacement/reuse smoke
- fixture: 50k 4D synthetic churn, isolated nlists=32 and nlists=64 tables/indexes, original drifting delete/refill shape
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.sql --raw --log-output review/30125-task28-ivf-adjacent-page-reuse/artifacts/ivf_sustained_churn_adjacent_smoke.log`
- timestamp: 2026-04-28 15:09-15:12 America/Los_Angeles
- isolation: isolated one-index-per-table surfaces
- key result lines:
  - `cycle0_build | task28_ivf_churn_n32_idx | 4464640 | 4360 kB`
  - `cycle0_build | task28_ivf_churn_n64_idx | 4472832 | 4368 kB`
  - `INSERT 0 25000` / `Time: 47827.475 ms (00:47.827)` for nlists=32 cycle1 refill
  - `INSERT 0 25000` / `Time: 31514.214 ms (00:31.514)` for nlists=64 cycle1 refill
  - `cycle1_refill | task28_ivf_churn_n64_idx | 4472832 | 4368 kB`
  - `INSERT 0 25000` / `Time: 47807.871 ms (00:47.808)` for nlists=32 cycle2 refill
  - `INSERT 0 25000` / `Time: 33221.881 ms (00:33.222)` for nlists=64 cycle2 refill
  - `cycle2_refill | task28_ivf_churn_n64_idx | 4481024 | 4376 kB`
  - `INSERT 0 25000` / `Time: 45764.644 ms (00:45.765)` for nlists=32 cycle3 refill
  - `INSERT 0 25000` / `Time: 34070.797 ms (00:34.071)` for nlists=64 cycle3 refill
  - `cycle3_refill | task28_ivf_churn_n32_idx | 4464640 | 4360 kB`
  - `cycle3_refill | task28_ivf_churn_n64_idx | 4497408 | 4392 kB`
