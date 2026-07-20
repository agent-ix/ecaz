---
task: 182
packet: 006-production-ab
role: coder
status: open
date: 2026-07-16
head: f02cf58a0
---

# Review request: Task 182 production A/B result

The checked-in `artifacts/production-ab-suite.json` is the only driver for the
Task 182 decision matrix. At each of 10k, 50k, and 100k it runs:

1. unchanged production `current_sample_graph` through the normal persisted
   head path;
2. production `training_landmarks_exact` through the new relation builder and
   manifest-selected exact persisted-head path; and
3. Task 181's benchmark-only owner scan as a diagnostic reference.

The two production arms set no benchmark seed or builder GUC. The oracle is a
separate explicitly tagged diagnostic step and cannot be promoted. Every step
uses 200 held-out queries, top-10 (2,000 distinct trials), 50 warm latency
iterations after 10 warmups, concurrency 1, BW4/H100, three owners, and cap
4,096. Each produces recall/CI, latency, physical and source storage, head
storage/cache estimate, topology, remote engagement, and installed release
provenance rows.

No hard recall or latency threshold is encoded. The closeout compares the
trained production arm with unchanged production as a relative Pareto tradeoff;
the earlier 0.9990 and 37.6 ms values are reported only as context. The suite
must still show valid topology, unanimous provenance, remote engagement, and
the manifest-backed policy/count/digest attestation before its performance rows
are decision-grade.

The suite dry-run at `8769d5783` expanded all nine steps with the intended
production and diagnostic policy arguments. The generated dry-run
`artifacts/run/suite-manifest.json` is included for review.

The release suite completed all nine cells with no failed, missing, stale, or
skipped step. All cells attested release extension SHA
`f02cf58a0224dc8a420dbb4964425fe31338e1e2` unanimously, published exactly
three disjoint owners, passed both remote-owner probes, and recorded the
expected production or diagnostic policy.

| Scale | Arm | Distinct recall@10 (95% CI) | Warm p50 / p95 / p99 / max |
| --- | --- | ---: | ---: |
| 10k | current production | 0.9990 (0.9964-0.9997) | 34.2 / 39.2 / 45.8 / 45.8 ms |
| 10k | trained production | 0.9990 (0.9964-0.9997) | 38.5 / 43.3 / 48.3 / 51.1 ms |
| 10k | owner oracle | 0.9995 (0.9972-0.9999) | 264.7 / 279.4 / 282.4 / 283.9 ms |
| 50k | current production | 0.9545 (0.9445-0.9628) | 44.1 / 54.5 / 57.5 / 58.1 ms |
| 50k | trained production | 0.9685 (0.9599-0.9753) | 39.3 / 48.3 / 55.8 / 59.7 ms |
| 50k | owner oracle | 0.9970 (0.9935-0.9986) | 1229.4 / 1265.1 / 1279.6 / 1286.1 ms |
| 100k | current production | 0.9275 (0.9153-0.9381) | 40.7 / 56.2 / 58.7 / 60.1 ms |
| 100k | trained production | 0.9625 (0.9532-0.9700) | 41.4 / 53.3 / 60.2 / 62.2 ms |
| 100k | owner oracle | 0.9970 (0.9935-0.9986) | 2554.1 / 2577.5 / 2647.6 / 2696.2 ms |

Trained production exactly reproduces Task 181's recall points: neutral at
10k, +0.0140 at 50k, and +0.0350 at 100k. Relative to current production it
costs 4.3 ms p50 at saturated 10k, improves 50k p50/p95 by 4.8/6.2 ms, and
costs 0.7 ms p50 while improving p95 by 2.9 ms at 100k. Physical storage is
+16,384 / +8,192 / +0 bytes at 10k/50k/100k; physical construction is
+1.9 / +7.4 / +3.1 seconds. Cached-head estimates differ by only +31,507 /
+86,201 / -2,404 bytes.

The decision is **PROMOTE** the bounded trained exact-landmark policy as a
production-supported, explicitly selected build policy. Existing/default
current-sample builds remain byte-compatible and unchanged because trained
builds intentionally require an explicit disjoint training relation. The
owner oracle is not promoted. Its remaining +0.0285/+0.0345 recall headroom at
50k/100k costs roughly 31x/62x trained p50 and remains diagnostic input to
Task 183.

Please review the production A/B evidence, relative decision, and provenance in
`artifacts/manifest.md`, `artifacts/run/results.jsonl`, and the cited compact
per-step logs. Proposed 0.9990 recall and 37.6 ms values are context only, not
hard gates.
