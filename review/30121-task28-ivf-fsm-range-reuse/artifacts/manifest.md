# Artifact Manifest

## ivf_sustained_churn_fsm_smoke.log

- head SHA: uncommitted pre-`fe2337a3` FSM-only trial
- packet/topic: `30121-task28-ivf-fsm-range-reuse`
- lane: Task 28 IVF F2/A3 range-reuse experiment
- fixture: synthetic 50k 4D, isolated one-index-per-table surfaces
- storage format: `turboquant`
- nlists: `32,64`
- nprobe: `8`
- rerank mode: `heap_f32`
- timestamp: `2026-04-28T14:18:28-07:00`
- surface: isolated one-index-per-table surfaces
- command:
  `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.sql --raw --log-output review/30121-task28-ivf-fsm-range-reuse/artifacts/ivf_sustained_churn_fsm_smoke.log`
- key result:
  FSM-only reproduced the same n64 shape as packet 30120: cycle3 size `4980736` bytes and cycle3 n64 refill `156585.394 ms`.

## ivf_sustained_churn_hint_smoke.log

- head SHA: `fe2337a3`
- packet/topic: `30121-task28-ivf-fsm-range-reuse`
- lane: Task 28 IVF F2/A3 range-reuse experiment
- fixture: synthetic 50k 4D, isolated one-index-per-table surfaces
- storage format: `turboquant`
- nlists: `32,64`
- nprobe: `8`
- rerank mode: `heap_f32`
- timestamp: `2026-04-28T14:18:28-07:00`
- surface: isolated one-index-per-table surfaces
- command:
  `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.sql --raw --log-output review/30121-task28-ivf-fsm-range-reuse/artifacts/ivf_sustained_churn_hint_smoke.log`
- key results:
  - `n32 cycle0/cycle1/cycle2/cycle3 = 4464640/4464640/4464640/4464640 bytes`
  - `n64 cycle0/cycle1/cycle2/cycle3 = 4472832/4579328/4751360/4997120 bytes`
  - `n64 refill times cycle1/cycle2/cycle3 = 35781.554/114028.280/135267.117 ms`

## ivf_sustained_churn_hint_smoke.sql

- head SHA: `fe2337a3`
- packet/topic: `30121-task28-ivf-fsm-range-reuse`
- lane / fixture / storage format / rerank mode: same as `ivf_sustained_churn_hint_smoke.log`
- command: input SQL for the logged run
