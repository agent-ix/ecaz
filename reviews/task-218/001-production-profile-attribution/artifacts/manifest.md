# Task 218 P1 packet manifest

- head SHA: `15834e2e4` for the lane implementation; updated with the evidence
  commit that adds this packet
- task bucket: `reviews/task-218/001-production-profile-attribution/`
- lane: ec_distann owner-side materialization attribution
- fixture: three-owner physical PG18, `ec_real_100k`, 200 held-out queries,
  top-k 10
- storage format: rabitq physical generation; sharded owner control; no
  traversal replica
- rerank mode: production lazy-10 (`materialization_batch_size=10`)
- shared surface: one physical generation, one production-lazy10 runtime arm
- SuiteConfig: `task218-lazy10-attribution.json`
- runner: `ecaz bench suite run`
- planned command:
  `ecaz bench suite run --config reviews/task-218/001-production-profile-attribution/artifacts/task218-lazy10-attribution.json --results-output reviews/task-218/001-production-profile-attribution/artifacts/run/results.jsonl`
- timestamp: 2026-08-08; final benchmark timestamp and cited stage/work lines
  are added when the PG18 run completes

The P1 decision must be based on the packet-local `results.jsonl` rows for
`physical_benchmark_stage` and `physical_benchmark_materialization_work`,
especially owner payload SQL, endpoint work, locator formatting, payload
counts, and executor rows. No eager `batch_size=0` result is admissible as the
P1 denominator.
