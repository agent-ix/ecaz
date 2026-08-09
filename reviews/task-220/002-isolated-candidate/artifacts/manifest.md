# Task 220 packet 002 manifest

- head SHA: `b043d06be04d628cd1f6a723f2d35f2d4c66342`
- task bucket: `reviews/task-220/`
- packet: `002-isolated-candidate/`
- source preregistration: `reviews/task-220/001-preregistration-and-screen/`
- lane: ec_distann owner-side payload materialization
- fixture: three-owner physical PG18, `ec_real_100k`, 200 held-out queries,
  top-k 10; 50 warm latency iterations plus 10 warmups
- storage format: rabitq physical generation; sharded owner control; no
  traversal replica
- rerank mode: production lazy-10 (`materialization_batch_size=10`)
- shared surface: one physical generation and one query surface across both
  arms; generation identity is recorded in `artifacts/correctness.md`
- arm delta: control `packed_payload=false`; candidate
  `packed_payload=true`; all other seed/search/materialization settings equal
- runner: `ecaz bench suite`
- command: `ecaz bench suite run --config reviews/task-220/001-preregistration-and-screen/artifacts/task220-mat16-100k.json --artifact-dir reviews/task-220/001-preregistration-and-screen/artifacts/run --manifest-output reviews/task-220/001-preregistration-and-screen/artifacts/run/suite-manifest.json --results-output reviews/task-220/001-preregistration-and-screen/artifacts/run/results.jsonl --continue-on-error`
- extension provenance: release profile, benchmark feature, SHA
  `b043d06be04d628cd1f6a723f2d35f2d4c66342`, unanimous across three nodes
- structured source: `artifacts/run/results.jsonl`
- cited correctness source: `artifacts/correctness.md`
- decision source: `artifacts/decision.md`
- decision: STOP; no packet 003 release matrix
- run captured: 2026-08-09

No corpus, cluster directory, polling snapshots, or PostgreSQL operational
logs are committed in this packet.
