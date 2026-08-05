# Artifact manifest

- head SHA: `3fb1319af`
- task bucket: `reviews/task-207`
- packet: `003-full-scale`
- lane / fixture: PG18; 3-owner distann local multinode; control/candidate A/B
- storage format / rerank mode: physical distann; `rabitq` neighbor scoring
- isolated one-index-per-table or shared-table surface: one fixture per A/B
  step; each suite step is isolated
- command: `target/debug/ecaz bench suite audit --config artifacts/task207-50k-union-ab.json`
- command: `target/debug/ecaz bench suite run --config artifacts/task207-50k-union-ab.json --dry-run`
- command: `target/debug/ecaz bench suite run --config artifacts/task207-50k-union-ab.json --artifact-dir artifacts/run-50k-final`
- command: `target/debug/ecaz bench suite run --config artifacts/task207-100k-union-ab-persisted.json --artifact-dir artifacts/run-100k-final`
- timestamp: `2026-08-03T20:35:08-07:00` (100k candidate completion)
- corpus: `ec_real_50k`, source directory
  `/home/peter/dev/ecaz/data/task111a_real50k`; corpus/query files are
  external and are not committed
- 100k corpus: `ec_real_100k`; query SHA:
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- benchmark extension SHA/profile: `4e1e889784a58a82c02d7df503468f95e7c733df`
  / release with `distann-head-attribution-benchmark`

Artifacts:

- `task207-50k-union-ab.json`: checked-in 50k persisted-head SuiteConfig
- `task207-100k-union-ab-persisted.json`: checked-in 100k persisted-head
  SuiteConfig
- `owner-scan-attempt.md`: disposition of the stopped full-scale owner-scan
  attempt
- `run-50k-feature/control/`: packet-local setup logs from that stopped
  attempt; no result numbers are claimed
- `run-50k/` and `run-100k/`: earlier suite manifests, structured results,
  and packet-local logs after execution
- `run-50k-final/results.jsonl`: completed 50k persisted-head A/B result rows
- `run-50k-final/control/` and `run-50k-final/candidate/`: completed 50k
  per-arm logs, predictions, topology, and storage evidence
- `result-summary-50k.md`: cited 50k result summary
- `run-100k-final/results.jsonl`: completed 100k persisted-head A/B result rows
- `run-100k-final/control/` and `run-100k-final/candidate/`: completed 100k
  per-arm logs, predictions, topology, and storage evidence
- `result-summary-100k.md`: cited 100k result summary
