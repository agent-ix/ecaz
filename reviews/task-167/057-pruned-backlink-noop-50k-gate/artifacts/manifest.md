# Task 167 packet 057 artifact manifest — preregistration

- Preregistration head: `2f305031a50828f93a788311d5a789da9827a999`.
- Candidate code checkpoints:
  `5e32a1dfb2e5d35ffe365c8bb013f43cc3bdbb34` and
  `3da6df06cd8f2428212e492535987e993a4658cf`.
- Owning packet:
  `reviews/task-167/057-pruned-backlink-noop-50k-gate/`.
- Suite config: `task167-pruned-backlink-noop-50k-suite.json`.
- Suite config SHA-256:
  `a4dab32c4387468bcc0c7b3a52a4013a13548811d3c3b1301d63d1fce7c99e1c`.
- Timestamp: `2026-08-22`.
- Lane: production physical distributed DistANN on PG18, three owners,
  RabitQ neighbor storage, exact fp32 truth, no rerank variant.
- Fixture: isolated `ec_real_50k`, one index per table, external run directory
  `/home/peter/.ecaz/clusters/task167-pruned-backlink-noop-50k-20260822`.
- Search regime: beam width 4, candidate heap 32, hop rounds 100, top-k 10,
  200 heldout queries plus 48 inserted-neighborhood queries.
- Insert regime at the quality checkpoint: 160 candidate-default inserts with
  ID base `2000000`. The append-when-room control ID base `3000000` is excluded
  until after the quality gate passes.
- Hard gates remain fixed from packet 045: inserted-neighborhood deficit at
  most `0.015`, heldout deficit at most `0.007`.
- Before comparators:
  - packet 047 robust-prune-all heldout deficit `0.008611`;
  - packet 051 append-when-room heldout deficit `0.010611`;
  - packet 054 conservative-admission heldout deficit `0.009611`.
- Runtime output will be packet-local under `artifacts/final-suite/`. Corpus
  data, truth caches, PGDATA, PostgreSQL operational logs, and polling output
  will not be committed. The external fixture will be stopped and removed
  after cited results are captured.

## Preregistered command

- Audit:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/task167-pruned-backlink-noop-50k-suite.json --log-file reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/suite-audit-preregister.log`.
- Audit result: passed, 1 step. Log SHA-256:
  `cdc6118948c73fd5ccc4fde9b50ad02afee5868378a7adeb9210edd7cbeb4c44`.
- Run, only after exact-runtime PG18 release installation, release CLI build,
  and repeated audit:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/task167-pruned-backlink-noop-50k-suite.json --log-file reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/suite-run.log`.
- Report after completion or an expected quality-gate failed-step exit:
  `/home/peter/.cargo-target/release/ecaz bench suite report --manifest reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/final-suite/suite-manifest.json --results-output reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/final-suite/results.jsonl --log-file reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/suite-report.log`.

## Decision rule

- If either fixed quality band fails, reject the candidate, skip the
  append-when-room control and post-gate drills, and keep Task 167 open.
- If both fixed quality bands pass, complete the packet's remaining drills and
  then preregister the isolated 10k/50k/100k recall, latency, and storage
  matrix. A 50k pass alone is not closeout.
