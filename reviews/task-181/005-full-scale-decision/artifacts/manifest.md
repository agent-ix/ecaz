# Task 181 packet 005 artifact manifest

This packet records the Phase 5 10k/50k/100k measurements and Task 181's
original NO-GO, superseded by packet 006 because the measurements were valid
but the proposed NFR-017 targets were not approved hard task gates.

## Provenance and execution

- Owning packet: `reviews/task-181/005-full-scale-decision/`.
- Measurement implementation / installed extension SHA:
  `e75dfc14bbf0c1ff406a7dc1795f7e1c2f4514d8`.
- Installed profile/features: release, PG18,
  `distann-head-attribution-benchmark`; every three-node step attested the SHA
  unanimously.
- Runner/config correction commit: `320e3f6ab`.
- Topology: three exact/disjoint physical owners, one index per table, graph
  degree 32, cap 4096, BW4/H100, RaBitQ search and neighbor codes.
- Evaluation: 200 held-out queries / 2,000 distinct top-10 trials per arm;
  latency uses 50 warm samples after 10 warmups at concurrency 1.
- Corrected config SHA-256:
  `0547e2ea58e23a8fde6c015ac572c4f8529a6bcc8884e0b23133b8a3adbd0310`.
- Final selected-run manifest SHA-256:
  `fa4c3feed1f219f787f1ab0ec88101f3515300bc9fbb6ba6ecef66f4afea1d68`.
- Final results SHA-256:
  `26e2d762a0c9d2d6a04ce7a81dcbc0e6b2e4d1378eda7e9e53aa9d6e45a44bbc`.
- Timestamp: 2026-07-15 PDT.

The initial config allowed the 10k current-production step to complete, then
correctly failed the candidate preflight because the 10k query file had no
rows 201-400. That successful current cell is retained under
`suite-manifest-pretraining-fix.json` (SHA-256
`f96df18eed42967dbbf9063c74b6515c190c478e0d9dafb53520d834f76c571d`)
and `current-10k-report.md`. Before any candidate cell was produced, commit
`320e3f6ab` froze the already-used, disjoint 100k rows 201-400 training slice
for 10k; exact line comparison found zero overlap with the 10k evaluation
file. The corrected config was re-audited and the remaining five steps ran via
`--only`, producing the final manifest/results. No measurement is fabricated
or merged across manifests.

Commands:

- `target/release/ecaz bench suite run --config reviews/task-181/005-full-scale-decision/artifacts/full-scale-suite.json`
- `target/release/ecaz bench suite run --config reviews/task-181/005-full-scale-decision/artifacts/full-scale-suite.json --only candidate-10k --only current-50k --only candidate-50k --only current-100k --only candidate-100k`

## Recall and latency

| Scale | Arm | Distinct recall@10 (95% CI) | Warm p50 / p95 / p99 / max |
| --- | --- | ---: | ---: |
| 10k | current production | 0.9990 (0.9964-0.9997) | 33.0 / 40.9 / 43.1 / 43.9 ms |
| 10k | training exact | 0.9990 (0.9964-0.9997) | 34.1 / 39.4 / 43.5 / 46.4 ms |
| 10k | owner oracle | 0.9995 (0.9972-0.9999) | 262.6 / 273.9 / 278.0 / 279.5 ms |
| 50k | current production | 0.9545 (0.9445-0.9628) | 42.6 / 57.3 / 59.8 / 60.3 ms |
| 50k | training exact | 0.9685 (0.9599-0.9753) | 35.8 / 47.8 / 51.5 / 51.7 ms |
| 50k | owner oracle | 0.9970 (0.9935-0.9986) | 1185.9 / 1214.5 / 1235.2 / 1236.0 ms |
| 100k | current production | 0.9275 (0.9153-0.9381) | 42.2 / 56.1 / 58.3 / 59.1 ms |
| 100k | training exact | 0.9625 (0.9532-0.9700) | 39.8 / 52.3 / 58.4 / 58.7 ms |
| 100k | owner oracle | 0.9970 (0.9935-0.9986) | 2449.5 / 2489.0 / 2500.7 / 2505.9 ms |

Training landmarks are recall-neutral at 10k, gain 0.0140 at 50k, and gain
0.0350 at 100k. They also improve p50 at 50k/100k relative to current, but
remain 0.0305 below the proposed target at 50k and 0.0365 below it at 100k. The
100k p50 is 2.2 ms above the proposed 37.6 ms IVF comparison anchor.

## Coverage and storage

| Scale | Policy | Owner top-32 membership | Zero represented | Physical generation bytes | Head cache bytes |
| --- | --- | ---: | ---: | ---: | ---: |
| 10k | current | 0.4483 | 0.000 | 242,745,344 | 25,794,572 |
| 10k | training | 0.4323 | 0.000 | 242,745,344 | 25,826,079 |
| 50k | current | 0.0098 | 0.835 | 1,242,750,976 | 25,814,193 |
| 50k | training | 0.3334 | 0.015 | 1,242,734,592 | 25,900,394 |
| 100k | current | 0.0470 | 0.435 | 2,496,659,456 | 25,894,567 |
| 100k | training | 0.5503 | 0.015 | 2,496,659,456 | 25,892,163 |

Control index bytes were 24,576 at every scale. Coordinator source bytes were
166,699,008 / 833,208,320 / 1,666,326,528 and single-index bytes were
115,687,424 / 444,186,624 / 854,810,624 at 10k/50k/100k. All ready/published
ownership sums, topology gates, remote engagement probes, and release
provenance checks passed.

Current/training construction and publish times were 69,447/82,045 and
71,521/84,160 ms at 10k; 394,122/457,262 and 401,917/466,038 ms at 50k; and
852,764/980,491 and 857,613/984,436 ms at 100k.

## Decision

Original decision (superseded): NO-GO. The precisely bounded training-landmark
candidate reaches the proposed 0.9990 target only at 10k and is above the
proposed 100k 37.6 ms p50 comparison anchor. The hierarchy was worse, and the
residual exact-neighbor trigger did not fire. The owner oracle remains
diagnostic and O(N). This original decision passed no candidate to Task 182;
packet 006 reverses that disposition based on the relative A/B improvement.

The compact suite manifests/results, reports, per-step summaries, and cited
recall/latency logs are retained. Corpus/query TSVs, truth caches, PostgreSQL
node logs, duplicate full fixture logs, and the reusable run directory are not
committed.
