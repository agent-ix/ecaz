# Task 221 packet 001 manifest

- packet: `reviews/task-221/001-preregistration-and-screen/`
- task bucket: `reviews/task-221/`
- preregistration: `artifacts/task221-mat22-100k.json`
- lane: ec_distann owner-side row materialization
- fixture: three-owner physical PG18, `ec_real_100k`, 200 held-out queries,
  top-k 10; 50 warm latency iterations plus 10 warmups
- storage format: rabitq physical generation; sharded owner control; no
  traversal replica
- rerank mode: production lazy-10 (`materialization_batch_size=10`)
- shared surface: one physical generation and one query surface across both
  arms; same-generation pair is enforced by the suite
- arm delta: control `expanded_locator=false`; candidate
  `expanded_locator=true`; typed locator, packed payload, owner plan cache,
  search, and materialization settings are equal
- runner: `ecaz bench suite`
- planned command: `ecaz bench suite run --config reviews/task-221/001-preregistration-and-screen/artifacts/task221-mat22-100k.json --artifact-dir reviews/task-221/001-preregistration-and-screen/artifacts/run --manifest-output reviews/task-221/001-preregistration-and-screen/artifacts/run/suite-manifest.json --results-output reviews/task-221/001-preregistration-and-screen/artifacts/run/results.jsonl --continue-on-error`
- decision rule: STOP on any recall, prediction/order, storage, NFR-021/022,
  or end-to-end/custom-scan regression or neutral result; only a useful
  isolated win may authorize packet 003's 10k/50k/100k matrix
- code checkpoint: `0b6a4bbbf` (extension); CLI runner checkpoint:
  `d1bd2a3bf`
- run artifacts: completed under the temporary `run-background/100k` directory;
  the suite manifest reported `succeeded`, and the decision-grade structured
  result stream was copied into packet 002 before cleanup
- run timestamp: 2026-08-10; extension SHA
  `5757ed6cb21b87ae5dae693327dcc8dbd72f8c72`; query SHA
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- recovery config: `artifacts/task221-mat22-100k-recovery.json`; setup completed
  during the initial invocation, which exited before measurements, so the
  exact immutable fixture is reused for the suite measurement rerun
- retry config: `artifacts/task221-mat22-100k-retry.json`; the reuse path was
  rejected because `materialization_correctness` is fixture-mutating, so the
  full gate remains enabled on a fresh fixture
- final config: `artifacts/task221-mat22-100k-background.json`; attached
  non-detached suite run completed the fresh 100k fixture and all measurements
- disposition: STOP; see `../002-isolated-candidate/` for the review packet

No corpus, cluster directory, polling snapshots, or PostgreSQL operational
logs belong in this packet.
