---
task: 191
packet: 003-production-full-scale
role: coder
status: review_requested
head_sha: bbea9dbd384457174af169285b7654f6c800056b
date: 2026-07-20
decision: PROMOTE
---

# Review request: full-scale production lazy10 A/B

## Decision

**PROMOTE** fixed global-ranked lazy payload windows of 10 as the production
physical `ec_distann` scan path.

The checked-in suite completed its required 10k/50k/100k matrix with 200
held-out queries (2,000 top-10 trials), 50 warm latency samples after 10
warmups, concurrency one, one shared byte-identical generation per scale, and
identical seed digests between eager and lazy arms. The manifest has a clean
runner descriptor and all three steps succeeded with no missing or stale
artifacts.

## A/B result

| Scale | Distinct recall eager/lazy | Mean eager → lazy | Mean improvement | p95 eager → lazy | p95 improvement |
| --- | --- | --- | ---: | --- | ---: |
| 10k | 0.9990 / 0.9990 | 34.00 → 21.70 ms | 36.2% | 39.70 → 25.10 ms | 36.8% |
| 50k | 0.9685 / 0.9685 | 36.90 → 22.70 ms | 38.5% | 44.20 → 26.20 ms | 40.7% |
| 100k | 0.9625 / 0.9625 | 39.00 → 23.70 ms | 39.2% | 49.20 → 27.20 ms | 44.7% |

Tail improvement is material at every scale: p99 improves by 39.5%, 47.9%,
and 49.0%; max improves by 42.5%, 49.3%, and 50.9% at 10k, 50k, and 100k.

| Scale | Remote candidates/scan eager → lazy | Payload bytes/scan eager → lazy | Reduction |
| --- | --- | --- | ---: |
| 10k | 23.68 → 6.58 | 437,606 → 121,598 | 72.2% |
| 50k | 26.36 → 6.72 | 487,133 → 124,186 | 74.5% |
| 100k | 26.84 → 6.64 | 496,003 → 122,707 | 75.3% |

Storage and construction are identical between arms because each A/B reuses
the same generation. Remote engagement and topology gates pass at every scale,
and duplicate remote candidate requests remain zero in all six physical-arm
measurements.

Attribution is non-overlapping: eager reports 50 `output_merge` samples and
zero `materialize_output_associate` samples per scale; lazy reports zero merge
samples and 100 associate samples per scale.

## Promotion rationale

Production lazy10 preserves the exact measured recall and Wilson interval at
every required scale while improving both mean and tail latency materially at
every scale. It reduces the intended remote payload work by 72%–75%, has no
storage or construction cost, and satisfies the semantic/failure evidence in
packet 002. This meets Task 191's relative Pareto gate without introducing an
absolute latency threshold.

Evidence and commands are indexed by
[`artifacts/manifest.md`](artifacts/manifest.md).
