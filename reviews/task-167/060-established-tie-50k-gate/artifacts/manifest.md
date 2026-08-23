# Task 167 packet 060 artifact manifest — preregistration

- Preregistration head: `45de2ed6f5c445265e44103fcaa845882ca90a87`.
- Candidate code checkpoints:
  `350385ce9fe7158286ce6570383f8f44828fe671` and
  `ddea621a61dd19e0c4c946b5a0627a57a5dae4dc`.
- Owning packet: `reviews/task-167/060-established-tie-50k-gate/`.
- Suite config: `task167-established-tie-50k-suite.json`.
- Suite config SHA-256:
  `552ec628fd0c68d8fefcac95a4ae2c20cb0c3e229fbda30abf92a1e8386dddc8`.
- Timestamp: `2026-08-22`.
- Lane: production physical distributed DistANN on PG18, three owners,
  RabitQ neighbor storage, exact fp32 truth, no rerank variant.
- Fixture: isolated `ec_real_50k`, one index per table, external run directory
  `/home/peter/.ecaz/clusters/task167-established-tie-50k-20260822`.
- Search regime: beam width 4, candidate heap 32, hop rounds 100, top-k 10,
  200 heldout queries plus 48 inserted-neighborhood queries.
- Insert regime at the quality checkpoint: 160 candidate-default exact-vector
  duplicate inserts with ID base `2000000`. The append-when-room control ID
  base `3000000` is excluded until after the quality gate passes.
- Hard gates remain fixed from packet 045: inserted-neighborhood deficit at
  most `0.015`, heldout deficit at most `0.007`.
- Before comparators:
  - packet 047 retained robust-prune heldout deficit `0.008611`;
  - packet 051 append-when-room heldout deficit `0.010611`;
  - packets 054 and 057 conservative-admission and full-target-no-op heldout
    deficits `0.009611`.
- Runtime output will be packet-local under `artifacts/final-suite/`. Corpus
  data, truth caches, PGDATA, PostgreSQL operational logs, and polling output
  will not be committed. The external fixture will be stopped and removed
  after cited results are captured.
- Execution is preregistered but not authorized until packet 059 receives an
  outside-review verdict and any findings are processed.

## Preregistered command

- Audit:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/060-established-tie-50k-gate/artifacts/task167-established-tie-50k-suite.json --log-file reviews/task-167/060-established-tie-50k-gate/artifacts/suite-audit-preregister.log`.
- Audit result: passed, 1 step. Log SHA-256:
  `d3565a0b4e681311ba7af4845c31a67d9a3ea60a07ac5ee3a7e17665a3c66a83`.
- Run, only after outside review, exact-runtime PG18 release installation,
  release CLI build, and repeated audit:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/060-established-tie-50k-gate/artifacts/task167-established-tie-50k-suite.json --log-file reviews/task-167/060-established-tie-50k-gate/artifacts/suite-run.log`.
- Report after completion or an expected quality-gate failed-step exit:
  `/home/peter/.cargo-target/release/ecaz bench suite report --manifest reviews/task-167/060-established-tie-50k-gate/artifacts/final-suite/suite-manifest.json --results-output reviews/task-167/060-established-tie-50k-gate/artifacts/final-suite/results.jsonl --log-file reviews/task-167/060-established-tie-50k-gate/artifacts/suite-report.log`.

## Decision rule

- If either fixed quality band fails, reject the candidate, skip the
  append-when-room control and post-gate drills, and keep Task 167 open.
- If both fixed quality bands pass, complete the packet's remaining drills and
  then preregister the isolated 10k/50k/100k recall, latency, and storage
  matrix. A 50k pass alone is not closeout.
