---
task: 192
packet: 005-paired-cache-ab
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Task 192 paired owner-validation-cache A/B

The reopened candidate passes its isolated 100k decision gate and advances to
the required full-scale and epoch-safety phase. This packet supersedes the
unmeasured STOP in packet 002; it does not yet request production promotion.

## Result

One release-profile, three-owner, byte-identical-generation suite compared the
same retained `persisted_head` / RaBitQ / lazy10 point with only the benchmark
owner schema-cache switch changed.

| Metric | uncached | cached | delta |
|---|---:|---:|---:|
| distinct recall@10 | 0.9625 | 0.9625 | 0.0000 |
| warm mean | 23.90 ms | 20.80 ms | -3.10 ms (-13.0%) |
| p50 | 23.80 ms | 20.60 ms | -3.20 ms |
| p95 | 27.10 ms | 24.30 ms | -2.80 ms |
| p99 | 28.60 ms | 24.70 ms | -3.90 ms |
| custom scan total | 21.329 ms | 18.167 ms | -3.162 ms |
| owner open/validate work | 6.960 ms | 0.026 ms | -6.934 ms |
| owner endpoint critical path | 9.424 ms | 6.015 ms | -3.409 ms |
| request wait | 10.490 ms | 7.116 ms | -3.374 ms |
| remote materialization | 10.522 ms | 7.149 ms | -3.373 ms |
| owner payload SQL | 8.764 ms | 8.927 ms | +0.164 ms |

The seed-ID digest is identical in both arms
(`488caa73ad3f6c22864f9af309569ba4fe6edd72c8d535e71eec7bff78af6d50`),
the work counters are identical (including 6.64 remote result rows and two
remote owners per scan), and physical storage is shared and unchanged at
2,496,626,688 bytes. The isolated signature therefore matches the
pre-registration: open/validate and its critical path move; payload SQL,
recall, work, and storage do not.

## Provenance and scope

- runner and extension SHA: `6578da92fdf43c14742e4395d71cb570bef31501`;
- extension profile: unanimous `release` on all three nodes;
- installed and target release binaries: 24,212,288 bytes, matching SHA-256
  `4065f1f55a89b43333facf3609e91499287d8f6f175a34c649dd4933cfa90e32`;
- protocol: 200 recall queries / 2,000 trials, then 10 warmups + 50 measured
  latency iterations per arm;
- generation: one isolated physical generation, shared by both arms;
- corpus/query: `ec_real_100k`, query SHA-256
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.

The implementation is feature-gated measurement code, bounded by the existing
four-entry backend-local retained-generation cache. It caches only the resolved
immutable row-schema descriptor; relation guards, generation fingerprint,
descriptor fingerprint, requested schema fingerprint, projection attnums,
directory lookup, and payload query continue to execute on every request.

## Safety analysis and remaining gate

ADR-085 D10 and FR-082 prohibit in-place mutation of build-time row-tier tuples
or schema during the Published lifetime. Separately, PostgreSQL relcache
invalidation already evicts retained entries when the index, row-tier, graph,
directory, or all relations are invalidated. A new fingerprint cannot hit the
old entry because fingerprint is part of the key.

Before disposition, the candidate will additionally make same-index epoch
replacement explicit (observing a new fingerprint discards older entries),
exercise a real publish-cycle/stale-fingerprint drill, and run the required
10k/50k/100k recall + latency + storage suite. Those results belong in a later
packet; this packet requests review of the isolated decision only.

