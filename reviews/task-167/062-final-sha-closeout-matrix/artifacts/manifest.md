# Task 167 packet 062 artifact manifest — preregistration

- Preregistration head: `2fb2972b86d26686ee9105b892de0c245994efab`.
- Product code checkpoint under measurement:
  `f58a69b41efbf5753b098b7476e7d7e7ba438c43` plus accepted packet 059
  checkpoint `350385ce9fe7158286ce6570383f8f44828fe671`.
- Owning packet: `reviews/task-167/062-final-sha-closeout-matrix/`.
- Suite config: `task167-final-sha-closeout-suite.json`.
- Suite config SHA-256:
  `8ffcbf7a0557281055d26ab1446685ace5ed3d7d9cae050885fcee976661c25e`.
- Timestamp: `2026-08-23` (America/Los_Angeles).
- Lane: production physical distributed DistANN on PG18, three owners, RabitQ
  neighbor storage, exact fp32 truth, no rerank variant.
- Fixtures: one isolated real-corpus step each at 10k, 50k, and 100k; one
  physical index and one single-control index per step, never shared across
  scales.
- External run directories:
  `/home/peter/.ecaz/clusters/task167-final-sha-20260823-{10k,50k,100k}`.
  They are disposable runtime state, not evidence, and will be removed after
  packet-local results are captured.
- Search regime: graph degree 32, head cap 4,096, beam width 4, candidate heap
  32, hop rounds 100, top-k 10.
- Measurement regime: 200 recall/latency/heldout queries, 48 additional
  inserted-neighborhood queries, latency iterations 10/5/5 at 10k/50k/100k,
  warmup 2, concurrency sweep 1.
- Task 167 semantics: inserted-neighborhood AC-4 hard gate; heldout
  `baseline_recording` disclosure; measurement-integrity failures remain hard.
- Corpus TSVs, truth caches, PGDATA, PostgreSQL operational logs, and polling
  output will not be committed.

## Commands

- Preregistration audit:
  `cargo run -p ecaz-cli --no-default-features -- bench suite audit --config reviews/task-167/062-final-sha-closeout-matrix/artifacts/task167-final-sha-closeout-suite.json --log-file reviews/task-167/062-final-sha-closeout-matrix/artifacts/suite-audit-preregister.log`.
- Audit result: passed, 3 steps. Log SHA-256:
  `8b392542b972b9146729f53c9abd0c4d00fcf32f13efecf0de0a6d153cc93f5c`.
- Exact runtime PG18 release install: pending.
- Exact runtime release CLI build and audit: pending.
- One matrix run after attestation:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/062-final-sha-closeout-matrix/artifacts/task167-final-sha-closeout-suite.json --continue-on-error --log-file reviews/task-167/062-final-sha-closeout-matrix/artifacts/suite-run.log`.
