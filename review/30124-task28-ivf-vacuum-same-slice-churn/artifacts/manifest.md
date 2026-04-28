# Artifacts Manifest

## ivf_same_slice_churn_smoke.sql

- head SHA: `e9ca634c`
- packet/topic: `30124-task28-ivf-vacuum-same-slice-churn`
- lane: Task 28 IVF vacuum replacement/reuse diagnostic
- fixture: 50k 4D synthetic churn, isolated nlists=32 and nlists=64 tables/indexes, repeated same first-half delete/refill slice
- storage format: `turboquant`
- rerank mode: `heap_f32`
- isolation: isolated one-index-per-table surfaces

## ivf_same_slice_churn_smoke.log

- head SHA: `e9ca634c`
- packet/topic: `30124-task28-ivf-vacuum-same-slice-churn`
- lane: Task 28 IVF vacuum replacement/reuse diagnostic
- fixture: 50k 4D synthetic churn, isolated nlists=32 and nlists=64 tables/indexes, repeated same first-half delete/refill slice
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30124-task28-ivf-vacuum-same-slice-churn/artifacts/ivf_same_slice_churn_smoke.sql --raw --log-output review/30124-task28-ivf-vacuum-same-slice-churn/artifacts/ivf_same_slice_churn_smoke.log`
- timestamp: 2026-04-28 14:48-14:54 America/Los_Angeles
- isolation: isolated one-index-per-table surfaces
- key result lines:
  - `cycle0_build | task28_ivf_same_slice_n32_idx | 4464640 | 4360 kB`
  - `cycle0_build | task28_ivf_same_slice_n64_idx | 4472832 | 4368 kB`
  - `INSERT 0 25000` / `Time: 47090.656 ms (00:47.091)` for nlists=32 cycle1 refill
  - `INSERT 0 25000` / `Time: 37788.675 ms (00:37.789)` for nlists=64 cycle1 refill
  - `cycle1_refill | task28_ivf_same_slice_n64_idx | 4579328 | 4472 kB`
  - `INSERT 0 25000` / `Time: 47997.044 ms (00:47.997)` for nlists=32 cycle2 refill
  - `INSERT 0 25000` / `Time: 115413.995 ms (01:55.414)` for nlists=64 cycle2 refill
  - `cycle2_refill | task28_ivf_same_slice_n64_idx | 4743168 | 4632 kB`
  - `INSERT 0 25000` / `Time: 45684.987 ms (00:45.685)` for nlists=32 cycle3 refill
  - `INSERT 0 25000` / `Time: 124588.292 ms (02:04.588)` for nlists=64 cycle3 refill
  - `cycle3_refill | task28_ivf_same_slice_n32_idx | 4464640 | 4360 kB`
  - `cycle3_refill | task28_ivf_same_slice_n64_idx | 4825088 | 4712 kB`
