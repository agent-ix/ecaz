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
- code checkpoint: `0b6a4bbbf`
- run artifacts: pending
- recovery config: `artifacts/task221-mat22-100k-recovery.json`; setup completed
  during the initial invocation, which exited before measurements, so the
  exact immutable fixture is reused for the suite measurement rerun

No corpus, cluster directory, polling snapshots, or PostgreSQL operational
logs belong in this packet.
