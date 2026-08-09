# Task 217 packet manifest

- head SHA: `15834e2e4` for the implementation checkpoint; updated with the
  evidence commit that adds the request packet
- task bucket: `reviews/task-217/002-lane-implementation/`
- lane: ec_distann same-generation A/A plus runtime-switch A/B
- fixture: three-owner physical PG18, `ec_real_100k`, 200 held-out queries,
  top-k 10
- storage format: rabitq physical generation; no traversal replica
- rerank mode: production owner path; A/B changes only read-time neighbor
  scoring
- shared surface: one physical generation and one active epoch are shared by
  all named physical variants in the step; no per-arm rebuild
- SuiteConfig: `task217-same-generation.json`
- runner: `ecaz bench suite run`
- planned command:
  `ecaz bench suite run --config reviews/task-217/002-lane-implementation/artifacts/task217-same-generation.json --results-output reviews/task-217/002-lane-implementation/artifacts/run/results.jsonl`
- timestamp: 2026-08-08; final benchmark timestamp and result lines are added
  when the PG18 run completes

The packet must cite `physical_benchmark_generation` rows for all physical
arms, one shared `generation_identity`, the A/A
`physical_benchmark_same_generation_recall byte_identical=true` row, and the
NFR-021 conformance rows in `results.jsonl` before review closure.
