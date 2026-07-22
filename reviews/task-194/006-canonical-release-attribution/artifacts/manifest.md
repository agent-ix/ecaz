# Task 194 canonical release attribution manifest

- Head SHA (suite runner): `5824a091a37d70967edb2419f46d67c8c597f8fb`.
- Extension SHA: `809db6716f1d31986f7e378320453781f1340987`.
- Extension build profile: `release`, unanimous on three owners.
- Task bucket / packet: `reviews/task-194/006-canonical-release-attribution/`.
- Canonical suite config:
  `reviews/task-194/002-nine-way-attribution/artifacts/suite/task194-suite.json`.
- Immutable suite evidence:
  `reviews/task-194/002-nine-way-attribution/artifacts/suite/run/`.
- Lane: Intel local, PG18, physical three-owner DistANN generation,
  `training_landmarks_exact`, RaBitQ stored neighbor values, lazy10 rerank.
- Fixture: `ec_real_100k`, one index per physical/source table plus the suite's
  same-data single-index comparison surface.
- Command: `target/debug/ecaz bench suite run --config
  reviews/task-194/002-nine-way-attribution/artifacts/suite/task194-suite.json
  --database tqvector_bench --log-file
  reviews/task-194/005-owner-sideband-rework/artifacts/suite-run-release.log`.
- Timestamp: 2026-07-21 21:29–22:03 America/Los_Angeles.
- Suite status: completed=1, failed=0, missing_artifacts=0, stale=0.

Key result rows from `results.jsonl` / `distann-multinode-summary.log`:

- recall: 0.9625 (CI95 0.9532–0.9700), 200 queries / 2,000 trials;
- latency: mean 23.50 ms, p50 23.50, p95 27.00, p99 27.70;
- storage: physical generation 2,496,626,688 bytes; control indexes 24,576
  bytes; same-data single index 854,810,624 bytes;
- traversal total 7.475629 ms/scan, remote expansion 6.066737;
- owner service 1.930342, transport wait 4.078199, straggler spread 0.394431;
- owner open/validate 6.926585, payload SQL 8.717163, node lookup 0.297166;
- traversal: 10 rounds, 40 nodes requested/returned, 0 repeated nodes per
  scan.

Durable files retained under packet 002:

- `suite-manifest.json` — status, command, runner SHA, config hash;
- `results.jsonl` — normalized suite result rows;
- `nine-way-attribution-100k/distann-multinode-summary.log` — canonical raw
  result lines;
- `physical-production-recall.log` and `physical-production-latency.log` —
  benchmark output and full attribution rows.

Operational PostgreSQL logs, fixture data, and generated corpus/truth data are
not part of the committed evidence.
