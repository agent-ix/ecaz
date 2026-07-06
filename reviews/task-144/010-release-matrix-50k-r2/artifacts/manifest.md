# Task 144 / Packet 010 Artifact Manifest

- Head SHA: `07835d7faf552c8abdc4ff3cb4106de9185c63b5`
- Task bucket: `reviews/task-144/010-release-matrix-50k-r2`
- Timestamp: 2026-07-05
- Database: `tqvector_bench_task144`
- Host/socket: `/home/peter/dev/ecaz/target/task144-pg18-socket`
- Port: `28818`
- Backend build profile: `release`
- Per-node build profile: `coordinator:28818:release`
- Runner: `target/release/ecaz 0.1.0`
- Suite config: `../008-release-matrix-config-r2/artifacts/suite-task144-release-matrix-r2.json`
- Surface: isolated one-index-per-table suite prefixes (`t144_50k_*`)

## Command

```text
target/release/ecaz --database tqvector_bench_task144 --host /home/peter/dev/ecaz/target/task144-pg18-socket --port 28818 bench suite run --config reviews/task-144/008-release-matrix-config-r2/artifacts/suite-task144-release-matrix-r2.json --artifact-dir reviews/task-144/010-release-matrix-50k-r2/artifacts --manifest-output reviews/task-144/010-release-matrix-50k-r2/artifacts/suite-manifest-50k-r2.json --results-output reviews/task-144/010-release-matrix-50k-r2/artifacts/results-50k-r2.jsonl --continue-on-error --only precheck-release-profile --only <all 50k-tagged steps>
```

The suite completed and wrote `suite-manifest-50k-r2.json` and
`results-50k-r2.jsonl`; `suite-run-50k-r2.log` ends with the final manifest and
results writes.

## Artifact Index

- `suite-manifest-50k-r2.json`: suite manifest for the 50k selected steps.
- `results-50k-r2.jsonl`: structured suite results; includes release profile,
  recall, production read latency, `spire_pipeline_row_scan`, storage, and load
  rows.
- `suite-run-50k-r2.log`: full suite stdout/stderr.
- `precheck-release-profile.log`: explicit release-profile precheck.
- `load-50k-*.log`: load/build logs for single, fixed_b2, and closure epsilon
  variants.
- `storage-50k-*.log`: corrected storage and replication summaries.
- `pipeline-50k-*.log`: pipeline logs for all 30 50k cells.
- `stage-containment-50k-*.jsonl`: per-query containment/probe-tail evidence.
- `result-identity-50k-*.jsonl`: per-query result identity evidence.
- `truth-cache-50k-k10.log`: truth-cache command log.

`truth-50k-k10.json` is a regenerable truth cache and is intentionally ignored
by `.gitignore`; it is not intended as committed review evidence.

## Result Row Coverage

`results-50k-r2.jsonl` contains:

```text
spire-pipeline             2130 rows
spire_pipeline_row_scan     900 rows
storage_field                45 rows
load_timing                  25 rows
storage_index                10 rows
storage_spire_replication     5 rows
recall                        1 row
```

All latency-emitting pipeline rows carry
`backend_build_profile=release` and
`backend_node_profiles=coordinator:28818:release`.

## Key 50k Results

Rows that reached `distinct_recall@10 >= 0.99`:

```text
cell                         nprobe  recall  candidate%  ready%   production_p50  recall_p50  result_source
closure_e025_b8-adaptive     96      0.9905  58.8173     22.3965  30.042 ms       454.994 ms  local_heap_candidates
closure_e050_b8-adaptive     96      0.9925  86.0754     27.6823  41.944 ms       604.198 ms  local_heap_candidates
closure_e050_b8-fixed        96      0.9900  82.2756     23.9815  38.064 ms       505.845 ms  local_heap_candidates
closure_e050_b8-ratio200     96      0.9900  80.9118     23.6585  37.820 ms       491.704 ms  local_heap_candidates
closure_e050_b8-ratio400     96      0.9900  82.2756     23.9815  38.924 ms       507.235 ms  local_heap_candidates
closure_e050_b8-ratio800     96      0.9900  82.2756     23.9815  38.382 ms       501.657 ms  local_heap_candidates
fixed_b2-adaptive            96      0.9900  35.6834     20.6858  20.434 ms       412.930 ms  local_heap_candidates
```

Representative nprobe 32 rows:

```text
cell                         recall  candidate%  ready%   production_p50  recall_p50
single-fixed                 0.8760  4.1195      4.1195   9.430 ms        81.281 ms
single-adaptive              0.8695  4.1369      4.1369   9.707 ms        82.563 ms
fixed_b2-fixed               0.9510  13.1338     8.4303   13.030 ms       170.708 ms
fixed_b2-adaptive            0.9585  13.1074     9.1753   13.183 ms       177.521 ms
closure_e010_b8-fixed        0.9200  10.1272     6.4842   11.904 ms       125.964 ms
closure_e010_b8-adaptive     0.9200  10.1616     7.0624   12.324 ms       141.871 ms
closure_e025_b8-fixed        0.9545  21.5104     9.7603   16.166 ms       191.908 ms
closure_e025_b8-adaptive     0.9600  21.4446     11.3180  14.837 ms       217.047 ms
closure_e050_b8-fixed        0.9685  31.7191     12.5962  19.070 ms       239.234 ms
closure_e050_b8-adaptive     0.9710  31.3748     14.8732  18.747 ms       293.966 ms
```

Corrected storage / replication rows:

```text
variant              index_size  mean_replicas_per_vector
single               50.4 MiB    1.0000
fixed_b2             129.0 MiB   3.0000
closure_e010_b8      91.6 MiB    2.0447
closure_e025_b8      168.1 MiB   4.0042
closure_e050_b8      242.7 MiB   5.9123
```

Interpretation for 50k only: the 10k AC rows do not carry forward. At 50k,
every row reaching 0.99 recall requires nprobe 96 and exceeds the 5%
candidate-row budget. `fixed_b2-adaptive` is the least expensive 0.99 recall
row by candidate percentage and production p50, while closure variants require
more candidates and more replicas. This is not Task 144 closeout; the approved
100k slice remains required before the promote / iterate / escalate decision.
