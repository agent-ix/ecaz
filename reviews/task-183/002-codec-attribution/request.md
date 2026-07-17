---
task: 183
packet: 002-codec-attribution
role: coder
status: open
date: 2026-07-17
head: 04cad857e
---

# Review request: same-seed codec attribution result

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
The release suite completed its one 100k step with no failed, skipped, missing,
or stale artifacts. The two trained arms emitted the identical aggregate seed
ID digest
`488caa73ad3f6c22864f9af309569ba4fe6edd72c8d535e71eec7bff78af6d50`.
The suite would have failed before results if those digests differed.

| Arm | Distinct recall@10 (95% CI) | Warm p50 / p95 / p99 / max |
| --- | ---: | ---: |
| trained RaBitQ | 0.9625 (0.9532--0.9700) | 43.8 / 55.6 / 62.3 / 63.2 ms |
| same-seed exact neighbor | 0.9605 (0.9510--0.9682) | 113.1 / 141.7 / 173.9 / 182.7 ms |
| owner-scan RaBitQ oracle | 0.9970 (0.9935--0.9986) | 2566.0 / 2601.3 / 2613.8 / 2614.3 ms |

Exact-neighbor changes recall by -0.0020 with overlapping intervals and costs
+69.3 ms / 2.58x trained p50. It therefore provides no evidence that replacing
RaBitQ neighbor scoring recovers the residual recall, and it is a measured
NO-GO for further codec work in this task. The owner oracle retains +0.0345
recall headroom but costs about 58.6x trained p50, confirming that bounded
entry coverage remains the useful next direction.

All arms share the same 2,496,626,688-byte physical generation, 25,892,203-byte
cached-head estimate, 907,537 ms physical construction, and 1,041,779 ms
publish time. All three owners unanimously attested installed release extension
SHA `c9011fb8b1e0e11ec2b94f2e45b29c0f0299f714`; Ready/Published topology,
physical serving, two remote materialization probes, and engagement gates
passed.

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

Decision: close Phase 1 and proceed to the pre-registered cap-4,096 coverage
policies in Phase 2. Do not advance exact-neighbor scoring or a new production
neighbor codec from this result.
