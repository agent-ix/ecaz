# Task 131 Packet 022 Artifact Manifest

- head SHA: `a7b847694854b7748b2c035f92246229f4e88684`
- task bucket: `reviews/task-131/022-real-scale-threshold-boundability`
- timestamp: 2026-07-01 PDT
- lane: local PG18 multi-instance, real staged corpora
- purpose: candidate-derived threshold boundability check for Phase 3 streaming top-k decision
- storage format: `rabitq`
- rerank mode: production read, `timeline_payload=none`, `ec_spire.remote_search_global_pre_heap_merge=off`
- sample: 20 queries per completed cell; this is a go/no-go boundability profile, not a task closeout latency matrix
- isolation: local multinode harness with one coordinator plus three remote PostgreSQL instances per cell

## Command

```sh
target/debug/ecaz bench suite run \
  --config reviews/task-131/022-real-scale-threshold-boundability/artifacts/task131-phase3-real-scale-boundability-suite.json \
  --manifest-output reviews/task-131/022-real-scale-threshold-boundability/artifacts/suite-manifest.json \
  --results-output reviews/task-131/022-real-scale-threshold-boundability/artifacts/results.jsonl \
  --log-file reviews/task-131/022-real-scale-threshold-boundability/artifacts/suite-run.log
```

The suite failed before completing all six cells because the workspace filesystem reached 100% usage during `100k-n128-b4` remote node 4 encoding:

- `df -h .` after failure: `/dev/sdd 1007G 955G 1.2G 100% /home/peter/dev/ecaz`
- `artifacts/100k-n128-b4/remote-load-node-4.log`: `ERROR: could not extend file "base/5/17625": No space left on device`
- `artifacts/100k-n128-b4/coord-postgres.log`: `could not write temporary statistics file "pg_stat/pgstat.tmp": No space left on device`

No benchmark or PG processes remained after failure.

## Suite Config

- `artifacts/task131-phase3-real-scale-boundability-suite.json`
- top-level manifest: `artifacts/suite-manifest.json`
- top-level run log: `artifacts/suite-run.log`

Configured cells:

- `10k-n128-b4`: `ec_real_10k`, `nlists=128`, `boundary_replica_count=4`, `nprobe=96`
- `10k-n1024-b2`: `ec_real_10k`, `nlists=1024`, `boundary_replica_count=2`, `nprobe=64`
- `50k-n128-b4`: `ec_real_50k`, `nlists=128`, `boundary_replica_count=4`, `nprobe=96`
- `50k-n1024-b2`: `ec_real_50k`, `nlists=1024`, `boundary_replica_count=2`, `nprobe=64`
- `100k-n128-b4`: `ec_real_100k`, `nlists=128`, `boundary_replica_count=4`, `nprobe=96` (failed before production-read result)
- `100k-n1024-b2`: not reached because suite stopped on the prior failure

## Completed Result Files

- `artifacts/10k-n128-b4/bench-suite/results.jsonl`
- `artifacts/10k-n1024-b2/bench-suite/results.jsonl`
- `artifacts/50k-n128-b4/bench-suite/results.jsonl`
- `artifacts/50k-n1024-b2/bench-suite/results.jsonl`
- compact summary: `artifacts/threshold-boundability-summary.md`

The local harness logs for those four cells report `HARNESS PASSED`.

## Key Boundability Result

Every completed 10k/50k real-scale cell reported zero usable sound bounds and zero threshold-skip opportunities:

- `sound_bound_available_sum = 0` for every node in every completed cell
- `threshold_block_available_sum = 0` and `threshold_block_skipped_sum = 0` for every node
- `threshold_row_available_sum = 0` and `threshold_row_skipped_sum = 0` for every node

Completed recall rows:

- `10k-n128-b4`: recall@10 `1.0000`, p50 `613.826 ms`, p95 `673.385 ms`
- `10k-n1024-b2`: recall@10 `0.9750`, p50 `528.689 ms`, p95 `672.335 ms`
- `50k-n128-b4`: recall@10 `1.0000`, p50 `2703.202 ms`, p95 `3371.223 ms`
- `50k-n1024-b2`: recall@10 `1.0000`, p50 `644.494 ms`, p95 `833.718 ms`

## Corpus Provenance

The suite used existing staged corpora:

- `data/task106_intel_dbpedia_staged/ec_real_10k_manifest.json`
- `data/task111a_real50k/ec_real_50k_manifest.json`
- `data/task106_full_sweep_100k/ec_real_100k_manifest.json`

Generated per-shard corpus TSVs under each `distributed-correctness/node-*` directory are ignored and intentionally not committed.
