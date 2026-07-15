# Task 181 packet 003 artifact manifest

This packet records the Phase 2 fixed-cap landmark screen and the mandatory
Phase 3 bounded-hierarchy trigger at 100k.

## Provenance and commands

- Owning packet: `reviews/task-181/003-fixed-cap-policy-screen/`.
- Head SHA / installed extension SHA: `e75dfc14bbf0c1ff406a7dc1795f7e1c2f4514d8`.
- Installed profile/features: release, PG18,
  `distann-head-attribution-benchmark`; all three nodes attested unanimously.
- Corpus: `ec_real_100k`; evaluation rows 1-200, SHA-256
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Training policy input: disjoint rows 201-400 of the same staged query file;
  slice SHA-256
  `30f11df03f6e988adfe531e2bf54b75b8515fa207fee1212dd0774acffec7471`.
- Topology: three exact/disjoint physical owners, one index per table, graph
  degree 32, BW4/H100, RaBitQ search and neighbor codes.
- Fixed-screen config SHA-256:
  `7f9542a151763f53eb49c55aa50b83a332b53ac53c78a9469a5e364d15ff933f`.
- Fixed-screen results SHA-256:
  `38362e47e9c7e43fbd477b4822eb3105a029f4cd33fae8fa4322244ac5c371b9`.
- Commands:
  - `target/release/ecaz bench suite run --config reviews/task-181/003-fixed-cap-policy-screen/artifacts/fixed-cap-100k-suite.json`
  - `target/release/ecaz bench suite run --config reviews/task-181/003-fixed-cap-policy-screen/artifacts/hierarchy-100k-suite.json`
- Timestamp: 2026-07-15 PDT.

## Fixed-cap results

All policies use a 4,096-entry head and exact scoring of that membership. The
compact `fixed-cap-100k/results.jsonl` and per-step summary logs are the cited
sources.

| Policy | Distinct recall@10 (95% CI) | Warm p50 / p95 | Owner top-32 membership | Zero represented |
| --- | ---: | ---: | ---: | ---: |
| current production control (packet 002) | 0.9275 (0.9153-0.9381) | 36.80 / 50.90 ms | 0.0470 | 0.435 |
| geometry landmarks | 0.9270 (0.9148-0.9376) | 41.60 / 55.70 ms | 0.0622 | 0.135 |
| graph landmarks | 0.9160 (0.9030-0.9274) | 40.40 / 49.30 ms | 0.0345 | 0.325 |
| disjoint-training landmarks | 0.9625 (0.9532-0.9700) | 38.80 / 45.70 ms | 0.5503 | 0.015 |

Training landmarks are the fixed-cap winner. They improve recall by 0.0350
absolute over the current control, increase owner top-32 membership from
4.70% to 55.03%, and nearly eliminate queries with zero represented owner
seeds. The remaining 0.0345 absolute gap to the same-run 0.9970 owner oracle
shows that broader membership helps materially but does not meet the 0.9900
fixed-cap escape gate.

Geometry improves coarse coverage without improving recall, while graph
landmarks regress both membership and recall. Coverage count alone is
therefore insufficient; recurring query-relevant owner seeds matter.

All three fixed-screen steps passed exact/disjoint topology, remote engagement,
and unanimous provenance. Physical storage remained approximately 2.497 GB;
the 4,096-entry head cache estimate remained approximately 25.9 MB.

## Bounded hierarchy

The fixed-cap maximum of 0.9625 is below 0.9900, so Task 181's Phase 3 trigger
fired. `hierarchy-100k-suite.json` preregisters a geometry-partitioned 16,384
entry second level with at most 256 first-level representatives scored, 16
regions opened, 512 second-level landmarks scored, and 32 seeds returned.

The suite completed successfully:

| Arm | Distinct recall@10 (95% CI) | Warm p50 / p95 | Cache estimate |
| --- | ---: | ---: | ---: |
| bounded two-level hierarchy | 0.9145 (0.9014-0.9260) | 98.10 / 108.90 ms | 103,584,578 B |
| exact 16,384-entry membership | 0.9440 (0.9330-0.9533) | 46.70 / 53.70 ms | 103,584,578 B |

The geometry hierarchy regresses 0.0480 absolute recall versus the 4,096-entry
training policy and is 2.5x slower at p50. Even exact scoring of all 16,384
geometry landmarks trails training landmarks by 0.0185 while using four times
the head cache. The hierarchy is rejected. Training landmarks with exact
bounded scoring are the Phase 5 candidate.

The hierarchy construction/publish times were 1,019,444 / 1,145,816 ms. The
head uses 101,122,048 sample bytes plus 2,462,530 graph/state bytes. Total
physical-generation storage was 2,496,626,688 bytes. Exact/disjoint topology,
remote engagement, and unanimous release provenance all passed.

`hierarchy-100k/results.jsonl`, its suite manifest, compact summary, per-arm
logs, `hierarchy-report.md`, and `hierarchy-status.log` are the cited durable
sources. Corpus/query TSVs, truth caches, node PostgreSQL logs, duplicate full fixture
logs, and stopped run directories are not committed.
