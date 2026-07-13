# Artifact manifest

- Task bucket / packet: `reviews/task-172/003-postfix-physical-matrix-acceptance`
- Evidence type: immutable cross-packet acceptance citation; no measurement is
  duplicated or rerun in this packet
- Current branch head when requested: `53984f2760cd58f5453ec0c1cb881615c7210119`
- Measured extension source: `9387f72b3209c751ba561f5f976f57954bd30b90`
- Prompt-cancellation implementation: `a94e5e9be83b523a907ca3590dc62cafeca3cb3a`
- Unchanged release runner commit: `24ec63788cc5c8ea361eb8c0ceff6c5a966e5323`
- Release runner SHA-256: `ad52902025faeed5c79629dabc23b8dd3e5a48d94d06e92f44e8af3259959320`
- Lane: local Intel PG18 physical DistANN versus same-data single-index control
- Host: x86_64 Intel Core i9-10900K, 20 logical CPUs, 62 GiB RAM,
  Linux 6.18.33.2 WSL2, 1 TiB ext4 virtual disk
- PostgreSQL: 18.3 release extension, three loopback owner instances per scale
- Storage format: WAL-logged distributed-control physical graph, row, and
  directory relations; degree 32; head cap 4096
- Rerank mode: exact frozen-row materialization from the physical owner
- Isolation surface: isolated source/control tables and one generation per
  owner; no shared-table benchmark surface

Subsequent source commit `e6e03dfc2` changes only the CLI physical publish
fault fixture and suite parser. It does not alter the measured production
extension read path, storage format, or benchmark SQL.

## Immutable evidence citations

### Final current matrix

`reviews/task-179/052-prompt-cancellation-ab/`

- `artifacts/candidate-suite.json` — canonical 10k/50k/100k config.
- `artifacts/candidate/suite-manifest.json` — 3/3 succeeded, 12/12 thresholds,
  exact commands, runner SHA, durations, expected artifacts.
- `artifacts/candidate/results.jsonl` — source rows for all current recall,
  latency, storage, topology, and engagement values.
- per-scale `physical-{recall,latency}.log` and
  `single-{recall,latency}.log` — detailed tables.
- `artifacts/comparison.md` — prompt-poll isolation against the already post-fix
  direct-reader baseline.
- `artifacts/manifest.md` — corpus hashes, commands, host, build profile, and
  exact source provenance.

Key state: 20 recall queries (200 recall@10 trials), 10 untimed warmups, 50
measured latency queries, concurrency 1, three owners, two remote owners,
degree 32, head cap 4096.

### Direct graph-reader isolation

`reviews/task-179/050-direct-graph-reader-ab/`

- immutable pre-poll current-design arm and direct-reader A/B;
- recall identical to its baseline at all scales; and
- no demonstrated latency regression after replacing dynamic SPI graph reads.

### Persisted-head production baseline

`reviews/task-179/048-persisted-head-ab/`

- same-source owner-scan versus persisted-head isolation;
- documents the bounded-work recall tradeoff explicitly; and
- establishes the production strategy used by packets 050 and 052.

### Prior outside-review ruling

`reviews/task-172/002-physical-multinode-benchmark/feedback/2026-07-12-01-reviewer.md`

- accepted recall, storage, and topology for Task 179 AC-13;
- rejected only the pre-fix latency arm; and
- required a post-fix run with real warmup and a decision-grade sample.

## Derived values

All values below derive directly from packet 052's committed JSONL/logs.

| Scale | Physical recall | Single recall | Physical mean/p50/p95/p99 ms | Single mean/p50/p95/p99 ms | Generation bytes | Raw f32 bytes | Amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 | 1.0000 | 43.50/44.00/55.70/56.10 | 2.83/2.77/3.43/3.57 | 242,761,728 | 61,440,000 | 3.9512x |
| 50k | 0.9800 | 0.9750 | 54.50/54.20/67.90/72.30 | 3.38/3.43/3.98/4.15 | 1,242,734,592 | 307,200,000 | 4.0454x |
| 100k | 0.9500 | 0.9450 | 49.50/46.90/67.40/75.90 | 3.56/3.39/4.55/4.88 | 2,496,634,880 | 614,400,000 | 4.0635x |

Raw f32 bytes are the explicit computed denominator `rows * 1536 * 4`,
correcting packet 002's implicit-denominator citation defect.

100k record-balance calculation:

```text
mean = 100000 / 3 = 33333.3333
max absolute deviation = 33333.3333 - 33195 = 138.3333
max deviation percentage = 138.3333 / 33333.3333 * 100 = 0.415%
```

No corpus, query, truth-cache, server log, or benchmark output is duplicated in
this packet. The cited immutable task packets remain the source of truth.
