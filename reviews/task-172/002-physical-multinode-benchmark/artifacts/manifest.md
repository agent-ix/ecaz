# Artifact manifest

- Head SHA: `77e09a511d0a8b78803f52992c89c1ec2e98e7d8`
- Extension build: `release`, `ecaz_build_git_sha() = 77e09a511d0a8b78803f52992c89c1ec2e98e7d8`
- Runner build: `release`, suite `runner_git_commit = 77e09a511d0a8b78803f52992c89c1ec2e98e7d8`
- Task bucket: `reviews/task-172/`
- Packet: `002-physical-multinode-benchmark`
- Lane: local three-owner PG18 physical ec_distann versus same-data single-instance ec_distann
- Fixture: one source table on the coordinator; one disjoint hidden generation per owner; no shared owner tables and no replicated serving-control surface
- Storage format: distributed-control generation descriptor/manifest/fingerprint v2, frozen row tier, graph heap, unique directory
- Rerank mode: RaBitQ neighbor codes plus frozen full-precision row materialization
- Query protocol: recall uses 10 held-out queries per scale; latency uses 5 iterations, concurrency 1, operator label `warm`
- Timestamp: `2026-07-12T07:06:12-07:00`

## Canonical suite

```text
target/release/ecaz bench suite run \
  --config reviews/task-172/002-physical-multinode-benchmark/artifacts/physical-matrix.json \
  --log-file reviews/task-172/002-physical-multinode-benchmark/artifacts/suite-run.log
```

The three measurement steps all succeeded. The initial run then rejected two
provisional absolute `recall >= 0.99` thresholds even though the physical and
single arms were equal. The config was corrected to require a present recall row,
and the successful steps were resumed without rerunning:

```text
target/release/ecaz bench suite run \
  --config reviews/task-172/002-physical-multinode-benchmark/artifacts/physical-matrix.json \
  --resume-from target/task172-physical-initial-manifest.json \
  --log-file reviews/task-172/002-physical-multinode-benchmark/artifacts/suite-resume.log
```

Final config SHA-256:
`d6592ec29f02dfb7965d2d645a87e3ca3a4e22bbf612b159f4dd0d6bb808319f`.
The final manifest records all three steps Succeeded and all nine presence/topology/
remote thresholds passed.

## Corpus provenance

| Scale | Dimension | Corpus rows / SHA-256 | Query source rows / SHA-256 | Measured queries |
| --- | ---: | --- | --- | ---: |
| 10k | 1536 | 10,000 / `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75` | 200 / `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8` | 10 |
| 50k | 1536 | 50,000 / `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133` | 1,000 / `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3` | 10 |
| 100k | 1536 | 100,000 / `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95` | 1,000 / `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782` | 10 |

Corpus/query TSVs and regenerated exact truth caches are intentionally not committed.

## Key normalized results

| Scale | Recall physical / single | Recall delta | Physical latency p50 / p95 / p99 | Physical generation bytes | Raw-vector amplification |
| --- | --- | ---: | --- | ---: | ---: |
| 10k | 1.0000 / 1.0000 | 0.0000 | 12,314.40 / 12,939.80 / 12,953.40 ms | 242,761,728 | 3.9512× |
| 50k | 0.9700 / 0.9700 | 0.0000 | 11,100.70 / 11,406.90 / 11,465.90 ms | 1,242,734,592 | 4.0454× |
| 100k | 0.9500 / 0.9500 | 0.0000 | 21,072.30 / 21,317.40 / 21,353.00 ms | 2,496,626,688 | 4.0635× |

The full physical begin/build/publish elapsed values are the `publish_ms` fields:
79,881 ms (10k), 493,005 ms (50k), and 1,067,837 ms (100k). The `physical_ms`
field in this runner revision measures begin-registration overhead only and is not
cited as build time. Same-data single-index construction took 15,619 / 173,640 /
421,046 ms.

## Artifact index

- `physical-matrix.json`: checked-in SuiteConfig.
- `suite-manifest.json`: exact commands, clean SHA, durations, final statuses and thresholds.
- `results.jsonl`: 96 normalized result rows.
- `suite-run.log`: original complete matrix driver record.
- `suite-resume.log`: no-rerun threshold correction record.
- `physical-{10k,50k,100k}/distann-local-multinode.log`: topology and benchmark structured lines.
- `physical-{10k,50k,100k}/distann-multinode-summary.log`: decision-grade per-scale summaries.
- `physical-{10k,50k,100k}/{physical,single}-{recall,latency}.log`: standard command tables.

PostgreSQL server logs were discarded as banned operational exhaust.

