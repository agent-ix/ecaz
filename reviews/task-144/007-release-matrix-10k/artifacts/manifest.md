# Task 144 Packet 007 Artifact Manifest

- head SHA: `f965c836ed9fdfa09fd7563dba695745f8b274b5`
- task bucket: `reviews/task-144`
- packet path: `reviews/task-144/007-release-matrix-10k`
- lane: local PG18 release lane
- database: `tqvector_bench_task144`
- host / port: `/home/peter/dev/ecaz/target/task144-pg18-socket` / `28818`
- fixture: `data/staged-current/ec_real_10k_{corpus,queries,manifest}.tsv/json`
- runner config: `reviews/task-144/006-release-matrix-config/artifacts/suite-task144-release-matrix.json`
- storage format: `rabitq`
- top-k / query count: `10` / `200`
- sweep: `nprobe=8,16,32,64,96`
- isolated surfaces: yes, one table/index per variant prefix
- timestamp: 2026-07-05

## Release Install And Profile

The current branch release backend was installed before the successful suite
slice:

```text
target/release/ecaz dev install ecaz-pg-test --pg 18 \
  --log-file reviews/task-144/007-release-matrix-10k/artifacts/install-release-pg18.log
```

Key evidence:

- `install-release-pg18.log`: `[install] sha256=a821e3ee67501cc7489dcc9380e2bfab867b33388f600ef1f8109d19751a5bf8`
- `precheck-release-profile.log`: `ecaz_build_profile() = release`

## Suite Runs

Load and storage were run through `ecaz bench suite` with the Task 144 release
matrix config:

```text
target/release/ecaz bench suite --database tqvector_bench_task144 \
  --host /home/peter/dev/ecaz/target/task144-pg18-socket --port 28818 \
  --log-file reviews/task-144/007-release-matrix-10k/artifacts/suite-run-10k-r3.log \
  run --config reviews/task-144/006-release-matrix-config/artifacts/suite-task144-release-matrix.json \
  --artifact-dir reviews/task-144/007-release-matrix-10k/artifacts \
  --manifest-output reviews/task-144/007-release-matrix-10k/artifacts/suite-manifest-10k-r3.json \
  --results-output reviews/task-144/007-release-matrix-10k/artifacts/results-10k-r3.jsonl \
  --continue-on-error --only precheck-release-profile --only load-10k-single \
  --only load-10k-fixed-b2 --only load-10k-closure-e010-b8 \
  --only storage-10k-single --only storage-10k-fixed-b2 \
  --only storage-10k-closure-e010-b8 --only pipeline-10k-single-fixed \
  --only pipeline-10k-single-ratio125 --only pipeline-10k-single-adaptive \
  --only pipeline-10k-fixed-b2-fixed --only pipeline-10k-fixed-b2-ratio125 \
  --only pipeline-10k-fixed-b2-adaptive --only pipeline-10k-closure-e010-b8-fixed \
  --only pipeline-10k-closure-e010-b8-ratio125 \
  --only pipeline-10k-closure-e010-b8-adaptive
```

That run completed load/storage and exposed that `spire-pipeline` requires the
truth-cache file to exist first. The cache was generated with:

```text
target/release/ecaz bench --database tqvector_bench_task144 \
  --host /home/peter/dev/ecaz/target/task144-pg18-socket --port 28818 \
  --log-file reviews/task-144/007-release-matrix-10k/artifacts/generate-truth-10k.log \
  recall --prefix t144_10k_single --profile ec_spire --k 10 --queries-limit 200 \
  --sweep 8 --truth-corpus-file data/staged-current/ec_real_10k_corpus.tsv \
  --truth-cache-file reviews/task-144/007-release-matrix-10k/artifacts/truth-10k-k10.json \
  --log-output reviews/task-144/007-release-matrix-10k/artifacts/generate-truth-10k-table.log
```

`truth-10k-k10.json` was intentionally not committed; AGENTS.md treats
truth-cache files as regenerable cache artifacts.

The nine pipeline cells were then rerun through the suite:

```text
target/release/ecaz bench suite --database tqvector_bench_task144 \
  --host /home/peter/dev/ecaz/target/task144-pg18-socket --port 28818 \
  --log-file reviews/task-144/007-release-matrix-10k/artifacts/suite-run-10k-r4-pipelines.log \
  run --config reviews/task-144/006-release-matrix-config/artifacts/suite-task144-release-matrix.json \
  --artifact-dir reviews/task-144/007-release-matrix-10k/artifacts \
  --manifest-output reviews/task-144/007-release-matrix-10k/artifacts/suite-manifest-10k-r4-pipelines.json \
  --results-output reviews/task-144/007-release-matrix-10k/artifacts/results-10k-r4-pipelines.jsonl \
  --resume-from reviews/task-144/007-release-matrix-10k/artifacts/suite-manifest-10k-r3.json \
  --only precheck-release-profile --only pipeline-10k-single-fixed \
  --only pipeline-10k-single-ratio125 --only pipeline-10k-single-adaptive \
  --only pipeline-10k-fixed-b2-fixed --only pipeline-10k-fixed-b2-ratio125 \
  --only pipeline-10k-fixed-b2-adaptive --only pipeline-10k-closure-e010-b8-fixed \
  --only pipeline-10k-closure-e010-b8-ratio125 \
  --only pipeline-10k-closure-e010-b8-adaptive
```

`suite-manifest-10k-r4-pipelines.json` status:

```text
completed=10 failed=0 skipped=36 dry_run=0 missing_artifacts=0 stale=0
```

## Load And Storage Results

| variant | prefix | index reloptions under review | storage total | index total | index per row |
|---|---|---|---:|---:|---:|
| single | `t144_10k_single` | `boundary_replica_count=0,closure_epsilon=0` | 177.2 MiB | 17.9 MiB | 1880.1 B |
| fixed_b2 | `t144_10k_fixed_b2` | `boundary_replica_count=2,closure_epsilon=0` | 194.1 MiB | 34.9 MiB | 3655.3 B |
| closure_e010_b8 | `t144_10k_closure_e010_b8` | `boundary_replica_count=8,closure_epsilon=0.10` | 177.7 MiB | 18.5 MiB | 1938.2 B |

## Pipeline Results

The table reports the best recall operating point in each 10k cell. `scanned`
is the sum over 200 queries, so 1,600 is 8 route/list probes per query and
19,200 is 96 route/list probes per query.

| cell | best distinct recall | nprobe | p50 | p95 | scanned |
|---|---:|---:|---:|---:|---:|
| single fixed | 0.9935 | 96 | 7.610 ms | 8.666 ms | 19200 |
| single ratio125 | 0.7635 | 96 | 7.166 ms | 7.784 ms | 411 |
| single adaptive | 0.9950 | 96 | 7.981 ms | 9.071 ms | 19200 |
| fixed_b2 fixed | 0.9955 | 96 | 9.761 ms | 10.776 ms | 19200 |
| fixed_b2 ratio125 | 0.9150 | 96 | 8.395 ms | 9.391 ms | 411 |
| fixed_b2 adaptive | 0.9965 | 96 | 9.774 ms | 10.654 ms | 19200 |
| closure_e010_b8 fixed | 0.9940 | 96 | 8.606 ms | 10.335 ms | 19200 |
| closure_e010_b8 ratio125 | 0.7680 | 96 | 7.863 ms | 8.668 ms | 411 |
| closure_e010_b8 adaptive | 0.9955 | 96 | 8.222 ms | 9.779 ms | 19200 |

Selected low-probe rows:

| cell | distinct recall | nprobe | p50 | p95 | scanned |
|---|---:|---:|---:|---:|---:|
| fixed_b2 fixed | 0.9860 | 8 | 7.713 ms | 8.690 ms | 1600 |
| fixed_b2 adaptive | 0.9745 | 8 | 5.231 ms | 6.282 ms | 1600 |
| closure_e010_b8 fixed | 0.9765 | 8 | 7.246 ms | 7.920 ms | 1600 |
| closure_e010_b8 adaptive | 0.9515 | 8 | 5.060 ms | 6.492 ms | 1600 |
| fixed_b2 ratio125 | 0.9115 | 8 | 7.145 ms | 7.954 ms | 331 |

## Artifact Index

- `suite-manifest-10k-r3.json`, `results-10k-r3.jsonl`: load/storage suite slice
- `suite-manifest-10k-r4-pipelines.json`, `results-10k-r4-pipelines.jsonl`: successful pipeline suite slice
- `load-10k-*.log`: load/build logs for the three variants
- `storage-10k-*.log`: storage logs for the three variants
- `pipeline-10k-*.log`: recall/latency/profile logs for the nine pipeline cells
- `stage-containment-10k-*.jsonl`: per-query recall and probed-list containment evidence
- `result-identity-10k-*.jsonl`: per-query result identity evidence
- `generate-truth-10k-table.log`: truth-cache generation command output

## Follow-Up From This Packet

Before scaling this exact config to 50k/100k, the suite needs a first-class
truth-cache prerequisite step or an equivalent runner change. Otherwise a clean
artifact directory fails the pipeline steps even though the suite config audits.
