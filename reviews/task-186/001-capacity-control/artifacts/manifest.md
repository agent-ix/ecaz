# Task 186 capacity-control measurement manifest

- Head SHA: `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7` (`origin/main` task baseline)
- Task bucket: `reviews/task-186/001-capacity-control/`
- Lane: PG18 local, three-owner physical DistANN surface with isolated per-arm ports
- Fixture: `ec_real_100k`; query SHA `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Storage/scoring: RabitQ neighbor codes, exact head scoring, `head_sample_exact`, beam 4, hop rounds 100, 32 returned seeds
- Surface: each arm used an isolated one-index-per-table physical surface; the runner also built the required single-index comparator in that arm
- Commands: `ecaz bench suite audit --config reviews/task-186/001-capacity-control/artifacts/task186-capacity-control-100k-suite.json`; the 4096/8192 arms were run with `ecaz bench suite run` into `artifacts/run-benchmark-feature`; the conditional 16384 arm was run with `--only trained-cap-16384-candidate-100k` into `artifacts/run-cap16384`
- Timestamp: `2026-07-26` (runs started at approximately 12:57 PDT, 13:35 PDT, and 14:16 PDT respectively)
- Suite configs/manifests/results: `task186-capacity-control-100k-suite.json`, `run-benchmark-feature/suite-manifest.json`, `run-benchmark-feature/results.jsonl`, `run-cap16384/suite-manifest.json`, and `run-cap16384/results.jsonl`
- Cited result summary: `task186-capacity-control-results.log`

The raw corpus/query TSVs and PostgreSQL operational logs are intentionally not part of the review evidence commit. The packet records the corpus/query identities and retains the structured suite results plus the cited result summary.
