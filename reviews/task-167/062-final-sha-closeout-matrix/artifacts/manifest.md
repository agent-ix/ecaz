# Task 167 packet 062 artifact manifest — final closeout evidence

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

## Run outcome

- Run status: passed; suite exit code 0; 3 succeeded, 0 failed, 0 skipped,
  0 missing artifacts, 0 stale.
- Step durations: 10k `1,052,992 ms`; 50k `2,665,721 ms`; 100k
  `4,871,139 ms`. Overall manifest start-to-finish wall time was
  `9,539,837 ms` (about 2h39m), including runner overhead.
- Runner commit recorded by the suite:
  `a1fcad4eea07e9e787f7ed094d3fd78789c91bb1`.
- Every result row reports extension SHA
  `3da8c572ec5a1034ef5563c661da201c8ad83efe`, build profile `release`.
- Canonical structured evidence: `final-suite/suite-manifest.json` and
  `final-suite/results.jsonl`.
- Human-readable suite report: `suite-report.log`.
- Compact cited extract: `cited-results.log`; its values are copied from
  `final-suite/results.jsonl`, which remains authoritative.
- Per-scale raw summary logs:
  `final-suite/physical-{10k,50k,100k}-final-sha/distann-multinode-summary.log`.
- Artifact SHA-256 values:
  - `cited-results.log`:
    `7096fe7bffc1af76793a1ea0d4039db8c1a95cf43662e15dc3df952eaa4b2bc7`
  - `suite-run.log`:
    `717f15d501cb6612e112865426d958596e2d7fc1eedf155d4e6dbea812a7a52b`
  - `suite-report.log`:
    `cb124bb048ddad2a1548c48323b4f806b4e6561b694e4eea4f330a7a0a87c63f`
  - `final-suite/suite-manifest.json`:
    `738dae1fb1e465a96e7802e15c56c0326be1960b07e7ba4ec5906162fd209691`
  - `final-suite/results.jsonl`:
    `835e93b7f01378b4ff72d6414bc0d1019cb6962df2accda68874813c2cd4b102`
  - 10k/50k/100k `distann-multinode-summary.log`:
    `7dfcc1363c28e3173afba7d3e28a755e2f861af1cae84304e6e596d2ffadfc35`,
    `0e97bd3b9442c1c11822b0db4c303d232f9336567e179c5805bede5a344b505b`,
    `ddbb9e72f10c4c216aa29c4709f90741fbc6d97d288d33c40d8f0f0cacb8dcf0`.

## Cited results

- 10k: ordinary physical recall `0.9990`; latency mean/p95
  `15.20/17.50 ms`; graph-side bytes `76,095,488`; raw-vector amplification
  `1.238533x`; inserted-neighborhood physical/fresh/deficit
  `0.931920/0.935681/0.003762`, AC-4 pass; heldout physical/fresh/delta
  `0.973000/0.974500/-0.001500`, baseline recording.
- 50k: ordinary physical recall `0.9545`; latency mean/p95
  `20.70/22.50 ms`; graph-side bytes `410,214,400`; raw-vector amplification
  `1.335333x`; inserted-neighborhood physical/fresh/deficit
  `0.916791/0.931052/0.014261`, AC-4 pass; heldout physical/fresh/delta
  `0.843722/0.857333/-0.013611`, baseline recording.
- 100k: ordinary physical recall `0.9295`; latency mean/p95
  `17.00/18.30 ms`; graph-side bytes `831,782,912`; raw-vector amplification
  `1.353813x`; inserted-neighborhood physical/fresh/deficit
  `0.916419/0.922082/0.005663`, AC-4 pass; heldout physical/fresh/delta
  `0.805500/0.767000/+0.038500`, baseline recording.
- Inserted-neighborhood allowed deficit was `0.015000` at every scale. All
  three quality gates report `quality_gate_pass=true` and
  `measurement_complete=true`.
- Heldout reports `quality_gate_applied=false`,
  `quality_gate_mode=baseline_recording`, as preregistered.

## Cleanup

- After durable packet-local results were captured, the three external run
  directories listed above were removed (about 15.4 GB total).
- Nine packet-local `node*-postgres.log` operational logs were removed and are
  not committed. Corpus TSVs, truth caches, PGDATA, and polling output are not
  present in this packet.
