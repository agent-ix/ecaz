# Task 144 Packet 011 Artifact Manifest

- head SHA: `8d6248246`
- task bucket: `reviews/task-144/011-release-matrix-100k-r2/`
- suite config: `reviews/task-144/008-release-matrix-config-r2/artifacts/suite-task144-release-matrix-r2.json`
- artifact dir: `reviews/task-144/011-release-matrix-100k-r2/artifacts/`
- database: `tqvector_bench_task144`
- PG host/port: `/home/peter/dev/ecaz/target/task144-pg18-socket:28818`
- storage format: `rabitq`
- corpus scale: staged real corpus `100k`
- query limit: `200`
- top-k: `10`
- suite surface: isolated one-index-per-table prefixes
- captured release profile: PostgreSQL 18.3, `ecaz_build_profile=release`
- timestamp: 2026-07-05

## Command

```bash
target/release/ecaz --database tqvector_bench_task144 \
  --host /home/peter/dev/ecaz/target/task144-pg18-socket --port 28818 \
  bench suite run \
  --config reviews/task-144/008-release-matrix-config-r2/artifacts/suite-task144-release-matrix-r2.json \
  --artifact-dir reviews/task-144/011-release-matrix-100k-r2/artifacts \
  --manifest-output reviews/task-144/011-release-matrix-100k-r2/artifacts/suite-manifest-100k-r2.json \
  --results-output reviews/task-144/011-release-matrix-100k-r2/artifacts/results-100k-r2.jsonl \
  --continue-on-error \
  --only precheck-release-profile \
  --only <100k-tagged-steps-from-suite-config>
```

The `--only` list was generated from the approved r2 suite config with:

```bash
jq -r '.steps[] | select((.tags // []) | index("100k")) | "--only " + .name' \
  reviews/task-144/008-release-matrix-config-r2/artifacts/suite-task144-release-matrix-r2.json
```

The full expanded command is recorded in `suite-run-100k-r2.log`.

## Artifact Index

- `precheck-release-profile.log`: release profile SQL check.
- `suite-run-100k-r2.log`: full suite stdout/stderr.
- `suite-manifest-100k-r2.json`: structured suite manifest; 124 configured steps, with non-100k steps skipped by selector.
- `results-100k-r2.jsonl`: structured suite results.
- `pipeline-100k-*.log`: 30 completed pipeline logs for 5 index variants x 6 probe modes.
- `stage-containment-100k-*.jsonl`: 30 stage containment identity streams.
- `result-identity-100k-*.jsonl`: 30 result identity streams.
- `storage-100k-*.log`: storage logs for single, fixed_b2, closure_e010_b8, closure_e025_b8, closure_e050_b8.
- `load-100k-*.log`: load/build logs for single, fixed_b2, closure_e010_b8, closure_e025_b8, closure_e050_b8.

`truth-100k-k10.json` was generated for the run but is gitignored by policy and is not committed.

## Result Row Counts

`results-100k-r2.jsonl` contains 3116 rows:

- `spire-pipeline`: 3030
- `spire_pipeline_row_scan`: 900
- `storage_field`: 45
- `storage_index`: 10
- `storage_spire_replication`: 5
- `load_timing`: 25
- `recall`: 1

## Key 100k Readout

Rows with recall >= 0.99:

| row | nprobe | recall | candidate rows | ready rows | production p50 | recall p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| closure_e050_b8-ratio200 | 96 | 0.9925 | 78.6594% | 31.7537% | 73.132 ms | 1424.491 ms |
| closure_e050_b8-ratio800 | 96 | 0.9925 | 79.1109% | 31.9043% | 71.754 ms | 1425.227 ms |
| closure_e050_b8-ratio400 | 96 | 0.9925 | 79.1109% | 31.9043% | 72.181 ms | 1437.245 ms |
| closure_e050_b8-fixed | 96 | 0.9925 | 79.1109% | 31.9043% | 72.587 ms | 1437.009 ms |
| closure_e050_b8-adaptive | 96 | 0.9925 | 79.5005% | 32.6196% | 73.023 ms | 1454.104 ms |

Best recall per other index family:

| row | nprobe | recall | candidate rows | ready rows | production p50 |
| --- | ---: | ---: | ---: | ---: | ---: |
| single-adaptive | 96 | 0.9240 | 10.9348% | 10.9348% | 14.224 ms |
| fixed_b2-adaptive | 96 | 0.9775 | 33.1731% | 21.3365% | 33.266 ms |
| closure_e010_b8-adaptive | 96 | 0.9680 | 26.4354% | 17.3397% | 27.806 ms |
| closure_e025_b8-adaptive | 96 | 0.9835 | 54.9097% | 25.9290% | 52.271 ms |

Storage and replication:

| index | index size | mean replicas/vector | leaf assignments | object count |
| --- | ---: | ---: | ---: | ---: |
| single | 89.8 MiB | 1.0000 | 100000 | 1034 |
| fixed_b2 | 246.0 MiB | 3.0000 | 300000 | 1034 |
| closure_e010_b8 | 204.0 MiB | 2.4607 | 246069 | 1034 |
| closure_e025_b8 | 408.6 MiB | 5.0811 | 508109 | 1034 |
| closure_e050_b8 | 568.8 MiB | 7.1315 | 713147 | 1034 |

Load/build totals:

| index | total load | build_index |
| --- | ---: | ---: |
| single | 116.68 s | 22.83 s |
| fixed_b2 | 383.26 s | 295.15 s |
| closure_e010_b8 | 377.61 s | 292.90 s |
| closure_e025_b8 | 391.76 s | 304.94 s |
| closure_e050_b8 | 389.13 s | 309.60 s |

## Decision Implication

At 100k, closure epsilon 0.50 is the only family that reaches recall >= 0.99, but it requires roughly 79% candidate row scan, about 72-73 ms production p50, 568.8 MiB index size, and 7.1315 mean replicas/vector. The lower-fanout rows do not meet recall 0.99, and ratio pruning does not produce a cheap operating point. This supports iterate/escalate, not promote.
