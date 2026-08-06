# Task 216 attribution manifest

- Task bucket: `reviews/task-216/001-attribution/`
- Packet status: attribution complete; no candidate implementation
- Control: normal PG18 release, sharded owners, no coordinator full-graph
  replica, current materialization/schema-cache behavior
- Required first scale: fresh physical 100k generation
- Runner: `ecaz bench suite` with a checked-in SuiteConfig
- Suite config: `artifacts/task216-100k-attribution.json`
- Diagnostic extension: PG18 release with
  `distann-head-attribution-benchmark`; suite uses `skip_install=true`
- Control parameters: BW4/H100, effective L32, persisted-head seeds 32,
  graph degree 32, build shards 1, top-k 10, 50 iterations / 10 warmups
- Candidate rule: pre-register at most three measured-stage candidates and
  advance at most one
- Hard separation: no Task 215 BW64/H8 defaults in the decision control or
  candidate
- Evidence to capture: owner graph read, scoring, response assembly/encoding,
  wire wait, coordinator decode/copy, executor residual, allocations/copies,
  request/response bytes, recall/result identity, tails, topology, and failure
  semantics where measurable

Run completed on 2026-08-06 America/Los_Angeles. The suite process completed
the physical step with exit code 0 and wrote `artifacts/run/results.jsonl` and
`artifacts/run/suite-manifest.json`; the final suite-level NFR-021 assertion
returned `actual_admissibility=unavailable` because this packet intentionally
contains only the required fresh 100k diagnostic scale. The topology gate and
serving/reconciliation drills passed. This is diagnostic evidence, not a
conforming release decision cell.

Packet-local source artifacts:

- `artifacts/run/results.jsonl` — structured source of truth; SHA-256
  `613201d00aa8186fd28b1b3e170a45991ba44d3da027d38ae1acaa71cc12f25e`.
- `artifacts/run/suite-manifest.json` — runner provenance; SHA-256
  `2ad23290172eafde4007ee9a777ef68fd72f22b8e5142fdaa85331517574740d`.
- `artifacts/run/100k-production-control/distann-multinode-summary.log` —
  topology, recall, latency, stage, work, storage, and gate rows; SHA-256
  `5be77a60bbdeb9528bbb9e2269636c33135ddf6437b2d5c08c9461a9585fde4a`.
- `artifacts/run/100k-production-control/physical-production-control-latency.log`
  — full stage counters and memory samples; SHA-256
  `b4b4df7564b986cf770400ae880895d3d4ac8e518dcd1e5b9312da309028a8ea`.
- `artifacts/attribution-disposition.md` — compact candidate screen and
  attribution decision.
- `artifacts/validation.md` — commands, gates, install/restore, and cleanup.

The benchmark command was driven exclusively by
`ecaz bench suite run --config artifacts/task216-100k-attribution.json` with
packet-local manifest, results, and log outputs. The transient cluster was
removed after capture; no corpus TSV, truth cache, node logs, predictions, or
polling exhaust is part of the packet.

Pre-registered candidate families before measurement: `MAT-15`, `MAT-21`, and
`TRAV-05`. The run must select candidates from measured dominant stages; this
list is not an implementation commitment.
