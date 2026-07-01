---
head_sha: e5ec01c6ecc429c14a178ff46db3bd3e73bb78ff
task: task-123
packet: reviews/task-123/019-dedupe-prune-multinode-ab
date: 2026-06-29
---

# Manifest

## Purpose

Packet 019 measures the Task 123 SPIRE pre-materialization prune fix from
commit `d2ffbdaa9` under local PG18 multi-instance production-read workloads.
The matrix repeats the packet 017 representative 100k corpus shape after
`pre_materialization_min_ip_to_keep()` was changed to engage under bounded
dedupe:

- `n128`, `boundary_replica_count=4`, `nprobe=96`, 200 queries.
- `n1024`, `boundary_replica_count=2`, `nprobe=64`, 200 queries.
- `storage_format=rabitq`.
- PG18 local multi-instance: one coordinator and three local worker instances.
- Production-read variants:
  - `source-prune-on`, projection `id,source`, `ec_spire.pre_materialization_prune=on`
  - `source-prune-off`, projection `id,source`, `ec_spire.pre_materialization_prune=off`
  - `id-prune-on`, projection `id`, `ec_spire.pre_materialization_prune=on`
  - `id-prune-off`, projection `id`, `ec_spire.pre_materialization_prune=off`
  - each with default routing cap and `max_routed_candidate_rows=25000`
- Shared production-read GUC:
  `ec_spire.max_remote_payload_bytes_per_row=16384`.

## Commands

Top-level dry run:

```sh
target/debug/ecaz bench suite run \
  --config reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/task123-dedupe-prune-multinode-ab-suite.json \
  --manifest-output reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/dryrun-manifest.json \
  --results-output reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/dryrun-results.jsonl \
  --log-file reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/dryrun-suite.log \
  --dry-run
```

Top-level suite:

```sh
target/debug/ecaz bench suite run \
  --config reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/task123-dedupe-prune-multinode-ab-suite.json \
  --manifest-output reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/suite-manifest.json \
  --results-output reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/results.jsonl \
  --log-file reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/suite-run.log \
  --continue-on-error
```

The top-level suite completed the n128 matrix and built the n1024 coordinator
and worker indexes. It then failed during n1024 remote materialization because
previous operator cleanup removed `coordinator-base-assignments.tsv` for node 2
and node 3 too early. Recovery used the already-built n1024 PG data directories:

1. Restarted the n1024 coordinator and three workers from
   `target/spire-local-multinode-task123-p19-mi-n1024-b2-200q`.
2. Re-exported node 2 and node 3 coordinator assignment TSVs from the
   coordinator index.
3. Re-ran remote materialization for node 2, node 3, and node 4.
4. Re-collected remote identities, published remote placements, rendered and
   applied remote registration SQL.
5. Ran the generated nested n1024 `ecaz bench suite` directly:

```sh
target/debug/ecaz --database postgres \
  --host /tmp/ecaz-task123/target/spire-local-multinode-sockets-task123-p19-mi-n1024-b2-200q \
  --port 40530 --user ecaz_coord \
  bench suite run \
  --config reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/n1024-b2-200q/bench-suite/local-real-production-read-suite.json \
  --manifest-output reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/n1024-b2-200q/bench-suite/suite-manifest.json \
  --results-output reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/n1024-b2-200q/bench-suite/results.jsonl \
  --log-file reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/n1024-b2-200q/bench-suite/suite-run.log
```

All regenerated TSV scratch files were deleted before this packet was committed.

## Primary Artifacts

- Suite config:
  `artifacts/task123-dedupe-prune-multinode-ab-suite.json`
- Top-level logs:
  `artifacts/suite-run.log`, `artifacts/suite-manifest.json`,
  `artifacts/results.jsonl`
- n128 nested suite:
  `artifacts/n128-b4-200q/bench-suite/local-real-production-read-suite.json`,
  `artifacts/n128-b4-200q/bench-suite/suite-run.log`,
  `artifacts/n128-b4-200q/bench-suite/suite-manifest.json`,
  `artifacts/n128-b4-200q/bench-suite/results.jsonl`
- n1024 recovered nested suite:
  `artifacts/n1024-b2-200q/bench-suite/local-real-production-read-suite.json`,
  `artifacts/n1024-b2-200q/bench-suite/suite-run.log`,
  `artifacts/n1024-b2-200q/bench-suite/suite-manifest.json`,
  `artifacts/n1024-b2-200q/bench-suite/results.jsonl`
- n1024 recovery logs:
  `artifacts/n1024-b2-200q/remote-leaf-materialization/*-recovery.log`,
  `artifacts/n1024-b2-200q/remote-identities/*.json`,
  `artifacts/n1024-b2-200q/publish-remote-placements-recovery.log`,
  `artifacts/n1024-b2-200q/register-remotes-recovery.log`

## Key Results

### n1024 / b2 / nprobe 64 / 200q

| variant | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: |
| source-prune-on-default | 777.223 ms | 854.805 ms | 1.0000 |
| source-prune-on-rowcap25k | 777.627 ms | 845.231 ms | 1.0000 |
| source-prune-off-default | 780.018 ms | 849.558 ms | 1.0000 |
| source-prune-off-rowcap25k | 777.575 ms | 852.266 ms | 1.0000 |
| id-prune-on-default | 731.133 ms | 813.083 ms | 1.0000 |
| id-prune-on-rowcap25k | 730.132 ms | 810.236 ms | 1.0000 |
| id-prune-off-default | 733.572 ms | 815.993 ms | 1.0000 |
| id-prune-off-rowcap25k | 730.743 ms | 817.757 ms | 1.0000 |

Per-node heap payload bytes in n1024 were:

- `id,source` variants: 2,000 payload rows and 24,632,000 bytes per worker.
- `id` variants: 2,000 payload rows and 16,000 bytes per worker.

All n1024 production-read profile rows reported `remote_heap_candidates`,
`remote_heap_ready_dispatch_sum=600`,
`remote_heap_failed_dispatch_sum=0`, `returned_sum=2000`, and no degraded
remote skips.

### n128 / b4 / nprobe 96 / 200q

| variant | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: |
| source-prune-on-default | 5220.889 ms | 5634.401 ms | 1.0000 |
| source-prune-on-rowcap25k | 5220.034 ms | 6103.406 ms | 1.0000 |
| source-prune-off-default | 5213.018 ms | 5618.722 ms | 1.0000 |
| source-prune-off-rowcap25k | 5200.651 ms | 5578.536 ms | 1.0000 |
| id-prune-on-default | 5124.063 ms | 5461.581 ms | 1.0000 |
| id-prune-on-rowcap25k | 5135.072 ms | 5497.886 ms | 1.0000 |
| id-prune-off-default | 5136.791 ms | 5503.452 ms | 1.0000 |
| id-prune-off-rowcap25k | 5163.412 ms | 5560.072 ms | 1.0000 |

Per-node heap payload bytes in n128 were the same shape as n1024:

- `id,source` variants: 2,000 payload rows and 24,632,000 bytes per worker.
- `id` variants: 2,000 payload rows and 16,000 bytes per worker.

## Verdict

The corrected pre-materialization threshold no longer gets suppressed by
bounded dedupe, but this representative local multi-instance A/B does not show
a meaningful prune-on latency win. Prune-on/off and rowcap/default results are
flat at both n1024 and n128. Projection width is measurable in the per-node
communication counters, but the latency gap is modest for n1024 and small
relative to the very high n128 latency.

This packet therefore closes the missing multi-instance measurement gap for the
core algorithm and retracts any earlier implication that enabling the prune gate
alone produced a latency win.
