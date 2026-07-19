# Task 184 packet 004 artifact manifest

## Provenance

- Task bucket / packet: `reviews/task-184/004-full-scale-decision/`
- Lane / fixture: Intel-local loopback, three PG18 physical owners, one
  immutable hash-sharded generation per scale; not a shared-table surface
- Index / format / rerank: `ec_distann`, persisted exact-scored
  `training_landmarks_exact` head (cap 4,096, 32 returned seeds), BW4/H100,
  RaBitQ neighbor records/traversal, exact final ranking
- Candidate extension head: `765f28a548b194f1bd1ba845ae06b2266d04153b`
- Installed extension: same full SHA, release profile, unanimous across all
  three nodes at every scale
- Runner correctness checkpoint: `b51b0ad4795290731bf1b8044117701af6527c8a`
- Decision / roadmap checkpoint: `98ff482f6209d1da2f12bd1ebc3d20ed4861ae69`
- Suite config SHA-256:
  `e2ee9b170b6ef716e632344f019116cc251dd3ccaa13974547932d5f81a335b0`
- Final results SHA-256:
  `3c5d812d5fbb4cd59ef5863b6700d5db0e0873e46284fca17cd583957faf5427`
- Execution window: 2026-07-19 12:54:23 through 13:23:18 PDT
- Command:
  `target/release/ecaz bench suite run --config reviews/task-184/004-full-scale-decision/artifacts/full-scale-suite.json`
- Measurement shape: 200 held-out queries / 2,000 distinct top-10 trials;
  50 warm latency samples after 10 warmups; concurrency one
- Result: all three selected steps succeeded; suite audit passed 3/3

The final suite manifest reports runner state as `b51b0ad...-dirty` because the
runner updates the tracked `full-run/suite-manifest.json` before collecting its
Git descriptor. The executed release CLI was built from committed checkpoint
`b51b0ad...`; this self-generated output dirtiness does not affect the installed
extension attestation, which is clean, unanimous, and emitted independently by
each scale's three PostgreSQL nodes.

## A/B result

| Scale | Recall eager / lazy10 (95% Wilson CI) | Mean ms | p50 ms | p95 ms | p99 ms | Max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 0.9990 / 0.9990 (0.9964–0.9997) | 34.10 / 20.70 | 34.10 / 20.40 | 40.10 / 23.80 | 43.20 / 25.00 | 44.70 / 25.30 |
| 50k | 0.9685 / 0.9685 (0.9599–0.9753) | 36.00 / 22.20 | 35.00 / 21.90 | 42.60 / 24.90 | 49.30 / 25.90 | 51.20 / 26.80 |
| 100k | 0.9625 / 0.9625 (0.9532–0.9700) | 38.30 / 22.40 | 38.30 / 22.20 | 48.40 / 25.60 | 52.90 / 26.30 | 54.40 / 26.80 |

| Scale | Remote materialize ms eager / lazy10 | Remote rows requested/query | Payload bytes/query | Physical generation bytes | Physical / publish / single build ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 22.497 / 9.901 | 23.68 / 6.58 | 437,606 / 121,598 | 242,745,344 | 70,971 / 83,556 / 14,859 |
| 50k | 24.244 / 10.037 | 26.36 / 6.72 | 487,133 / 124,186 | 1,242,734,592 | 398,792 / 462,559 / 157,466 |
| 100k | 25.596 / 10.179 | 26.84 / 6.64 | 496,003 / 122,707 | 2,496,659,456 | 854,786 / 982,206 / 394,675 |

All pairs report identical evaluation query SHA, seed digest, head sample
digest, generation/storage values, topology, and release provenance. Topology
passed with three physical owners and two explicitly verified remote owners at
each scale. Materialization engagement passed for eager and lazy10.

## Artifacts

| Artifact | Purpose / cited result |
| --- | --- |
| `full-scale-suite.json` | Checked-in `ecaz bench suite` definition for the isolated 10k/50k/100k eager-vs-lazy10 matrix |
| `suite-dry-run.log` | Preflight expansion of all three scales and both explicit materialization arms |
| `suite-audit-preflight.log` | Pre-run input and shape audit |
| `full-run/suite-manifest.json` | Final non-dry manifest: 3 selected, 3 succeeded, 0 failed |
| `full-run/results.jsonl` | 279 structured suite result rows; sole structured source for the tables above |
| `full-run/materialization-ab-10k/distann-multinode-summary.log` | Compact 10k recall, latency, stage/work, storage/build, topology, engagement, and provenance evidence |
| `full-run/materialization-ab-50k/distann-multinode-summary.log` | Compact 50k evidence on the same fields |
| `full-run/materialization-ab-100k/distann-multinode-summary.log` | Compact 100k evidence on the same fields |
| `suite-audit.log` | Post-run audit: 3 steps pass |
| `suite-status.log` | Post-run status: all three steps succeeded |
| `suite-report.md` | Human-readable projection of the final structured results |

Regenerable per-arm recall/latency logs, PostgreSQL logs, and the suite driver's
combined child logs were pruned after `results.jsonl` and the compact summaries
were finalized. No corpus, query, ground-truth, polling, tunnel, or raw SSM data
is committed.

## Decision

**PROMOTE to a separately reviewed productionization task.** Fixed batch 10 is
the selected MAT-01/MAT-02/MAT-04 realization. It preserves observed recall and
all required semantics while materially improving end-to-end mean and tails at
every required scale, moving the attributed target consistently, reducing
remote work/bytes, and changing neither storage nor construction. The 1m cell
is conditionally skipped because no attested 1m corpus is staged on this host.
ADR-085 D12 records the fixed window and existing corpus-independent deepening
ceiling; Task 191 owns the normative NFR-019 reconciliation and production
default. Task 187 waits for that productionized retained baseline.
