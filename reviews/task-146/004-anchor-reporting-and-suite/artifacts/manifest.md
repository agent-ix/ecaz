# Task 146 Packet 004 Artifact Manifest

- head SHA: `aa6012859`
- task bucket: `reviews/task-146/004-anchor-reporting-and-suite/`
- packet type: suite config / reporting contract
- timestamp: 2026-07-06
- lane: local Intel, PG18, release-substrate Task 146 follow-on
- measurement status: config and dry-run only; no benchmark results in this packet

## Artifacts

| artifact | purpose |
| --- | --- |
| `suite-task146-release-anchors.json` | `ecaz bench suite` config for matched-scale ecaz IVF/HNSW release anchors |
| `audit.log` | suite audit output |
| `dry-run.log` | dry-run command output |
| `dry-run-suite-manifest.json` | dry-run manifest emitted by the suite runner |

## Commands

```bash
jq empty reviews/task-146/004-anchor-reporting-and-suite/artifacts/suite-task146-release-anchors.json
target/release/ecaz bench suite audit --config reviews/task-146/004-anchor-reporting-and-suite/artifacts/suite-task146-release-anchors.json
target/release/ecaz bench suite run --config reviews/task-146/004-anchor-reporting-and-suite/artifacts/suite-task146-release-anchors.json --dry-run --manifest-output reviews/task-146/004-anchor-reporting-and-suite/artifacts/dry-run-suite-manifest.json --log-file reviews/task-146/004-anchor-reporting-and-suite/artifacts/dry-run.log
```

## Anchor Status

Task76 provides historical ecaz controls for 10k and 100k only:

| source | scale | engine | status |
| --- | --- | --- | --- |
| `benchmarks/task76-intel-local-spire-pareto/` | 10k | ec_ivf, ec_hnsw | available historical anchor |
| `benchmarks/task76-intel-local-spire-pareto/` | 50k | ec_ivf, ec_hnsw | missing |
| `benchmarks/task76-intel-local-spire-pareto/` | 100k | ec_ivf, ec_hnsw | available historical anchor |

Task 146 needs matched-scale frontier reporting at 10k / 50k / 100k. This packet
therefore adds a current-substrate anchor suite instead of interpolating the
missing 50k row. The matrix result packet must treat any missing anchor as a
gap, not as estimated evidence.

## Suite Shape

The anchor suite contains 24 steps:

- scales: 10k, 50k, 100k
- engines: `ec_ivf`, `ec_hnsw`
- per engine/scale: load, recall, latency, storage

All cells use `data/staged-current/ec_real_{scale}_*.tsv` and 200 queries to
match the Task 146 single-instance SPIRE matrix. IVF controls follow the Task76
release control pattern (`pq_fastscan`, `pq_group_size=8`, `heap_f32` rerank).
HNSW controls follow the Task76 release control pattern (`m=16`,
`ef_construction=128`).

## Required Result Reporting

The Task 146 result packet must report:

- ecaz IVF and HNSW anchors at 10k / 50k / 100k in the same frontier table as
  SPIRE shapes.
- the 100k IVF p50 comparison gate as a viability band, not Pareto dominance.
- both the 15% gate and a stricter 10% secondary line for row-instance scan
  fraction.
- per-node `ecaz_build_profile()` and release-guarded row profile fields for
  every latency-emitting step.
- Task 142 epoch-cache engagement via live profile fields, not assumption:
  `manifest_cache_hit_sum`, `manifest_cache_miss_sum`,
  `routing_hierarchy_load_sum`, `socket_open_sum`, and
  `endpoint_identity_query_sum`.
- non-engaged mechanisms as faulty/null evidence. If an engagement counter is
  zero, the row cannot support recall-safety, latency, or promotion claims for
  that mechanism.

## Carry-Forward From Task 145

Task 145 closeout approved do-not-promote, with a non-blocking request to carry
the structural transport-floor lesson forward. Task 146 must report that remote
scan/leaf/pruning economy does not by itself attack the floor when the invariant
counters stay fixed: dispatch fan-out and the 30000 remote heap frontier. Any
candidate promotion must therefore show it moves recall/scan/latency with live
engagement counters, and must not infer a win from unchanged remote-path
mechanisms.
