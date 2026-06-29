# Task 123 Packet 017 Artifact Manifest

- Head SHA: `7896f77b13d31a0563ebdf3579c75514522f313d`
- Task bucket: `reviews/task-123/017-multinode-communications-prune-ab`
- Timestamp: `2026-06-29T09:41:52-07:00`
- Host/worktree: `/tmp/ecaz-task123`
- Drive during closeout: `/dev/sdf` mounted at `/tmp`, `1007G` size, `909G` used, `48G` available after packet TSV cleanup.
- Scope: local PG18 multi-instance SPIRE production-read measurement. This exercises the core routing/materialization/remote heap algorithm and local socket communication path. It is not a true cross-network measurement.
- Corpus: staged real corpus `ec_real_100k` from `/home/peter/dev/ecaz/data/staged-current`; generated corpus/query TSVs were not committed.
- Storage format: `rabitq`
- Surface isolation: one local multinode fixture per surface, with one coordinator and three remotes.
- Rerank mode: production-read-only, `top_k=10`, recall enabled against staged truth corpus.
- Shared session GUC: `ec_spire.max_remote_payload_bytes_per_row=16384`.
- Variant axis:
  - `source-prune-on`: projection `id,source`, `ec_spire.pre_materialization_prune=on`
  - `source-prune-off`: projection `id,source`, `ec_spire.pre_materialization_prune=off`
  - `id-prune-on`: projection `id`, `ec_spire.pre_materialization_prune=on`
  - `id-prune-off`: projection `id`, `ec_spire.pre_materialization_prune=off`
  - Each variant also ran default and `--max-routed-candidate-rows 25000` rowcap forms.

## Commands

Dry run:

```sh
target/debug/ecaz bench suite run \
  --config reviews/task-123/017-multinode-communications-prune-ab/artifacts/task123-mi-communications-prune-ab-suite.json \
  --manifest-output reviews/task-123/017-multinode-communications-prune-ab/artifacts/dryrun-manifest.json \
  --results-output reviews/task-123/017-multinode-communications-prune-ab/artifacts/dryrun-results.jsonl \
  --log-file reviews/task-123/017-multinode-communications-prune-ab/artifacts/dryrun-suite.log \
  --dry-run
```

Measurement:

```sh
target/debug/ecaz bench suite run \
  --config reviews/task-123/017-multinode-communications-prune-ab/artifacts/task123-mi-communications-prune-ab-suite.json \
  --manifest-output reviews/task-123/017-multinode-communications-prune-ab/artifacts/suite-manifest.json \
  --results-output reviews/task-123/017-multinode-communications-prune-ab/artifacts/results.jsonl \
  --log-file reviews/task-123/017-multinode-communications-prune-ab/artifacts/suite-run.log \
  --continue-on-error
```

## Source Artifacts

- Suite config: `task123-mi-communications-prune-ab-suite.json`
- Top-level suite manifest/log: `suite-manifest.json`, `suite-run.log`
- Dry run manifest/log: `dryrun-manifest.json`, `dryrun-suite.log`; the dry run did not emit a `dryrun-results.jsonl` file.
- Note: top-level `results.jsonl` is empty because the local multinode harness writes production-read suite results inside each fixture artifact directory.
- n128 fixture:
  - Harness log: `n128-b4-200q/local-multinode.log`
  - Nested suite config: `n128-b4-200q/bench-suite/local-real-production-read-suite.json`
  - Nested suite manifest/results/log: `n128-b4-200q/bench-suite/suite-manifest.json`, `n128-b4-200q/bench-suite/results.jsonl`, `n128-b4-200q/bench-suite/suite-run.log`
  - Production-read logs: `n128-b4-200q/bench-suite/production-read-k10-*.log`
  - Storage log: `n128-b4-200q/bench-suite/storage.log`
- n1024 fixture:
  - Harness log: `n1024-b2-200q/local-multinode.log`
  - Nested suite config: `n1024-b2-200q/bench-suite/local-real-production-read-suite.json`
  - Nested suite manifest/results/log: `n1024-b2-200q/bench-suite/suite-manifest.json`, `n1024-b2-200q/bench-suite/results.jsonl`, `n1024-b2-200q/bench-suite/suite-run.log`
  - Production-read logs: `n1024-b2-200q/bench-suite/production-read-k10-*.log`
  - Storage log: `n1024-b2-200q/bench-suite/storage.log`

Both fixtures ended with:

```text
SPIRE local multinode fixture passed
HARNESS PASSED
```

## n128 / b4 / nprobe 96 / 200 Queries

Storage summary from `n128-b4-200q/bench-suite/results.jsonl`:

| field | value | value_bytes |
| --- | ---: | ---: |
| rows | 100000 | 100000 |
| table (heap + toast + fsm/vm) | 1.6 GiB | 1717986918 |
| total | 1.9 GiB | 2040109466 |

Latency and recall from `n128-b4-200q/bench-suite/results.jsonl`:

| step | p50 | p95 | p99 | max | recall@k |
| --- | ---: | ---: | ---: | ---: | ---: |
| source-prune-on-default | 5524.614 ms | 5933.579 ms | 6066.021 ms | 6149.005 ms | 1.0000 |
| source-prune-on-rowcap25k | 5520.691 ms | 5946.800 ms | 6141.474 ms | 6252.829 ms | 1.0000 |
| source-prune-off-default | 5508.976 ms | 5902.777 ms | 6103.653 ms | 6142.842 ms | 1.0000 |
| source-prune-off-rowcap25k | 5518.986 ms | 5958.717 ms | 6035.074 ms | 6089.484 ms | 1.0000 |
| id-prune-on-default | 5472.119 ms | 5940.613 ms | 6048.305 ms | 6079.655 ms | 1.0000 |
| id-prune-on-rowcap25k | 5513.885 ms | 5953.022 ms | 6438.731 ms | 7422.320 ms | 1.0000 |
| id-prune-off-default | 5690.467 ms | 7266.924 ms | 7988.415 ms | 8640.454 ms | 1.0000 |
| id-prune-off-rowcap25k | 5605.199 ms | 6398.346 ms | 6665.309 ms | 7330.017 ms | 1.0000 |

Profile counters, all variants:

- `remote_heap_ready_dispatch_sum=600`
- `remote_heap_failed_dispatch_sum=0`
- `remote_heap_candidate_sum=6000`
- `payload_rows_sum=6000`
- `returned_sum=2000`

Per-node heap payload from `n128-b4-200q/bench-suite/results.jsonl`:

- `source` projection: `24632000` bytes per remote node, `73896000` bytes total across three remotes for 200 queries.
- `id` projection: `16000` bytes per remote node, `48000` bytes total across three remotes for 200 queries.

## n1024 / b2 / nprobe 64 / 200 Queries

Storage summary from `n1024-b2-200q/bench-suite/results.jsonl`:

| field | value | value_bytes |
| --- | ---: | ---: |
| rows | 100000 | 100000 |
| table (heap + toast + fsm/vm) | 1.6 GiB | 1717986918 |
| total | 1.8 GiB | 1932735283 |

Latency and recall from `n1024-b2-200q/bench-suite/results.jsonl`:

| step | p50 | p95 | p99 | max | recall@k |
| --- | ---: | ---: | ---: | ---: | ---: |
| source-prune-on-default | 851.406 ms | 1002.419 ms | 1115.553 ms | 1319.433 ms | 1.0000 |
| source-prune-on-rowcap25k | 851.568 ms | 1040.857 ms | 1118.107 ms | 1138.855 ms | 1.0000 |
| source-prune-off-default | 836.644 ms | 963.494 ms | 1048.749 ms | 1065.336 ms | 1.0000 |
| source-prune-off-rowcap25k | 842.157 ms | 1028.240 ms | 1182.437 ms | 1205.402 ms | 1.0000 |
| id-prune-on-default | 803.108 ms | 973.497 ms | 1063.741 ms | 1116.989 ms | 1.0000 |
| id-prune-on-rowcap25k | 793.343 ms | 1088.522 ms | 1158.382 ms | 1195.032 ms | 1.0000 |
| id-prune-off-default | 796.862 ms | 914.903 ms | 1006.874 ms | 1082.376 ms | 1.0000 |
| id-prune-off-rowcap25k | 785.917 ms | 876.236 ms | 911.363 ms | 997.308 ms | 1.0000 |

Profile counters, all variants:

- `remote_heap_ready_dispatch_sum=600`
- `remote_heap_failed_dispatch_sum=0`
- `remote_heap_candidate_sum=6000`
- `payload_rows_sum=6000`
- `returned_sum=2000`

Per-node heap payload from `n1024-b2-200q/bench-suite/results.jsonl`:

- `source` projection: `24632000` bytes per remote node, `73896000` bytes total across three remotes for 200 queries.
- `id` projection: `16000` bytes per remote node, `48000` bytes total across three remotes for 200 queries.

## Interpretation

- The earlier 32-query claim that multi-instance reversed the latency no-go is not supported by this 200-query evidence.
- The prior projection-column failure is resolved for this local multi-instance path: both `id,source` and `id` projections complete with `remote_heap_failed_dispatch_sum=0`.
- The `id` projection greatly reduces measured heap payload bytes, from `73896000` total bytes to `48000` total bytes over 200 queries.
- Per reviewer feedback now committed at `reviews/task-123/017-multinode-communications-prune-ab/feedback/2026-06-28-01-reviewer.md`, the current prune guard is structurally inert for these b2/b4 configs because `boundary_replica_count > 0` selects `VecIdDedupeEnabled`. The on/off rows below are therefore a no-op confirmation, not a meaningful prune efficacy A/B:
  - n128 source default: prune on `5524.614 ms` p50 vs prune off `5508.976 ms`.
  - n1024 source default: prune on `851.406 ms` p50 vs prune off `836.644 ms`.
  - n1024 id default: prune on `803.108 ms` p50 vs prune off `796.862 ms`.
- n1024/b2 is substantially faster than n128/b4 in this local multi-instance core path, but still far above the earlier 32-query optimistic numbers.
- The communications/projection result is the usable signal from this packet: shrinking remote heap payload from `73896000` bytes to `48000` bytes did not produce a matching latency collapse, so transport payload bytes are unlikely to be the dominant local core-path cost at this scale.
