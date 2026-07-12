# Artifact manifest

- Head SHA: `83a40f8fdaaf77efe2aa5ce6d735bf34292098bf`
- Cache implementation commit: `d4d38a865` (`feat(distann): cache validated physical epoch heads`)
- Fixture remediation commit: `83a40f8fd` (`fix(distann): prove remote owner with constant query`)
- Runner build: release; `suite-manifest.json` records `runner_git_commit = 83a40f8fdaaf77efe2aa5ce6d735bf34292098bf`
- Extension build: release PG18 install from the same clean SHA before the run
- Task bucket / packet: `reviews/task-179/035-physical-epoch-cache`
- Lane: local three-owner PG18 physical ec_distann, cache disabled versus enabled
- Fixture: one source table on the coordinator; one disjoint hidden generation per owner; fresh three-instance cluster for every arm
- Storage format: distributed-control generation descriptor/manifest/fingerprint v2, frozen row tier, graph heap, unique directory
- Rerank mode: RaBitQ neighbor codes plus frozen full-precision row materialization
- Query protocol: 20 held-out queries (200 recall trials) and 20 latency iterations per arm; concurrency 1
- Timestamp: `2026-07-12T11:59:24-07:00`

## Canonical suite

```text
target/release/ecaz bench suite run \
  --config reviews/task-179/035-physical-epoch-cache/artifacts/cache-ab-matrix.json \
  --log-file reviews/task-179/035-physical-epoch-cache/artifacts/suite-run.log
```

Config SHA-256:
`5e2268c67bb98ce552714213df84154232473dfa509b178f93b70806a5eb6cd1`.

The six steps are the Cartesian product of 10k/50k/100k and:

- `PGOPTIONS="-c ec_distann.physical_epoch_cache=off"`; and
- `PGOPTIONS="-c ec_distann.physical_epoch_cache=on"`.

Every step uses its own ports, run directory, and freshly initialized three-node
cluster. `suite-manifest.json` records all six steps Succeeded. All 18 presence,
topology, and remote-engagement thresholds passed. `suite-audit-final.log` is the
post-run config audit.

## Corpus provenance

| Scale | Dimension | Corpus rows / SHA-256 | Query source rows / SHA-256 | Measured queries |
| --- | ---: | --- | --- | ---: |
| 10k | 1536 | 10,000 / `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75` | 200 / `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8` | 20 |
| 50k | 1536 | 50,000 / `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133` | 1,000 / `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3` | 20 |
| 100k | 1536 | 100,000 / `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95` | 1,000 / `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782` | 20 |

Corpus/query TSVs and regenerated truth caches are intentionally not committed.

## Key A/B results

| Scale | Recall off / on | Physical p50 off / on | p50 reduction | Mean reduction | p95 reduction | Storage off / on | Storage delta |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 / 1.0000 | 11,302.8 / 70.4 ms | 99.38% | 94.33% | 94.38% | 242,761,728 / 242,745,344 B | -16,384 B |
| 50k | 0.9800 / 0.9800 | 12,041.0 / 100.9 ms | 99.16% | 94.25% | 94.86% | 1,242,742,784 / 1,242,750,976 B | +8,192 B |
| 100k | 0.9500 / 0.9500 | 17,907.0 / 82.2 ms | 99.54% | 94.58% | 94.91% | 2,496,643,072 / 2,496,626,688 B | -16,384 B |

The cache-on latency children each start a new backend and therefore include one
cold validated-head reconstruction. That cold sample dominates `max`/`p99` and
materially contributes to the mean and p95; p50 represents the warm steady-state
path. The raw 20-sample summaries remain in each arm's
`distann-multinode-summary.log` and `physical-latency.log`.

At all scales, Ready and Published owner row counts sum exactly to the corpus
size, non-owned/orphan counts are zero, and two remote owners pass both an
`EcDistannDistributedScan` EXPLAIN assertion and exact row-identity materialization.

## Failed proof and remediation

The first cache-off 10k attempt built and published valid topology but failed the
new remote-owner plan assertion. The proof used an extended-protocol `$1` Param;
the distributed planner intentionally requires a constant vector at path creation,
so PostgreSQL selected the local AM path. Commit `83a40f8fd` changed the proof to
use its already strict-allowlisted numeric array as a literal for both EXPLAIN and
execution. `suite-manifest.failed-explain.json`,
`suite-run.failed-explain.log`, and `cache-off-10k/failed-explain.log` preserve
that regression. All six exact-SHA rerun arms then passed the CustomScan proof.

## Artifact index

- `cache-ab-matrix.json`: checked-in SuiteConfig.
- `suite-manifest.json`: exact commands, PGOPTIONS, clean runner SHA, durations, statuses, and thresholds.
- `results.jsonl`: normalized result rows. Metrics are mirrored from both the main fixture log and packet summary, so equivalent rows appear twice with different artifact provenance.
- `suite-run.log`: complete canonical matrix driver record.
- `suite-report.md`: runner-generated report.
- `suite-audit.log`, `suite-audit-final.log`: pre-run and post-run config audits.
- `cache-{off,on}-{10k,50k,100k}/distann-local-multinode.log`: topology, CustomScan proof, and benchmark structured lines.
- `cache-{off,on}-{10k,50k,100k}/distann-multinode-summary.log`: concise decision-grade per-arm summaries.
- `cache-{off,on}-{10k,50k,100k}/{physical,single}-{recall,latency}.log`: standard benchmark command tables.
- `suite-manifest.failed-explain.json`, `suite-run.failed-explain.log`, `cache-off-10k/failed-explain.log`: preserved failed proof and diagnosis.

PostgreSQL server logs, corpus TSVs, truth caches, and run directories were not
committed.
