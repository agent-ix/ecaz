---
task: 182
packet: 006-production-ab
role: coder
status: open
date: 2026-07-16
head: 8769d5783
---

# Review request: Task 182 production A/B plan

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

Measurements are pending. This request and manifest will be updated with the
actual measurement head SHA, suite manifest, results, compact logs, and result
summary after the run.
