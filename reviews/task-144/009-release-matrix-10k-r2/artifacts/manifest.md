# Task 144 / Packet 009 Artifact Manifest

- Head SHA: `9beb461aa8659017393121b3ca35a4687a063f28`
- Task bucket: `reviews/task-144/009-release-matrix-10k-r2`
- Timestamp: 2026-07-05
- Database: `tqvector_bench_task144`
- Host/socket: `/home/peter/dev/ecaz/target/task144-pg18-socket`
- Port: `28818`
- Backend build profile: `release` (`precheck-before-suite.log`; suite precheck)
- Runner: `target/release/ecaz 0.1.0`
- Suite config: `../008-release-matrix-config-r2/artifacts/suite-task144-release-matrix-r2.json`

## Commands

Release build:

```text
cargo build --release -p ecaz-cli
```

Precheck:

```text
target/release/ecaz --database tqvector_bench_task144 --host /home/peter/dev/ecaz/target/task144-pg18-socket --port 28818 dev sql --pg 18 --db tqvector_bench_task144 --socket-dir /home/peter/dev/ecaz/target/task144-pg18-socket --raw --sql "LOAD 'ecaz'; SELECT ecaz_build_profile();" --log-output reviews/task-144/009-release-matrix-10k-r2/artifacts/precheck-before-suite.log
```

10k slice:

```text
target/release/ecaz --database tqvector_bench_task144 --host /home/peter/dev/ecaz/target/task144-pg18-socket --port 28818 bench suite run --config reviews/task-144/008-release-matrix-config-r2/artifacts/suite-task144-release-matrix-r2.json --artifact-dir reviews/task-144/009-release-matrix-10k-r2/artifacts --manifest-output reviews/task-144/009-release-matrix-10k-r2/artifacts/suite-manifest-10k-r2.json --results-output reviews/task-144/009-release-matrix-10k-r2/artifacts/results-10k-r2.jsonl --continue-on-error --only precheck-release-profile --only <all 10k-tagged steps>
```

Corrected storage reruns after `f7cbc7711`:

```text
target/release/ecaz --database tqvector_bench_task144 --host /home/peter/dev/ecaz/target/task144-pg18-socket --port 28818 --log-file reviews/task-144/009-release-matrix-10k-r2/artifacts/storage-10k-<variant>.log bench storage --prefix t144_10k_<variant>
```

Validation:

```text
cargo test -p ecaz-cli storage
```

## Artifact Index

- `suite-manifest-10k-r2.json`: release suite manifest for the 10k selected steps.
- `results-10k-r2.jsonl`: suite result rows, including `spire_pipeline_row_scan` rows.
- `suite-run-10k-r2.log`: full 10k suite stdout/stderr.
- `precheck-before-suite.log`: explicit release-profile precheck.
- `cargo-test-ecaz-cli-storage-r2.log`: focused validation for corrected replica denominator.
- `load-10k-*.log`: load/build logs for single, fixed_b2, and closure epsilon variants.
- `storage-10k-*.log`: corrected storage and replication summaries.
- `pipeline-10k-*.log`: pipeline logs for all 30 10k cells.
- `stage-containment-10k-*.jsonl`: per-query containment/probe-tail evidence for each cell.
- `result-identity-10k-*.jsonl`: per-query result identity evidence for each cell.

No corpus `.tsv`, raw truth cache JSON, PostgreSQL server logs, SSM state, or polling exhaust are intended review artifacts.

## Key Results

AC candidate rows, using `distinct_recall@10 >= 0.99` and `candidate_row_instances_percent <= 5`:

```text
cell                         nprobe  recall  candidate%  ready%  production_p50  recall_p50
single-adaptive              32      0.9925  3.70        3.70    7.720 ms        21.728 ms
closure_e010_b8-fixed        32      0.9905  3.79        3.69    8.122 ms        21.149 ms
closure_e010_b8-adaptive     32      0.9930  3.86        3.76    7.767 ms        20.952 ms
closure_e010_b8-ratio800     32      0.9905  3.71        3.60    8.740 ms        22.025 ms
closure_e025_b8-fixed        32      0.9910  4.24        3.82    8.337 ms        22.347 ms
closure_e025_b8-adaptive     32      0.9935  4.36        3.95    7.332 ms        20.881 ms
closure_e025_b8-ratio800     32      0.9910  4.14        3.73    7.142 ms        20.397 ms
closure_e050_b8-fixed        16      0.9915  2.96        2.25    7.833 ms        16.286 ms
closure_e050_b8-ratio400     16      0.9900  2.57        1.94    7.670 ms        15.350 ms
closure_e050_b8-ratio400     32      0.9905  4.45        3.30    7.789 ms        21.482 ms
closure_e050_b8-ratio800     16      0.9915  2.91        2.21    7.725 ms        15.745 ms
```

Ratio sweep readout at nprobe 96:

```text
cell                         recall  candidate%  ready%
single-ratio125              0.7635  0.26        0.26
single-ratio200              0.9575  1.78        1.78
single-ratio400              0.9895  6.95        6.95
single-ratio800              0.9935  10.09       10.09
closure_e010_b8-ratio125     0.7680  0.27        0.27
closure_e010_b8-ratio200     0.9585  1.87        1.81
closure_e010_b8-ratio400     0.9900  7.27        7.03
closure_e010_b8-ratio800     0.9940  10.53       10.19
closure_e025_b8-ratio400     0.9915  8.18        7.22
closure_e050_b8-ratio400     0.9925  10.86       7.71
```

Corrected storage / replication rows:

```text
variant              index_size  mean_replicas_per_vector
single               17.9 MiB    1.0000
fixed_b2             34.9 MiB    3.0000
closure_e010_b8      18.5 MiB    1.0549
closure_e025_b8      20.4 MiB    1.2593
closure_e050_b8      26.0 MiB    1.9064
```

Interpretation for 10k only: ratio 1.25 remains a hard recall failure. Ratio 2.0 remains below 0.99. Ratio 4.0/8.0 can recover recall, but most nprobe-96 rows exceed the 5% candidate scan AC. The clearest 10k AC points are closure epsilon 0.10/0.25 at nprobe 32 and closure epsilon 0.50 at nprobe 16 or ratio 4.0 nprobe 16. This packet does not close Task 144; 50k/100k remain required.
