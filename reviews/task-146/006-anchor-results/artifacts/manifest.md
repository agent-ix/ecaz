# Task 146 Packet 006 Artifact Manifest

- Head SHA: `f18aac406176e31dc2b384d50637d6fe1118ba4e`
- Task bucket: `reviews/task-146/006-anchor-results/`
- Packet type: release anchor measurement evidence for Task 146
- Host/lane: local Intel PG18, single coordinator over TCP `127.0.0.1:28818`
- Database: `tqvector_bench_task146`
- Runner: `target/release/ecaz bench suite`
- Suite config: `artifacts/suite-task146-release-anchors.json`
- Suite manifest: `artifacts/suite-manifest.json`
- Results: `artifacts/results.jsonl`
- Derived summaries:
  - `artifacts/anchor-recall-latency.txt`
  - `artifacts/anchor-storage-index.txt`
- Isolated surfaces: yes. Each anchor uses its own prefix/table/index:
  `t146_anchor_{10k,50k,100k}_{ivf,hnsw}`.

## Backend Provenance

The authoritative run is `suite-run-r4.log`; earlier `suite-run*.log` attempts
were setup/preflight failures before benchmark execution completed.

`suite-manifest.json` records one configured backend node:

| node | database | host | port | build_profile | library |
| --- | --- | --- | --- | --- | --- |
| coordinator | `tqvector_bench_task146` | `127.0.0.1` | `28818` | `release` | `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so` |

Library SHA256:
`b261b873f3db494f7c56a3894cda5b4344f078447c1ccce6bff7530fb013d27a`.

Because this anchor suite used a TCP host, socket-lock discovery did not attach
extra local nodes. This is expected for this single-node anchor suite; it is not
multinode SPIRE evidence.

## Commands

Audit:

```bash
target/release/ecaz bench suite audit \
  --config reviews/task-146/006-anchor-results/artifacts/suite-task146-release-anchors.json \
  --artifact-dir reviews/task-146/006-anchor-results/artifacts/top-suite \
  --log-file reviews/task-146/006-anchor-results/artifacts/audit.log
```

Successful run:

```bash
target/release/ecaz bench suite run \
  --config reviews/task-146/006-anchor-results/artifacts/suite-task146-release-anchors.json \
  --artifact-dir reviews/task-146/006-anchor-results/artifacts/top-suite \
  --manifest-output reviews/task-146/006-anchor-results/artifacts/suite-manifest.json \
  --results-output reviews/task-146/006-anchor-results/artifacts/results.jsonl \
  --database tqvector_bench_task146 \
  --host 127.0.0.1 \
  --port 28818 \
  --log-file reviews/task-146/006-anchor-results/artifacts/suite-run-r4.log
```

Status:

```text
[suite:task146-release-anchors] completed=24 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Anchor Results

These are release anchor controls only. They do not measure SPIRE shapes and do
not support a Task 146 promote/do-not-promote verdict by themselves.

### IVF

| scale | index reloptions | sweep | distinct recall@10 | p50 | p95 | index size | per row |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 10k | nlists=32, rerank_width=500 | nprobe=16 | 1.0000 | 17.3 ms | 18.4 ms | 2.3 MiB | 237.6 B |
| 10k | nlists=32, rerank_width=500 | nprobe=24 | 1.0000 | 17.7 ms | 19.0 ms | 2.3 MiB | 237.6 B |
| 10k | nlists=32, rerank_width=500 | nprobe=32 | 1.0000 | 18.3 ms | 20.5 ms | 2.3 MiB | 237.6 B |
| 50k | nlists=64, rerank_width=750 | nprobe=32 | 0.9900 | 30.6 ms | 32.8 ms | 9.7 MiB | 203.8 B |
| 50k | nlists=64, rerank_width=750 | nprobe=48 | 0.9975 | 33.6 ms | 40.6 ms | 9.7 MiB | 203.8 B |
| 50k | nlists=64, rerank_width=750 | nprobe=64 | 1.0000 | 37.2 ms | 43.2 ms | 9.7 MiB | 203.8 B |
| 100k | nlists=128, rerank_width=500 | nprobe=48 | 0.9805 | 27.6 ms | 29.1 ms | 19.4 MiB | 202.9 B |
| 100k | nlists=128, rerank_width=500 | nprobe=64 | 0.9880 | 31.7 ms | 35.7 ms | 19.4 MiB | 202.9 B |
| 100k | nlists=128, rerank_width=500 | nprobe=80 | 0.9950 | 34.9 ms | 37.5 ms | 19.4 MiB | 202.9 B |
| 100k | nlists=128, rerank_width=500 | nprobe=96 | 0.9980 | 37.6 ms | 44.0 ms | 19.4 MiB | 202.9 B |
| 100k | nlists=128, rerank_width=500 | nprobe=128 | 1.0000 | 42.2 ms | 46.6 ms | 19.4 MiB | 202.9 B |

### HNSW

| scale | index reloptions | sweep | recall@10 | p50 | p95 | index size | per row |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 10k | m=16, ef_construction=128 | ef_search=64 | 0.9620 | 3.99 ms | 4.59 ms | 13.0 MiB | 1366.4 B |
| 10k | m=16, ef_construction=128 | ef_search=128 | 0.9935 | 5.28 ms | 6.22 ms | 13.0 MiB | 1366.4 B |
| 10k | m=16, ef_construction=128 | ef_search=200 | 0.9950 | 6.45 ms | 7.56 ms | 13.0 MiB | 1366.4 B |
| 10k | m=16, ef_construction=128 | ef_search=400 | 0.9960 | 9.42 ms | 10.8 ms | 13.0 MiB | 1366.4 B |
| 50k | m=16, ef_construction=128 | ef_search=64 | 0.9375 | 4.53 ms | 5.77 ms | 65.1 MiB | 1365.6 B |
| 50k | m=16, ef_construction=128 | ef_search=128 | 0.9570 | 6.37 ms | 8.31 ms | 65.1 MiB | 1365.6 B |
| 50k | m=16, ef_construction=128 | ef_search=200 | 0.9735 | 7.49 ms | 10.0 ms | 65.1 MiB | 1365.6 B |
| 50k | m=16, ef_construction=128 | ef_search=400 | 0.9815 | 11.5 ms | 14.0 ms | 65.1 MiB | 1365.6 B |
| 100k | m=16, ef_construction=128 | ef_search=64 | 0.8535 | 7.87 ms | 15.2 ms | 130.2 MiB | 1365.4 B |
| 100k | m=16, ef_construction=128 | ef_search=128 | 0.9370 | 11.1 ms | 19.9 ms | 130.2 MiB | 1365.4 B |
| 100k | m=16, ef_construction=128 | ef_search=200 | 0.9630 | 13.4 ms | 22.7 ms | 130.2 MiB | 1365.4 B |
| 100k | m=16, ef_construction=128 | ef_search=400 | 0.9795 | 20.4 ms | 33.5 ms | 130.2 MiB | 1365.4 B |

## Non-Claims

- This packet does not close Task 146.
- This packet does not benchmark SPIRE S1-S6 shapes.
- This packet does not reuse Task 145 packet 008 bound-prune data as evidence;
  that A/B is treated as null/faulty because the mechanism did not engage.
- Final Task 146 conclusions still require the preregistered single-instance
  and multinode SPIRE matrices plus review of the anchor/result packets.
