# Review request: persisted-head versus owner-scan A/B

## Scope

Please review this isolated 10k/50k/100k measurement packet for the remaining
half of packet 030 P2-1: current persisted-head seeding versus the removed
owner-wide O(N) seed scan.

Both arms use the exact same source head and release runner
`24ec63788`. The baseline enables only the default-off
`distann-legacy-seed-benchmark` control from implementation commit
`2bf203e4c`; the candidate installs normal production features. Build-time
generation construction, persisted head data, current bounded/concurrent
transport, graph search, corpus, queries, and suite parameters are otherwise
identical.

Every scale uses three physical owners, graph degree 32, head index cap 4096,
20 recall queries (200 recall trials), 10 untimed same-connection warmups, and
50 measured latency queries at concurrency 1. The suite requires the runtime
rows to identify the arms as `owner_scan` and `persisted_head`.

## Result

Persisted-head seeding removes the per-query O(N) owner scans and produces a
large, scale-increasing warmed-latency reduction. It is not recall-neutral:

| Scale | Recall owner scan → persisted head | Recall delta | Mean ms owner scan → persisted head | Mean delta | p95 ms owner scan → persisted head | p95 delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 → 1.0000 | 0.0 pp | 283.20 → 42.40 | -85.0% | 296.20 → 55.00 | -81.4% |
| 50k | 1.0000 → 0.9800 | -2.0 pp | 1266.60 → 57.10 | -95.5% | 1301.40 → 74.40 | -94.3% |
| 100k | 0.9950 → 0.9500 | -4.5 pp | 2613.40 → 50.90 | -98.1% | 2663.00 → 69.10 | -97.4% |

At 50k this is four fewer correct top-10 memberships across 200 trials; at
100k it is nine fewer. The candidate values reproduce packet 038's cap-4096
recall rather than introducing a new regression relative to the accepted
head-cap matrix, and all configured recall floors remain satisfied. The A/B
nevertheless establishes that the bounded head trades recall for bounded
per-query work as scale grows.

Physical storage is identical at 10k and 100k. At 50k the candidate differs
by one 8 KiB page across approximately 1.24 GB (+0.000659%), consistent with
relation page-allocation noise rather than a format difference. Same-run
single-index mean latency moves +4.5%, -9.5%, and 0.0%, which does not explain
the 85-98% physical improvement. See `artifacts/comparison.md` and
`artifacts/manifest.md` for p99, build-time, storage, topology, and provenance
details.

## Validation state

Both suite manifests report three completed steps, zero failures, zero missing
artifacts, and zero stale steps. All 24 configured topology, recall, latency,
remote-engagement, and strategy-specific thresholds pass. Every scale has the
exact source row count across three Published owners, zero non-owned rows,
zero orphans, and both remote owners verified.

## Requested decision

Please confirm that this matrix closes the outstanding packet 030 P2-1
measurement requirement and explicitly decide whether the measured
scale-dependent recall cost is acceptable under FR-080's retained cap-4096
outcome. This packet does not by itself close Task 179 or any unrelated open
review finding.
