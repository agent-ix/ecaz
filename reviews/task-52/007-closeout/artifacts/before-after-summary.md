# Task 52 Bench Window — Before/After vs Post-Task-50 M5 Baseline

Date: 2026-05-23.
Host: Peters-MBP (Apple M5 Pro, 64 GiB, macOS 26.4.1).
PG: 18 on pgrx socket `/Users/peter/.pgrx`, port 28818.
Baseline: `benchmarks/task-50-m5-hnsw-baseline/artifacts/` (HEAD `18acf379a`).
Task 52 candidate HEAD: see `reviews/task-52/007-closeout` commit parent.

Same 8-step suite shape as the baseline (same prefixes, same `m`,
ef_construction, ef sweep, k, recall trials, latency trials).

## §10k corpus

### Recall@10 (`ec_real_10k_hnsw`, 200 queries × 2000 trials per ef)

| ef | Baseline recall@k | Task 52 recall@k | Δ | Baseline ndcg@k | Task 52 ndcg@k |
| --- | ---: | ---: | ---: | ---: | ---: |
| 40  | 0.9040 | 0.9040 | 0.0000 | 0.9617 | 0.9617 |
| 80  | 0.9530 | 0.9530 | 0.0000 | 0.9821 | 0.9821 |
| 120 | 0.9605 | 0.9605 | 0.0000 | 0.9866 | 0.9866 |
| 200 | 0.9775 | 0.9775 | 0.0000 | 0.9933 | 0.9933 |
| 400 | 0.9950 | 0.9950 | 0.0000 | 0.9996 | 0.9996 |

**Identical to four decimals.**

### Latency (`ec_real_10k_hnsw`, 1000 trials per ef, ms)

| ef | Base p50 | T52 p50 | Δ% | Base p95 | T52 p95 | Δ% | Base p99 | T52 p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 0.57 | 0.57 | 0.0   | 0.74 | 0.75 | +1.4 | 0.88 | 0.89 |
| 80  | 0.89 | 0.91 | +2.2  | 1.15 | 1.21 | +5.2 | 1.37 | 1.42 |
| 120 | 0.82 | 0.81 | -1.2  | 1.08 | 1.12 | +3.7 | 1.28 | 1.30 |
| 200 | 1.05 | 1.07 | +1.9  | 1.36 | 1.37 | +0.7 | 1.58 | 1.60 |
| 400 | 1.69 | 1.73 | +2.4  | 2.02 | 2.05 | +1.5 | 2.26 | 2.25 |

Mixed-sign drift ≤5.2% on every bucket — within run-to-run noise on
this host.

### Storage (`ec_real_10k_hnsw`)

| Field | Baseline | Task 52 |
| --- | --- | --- |
| heap | 1.3 MiB | 1.3 MiB |
| table (heap + toast + fsm/vm) | 159.4 MiB | 159.4 MiB |
| indexes total | 25.3 MiB | 25.3 MiB |
| total | 184.6 MiB | 184.6 MiB |
| per row (total) | 19359.3 B | 19361.0 B |
| per row (heap only) | 136.8 B | 136.8 B |
| `m=16` idx | 13.0 MiB / 1366.4 B/row | 13.0 MiB / 1366.4 B/row |
| `m=8` idx | 11.8 MiB / 1235.4 B/row | 11.8 MiB / 1235.4 B/row |
| `corpus_pkey` btree | 456.0 KiB / 46.7 B/row | 456.0 KiB / 46.7 B/row |

**Identical to the B/row.** The +1.7 B/row delta in total-per-row
is FSM/VM noise (random VACUUM state).

## §100k corpus

### Recall@10 (`ec_real_100k_hnsw`, 1000 queries × 10000 trials per ef)

| ef | Baseline recall@k | Task 52 recall@k | Δ | Within ci95? |
| --- | ---: | ---: | ---: | --- |
| 40  | 0.7426 | 0.7434 | +0.0008 | yes (ci95 ±0.0086) |
| 80  | 0.8506 | 0.8503 | -0.0003 | yes (ci95 ±0.0070) |
| 120 | 0.8973 | 0.8973 | 0.0000  | yes |
| 200 | 0.9414 | 0.9419 | +0.0005 | yes (ci95 ±0.0046) |
| 400 | 0.9676 | 0.9678 | +0.0002 | yes (ci95 ±0.0035) |

**All deltas inside ci95.** The sub-0.001 jitter is from
worker-scheduling order producing tiny variations in neighbor
selection during the parallel build; mathematically equivalent
graphs by recall.

### Latency (`ec_real_100k_hnsw`, 1000 trials per ef, ms)

| ef | Base p50 | T52 p50 | Δ% | Base p95 | T52 p95 | Δ% | Base p99 | T52 p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 0.92 | 0.96 | +4.3 | 1.61 | 1.62 | +0.6  | 2.25 | 2.40 |
| 80  | 1.57 | 1.60 | +1.9 | 2.56 | 2.62 | +2.3  | 3.50 | 3.68 |
| 120 | 1.96 | 1.96 | 0.0  | 3.11 | 3.16 | +1.6  | 4.19 | 4.19 |
| 200 | 2.80 | 2.69 | -3.9 | 4.03 | 3.88 | -3.7  | 5.54 | 5.35 |
| 400 | 4.92 | 4.66 | -5.3 | 6.38 | 6.12 | -4.1  | 7.98 | 7.33 |

Mixed-sign drift, ≤5.3%. At higher ef (200, 400) Task 52 is slightly
**faster** than baseline (worker tail call simplification?
measurement noise?). At lower ef Task 52 is slightly slower.
Net no-regression.

### Storage (`ec_real_100k_hnsw`)

| Field | Baseline | Task 52 |
| --- | --- | --- |
| heap | 13.0 MiB | 13.0 MiB |
| table (heap + toast + fsm/vm) | 1.6 GiB | 1.6 GiB |
| indexes total | 134.5 MiB | 134.5 MiB |
| total | 1.7 GiB | 1.7 GiB |
| per row (total) | 18117.1 B | 18117.4 B |
| per row (heap only) | 136.8 B | 136.8 B |
| `m=16` idx | 130.2 MiB / 1365.4 B/row | 130.2 MiB / 1365.4 B/row |
| `corpus_pkey` btree | 4.3 MiB / 45.1 B/row | 4.3 MiB / 45.1 B/row |

**Identical to the B/row.** The +0.3 B/row delta in total-per-row
is FSM/VM noise.

## Disposition

**No regression on either corpus size.**

- 10k recall: exact-equal to 4 decimals.
- 100k recall: deltas <0.001, inside ci95 confidence envelope.
- 10k + 100k storage: identical index sizes B/row; total-per-row
  within FSM/VM noise (<2 B).
- 10k + 100k latency: mixed-sign drift, ≤5.3% on every p50/p95/p99
  bucket, within typical run-to-run noise on the M5 Pro host. At
  100k ef≥200, Task 52 is slightly faster than baseline.

Task 52 §Exit Criterion #3 — "HNSW recall + QPS no regression vs
post-Task-50 baseline" — is satisfied with the full 8-step bench
matching the baseline shape (load + recall + latency + storage at
both 10k and 100k, same `m`, ef_construction, ef sweep).

The acceptance tolerance matches the Task 50 closeout's "functional +
forward-baseline" disposition (see
`benchmarks/task-50-m5-hnsw-baseline/feedback/2026-05-23-01-reviewer.md`
§"Tolerance note").

## Build-path coverage

The bench's `load-10k-hnsw` and `load-100k-hnsw` steps each ran
`CREATE INDEX ... USING ec_hnsw (...)` invocations going through the
Task 52-migrated parallel-build path:

- `enter_parallel_mode` → `CreateParallelContext` →
  `ParallelContextRef::new` → estimator phase (slice 006 safe
  `estimate_chunk` / `estimate_keys` via `pcxt_ref.estimator_mut()`)
  → `initialize_dsm` → `ShmTocBuilder::{allocate_typed, insert}` →
  `view.init_synchronization` (leader, slice 005) → `launch_workers`
  → workers call `view.record_workers_done` at scan tail (slice 005)
  → `wait_for_workers_to_finish` → `destroy` → `exit_parallel_mode`.
- Both `parallel_build_worker_main` (heap-scan phase) and
  `parallel_graph_build_worker_main` (graph-build phase) run with
  slice-003+005 view methods.

A build-path semantics regression would surface as recall@k
mismatch (different neighbor topology). Recall matched exactly on
10k and within ci95 on 100k. Storage identical to B/row. Latency
within stddev.

## Cross-references

- Baseline: `benchmarks/task-50-m5-hnsw-baseline/manifest.md` and
  `benchmarks/task-50-m5-hnsw-baseline/artifacts/`.
- Bench logs in this packet:
  - `corpus-load-ec_real_10k-hnsw.log`
  - `recall-ec_real_10k-hnsw.log`
  - `latency-ec_real_10k-hnsw.log`
  - `storage-ec_real_10k-hnsw.log`
  - `corpus-load-ec_real_100k-hnsw.log`
  - `recall-ec_real_100k-hnsw.log`
  - `latency-ec_real_100k-hnsw.log`
  - `storage-ec_real_100k-hnsw.log`
  - `suite-manifest.json` — `ecaz bench suite` audit trail.
  - `results.jsonl` — structured per-step results.
