# Artifact Manifest

- Head SHA: `fe57bb57d291123873819330d206acea3d2b8a14`
- Task bucket: `reviews/task-123/009-multi-instance-phase-a-baseline/`
- Timestamp: `2026-06-27T20:33:59Z`
- Lane: contained local multi-instance PG18, one coordinator and three worker instances on one host
- Fixture: staged representative 100k corpus, prepared prefix `ec_real_100k`, prepared dir `/home/peter/dev/ecaz/data/staged-current`
- Corpus/query SHA: corpus `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`, queries `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Storage format: `rabitq`
- Rerank mode: production read, `top_k=10`, query limit 32, default `rerank_width=0`
- Isolation: each measured config used its own four-instance local PG run and its own coordinator/remote indexes. The prepared corpus prefix stayed `ec_real_100k` because the staged manifest enforces that prefix.

## Suite Configs

| Artifact | Purpose |
| --- | --- |
| `task123-multi-instance-phase-a-suite.json` | Original two-cell suite config for `n128 b4/tr50/f8` and `n1024 b2/tr50/f8`. |
| `task123-multi-instance-n1024-rerun-suite.json` | Focused rerun config for `n1024 b2/tr50/f8` after disk cleanup. |
| `dryrun-manifest-r2.json` | Dry-run expansion for the corrected two-cell suite. |
| `n1024-rerun-dryrun-manifest.json` | Dry-run expansion for the focused `n1024` rerun. |

## Executed Commands

Corrected two-cell run:

```text
/home/peter/dev/ecaz/target/debug/ecaz bench suite run --config reviews/task-123/009-multi-instance-phase-a-baseline/artifacts/task123-multi-instance-phase-a-suite.json --database postgres --host /tmp --port 28818 --manifest-output reviews/task-123/009-multi-instance-phase-a-baseline/artifacts/suite-manifest-r2.json --results-output reviews/task-123/009-multi-instance-phase-a-baseline/artifacts/results-r2.jsonl --log-file reviews/task-123/009-multi-instance-phase-a-baseline/artifacts/suite-run-r2.log
```

The `n128 b4/tr50/f8` cell completed in that run. The `n1024 b2/tr50/f8` cell failed during remote node 2 encoding because the filesystem filled:

```text
PANIC: could not write to file "pg_wal/xlogtemp.1657600": No space left on device
```

After pruning generated shard TSVs and local PG runtime directories, the focused rerun command completed:

```text
/home/peter/dev/ecaz/target/debug/ecaz bench suite run --config reviews/task-123/009-multi-instance-phase-a-baseline/artifacts/task123-multi-instance-n1024-rerun-suite.json --database postgres --host /tmp --port 28818 --manifest-output reviews/task-123/009-multi-instance-phase-a-baseline/artifacts/n1024-rerun-suite-manifest.json --results-output reviews/task-123/009-multi-instance-phase-a-baseline/artifacts/n1024-rerun-results.jsonl --log-file reviews/task-123/009-multi-instance-phase-a-baseline/artifacts/n1024-rerun-suite-run.log
```

## Primary Result Artifacts

| Artifact | Description |
| --- | --- |
| `extracted-results.md` | Human-readable extracted latency, recall, production-read profile, storage, and fixture-load summary. |
| `n128-b4-r2/bench-suite/results.jsonl` | Nested suite JSONL for completed `n128 b4/tr50/f8`, nprobe 8 and 96, default and rowcap25k production-read steps. |
| `n128-b4-r2/bench-suite/production-read-k10-default.log` | Default production-read log for `n128 b4/tr50/f8`; includes tuple transport identity, query metrics, and production-read profile. |
| `n128-b4-r2/bench-suite/production-read-k10-rowcap25k.log` | Rowcap25k production-read log for `n128 b4/tr50/f8`. |
| `n128-b4-r2/bench-suite/storage.log` | Coordinator storage report for `n128 b4/tr50/f8`. |
| `n1024-b2-r3/bench-suite/results.jsonl` | Nested suite JSONL for completed `n1024 b2/tr50/f8`, nprobe 8 and 64, default and rowcap25k production-read steps. |
| `n1024-b2-r3/bench-suite/production-read-k10-default.log` | Default production-read log for `n1024 b2/tr50/f8`; includes tuple transport identity, query metrics, and production-read profile. |
| `n1024-b2-r3/bench-suite/production-read-k10-rowcap25k.log` | Rowcap25k production-read log for `n1024 b2/tr50/f8`. |
| `n1024-b2-r3/bench-suite/storage.log` | Coordinator storage report for `n1024 b2/tr50/f8`. |
| `n128-b4-r2/{coordinator-load,remote-load-node-2,remote-load-node-3,remote-load-node-4}.log` | Fixture load/index build logs for completed `n128 b4/tr50/f8`. |
| `n1024-b2-r3/{coordinator-load,remote-load-node-2,remote-load-node-3,remote-load-node-4}.log` | Fixture load/index build logs for completed `n1024 b2/tr50/f8`. |
| `n1024-b2-r2/remote-load-node-2.log` | Failed first `n1024` attempt showing the no-space failure before rerun. |

## Key Result Lines

Default production-read latency/recall:

| Config | nprobe | Recall@10 | p50 | p95 |
| --- | ---: | ---: | ---: | ---: |
| n128 b4/tr50/f8 | 8 | 0.9781 | 69.620 ms | 78.007 ms |
| n128 b4/tr50/f8 | 96 | 1.0000 | 337.096 ms | 479.785 ms |
| n1024 b2/tr50/f8 | 8 | 0.9406 | 75.196 ms | 85.457 ms |
| n1024 b2/tr50/f8 | 64 | 1.0000 | 87.323 ms | 90.365 ms |

Production-read profile, default step:

| Config | nprobe | Result source | Remote pids | Dispatches | Remote heap candidates | Candidate p50/p95 | Heap p50/p95 | Total p50/p95 | Payload bytes |
| --- | ---: | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| n128 b4/tr50/f8 | 8 | remote_heap_candidates | 256 | 95 | 950 | 34/41 ms | 34/39 ms | 62/74 ms | 0 |
| n128 b4/tr50/f8 | 96 | remote_heap_candidates | 3072 | 96 | 960 | 386/545 ms | 400/539 ms | 339/437 ms | 0 |
| n1024 b2/tr50/f8 | 8 | remote_heap_candidates | 256 | 93 | 930 | 6/7 ms | 6/7 ms | 52/54 ms | 0 |
| n1024 b2/tr50/f8 | 64 | remote_heap_candidates | 2048 | 96 | 960 | 20/24 ms | 20/24 ms | 63/68 ms | 0 |

The profile timing buckets are aggregate profile percentiles, not additive
sub-stages; they should be read as attribution signals, not summed into the
coordinator query p50.

Coordinator index storage:

| Config | Index size | Per row |
| --- | ---: | ---: |
| n128 b4/tr50/f8 | 392.2 MiB | 4112.6 B |
| n1024 b2/tr50/f8 | 246.1 MiB | 2580.9 B |

## Instrumentation Limits

The local multi-instance production-read profile does not currently report the requested per-worker object bytes shipped. It reports `payload_rows_sum` and `payload_bytes_sum`, but the nested suite projects only `id`, so `payload_bytes_sum=0` is not an object-byte counter. It also exposes candidate/heap/endpoint/total timings, not a full leaf-read / materialize+transport-encode / candidate-score / heap split.

## Artifact Hygiene

Generated shard TSVs and local PG runtime directories were removed after the run. They are regenerable corpus/runtime data and must not be committed. Current packet TSV count after cleanup: 0.
