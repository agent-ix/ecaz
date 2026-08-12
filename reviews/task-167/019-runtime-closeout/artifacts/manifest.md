# Task 167 runtime closeout artifacts

- head SHA: `b1b989016`
- task bucket: `reviews/task-167/`
- packet: `019-runtime-closeout`
- suite config: `task167-physical-suite.json`
- runner: `ecaz bench suite run`
- run timestamp: 2026-08-11 (America/Los_Angeles)
- fixture: three-node local PG18 `distann-multicluster`, one physical index per table, shared-table control arm
- storage format: `rabitq`
- rerank mode: physical owner-routed exact rerank; control arm is local single-index
- scales: 10k, 50k, 100k
- command: `/home/peter/.cargo-target/debug/ecaz bench suite run --config reviews/task-167/016-physical-benchmark-suite/artifacts/task167-physical-suite.json --artifact-dir reviews/task-167/016-physical-benchmark-suite/artifacts/run29 --manifest-output reviews/task-167/016-physical-benchmark-suite/artifacts/run29/suite-manifest.json --results-output reviews/task-167/016-physical-benchmark-suite/artifacts/run29/results.jsonl --log-file reviews/task-167/016-physical-benchmark-suite/artifacts/run29/suite.log`
- corpus provenance: staged real `ec_real_{10k,50k,100k}`; corpus TSVs are not committed
- durable sources: `suite-manifest.json`, `results.jsonl`, and `cited-results.log`

The cited result lines include recall, latency at concurrency 1 and 4, storage, insert-throughput A/B, insert-work counters, fresh-rebuild parity, TC-043 concurrent insert/query, and the physical topology gate. `results.jsonl` is the structured suite source of truth; `cited-results.log` is the compact human-readable extract.
