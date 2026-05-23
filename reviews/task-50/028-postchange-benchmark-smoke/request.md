# Task 50 Postchange Benchmark Smoke

This packet adds a narrow postchange benchmark smoke for Task 50 after the unsafe-block cleanup closeout packet.

Scope:

- fixture: `ec_real_10k`
- surfaces: `ivfrabitq`, `spirerabitq`, `hnsw`, `diskann`
- steps per surface: load, recall, latency, storage
- latency smoke: `50` iterations per sweep value
- PG target: local PG18 pgrx instance on `/home/peter/.pgrx`, port `28818`

Authoritative evidence is under `artifacts/tight/`.

Summary:

- `ecaz bench suite audit` passed for `suite-tight.json`.
- `ecaz bench suite run` completed all 16 selected steps with `failed=0`, `missing_artifacts=0`, and `stale=0`.
- SPIRE/RabitQ, the production target lane, completed load/recall/latency/storage successfully. Recall reached `1.0000` from nprobe `24` upward; the nprobe `64` smoke latency mean was `241.7 ms` with p95 `258.4 ms` over 50 iterations.
- IVF/RabitQ, HNSW, and DiskANN also completed their selected 10k load/recall/latency/storage smokes.

The packet intentionally does not claim a full benchmark-regression comparison. It is the requested narrow smoke to catch major postchange issues after installing the branch build into PG18.

Primary artifacts:

- `artifacts/manifest.md`
- `artifacts/tight/suite-manifest.json`
- `artifacts/tight/suite-tight-status.log`
- `artifacts/tight/suite-tight-report.md`
- `artifacts/tight/results.jsonl`
- `artifacts/tight/results-report.jsonl`
