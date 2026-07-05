# Task 107 packet 003 - AWS benchmark manifest

- Head SHA: `ef916cea7`.
- Task bucket: `reviews/task-107/003-aws-benchmarks/`.
- Date: 2026-06-15.
- Purpose: package Task 107 AWS benchmark evidence gathered so far and make
  the scope limits explicit.

## Scope Boundary

No additional single-node benchmark runs should be started from this packet.
Task 106 already contains single-node SPIRE RaBitQ AWS coverage at 10k, 50k,
100k, and 1m on AWS Intel and Graviton under
`reviews/task-106/004-aws-targeted-bench/`.

The Task 107 topology benchmark runs in this packet are one quantization
setting only: `--bits 4`. They are not a 2/4/8 bit-depth sweep and should not
be cited as quantization sensitivity evidence. Task 106 covers IVF RaBitQ
bit-depth/kernel routing, but not a distributed SPIRE topology bit-depth sweep.

All Task 107 index builds were run as separate lanes, not as one all-index
build. Large generated TSV/parquet/tar/state/key intermediates, truth caches,
and row-id helper files were pruned before commit.

## Completed Distributed Result

### RaBitQ 100k, local_store_count=1, 1 coordinator + 2 remotes

- Artifact directory: `rabitq-100k-l1/`.
- Prefix: `task107_rabitq_100k_l1`.
- Storage format: `rabitq`.
- Bits: `4`.
- Coordinator reloptions: `local_store_count=1`, `storage_format=rabitq`.
- Remote reloptions: `local_store_count=1`, `storage_format=rabitq`.
- Suite config: `rabitq-100k-l1/suite-representative-priority.json`.
- Suite manifest/results:
  - `rabitq-100k-l1/suite-manifest-representative-priority.json`
  - `rabitq-100k-l1/suite-results-representative-priority.jsonl`

Load/build:

- Coordinator: 100000 rows, copied in 32.25s, encoded in 32.68s, built
  `task107_rabitq_100k_l1_idx` in 89.87s, total 168.16s.
- Remote node 2: 52031 rows, copied in 16.80s, encoded in 17.33s, built
  `task107_rabitq_100k_l1_remote_idx` in 36.31s, total 77.08s.
- Remote node 3: 47969 rows, copied in 15.41s, encoded in 11.75s, built
  `task107_rabitq_100k_l1_remote_idx` in 32.27s, total 65.56s.

Recall:

| Step | nprobe | recall | mean q-time |
| --- | ---: | ---: | ---: |
| k10 | 8 | 0.7873 | 80.80 ms |
| k10 | 16 | 0.8626 | 84.19 ms |
| k10 | 24 | 0.8957 | 85.88 ms |
| k10 | 32 | 0.9180 | 88.52 ms |
| k10 | 64 | 0.9560 | 99.27 ms |
| k100 | 8 | 0.6860 | 84.77 ms |
| k100 | 16 | 0.7895 | 87.06 ms |
| k100 | 24 | 0.8358 | 89.99 ms |
| k100 | 32 | 0.8682 | 92.55 ms |
| k100 | 64 | 0.9331 | 101.79 ms |

Latency:

| Step | nprobe | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| k10 c1 | 8 | 81.9 ms | 80.2 ms | 92.5 ms | 99.2 ms |
| k10 c1 | 16 | 84.6 ms | 83.1 ms | 94.4 ms | 102.3 ms |
| k10 c1 | 24 | 87.4 ms | 85.9 ms | 97.8 ms | 107.4 ms |
| k10 c1 | 32 | 89.2 ms | 87.8 ms | 99.8 ms | 105.8 ms |
| k10 c4 | 8 | 83.0 ms | 81.5 ms | 94.5 ms | 105.1 ms |
| k10 c4 | 16 | 86.6 ms | 84.2 ms | 101.8 ms | 121.2 ms |
| k10 c4 | 24 | 89.5 ms | 87.6 ms | 102.9 ms | 117.7 ms |
| k10 c4 | 32 | 92.3 ms | 90.1 ms | 107.0 ms | 130.1 ms |
| k10 c8 | 8 | 124.3 ms | 124.2 ms | 138.7 ms | 159.3 ms |
| k10 c8 | 16 | 123.4 ms | 123.6 ms | 138.4 ms | 152.5 ms |
| k10 c8 | 24 | 122.7 ms | 122.9 ms | 138.0 ms | 154.4 ms |
| k10 c8 | 32 | 122.6 ms | 122.5 ms | 136.4 ms | 148.0 ms |
| k1 c32 | 32 | 490.9 ms | 468.6 ms | 873.2 ms | 962.2 ms |

Production remote read profile:

- `13e3-production-read-profile-k10.log`: status `ready`,
  `result_source=remote_heap_candidates`, `dispatch_sum=2000`,
  total p50/p95 `48.000 ms` / `51.000 ms`, recall@10 `0.9560`.
- `13e3-production-read-profile-k100.log`: status `ready`,
  `result_source=remote_heap_candidates`, `dispatch_sum=2000`,
  total p50/p95 `49.000 ms` / `52.000 ms`, recall@100 `0.9331`.

Cleanup:

- Coordinator and remote objects for this lane were dropped after measurement.
- Disk cleanup logs are under `rabitq-100k-l1/cleanup-*`.

## Partial / Non-Decision Evidence

### RaBitQ 100k, local_store_count=4

- Artifact directory: `rabitq-100k-l4/`.
- This became single-node-only evidence and should not be extended here.
- Distributed placement export failed before remote benchmark:
  `ec_spire placement local_store_id 1 does not match relation object store id 0`
  in
  `rabitq-100k-l4/distributed-representative/node-2/coordinator-base-assignments.stderr.log`.
- Single-node suite files are preserved for audit trail only because Task 106
  already owns single-node coverage.

### TurboQuant 100k, local_store_count=1

- Artifact directory: `turboquant-100k-l1/`.
- Coordinator load completed: 100000 rows, copied in 32.17s, encoded in
  23.78s, built `task107_turboquant_100k_l1_idx` in 89.58s, total 158.93s.
- The completed suite is single-node only:
  `turboquant-100k-l1/suite-results-single-node.jsonl`.
- Distributed benchmark work was not completed and should not be cited as
  TurboQuant distributed evidence.

### RaBitQ 1m, local_store_count=1

- Artifact directory: `rabitq-1m-l1/`.
- Prefix: `task107_rabitq_1m_l1`.
- The run reached coordinator load/build and was cancelled after the user
  clarified not to rerun already-covered single-node work:
  - copied 990000 corpus rows in 318.45s;
  - encoded corpus in 429.44s;
  - copied 10000 queries in 3.35s;
  - cancelled during `CREATE INDEX task107_rabitq_1m_l1_idx`.
- SSM command:
  `a964ff00-f47f-4770-b871-d6ab4e1543e5`, status `Cancelled`,
  response code `137`.
- Cleanup required terminating the stuck backend and restarting coordinator
  PostgreSQL:
  - `cancelled-load-command-invocation.json`
  - `postgres-restart-command-invocation.json`
  - `force-kill-backend-command-invocation.json`
  - `cleanup-drop-after-restart.log`
  - `cleanup-verify-objects.log`
  - `cleanup-df-after-cancel.json`
- Final verification: zero `task107_rabitq_1m_l1%` relations remained.
  Coordinator disk sample: root `27G/400G` used; each 200G store volume
  `1.5G` used.

## Reused Task 106 Evidence

Task 106 packet 004 is the current source for single-node SPIRE RaBitQ AWS
evidence:

- `reviews/task-106/004-aws-targeted-bench/request.md`
- `reviews/task-106/004-aws-targeted-bench/artifacts/manifest.md`
- `reviews/task-106/004-aws-targeted-bench/artifacts/aws-intel/results.jsonl`
- `reviews/task-106/004-aws-targeted-bench/artifacts/aws-graviton/results.jsonl`

At 1m, Task 106 single-node SPIRE RaBitQ recall includes:

- AWS Intel, batch-off: nprobe 32/64/96/128 recall
  `0.9760` / `0.9850` / `0.9870` / `0.9910`.
- AWS Graviton, batch-off: nprobe 32/64/96/128 recall
  `0.9760` / `0.9850` / `0.9870` / `0.9910`.

Those runs are single-node and should not be treated as distributed Task 107
evidence.

## Remaining Gaps

- No Task 107 distributed 1m result is complete.
- No Task 107 TurboQuant distributed result is complete.
- No Task 107 SPIRE topology bit-depth sweep exists; topology measurements in
  this packet are `bits=4` only.
- Any additional benchmark work should be distributed-only, one index lane at a
  time, after an explicit decision that the missing evidence is worth the AWS
  cost.

## AWS Stop State

After cleanup and packaging, the three Task 107 EC2 instances were stopped to
conserve spend:

- `aws-stop/stop-instances.json`
- `aws-stop/describe-stopped-instances.json`

Stopped instances: coordinator `i-0b4386fa5017f1363`, remote node 2
`i-07bcc98c3d5d027ee`, and remote node 3 `i-00c2f2aca9dbdd6bd`.
