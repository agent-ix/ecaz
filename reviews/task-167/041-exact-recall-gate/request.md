---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 exact-ground-truth post-insert gate

Status: review-open; corrected production 10k gate executed and found genuine
incremental graph quality loss; 50k/100k correctly stopped.

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

The production 10k run passed release provenance, live PostgreSQL
normalization, three-owner topology, remote-owner serving, rollback, update,
and the ordinary physical/single recall children (`0.9990` each). The corrected
post-insert gate then failed closed:

- inserted-neighborhood physical `0.805382` versus fresh `0.954985`, delta
  `-0.149603`; 214 of 480 exact-truth slots were duplicate source keys and the
  distinct per-query denominator handled them explicitly;
- held-out physical `0.973684` versus fresh `0.977632`, delta `-0.003947`;
- suite step exit 1 after 599,805 ms; 50k and 100k remain unrun.

This is now a supported product finding, not the prior unsatisfiable metric.
Static diagnosis found that incremental forward/backlink pruning feeds raw
negative inner-product distance into `robust_prune`, while batch construction
uses the required nonnegative `max(0, 1 - inner_product)` distance. That
algorithm correction will be a separate checkpoint and packet before rerun.

The immutable failure evidence is summarized in
[`artifacts/cited-results.log`](artifacts/cited-results.log) and traced to the
suite manifest and packet-local raw logs. This packet makes no Task 167
closeout claim.
