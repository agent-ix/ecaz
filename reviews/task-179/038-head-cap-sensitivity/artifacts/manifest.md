# Artifact manifest

- Documentation head SHA: `5a66a47b67b6437fd3d80d840dff25f68e4dd139`
- Benchmark extension/runner SHA: `6c25e55a22a7828ae5b3bb2c8309e15b3738d2d3`
- Suite-runner implementation: `efb0aa8cb` (`feat(cli): expose DistANN head cap in suites`)
- Task bucket / packet: `reviews/task-179/038-head-cap-sensitivity`
- Lane: local real three-owner PG18 physical ec_distann, cap 64/256/4096
- Fixture: one source table on the coordinator, disjoint hidden generation per
  owner, fresh three-instance cluster for every arm
- Storage format: distributed-control generation descriptor/manifest/fingerprint
  v2, frozen row tier, graph heap, unique directory
- Rerank mode: RaBitQ neighbor codes plus frozen full-precision row materialization
- Query protocol: 20 held-out queries (200 recall trials) and 20 latency
  iterations per arm; concurrency 1
- Run start: `2026-07-12T13:47:26-07:00`
- Config SHA-256: `9b88d0f461a3b6aa2984a0a11bfc6c169e06ceb0b2552dac128e0f4c5541df24`

The release PG18 extension and release CLI were built from the clean benchmark
SHA before the run. A reviewer-only feedback commit landed while the suite was
running; it did not change the installed extension, executable, SuiteConfig, or
working-tree measurement files. `suite-manifest.json` records the exact runner
SHA and every expanded command.

## Canonical suite

```text
target/release/ecaz bench suite run \
  --config reviews/task-179/038-head-cap-sensitivity/artifacts/head-cap-sensitivity.json \
  --continue-on-error \
  --log-file reviews/task-179/038-head-cap-sensitivity/artifacts/suite-run.log
```

All nine steps use one sequential `run_dir`. The fixture deletes and recreates
that directory before each step, so every arm uses an isolated fresh cluster
without retaining nine generated PostgreSQL trees. The nine cells are the
Cartesian product of 10k/50k/100k and head caps 64/256/4096. Every step applies
the same cap to the physical distributed-control index and same-data local
control index.

Post-run status: 9 succeeded, 0 failed, 0 missing/stale artifacts; 27 of 27
topology, recall-presence, and remote-engagement thresholds passed.

## Corpus provenance

| Scale | Dimension | Corpus rows / SHA-256 | Query source rows / SHA-256 | Measured queries |
| --- | ---: | --- | --- | ---: |
| 10k | 1536 | 10,000 / `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75` | 200 / `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8` | 20 |
| 50k | 1536 | 50,000 / `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133` | 1,000 / `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3` | 20 |
| 100k | 1536 | 100,000 / `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95` | 1,000 / `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782` | 20 |

Corpus/query TSVs and regenerated truth caches are intentionally not committed.

## Key physical results

| Scale | Cap | Recall | p50 ms | Mean ms | p95 ms | Physical generation bytes | Local control index bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 64 | 0.9950 | 86.9 | 89.4 | 145.4 | 242,794,496 | 82,657,280 |
| 10k | 256 | 0.9950 | 81.6 | 89.3 | 121.5 | 242,745,344 | 84,230,144 |
| 10k | 4096 | 1.0000 | 70.7 | 653.1 | 679.8 | 242,761,728 | 115,687,424 |
| 50k | 64 | 0.9750 | 106.2 | 110.0 | 138.2 | 1,242,734,592 | 411,156,480 |
| 50k | 256 | 0.9800 | 102.0 | 106.5 | 132.4 | 1,242,734,592 | 412,729,344 |
| 50k | 4096 | 0.9800 | 100.8 | 831.3 | 868.6 | 1,242,734,592 | 444,186,624 |
| 100k | 64 | 0.9200 | 99.4 | 103.1 | 136.6 | 2,496,626,688 | 821,780,480 |
| 100k | 256 | 0.9450 | 105.0 | 112.9 | 178.4 | 2,496,626,688 | 823,353,344 |
| 100k | 4096 | 0.9500 | 78.9 | 985.1 | 1,031.0 | 2,496,626,688 | 854,810,624 |

The cache-on latency child starts a new backend, so its first query performs a
cold validated-head load/construction. At cap 4096 that one sample dominates
mean/p95/p99; p50 describes the warm-majority steady path. This packet does not
use these samples to claim AC-13 latency closure or a fully warmed latency run.

`physical_generation_bytes` is the standard sum of owner graph, frozen row,
directory, and control-index bytes. The persisted coordinator head lives in
shared extension catalog tables and is not apportioned by this metric. Therefore
the near-identical physical-generation values do **not** prove the head object
storage-neutral. The same-data local control index does include its head object
and grows by about 33 MB from cap 64 to 4096 at every scale. No physical
coordinator-head storage-neutrality claim is made.

At every scale/cap, Ready and Published owner row counts sum exactly to the
corpus size, non-owned/orphan counts are zero, and two remote owners pass both
CustomScan EXPLAIN and exact-row materialization probes.

## Artifact index

- `head-cap-sensitivity.json`: checked-in SuiteConfig.
- `suite-manifest.json`: exact commands, runner SHA, durations, statuses, and
  threshold results.
- `results.jsonl`: normalized result rows; equivalent metrics occur once from
  the main fixture log and once from the packet summary with distinct artifact
  provenance.
- `suite-run.log`, `suite-status.log`, `suite-report.md`: canonical driver,
  completion status, and runner-generated report.
- `suite-audit.log`, `suite-audit-final.log`: pre-run and post-run audits.
- `suite-manifest.dry-run.json`, `suite-dry-run.log`: audited expanded commands.
- `cap{64,256,4096}-{10k,50k,100k}/distann-local-multinode.log`: topology,
  CustomScan proofs, and structured measurement lines.
- Each arm's `distann-multinode-summary.log`, `{physical,single}-recall.log`,
  and `{physical,single}-latency.log`: concise result and standard benchmark
  command artifacts.

Generated PostgreSQL clusters, PostgreSQL server logs, corpus TSVs, and truth
caches were pruned and are not committed.
