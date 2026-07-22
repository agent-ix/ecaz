---
task: 193
packet: 005-owner-plan-candidate
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Task 193 owner payload prepared-plan candidate

This checkpoint implements and pre-registers the remaining in-scope MAT-19
candidate, refined by MAT-20: reuse the owner payload endpoint's prepared SPI
statement within one retained generation, keyed by the immutable generation
fingerprint and the exact projection/SQL fingerprint.

The cache is benchmark-feature-only and defaults off. Its lifecycle is owned by
the retained-generation entry, so generation invalidation drops the cache while
an already-running scan may safely retain its existing entry. The cache has a
four-plan LRU bound. The control and candidate use identical generations,
seeds, RaBitQ neighbor values, lazy10 materialization, schema-validation mode,
and BW=4/H=100 traversal; only prepared-plan reuse differs.

Pre-registered prediction: `owner_payload_sql_work` and end-to-end latency
should fall, while open/validate, node lookup, traversal counters, recall,
storage, ordering, and failure semantics remain unchanged. A stage-local win
without an end-to-end latency improvement is a STOP. A useful 100k result
advances to the required 10k/50k/100k matrix; otherwise the measured negative
result closes the candidate.

The first run attempted the optional promotion correctness drills after the
performance A/B. The cache-off lazy10 control exposed a pre-existing
stable-prefix duplicate-request invariant before the cache-on semantic arm;
this is not attributable to MAT-19. Because the performance result failed the
pre-registered usefulness gate and therefore cannot be promoted, the decision
rerun omits those promotion-only drills and records the baseline failure as a
separate follow-up rather than expanding this candidate's scope.

## Decision: STOP

The successful decision rerun completed at 100k with no failed, missing, or
stale artifacts. Prepared-plan reuse preserved recall (`0.9625/0.9625`) and
storage (`2,496,659,456` bytes both), but moved warm mean only
`23.60 -> 23.50 ms` and p95 `26.80 -> 26.40 ms`. The intended stage moved
only `8.746651 -> 8.599735 ms/scan` (0.147 ms, 1.7%).

This fails the pre-registered usefulness gate, so MAT-19/MAT-20 do not advance
to full scale or productionization. MAT-23/MAT-24 remain the already-existing
production batching mechanism. Task 196 owns the independent cache-off lazy10
duplicate follow-up.

Implementation: `e444f6474`.
Evidence metadata and the checked-in suite are in `artifacts/manifest.md`.
