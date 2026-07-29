# Task 188 packet 007 artifact manifest

- Head SHA: `0a270a4b3` (code checkpoint `2d9f6099b`)
- Task bucket: `reviews/task-188/007-review-fixes/`
- Lane: local PG18 HNSW latency-worker equivalence
- Fixture: 10k DBpedia, source SHA-256 `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`, 200 queries SHA-256 `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- Storage format: HNSW `m=16`, `ef_construction=128`; rerank mode: not applicable
- Surface: isolated one-index-on-one-table (`task188_review_fixes_hnsw`)
- Backend: PG18 `/home/peter/.pgrx:28818`, release extension SHA `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7`
- Config: `artifacts/task188-review-fixes-equivalence-suite.json`, SHA-256 `811820e620c57ff9ea698912d4c280d694c75c3d7054691dcfcc2d39a5110d4e`

Commands used, with the suite config selected to the latency step:

```text
PGHOST=/home/peter/.pgrx PGPORT=28818 ecaz --database tqvector_bench bench suite run --config reviews/task-188/007-review-fixes/artifacts/task188-review-fixes-equivalence-suite.json --only latency-10k-hnsw --artifact-dir reviews/task-188/007-review-fixes/artifacts/run/equivalence/pre-refactor-1426c838b --manifest-output reviews/task-188/007-review-fixes/artifacts/run/equivalence/pre-refactor-1426c838b/suite-manifest.json --results-output reviews/task-188/007-review-fixes/artifacts/run/equivalence/pre-refactor-1426c838b/results.jsonl
PGHOST=/home/peter/.pgrx PGPORT=28818 ecaz --database tqvector_bench bench suite run --config reviews/task-188/007-review-fixes/artifacts/task188-review-fixes-equivalence-suite.json --only latency-10k-hnsw --artifact-dir reviews/task-188/007-review-fixes/artifacts/run/equivalence/current-r2 --manifest-output reviews/task-188/007-review-fixes/artifacts/run/equivalence/current-r2/suite-manifest.json --results-output reviews/task-188/007-review-fixes/artifacts/run/equivalence/current-r2/results.jsonl
```

Result lines:

- pre-refactor: `ef_search=64 count=30 mean=2.56 ms stddev=0.98 ms min=2.15 ms p50=2.38 ms p95=2.81 ms p99=6.34 ms max=7.74 ms cache_state=post_recall_warm`
- review fix: `ef_search=64 count=30 mean=2.57 ms stddev=1.00 ms min=2.12 ms p50=2.38 ms p95=2.82 ms p99=6.47 ms max=7.89 ms cache_state=post_recall_warm worker_batch_size=0`

Artifacts were generated on 2026-07-27. The suite manifests under `artifacts/run/equivalence/` record the exact runner commit, connection, backend provenance, command, and successful status.

Artifact SHA-256:

- `run/equivalence/current-load/load-10k-hnsw.log`: `213142c0abb20a72779fa4e9f9cf6ea6fec337698e44e2fac708dabfefe923f1`
- `run/equivalence/current-load/suite-manifest.json`: `df4d02b8ac0e12410c34b5f70fa50db708b39f73fb87d4aa34b9a64c0939d150`
- `run/equivalence/current-r2/latency-10k-hnsw.log`: `dee93914e68534326318adc8b2d23b7d86f2366d15189a9995a9cb8e7a42c65f`
- `run/equivalence/current-r2/results.jsonl`: `97eb86c8002598d7535be224e4b7112140ef5c5c7262bcc98d0884949c4ba5b0`
- `run/equivalence/current-r2/suite-manifest.json`: `b60a7c8a7f63deb62b542368b3de21c3fe801fa0e26b00926a51483c1fb90579`
- `run/equivalence/pre-refactor-1426c838b/latency-10k-hnsw.log`: `643b871a17a639d420f230261b532dcb6cd072a1a56ceb41b0dddbd06de27bb1`
- `run/equivalence/pre-refactor-1426c838b/results.jsonl`: `2043e8e5cf9277f6630e0c6335a04db61c9c49c96bf91d163910d40cad981374`
- `run/equivalence/pre-refactor-1426c838b/suite-manifest.json`: `ee0e8ac11c3fbce6240c1124953be1f8d8dbaf3c63e032b63101e1d1e9452db1`
