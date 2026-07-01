# Review Request: Task 123 Multi-Instance Phase A Baseline

## Scope

This packet addresses the reopened Task 121/123 reviewer feedback in
`reviews/task-123/008-completion-record/feedback/2026-06-27-03-reviewer.md`.
It re-runs the requested 100k Phase A efficiency baseline on the contained
local multi-instance lane: one coordinator PG18 instance plus three worker PG18
instances on the same host.

No AWS or cross-network measurement was used. This is intentionally the
same-machine multi-instance substrate: real distributed executor path, with
network RTT removed.

## Configs

- `n128 b4/tr50/f8`: `nlists=128`, `boundary_replica_count=4`,
  `training_sample_rows=50000`, `recursive_fanout=8`,
  `top_graph_search_list_size=96`, nprobe 8 and 96.
- `n1024 b2/tr50/f8`: `nlists=1024`, `boundary_replica_count=2`,
  `training_sample_rows=50000`, `recursive_fanout=8`,
  `top_graph_search_list_size=96`, nprobe 8 and 64.
- Fixture: staged `ec_real_100k`, 32 queries, `top_k=10`, `rabitq`.

## Results

| Config | nprobe | Recall@10 | p50 | p95 | Coordinator index |
| --- | ---: | ---: | ---: | ---: | ---: |
| n128 b4/tr50/f8 | 8 | 0.9781 | 69.620 ms | 78.007 ms | 392.2 MiB |
| n128 b4/tr50/f8 | 96 | 1.0000 | 337.096 ms | 479.785 ms | 392.2 MiB |
| n1024 b2/tr50/f8 | 8 | 0.9406 | 75.196 ms | 85.457 ms | 246.1 MiB |
| n1024 b2/tr50/f8 | 64 | 1.0000 | 87.323 ms | 90.365 ms | 246.1 MiB |

Production-read profile confirms the distributed read path:

| Config | nprobe | Result source | Remote pids | Dispatches | Remote heap candidates | Candidate p50/p95 | Heap p50/p95 | Total p50/p95 |
| --- | ---: | --- | ---: | ---: | ---: | --- | --- | --- |
| n128 b4/tr50/f8 | 8 | remote_heap_candidates | 256 | 95 | 950 | 34/41 ms | 34/39 ms | 62/74 ms |
| n128 b4/tr50/f8 | 96 | remote_heap_candidates | 3072 | 96 | 960 | 386/545 ms | 400/539 ms | 339/437 ms |
| n1024 b2/tr50/f8 | 8 | remote_heap_candidates | 256 | 93 | 930 | 6/7 ms | 6/7 ms | 52/54 ms |
| n1024 b2/tr50/f8 | 64 | remote_heap_candidates | 2048 | 96 | 960 | 20/24 ms | 20/24 ms | 63/68 ms |

These profile timing buckets are aggregate profile percentiles, not additive
sub-stages; they should be read as attribution signals, not summed into the
coordinator query p50.

## Read

This changes the cost picture materially from the single-instance no-go:

- The local multi-instance production-read lane is not showing the multi-second
  local scan wall from the single-instance path.
- `n1024 b2` reaches recall 1.0000 at nprobe 64 with p50 87.323 ms / p95
  90.365 ms on the contained distributed path, versus `n128 b4` needing
  nprobe 96 and p50 337.096 ms / p95 479.785 ms.
- The dominant measured stage is the remote candidate/heap phase, and it scales
  with selected remote pids: `n128 b4` at nprobe 96 selects 3072 remote pids and
  reports candidate/heap p50 386/400 ms; `n1024 b2` at nprobe 64 selects 2048
  remote pids and reports candidate/heap p50 20/20 ms.
- The finer `n1024 b2` cell is both faster at recall 1.0 and smaller on the
  coordinator index than `n128 b4`: 246.1 MiB vs 392.2 MiB.

## Instrumentation Gap

This packet does not fully satisfy the requested stage taxonomy. The existing
local multi-instance nested suite reports candidate/heap/endpoint/total timing,
remote dispatch/candidate counts, and projected payload rows/bytes. It does not
report:

- per-worker object bytes shipped;
- separate leaf-read timing;
- separate materialize+transport-encode timing;
- separate candidate-score timing apart from the profile's `candidate_*`
  bucket.

The `payload_bytes_sum` column is `0` because the nested suite currently sets
`query_metric_projection_columns=["id"]`; this is not an object-byte
measurement. Follow-up instrumentation should widen the production-read profile
or nested suite to expose per-worker object bytes and the requested stage split.

## Evidence

See `artifacts/manifest.md` and `artifacts/extracted-results.md`.

Primary artifacts:

- `artifacts/n128-b4-r2/bench-suite/results.jsonl`
- `artifacts/n128-b4-r2/bench-suite/production-read-k10-default.log`
- `artifacts/n1024-b2-r3/bench-suite/results.jsonl`
- `artifacts/n1024-b2-r3/bench-suite/production-read-k10-default.log`

The first `n1024` attempt failed because generated runtime data filled the
filesystem. Generated shard TSVs and local PG runtime directories were pruned,
then the focused `n1024` rerun completed. No TSV corpus/shard data is included
in this packet commit.
