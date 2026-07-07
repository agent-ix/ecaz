# Task 168 Packet 001 — rabitq streaming-path characterization (Phase 1)

- Task: `plan/tasks/168-diskann-batched-beam-and-prefetch.md` (task file on
  branch `task-161-ec-distann-specs`; this work branch is
  `task-168-diskann-batched-beam` off main `b891c3743`).
- Head SHA at run time: `da3c107db` (flush-width instrumentation commit).
- Timestamp: 2026-07-06 (run completed same day, Intel desktop local time).
- Host: Intel desktop (local bench host), PG18 pgrx install
  `/home/peter/.pgrx/18.3/pgrx-install`, port 28818, socket
  `/home/peter/.pgrx`, db `tqvector_bench`.
- Backend: **release** build verified via `SELECT ecaz_build_profile()` →
  `release` (`build-profile.log`).
- Fixture: staged real corpora `data/staged-current/ec_real_{10k,50k,100k}_*`
  (isolated one-index-per-table surfaces, prefixes
  `t168_p1_real{10k,50k,100k}_diskann`; worktree symlinks `data/staged-current`
  to the shared checkout's staged dir).
- Access method: `ec_diskann`, storage_format **rabitq** (streaming
  RelationGraphReader path), reloptions pinned to the Task 70 reference:
  `graph_degree=32, build_list_size=100, alpha=1.2, rerank_budget=64, top_k=10`.
- Command: `ecaz --host /home/peter/.pgrx --port 28818 bench suite run
  --config reviews/task-168/001-rabitq-characterization/artifacts/suite.json
  --artifact-dir reviews/task-168/001-rabitq-characterization/artifacts`
  (18/18 steps passed; `suite-run.log`, `suite-manifest.json`,
  `results.jsonl`).
- Bespoke SuiteConfig justification (per CLAUDE.md): Task 168 Phase 1
  characterization, not the standard lane sweep — adds scan-profile NOTICE
  capture steps (raw `dev sql` steps at L=64/L=800 per scale, Task 70
  packet-012 pattern) and pins `storage_format=rabitq` explicitly because the
  standard lane config's diskann load step currently inherits the stale
  `StorageFormat::DEFAULT = PqFastScan` (`options.rs:66`; flipped later in
  this task). Recall/latency steps use the registered `ec_diskann`
  `default_sweep` `[64,128,200,400,800]` verbatim.
- Instrumentation: head includes `da3c107db` (per-hop flush-width histogram
  in FrontierProfile; NOTICE fields `flush_width_zero`, `flush_width_1_7`,
  `flush_width_8_15`, `flush_width_16_31`, `flush_width_ge32`). Profiled
  path only; unprofiled loop untouched. The profile NOTICE runs are separate
  raw steps — the latency steps ran **without** the profile GUC.

## Artifacts

| Artifact | Producer | Key result lines |
|---|---|---|
| `suite.json` | hand-authored | 18 steps: load/recall/latency/storage + profile-notice raw steps (L=64, L=800) × 3 scales |
| `suite-manifest.json`, `results.jsonl`, `suite-run.log` | suite runner | 18/18 ok; 81 normalized rows |
| `load-{10k,50k,100k}-diskann-rabitq.log` | corpus load | builds 22.75 s / 200.24 s / 473.11 s |
| `recall-{scale}-diskann-rabitq.log`, `truth-{scale}-k10.json` | bench recall | recall@10 references (table below) |
| `latency-{scale}-diskann-rabitq.log` | bench latency | baseline latency per L (table below) |
| `storage-{scale}-diskann-rabitq.log` | bench storage | index 431.3–432.5 B/row (R=32 rabitq) at all scales |
| `profile-notices-{scale}-l{64,800}.{sql,log}` | dev sql raw steps | 200 NOTICE rows each; split + width histogram below |
| `aggregate-profile-notices.py`, `profile-summary.txt` | aggregation | per-key mean/p50/p95 + shares |
| `build-profile.log` | dev sql | `ecaz_build_profile()` = `release` |

## Aggregation command

```
python3 reviews/task-168/001-rabitq-characterization/artifacts/aggregate-profile-notices.py \
  reviews/task-168/001-rabitq-characterization/artifacts/profile-notices-{10k,50k,100k}-l{64,800}.log
```

## Key results

### Recall / latency references (rabitq, k=10, 200 queries, 200 iterations)

| scale | L | recall@10 | p50 | mean | p95 |
|---|---|---|---|---|---|
| 10k | 64 | 0.9990 | 3.16 ms | 3.19 ms | 3.42 ms |
| 10k | 128 | 0.9995 | 3.51 ms | 3.55 ms | 4.02 ms |
| 10k | 200 | 1.0000 | 3.89 ms | 3.94 ms | 4.50 ms |
| 10k | 400 | 1.0000 | 4.60 ms | 4.62 ms | 5.38 ms |
| 10k | 800 | 1.0000 | 6.14 ms | 6.32 ms | 7.59 ms |
| 50k | 64 | 0.9685 | 3.79 ms | 3.80 ms | 4.37 ms |
| 50k | 128 | 0.9865 | 4.30 ms | 4.33 ms | 5.05 ms |
| 50k | 200 | 0.9905 | 5.06 ms | 5.12 ms | 6.16 ms |
| 50k | 400 | 0.9950 | 6.89 ms | 6.93 ms | 8.60 ms |
| 50k | 800 | 0.9965 | 10.0 ms | 10.3 ms | 13.2 ms |
| 100k | 64 | 0.9275 | 4.21 ms | 4.22 ms | 4.75 ms |
| 100k | 128 | 0.9665 | 5.34 ms | 5.35 ms | 5.92 ms |
| 100k | 200 | 0.9845 | 7.02 ms | 7.31 ms | 10.0 ms |
| 100k | 400 | 0.9940 | 9.22 ms | 9.25 ms | 10.9 ms |
| 100k | 800 | 0.9975 | 14.6 ms | 14.5 ms | 17.8 ms |

Recall floor for later phases: recall@10 within 0.5 pp of this table at each
(scale, L).

### Per-query wall-time split (profile NOTICE means, µs; share of total)

| scale/L | graph_read_decode | prefilter_score | frontier residual | exact_rerank | total | frontier share |
|---|---|---|---|---|---|---|
| 10k/64 | 59 (2.1%) | 49 (1.7%) | 775 (27.6%) | 1890 (67.3%) | 2809 | 27.6% |
| 10k/800 | 236 (3.0%) | 189 (2.4%) | 5420 (69.3%) | 1926 (24.6%) | 7817 | 69.3% |
| 50k/64 | 105 (3.2%) | 85 (2.6%) | 1183 (36.0%) | 1875 (57.1%) | 3284 | 36.0% |
| 50k/800 | 596 (4.8%) | 524 (4.3%) | 9217 (74.9%) | 1920 (15.6%) | 12303 | 74.9% |
| 100k/64 | 315 (8.5%) | 99 (2.7%) | 1322 (35.5%) | 1940 (52.2%) | 3720 | 35.5% |
| 100k/800 | 2200 (12.9%) | 787 (4.6%) | 12065 (70.6%) | 1971 (11.5%) | 17079 | 70.6% |

`frontier residual` = `frontier_us` (loop wall time minus reads, scoring,
prefetch, rerank — i.e. frontier maintenance + per-hop allocation). Its
sub-counters (`candidate_heap_us`, `visited_set_us`) are each < 3% of it,
so the residual is dominated by per-hop allocation/move work, not the heap
or hash operations themselves.

### Per-hop flush-width histogram (share of hops)

| scale/L | zero | 1–7 | 8–15 | 16–31 | ≥32 |
|---|---|---|---|---|---|
| 10k/64 | 1.3% | 33.3% | 41.0% | 22.8% | 1.5% |
| 10k/800 | 15.2% | 62.5% | 18.5% | 3.7% | 0.1% |
| 50k/64 | 0.3% | 13.3% | 30.9% | 53.0% | 2.4% |
| 50k/800 | 5.2% | 41.3% | 31.4% | 21.8% | 0.2% |
| 100k/64 | 0.2% | 10.3% | 23.3% | 64.2% | 2.1% |
| 100k/800 | 1.9% | 27.1% | 34.1% | 36.8% | 0.2% |

The 32-wide SIMD block fires on ≤ 2.4% of hops everywhere; at high L the
modal flush is 1–7 wide (frontier dedup starves the batch).

### Storage

Index 431.3–432.5 B/row (R=32, rabitq) at all three scales
(`storage-*-diskann-rabitq.log`); scale-invariant per-row cost as expected.
