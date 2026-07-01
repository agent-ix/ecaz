# Task 121 Phase 3 100k Retuned Latency/Pipeline Summary

## Setup

- Prefix: `t121_s3_100k_b4_tr50_f8_b64`
- Rows: 100000
- Storage format: RaBitQ, 4-bit
- Training: `training_sample_rows=50000`
- Block summaries: leaf block rows 64, representatives 2
- Queries: 200
- Sweep: `nprobe=8,16,32,48,96`
- Policy off: all leaf-block pruning knobs set to 0.
- Policy retuned: `max_global_blocks=4096`, `global_probe_blocks=8192`,
  `sample_rows_per_block=4`, `sample_summary_prior_weight=0.8`,
  `summary_radius_weight=0.25`, `route_prior_weight=0.0`.

## Suite Status

```text
[suite:task121-phase3-local-100k-retuned-latency-pipeline] completed=7 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Storage

```text
index=404.8 MiB, index_per_row=4244.8 B
total=1.9 GiB, total_per_row=20915.2 B
```

The policy is a query-time gate, so it does not change storage.

## Truth-Cache Seed

```text
nprobe=96 recall@10=1.0000 mean_q_time=4639.98 ms
```

The generated `truth-cache-100k-q200-k10.json` is local-only and is not
committed.

## Clean Latency A/B

```text
nprobe  off_p50   retuned_p50  p50_delta   off_mean   retuned_mean  mean_delta
8       960.5 ms  951.4 ms     -0.9%       966.5 ms   969.9 ms      +0.4%
16      1624.1 ms 1641.3 ms    +1.1%       1634.1 ms  1649.1 ms     +0.9%
32      2678.3 ms 2685.4 ms    +0.3%       2639.4 ms  2649.4 ms     +0.4%
48      3411.2 ms 3367.9 ms    -1.3%       3425.7 ms  3380.6 ms     -1.3%
96      4622.4 ms 4200.8 ms    -9.1%       4653.1 ms  4240.1 ms     -8.9%
```

The retuned policy increases backend RSS at every checkpoint, from 254724 KiB
at `nprobe=8` to 643488 KiB at `nprobe=96`.

## Pipeline A/B

```text
nprobe  off_p50     retuned_p50  p50_delta   off_recall  retuned_recall
8       954.048 ms  946.735 ms   -0.8%       0.9330      0.9330
16      1603.581 ms 1602.378 ms  -0.1%       0.9670      0.9670
32      2618.912 ms 2616.362 ms  -0.1%       0.9895      0.9895
48      3368.828 ms 3372.773 ms  +0.1%       0.9945      0.9945
96      4607.716 ms 4211.934 ms  -8.6%       1.0000      1.0000
```

## Pipeline Counters

```text
nprobe  off_candidates  retuned_candidates  off_heap_rerank  retuned_heap_rerank  off_object_bytes  retuned_object_bytes
8       6550241         6550241             4661921          4661921              5504487114        5504487114
16      13261932        13261932            7609988          7609988              11144593728       11144593728
32      26533390        26533390            11786775         11786775             22297157442       22297157442
48      39356548        39356548            14722347         14722347             33073013046       33073013046
96      76623116        56982159            19307246         17473898             64389908578       64389908578
```

The retuned policy does not reduce read/object bytes at any checkpoint. It only
cuts candidate and heap-rerank work at `nprobe=96`, where candidates drop 25.6%
and heap-rerank rows drop 9.5%.

## Read

This packet answers the packet-022 operating-point question for 100k. The
retuned global sampled gate is recall-neutral across the measured sweep, but it
does not materially improve the likely low operating point (`nprobe=8..32`).
The measurable win is concentrated at the high-recall endpoint (`nprobe=96`),
with a smaller clean-latency hint at `nprobe=48` that does not survive as a
pipeline win.

The mechanism is compute/rerank reduction, not I/O reduction: object bytes are
unchanged, while candidate and heap-rerank rows fall only at `nprobe=96`.
