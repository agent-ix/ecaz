---
task: 183
packet: 003-fixed-budget-coverage
role: coder
status: open
date: 2026-07-17
head: f8612af1f
---

# Review request: fixed-budget coverage result

Phase 1 found no positive recall contribution from exact-neighbor traversal, so
Phase 2 keeps RaBitQ traversal and freezes three cap-4,096 landmark builders
before reading any held-out result. All builders use only the canonical 200
training rows and the frozen index codec artifact.

Common candidate extraction ranks every corpus code for each training query by
`(RaBitQ score, vec_id)`, retains exactly the best 32, and records candidate
frequency and best rank. Evaluation rows never enter construction.

1. `training_landmarks` is the Task 182 control: descending frequency, ascending
   best rank, then ascending `vec_id`, followed by the existing deterministic
   geometry fill.
2. `training_region_balanced` assigns every unique training candidate to its
   existing 12-hyperplane geometry region. Within each region it uses the
   control ordering. It selects depth 0 from every nonempty region in ascending
   region ID, then depth 1, and so on until cap; any tail uses the existing
   geometry fill over unseen nodes.
3. `training_query_facility` treats each training query's ordered top-32 list as
   a query-relevant seed neighborhood. For rank rounds 0 through 31 it visits
   all 200 canonical query ordinals cyclically, starting round `r` at
   `(97 * r) mod 200`, and adds that query's rank-`r` seed if unseen. It stops at
   cap and uses the existing geometry fill over unseen nodes only if all 6,400
   ranked slots do not fill the cap. The coprime rotation prevents the partial
   final round from always favoring early training ordinals.

Every comparison holds exact scoring of all 4,096 persisted landmarks, 32
returned seeds, BW4/H100, graph degree 32, RaBitQ neighbor traversal, exact
final rerank, topology, corpus, training/evaluation identities, and tie-breaks
constant. The two new builders remain benchmark-only and cannot change a
production policy or persisted format.

Initial screening is at 100k. Only a useful fixed-cap winner proceeds to the
required 10k/50k/100k confirmation and Phase 3 capacity/routing conditions.
Selection is held-out recall first, then warm p50, cached bytes, and build time
for overlapping quality; training diagnostics never break an evaluation tie.

Implementation checkpoint `f8612af1f` adds the two benchmark-only policy names,
shares the frequency-control selection helper with the existing production
builder, validates suite/CLI inputs fail-closed, and emits the persisted sample
digest with benchmark head metrics. Focused PG18 feature compilation and both
policy/runner tests pass; their packet-local logs are listed in the manifest.

The checked-in suite ran three fresh 100k physical generations with 50 timed
iterations after 10 warmups. Every arm explicitly uses `head_sample_exact` so
the builder is the only intended variable.

## Result

| Policy | Recall (95% CI) | Warm p50 / p95 | Head digest | Physical bytes |
| --- | ---: | ---: | --- | ---: |
| control | 0.9625 (0.9532--0.9700) | 41.0 / 54.5 ms | `50261d76...3b54` | 2,496,659,456 |
| region-balanced | 0.9625 (0.9532--0.9700) | 42.8 / 55.1 ms | `5cf7924a...e109` | 2,496,659,456 |
| query-facility | 0.9625 (0.9532--0.9700) | 41.6 / 55.7 ms | `2d796191...2c44` | 2,496,659,456 |

All three persisted head digests differ, proving that the builders selected
different 4,096-row samples. Exact scoring nevertheless returned the same
top-32 seed IDs for every held-out evaluation query: the suite's aggregate seed
digest is byte-identical across all arms. Consequently recall is byte-for-byte
identical, while neither alternative improves the matched latency distribution.
The owner-membership rate (0.5503125), zero-representation fraction (0.015),
and exact overlap (0.51328125) are also unchanged; only bounded overlap moves
slightly downward from 0.50671875 to 0.5065625 and 0.5059375.

The two alternative coverage builders are therefore **NO-GO**. Phase 2 has no
fixed-cap winner, so Task 183's explicit prerequisite for Phase 3 is not met:
no 8,192-cap or query-conditioned-routing experiment will run. Task 183 moves
to Phase 4 latency attribution using the retained Task 182/control policy.

Please review the deterministic algorithms, cap/fill behavior, query rotation,
training/evaluation separation, unchanged query-work contract, suite
provenance, and the measured no-go decision. The packet-local report,
`results.jsonl`, summaries, recall logs, and latency logs are listed in the
manifest.
