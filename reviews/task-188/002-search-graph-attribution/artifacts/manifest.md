# Task 188 search-graph-attribution measurement manifest

- Head SHA: `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7` (`origin/main`)
- Task bucket: `reviews/task-188/002-search-graph-attribution/`
- Lane: PG18 local, three-owner fresh physical DistANN generation
- Fixture: `ec_real_100k`; query SHA `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Storage/scoring: RabitQ neighbor codes, training-landmark head cap 16,384, exact head seeds for bounded arms, BW/H isolated controls, 32 returned seeds
- Surface: shared physical generation within one suite, isolated one-index-per-table benchmark surface; single-index comparator also measured
- Command: `ecaz bench suite run --config reviews/task-188/001-entry-and-residual-plan/artifacts/task188-residual-attribution-100k-suite.json --artifact-dir reviews/task-188/002-search-graph-attribution/artifacts/run --manifest-output reviews/task-188/002-search-graph-attribution/artifacts/run/suite-manifest.json --results-output reviews/task-188/002-search-graph-attribution/artifacts/run/results.jsonl --log-file reviews/task-188/002-search-graph-attribution/artifacts/run/suite-run.log`
- Timestamp: `2026-07-26` (completed during this session)
- Structured evidence: `run/suite-manifest.json`, `run/results.jsonl`, and the packet-local physical summary log
- Cited result summary: `task188-residual-attribution-results.log`

The raw corpus/query TSVs and PostgreSQL operational logs are intentionally
not part of the evidence commit. The complete per-arm stage and materialization
work rows remain in the generated packet-local structured results/summary.
