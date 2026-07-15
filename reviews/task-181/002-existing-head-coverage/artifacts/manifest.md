# Task 181 packet 002 artifact manifest

This packet records the Phase 1 100k audit of the unchanged production head.

## Provenance and command

- Owning packet: `reviews/task-181/002-existing-head-coverage/`.
- Head SHA / installed extension SHA: `e75dfc14bbf0c1ff406a7dc1795f7e1c2f4514d8`.
- Installed profile/features: release, PG18,
  `distann-head-attribution-benchmark`; all three nodes attested unanimously.
- Corpus: `ec_real_100k`; evaluation rows 1-200 of the staged query file,
  SHA-256 `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Topology: three exact/disjoint physical owners, one index per table, graph
  degree 32, cap 4096, BW4/H100, RaBitQ search and neighbor codes.
- Config: `phase1-current-100k-suite.json`.
- Command: `target/release/ecaz bench suite run --config reviews/task-181/002-existing-head-coverage/artifacts/phase1-current-100k-suite.json`.
- Timestamp: 2026-07-15 PDT.

## Key results

The compact `phase1-current-100k/results.jsonl` and
`phase1-current-100k/100k/distann-multinode-summary.log` are the cited sources.

| Arm | Distinct recall@10 (95% CI) | Warm p50 / p95 |
| --- | ---: | ---: |
| persisted production | 0.9275 (0.9153-0.9381) | 39.60 / 53.10 ms |
| exact same membership | 0.9275 (0.9153-0.9381) | 36.80 / 50.90 ms |
| owner oracle | 0.9970 (0.9935-0.9986) | 2467.00 / 2512.20 ms |

The new per-query coverage row reports:

- owner top-32 membership in the head: `0.04703125`;
- exact-head/owner top-32 overlap: `0.04703125`;
- persisted-head/owner top-32 overlap: `0.045`;
- queries with zero owner top-32 seeds represented: `0.435`;
- best exact-head versus owner code-score gap: mean `0.1684243`, p50
  `0.1496532`, p95 `0.3512916`;
- 182 deterministic query-geometry regions represented in the 200-query
  evaluation set; the compact region histogram is embedded in the coverage
  result row.

The build/publish times were 860,972 / 987,997 ms. The head contains 4,096
unique entries, occupies 25,280,512 sample bytes plus 614,055 graph/state
bytes, and has a 25,894,567-byte cache estimate. Total physical-generation
bytes were 2,496,659,456. Ready/published ownership summed to exactly 100,000
with zero non-owned rows or orphans; both remote materialization probes passed.

## Loss decomposition

Exact head scoring and persisted head search produce byte-identical 0.9275
recall, while 43.5% of queries have no owner top-32 seed in the fixed head at
all. The miss is therefore dominated by absent landmark membership. It is
broadly distributed across query-geometry regions rather than isolated to a
hash owner or a single recurring region. Phase 2 should screen new membership
policies; another width/seed sweep is not justified.

`implementation-validation.log`, `implementation-install.log`, and
`implementation-cli-build.log` record the focused test and release builds.
Corpus/query TSVs, truth caches, node PostgreSQL logs, the duplicate full
fixture log, and the stopped run directory are not committed.
