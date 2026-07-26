# Task 186 hierarchy-screen measurement manifest

- Head SHA: `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7` (`origin/main` task baseline)
- Task bucket: `reviews/task-186/002-hierarchy-screen/`
- Lane: PG18 local, three-owner physical DistANN surface
- Fixture: `ec_real_100k`; query SHA `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Storage/scoring: RabitQ neighbor codes, training-landmark head, bounded hierarchy seed selection, beam 4, hop rounds 100, 32 returned seeds
- Surface: isolated one-index-per-table physical surface; shared generation across benchmark arms; required single-index comparator also ran
- Commands: `ecaz bench suite audit --config reviews/task-186/002-hierarchy-screen/artifacts/task186-hierarchy-100k-suite.json`; `ecaz bench suite run --config reviews/task-186/002-hierarchy-screen/artifacts/task186-hierarchy-100k-suite.json --artifact-dir reviews/task-186/002-hierarchy-screen/artifacts/run --manifest-output reviews/task-186/002-hierarchy-screen/artifacts/run/suite-manifest.json --results-output reviews/task-186/002-hierarchy-screen/artifacts/run/results.jsonl --log-file reviews/task-186/002-hierarchy-screen/artifacts/run/suite-run.log`
- Timestamp: `2026-07-26` (run completed during this session)
- Suite config/manifests/results: `task186-hierarchy-100k-suite.json`, `run/suite-manifest.json`, and `run/results.jsonl`
- Cited result summary: `task186-hierarchy-screen-results.log`

The raw corpus/query TSVs and PostgreSQL operational logs are intentionally
not part of the review evidence commit. The packet records corpus/query
identity and retains the structured suite results plus the cited summary.
