# Task 221 packet 002 manifest

- packet: `reviews/task-221/002-isolated-candidate/`
- task bucket: `reviews/task-221/`
- source packet: `reviews/task-221/001-preregistration-and-screen/`
- lane: ec_distann owner-side row materialization
- fixture: three-owner physical PG18, `ec_real_100k`, 100,000 rows, 200
  queries, top-k 10; 50 warm latency iterations plus 10 warmups
- storage format: rabitq physical generation; sharded owner control; no
  traversal replica
- rerank mode: production lazy-10 (`materialization_batch_size=10`)
- shared surface: one immutable physical generation and one query surface;
  same-generation pair verified by the suite
- arm delta: control `expanded_locator=false`; candidate
  `expanded_locator=true`; all other search/materialization settings equal
- runner: `ecaz bench suite`
- source command: see `../001-preregistration-and-screen/artifacts/task221-mat22-100k-background.json`
- source suite artifact: temporary `run-background/100k`, cleaned after the
  decision-grade outputs were copied into this packet
- packet-local structured evidence: `artifacts/results.jsonl`
- extension SHA: `5757ed6cb21b87ae5dae693327dcc8dbd72f8c72`
- CLI runner SHA: `d1bd2a3bf`
- query SHA: `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- decision: STOP under the preregistered neutral/regression rule; no matrix

The committed packet contains only decision-grade structured output and
summaries. Corpus files, cluster directories, node PostgreSQL logs, and
polling exhaust are excluded.
