---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 exact-ground-truth post-insert gate

Status: review-open; code and suite-instrument remediation complete; production
10k/50k/100k execution pending from this packet.

Please review checkpoint `f83110078` against
`reviews/task-167/039-post-insert-parity-gate/feedback/2026-08-22-01-reviewer.md`.

The pairwise ANN-overlap gate is removed. The replacement constructs exact
fp32 inner-product ground truth over the staged corpus plus the 320 rows
actually inserted by the disabled and enabled throughput arms. It reports the
incrementally built physical index and a same-row fresh rebuild side by side,
and verifies with `EXPLAIN` that the two arms use
`EcDistannDistributedScan` and a local `Index Scan`, respectively.

The result is split into two populations: 48 inserted-neighborhood queries and
152 held-out queries. Held-out queries are required to outnumber inserted
queries before the run starts. Exact duplicate truth slots are collapsed by
source fingerprint, the denominator is computed per query from the distinct
exact-truth keys, and emitted `truth_slots`, `truth_distinct_keys`, and
`truth_duplicate_slots` make the duplicate effect visible. The obsolete
`append_enabled_minus_disabled` claim is gone because both throughput arms
intentionally share one physical index; the gate makes no false per-arm
quality attribution.

The fresh index explicitly matches graph degree, head cap, build shards, head
construction, neighbor codec, and optional head-sizing reloptions. A physical
recall degradation greater than 0.002 versus the fresh index fails the process
for either population.

The insert workload now records the actual successful INSERT count, checks all
three arms against the preregistered 160-row sample, emits that observed count,
and uses it for counter bounds. A live PostgreSQL fixture preflight evaluates
32 generated synthetic vectors and fails unless their computed L2 norms are
within `1e-5` of one.

Packet 040 separately fixed the extension-level reset alias, documented the
coordinator-backend-only counter scope, and placed retry attribution behind an
off-by-default GUC. Together, packets 040 and 041 address findings 1–7.3.

Packets 027–030 are explicitly superseded by packets 031–041. No current claim
depends on their review-open requests; please disposition those historical
packets as superseded when reviewing this round, satisfying feedback section 8.

The corrected release matrix is
[`artifacts/task167-exact-recall-suite.json`](artifacts/task167-exact-recall-suite.json).
No runtime result is claimed yet.
