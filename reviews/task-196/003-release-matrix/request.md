---
task: 196
packet: 003-release-matrix
role: coder
status: review_requested
date: 2026-07-22
seq: 1
---

# Task 196 release-matrix closeout candidate

This packet requests outside closeout review and recommends **PROMOTE**. The
identity-keyed stable-prefix fix preserves exact production recall and all 78
materialization work counters at 10k, 50k, and 100k while packet 002 proves the
formerly failing real-100k rejected-prefix case now makes zero duplicate remote
requests.

## A/B result

| Scale | Recall, before -> after | Warm mean, before -> after | p95, before -> after | Storage delta | Duplicate requests |
|---|---:|---:|---:|---:|---:|
| 10k | 0.9990 -> 0.9990 | 20.90 -> 19.10 ms | 25.50 -> 22.40 ms | +16 KiB | 0 |
| 50k | 0.9685 -> 0.9685 | 20.90 -> 19.90 ms | 24.30 -> 23.10 ms | 0 | 0 |
| 100k | 0.9625 -> 0.9625 | 19.90 -> 19.80 ms | 23.30 -> 23.20 ms | 0 | 0 |

The latency observations are neutral-to-favorable, but the patch is off the
ordinary one-window path and this packet does not claim a causal speedup. The
10k storage difference is two PostgreSQL pages from independent clean builds;
the patch changes no stored representation, and 50k/100k storage is exact.

## Common provenance

The pre-fix side reuses the same-day Task 195 candidate matrix at
`reviews/task-195/002-release-matrix/artifacts/candidate`. Task 196 branched
from that accepted production code, and `custom_scan.rs` has no diff between
the Task 195 measurement head and the Task 196 base. The owning packet contains
a byte-identical copy of the same SuiteConfig (SHA-256 `7b5ae0f2...`), so both
sides use three independent PG18 owners, one index per table, exact training
landmarks, RaBitQ neighbors, lazy10, BW4/H100, 200 recall queries, 2,000 trials,
10 warmups, and 50 measured iterations.

Query, training-slice, head-sample, and seed-ID digests match at every scale.
All topology, engagement, and traversal-reconciliation gates pass. Comparing
all 78 materialization work rows produces zero mismatches.

The candidate attribution binary and runner were clean release builds at SHA
`a5e567c45`; target and installed extension were byte-identical. After the
matrix, a normal `pg18` release was restored and byte-verified. Its installed
SQL contains none of the profile endpoint, benchmark counters, or rank-shift
diagnostic fields; identity-keyed reuse remains production code.

Please review baseline reuse/common provenance, exact recall and work equality,
the storage interpretation, normal-build isolation, and packet 002's nine-case
semantic result. If accepted, Task 196 is ready to merge after its parent Task
195.
