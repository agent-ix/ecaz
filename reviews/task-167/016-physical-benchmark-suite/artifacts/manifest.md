# Task 167 packet 016 manifest

- head SHA: `5b9c1ea8f`
- task bucket: `reviews/task-167/`
- packet: `reviews/task-167/016-physical-benchmark-suite/`
- lane: PG18 physical-generation benchmark suite
- fixture: three sequential `distann-local-multinode` physical steps
- scales: 10k, 50k, 100k
- storage format: Task 179 physical generation format
- rerank mode: co-located row-tier exact rerank
- command: `jq empty reviews/task-167/016-physical-benchmark-suite/artifacts/task167-physical-suite.json`
- timestamp: `2026-08-11` America/Los_Angeles
- shared-table/isolated-table surface: each step builds an isolated cluster;
  the throughput A/B uses its physical and local control tables within that step

Required runtime result files are intentionally absent until a benchmark host
executes the suite. The packet does not claim AC-4, AC-7, or benchmark closeout.
