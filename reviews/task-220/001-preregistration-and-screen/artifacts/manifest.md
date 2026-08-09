# Task 220 packet 001 manifest

- head SHA: `9bc0b05eb`
- task bucket: `reviews/task-220/`
- packet: `001-preregistration-and-screen/`
- lane: ec_distann owner-side payload materialization
- fixture: three-owner physical PG18, `ec_real_100k`, 200 held-out queries,
  top-k 10
- storage format: rabitq physical generation; sharded owner control; no
  traversal replica
- rerank mode: production lazy-10 (`materialization_batch_size=10`)
- shared surface: one physical generation and one query surface across the
  control/candidate pair
- arm delta: control `packed_payload=false`; candidate
  `packed_payload=true`; all other seed/search/materialization settings equal
- SuiteConfig: `artifacts/task220-mat16-100k.json`
- runner: `ecaz bench suite`
- planned command: `ecaz bench suite run --config reviews/task-220/001-preregistration-and-screen/artifacts/task220-mat16-100k.json --artifact-dir reviews/task-220/001-preregistration-and-screen/artifacts/run --manifest-output reviews/task-220/001-preregistration-and-screen/artifacts/run/suite-manifest.json --results-output reviews/task-220/001-preregistration-and-screen/artifacts/run/results.jsonl`
- timestamp: preregistered 2026-08-09 before result inspection
- evidence status: pending isolated 100k screen

No corpus, cluster directory, polling snapshots, or raw operational output is
part of this packet. The eventual `results.jsonl` and cited compact logs will
be the decision evidence.
