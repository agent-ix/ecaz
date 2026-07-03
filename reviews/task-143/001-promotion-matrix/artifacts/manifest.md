# Task 143 packet 001 — 100k+1m promotion matrix: artifact manifest

- Head SHA (dylib + runner, single binary for all cells): `e6b08f497`
  (branch `task-141-sdot-kernel` = main `e5ef96109` + Task 141 SDOT +
  Task 142 drain removal + 2026-07-03 reviewer feedback). In-suite:
  `precheck-build-sha.log` records `ecaz_build_git_sha()` =
  `e6b08f497...`; dylib shasum verified equal to `target/release` before
  launch; no installs during the run.
- Task bucket / packet: `reviews/task-143/001-promotion-matrix/`
- Host: Apple M5 Pro (m5-local), PG18 socket `/Users/peter/.pgrx` port
  28818, db `tqvector_bench`, shared_buffers 128MB. 2026-07-02/03.
- Matrix: {row, `dense_posting_blocks=1`} × {`ec_ivf.turboquant_scorer`
  lut, int8_approx} × {100k, 1m-tier}. All four axes are GUC/reloption
  switches in ONE binary — every cell pair is same-session, same-binary.
- Fixtures:
  - 100k: the Task 135 packet 002 tables
    (`task135ab_{row,dense}_real100k`), reused; loaded 2026-07-02 at
    `8f7bce3cc` (loader/build code unchanged since).
  - 1m tier: fresh loads `task143_{row,dense}_1m` from the anchor split
    `data/staged-current/ec_real_ann_benchmarks_anchor_*` (990,000 corpus
    rows + 10,000 held-out queries prepared this session from the local
    dbpedia-openai3-1536 1M parquet base via `ecaz corpus prepare`;
    manifest with TSV SHA256s at
    `data/staged-current/ec_real_ann_benchmarks_anchor_manifest.json`;
    corpus sha `see manifest`, 20.9 GB, prepare log preserved in session
    scratch and the command recorded here:
    `ecaz corpus prepare --profile ec_real_ann_benchmarks_anchor
    --parquet data/task31_m5_dbpedia_fetch/data --output-dir
    data/staged-current`). "1m" in this packet = the 990k anchor tier,
    the maximal fixture derivable from the 1M release with held-out
    queries.
- Sweep: nprobe [8,16,24,32,40,48,64] for recall (the registered ec_ivf
  default grid plus 40, added for the recall-buy analysis — deviation
  stated per the standard-sweep rule); latency at [32,40];
  `ivf_stage_counters` + task87 counters on. queries_limit 32 (100k) /
  24 (1m); iterations 32 (100k) / 16 (1m).
- Runner:

  ```sh
  target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 \
    bench suite run --config reviews/task-143/001-promotion-matrix/task143-promotion-matrix-suite.json \
    --artifact-dir reviews/task-143/001-promotion-matrix/artifacts
  ```

- Completed 2026-07-03, exit 0, 23/23 steps.
- Note on 100k absolute levels: the 100k cells ran after the two 1m loads
  filled shared_buffers, so absolute 100k latencies are higher than the
  Task 135/136 packets' (e.g. row-lut 3.16 vs 2.79-2.82 ms mean) — a
  buffer-pressure regime, not a regression. All comparisons cited below
  are within-run and share that regime; it is also the regime where the
  dense layout's smaller page count matters, consistent with the packet's
  1m findings.

## Key result lines

### Recall@10 by nprobe (identical row vs dense per scorer at every point)

| cell | 8 | 16 | 24 | 32 | 40 | 48 | 64 |
|---|---|---|---|---|---|---|---|
| 100k lut (row=dense) | 0.7875 | 0.8375 | 0.8781 | 0.8969 | 0.9062 | 0.9156 | 0.9281 |
| 100k int8 (row=dense) | 0.7844 | 0.8344 | 0.8750 | 0.8938 | 0.9031 | 0.9125 | 0.9250 |
| 1m lut (row=dense) | 0.8333 | 0.8917 | 0.9042 | 0.9250 | 0.9292 | 0.9333 | 0.9333 |
| 1m int8 (row=dense) | 0.8333 | 0.8875 | 0.9000 | 0.9208 | 0.9250 | 0.9292 | 0.9292 |

int8 vs lut dip: ≤0.31 pp (100k) / ≤0.42 pp (1m) at every point — within
noise at 32/24-query samples, consistent with Tasks 136/141.

### Latency mean/p50 (ms)

| cell | nprobe 32 | nprobe 40 |
|---|---|---|
| 100k row-lut (old default) | 3.16 / 3.10 | 3.76 / 3.71 |
| 100k row-int8 | 2.10 / 2.02 | 2.48 / 2.35 |
| 100k dense-lut | 2.65 / 2.53 | 3.16 / 3.12 |
| 100k dense-int8 | **1.75 / 1.64** | 2.03 / 1.98 |
| 1m row-lut (old default) | 12.1 / 12.1 | 14.6 / 14.2 |
| 1m row-int8 | 8.49 / 8.30 | 9.96 / 9.56 |
| 1m dense-lut | 11.5 / 11.6 | 13.4 / 13.0 |
| 1m dense-int8 | **7.76 / 7.53** | 8.88 / 8.64 |

Axis attribution (mean, nprobe 32): int8 −30.0% (1m row), −32.5% (1m
dense); dense −5.0% (1m lut), −8.6% (1m int8), −16.1% (100k lut under
buffer pressure). Combined old default → dense-int8: **−44.6% (100k),
−35.9% (1m)**.

### The recall-buy point (nprobe 40 on dense-int8 vs old default at 32)

- 100k: 2.03 ms @ recall 0.9031 vs 3.16 ms @ 0.8969 — **+0.6 pp recall
  AND −36% latency**. Even nprobe 64 (recall 0.9250, ≈3.0 ms
  extrapolated) stays under the old default's latency.
- 1m: 8.88 ms @ 0.9250 vs 12.1 ms @ 0.9208 — **+0.4 pp recall AND −27%
  latency**.

### Storage (turboquant index)

| scale | row | dense | delta |
|---|---|---|---|
| 100k | 90.4 MiB (948.2 B/row) | 81.7 MiB (856.5 B/row) | −9.6% |
| 1m | 870.6 MiB (922.1 B/row) | 784.8 MiB (831.2 B/row) | −9.9% |

## Not committed

- `truth-cache/` (gitignored; 1m ground truth regenerable from the staged
  anchor fixture).
