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
- Exact runtime head:
  `3da8c572ec5a1034ef5563c661da201c8ad83efe`; installed and attested at
  `2026-08-23T09:23:18-07:00`.
- PG18 extension install command:
  `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
- Install result: passed. `install-extension.log` SHA-256:
  `02f80a8566e6648934246d9c223d85cc858230fdd576e7ff1c0f31aab160a9f1`.
- Installed `ecaz.so` embeds `3da8c572e.../release`; binary SHA-256:
  `146a108df1213535721c1bf962570e71fdaa9918cc3bf3cb16974d76c9f06f0e`.
- Release CLI build command:
  `cargo build -p ecaz-cli --release --no-default-features`.
- Build result: passed with the pre-existing unrelated dead-code warning at
  `commands/corpus/load.rs:190`. `build-cli.log` SHA-256:
  `f242c079f081bbf2e669f4a706124a368b7a6fe42d733e26e97ffb51468b08c2`.
- Release CLI embeds `3da8c572e.../release`; binary SHA-256:
  `6c0eb4f90acc870b1c2b6c20110f467db9f397a19ddccd97b9f863cb87bfd868`.
- Exact-runtime audit passed all 3 steps. `suite-audit-runtime.log` SHA-256:
  `8b392542b972b9146729f53c9abd0c4d00fcf32f13efecf0de0a6d153cc93f5c`.
- One matrix run after attestation:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/062-final-sha-closeout-matrix/artifacts/task167-final-sha-closeout-suite.json --continue-on-error --log-file reviews/task-167/062-final-sha-closeout-matrix/artifacts/suite-run.log`.
