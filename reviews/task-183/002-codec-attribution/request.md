---
task: 183
packet: 002-codec-attribution
role: coder
status: open
date: 2026-07-17
head: 229e7d7a5
---

# Review request: same-seed codec attribution

This checkpoint adds the missing fail-closed provenance contract for Task 183
Phase 1. A benchmark-only extension endpoint hashes the ordered seed IDs chosen
by the same helper used by production scans. The suite hashes those per-query
digests over the ordered 200-query evaluation set and rejects neighbor-score
arms with the same seed strategy/search width/count if their digests differ.

The checked-in suite config builds one production `training_landmarks_exact`
100k generation and measures three arms against it:

1. persisted trained seeds with normal RaBitQ neighbor traversal;
2. the byte-identical persisted trained seeds with benchmark-only exact
   neighbor traversal; and
3. the O(N) owner scan with normal RaBitQ traversal as a non-selectable upper
   reference.

All arms hold cap 4,096, exact head scoring, 32 returned seeds, BW4/H100, graph
degree 32, top-10, three owners, the staged corpus/training/evaluation identity,
and exact final rerank constant. The measurement uses 200 held-out queries and
50 warm latency samples after 10 warmups. No production policy, format,
default, traversal budget, graph, or codec changes in this packet.

Please review the shared seed-selection refactor, digest domain/order, suite
fail-closed equality check, and the single-generation attribution config.
Measurement results will be appended to this packet after the committed config
and release build complete.

PG18 benchmark-feature compilation, CLI compilation, both fail-closed digest
unit tests, and the checked-in suite dry run pass. The dry run expands exactly
one physical-generation step and the three pre-registered variants above; the
packet-local logs and generated suite manifest are recorded in
`artifacts/manifest.md`.

The first release invocation stopped before generation construction because
the fixture's 100k bulk load raced its own autovacuum and the production build
gate correctly returned `EC_BUILD_BUSY`. The fixture now disables autovacuum
for the fresh source and TOAST tables and performs explicit `ANALYZE` before
the build. This setup remediation is committed separately at `04cad857e`; it
does not alter the index, build gate, measured algorithms, or work budgets.
