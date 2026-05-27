# Artifact Manifest: Task 63 HNSW RaBitQ M5 Suite Config

- head SHA: `b0c7deeeadbf55cd72ef3376e11f44477fc4dfc9`
- task bucket: `reviews/task-63/020-hnsw-rabitq-m5-suite-config/`
- lane: HNSW RaBitQ benchmark handoff config
- fixture: `ec_real_50k`, `ec_real_100k`
- storage format: `turboquant`, `pq_fastscan`, `rabitq`
- rerank mode: unchanged
- timestamp: 2026-05-27T07:55:38-07:00
- isolated one-index-per-table surface: yes; one suite prefix per size and
  storage format

## Artifacts

No generated benchmark artifacts. This packet adds a checked-in m5 laptop
SuiteConfig and records static validation of its scope.

## Commands

- `jq empty benchmarks/task63-hnsw-rabitq-format/suite-m5.json`
- `jq -r '[.steps[].kind] | group_by(.)[] | "\(.[0]) \(length)"' benchmarks/task63-hnsw-rabitq-format/suite-m5.json`
- `jq -r '.steps as $s | ($s | map(select((.kind != "raw") and (.name != "precheck-host"))) | length) as $measured | ($s | map(select((.kind != "raw") and (.name != "precheck-host") and ((.tags // []) | index("hnsw")))) | length) as $hnsw | "measured_steps=\($measured) measured_hnsw_tagged=\($hnsw)"' benchmarks/task63-hnsw-rabitq-format/suite-m5.json`
- `jq -r '[.steps[] | .profile? // empty] | unique | .[]' benchmarks/task63-hnsw-rabitq-format/suite-m5.json`
- `jq -r '[.steps[] | select(.sweep? != null) | .sweep[]] | unique | @csv' benchmarks/task63-hnsw-rabitq-format/suite-m5.json`
- `rg -n "/proc|/var/run/postgresql|/var/lib/pgsql" benchmarks/task63-hnsw-rabitq-format/suite-m5.json`
