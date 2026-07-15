# Task 181: ec_distann Head-Coverage and Landmark Selection Benchmark

Status: proposed (2026-07-15). Depends on Task 180's reviewed NO-GO and the
completed Task 179 physical hash-shard lane. Feeds conditional Task 182 and
Task 172's distributed performance gate. Priority: P1 measurement-first
quality follow-up.

## Why

Task 180 isolated the 100k bounded-head recall loss:

| Entry strategy | Distinct recall@10 | Warm p50 | Query-time entry work |
| --- | ---: | ---: | --- |
| persisted cap 4096, width32/seeds32 | 0.9275 | 40.3 ms | bounded |
| exact scoring of the same cap-4096 sample | 0.9275 | 42.2 ms | bounded |
| persisted cap 4096, width64/seeds64 | 0.9280 | 40.9 ms | bounded |
| exact scoring of a cap-16384 sample | 0.9440 | 45.2 ms | bounded, 4x head |
| full-owner RaBitQ seed scan | 0.9970 | 2445.2 ms | O(N), diagnostic only |

Head-search width 32/64/128/256 and returned seed counts 32/64/128 were flat.
Exact scoring cannot select useful entry nodes that are absent from the
persisted sample. Conversely, the owner oracle uses the same graph, BW4/H100,
and RaBitQ search/neighbor codes and nearly recovers the quality floor when it
can choose seeds from every node. The next uncertainty is therefore whether a
better fixed-cap landmark set can cover the query-relevant graph regions, or
whether a bounded hierarchical head is required.

Task 180 closes as a measured NO-GO. Do not reopen it, promote width64/seeds64,
or reinterpret its owner scan as a production candidate.

## Goal

Measure the bounded head's entry-region coverage, compare deterministic
coverage-aware landmark policies at fixed query work, and issue a reviewed
GO/NO-GO for exactly one production candidate to conditional Task 182.

This task is measurement-only. It may add benchmark-only construction,
inspection, and suite surfaces, but SHALL NOT change the production head
policy, production defaults, persisted format, query path, graph, traversal
budget, or neighbor codec.

## Required diagnostic surface

Extend `ecaz bench suite` and the existing `distann-local-multinode` step. Do
not add packet-local sweep scripts. For each held-out query, compute compact
aggregate diagnostics comparing the bounded head with the same-run owner
oracle:

1. owner-oracle top-seed membership and overlap@k in the bounded landmark set;
2. overlap between bounded returned seeds and owner-oracle returned seeds;
3. best bounded-seed versus best owner-seed score gap and rank histogram;
4. fraction of queries with zero owner-oracle seeds represented in the head;
5. coverage broken down by deterministic corpus/graph region, never by owner
   alone (hash ownership is not a semantic cluster); and
6. landmark selection frequency, duplicate suppression, sample count, sample
   bytes, graph bytes, cached bytes, and construction time.

Raw per-query/corpus rows remain uncommitted. Emit compact JSONL result rows and
histograms with corpus, held-out-query, training-query, extension, graph, and
policy provenance. The owner oracle remains feature-gated and O(N); it is used
only to label diagnostics and is never eligible for selection.

## Query separation

Landmark policies may not train on evaluation queries.

- Preserve Task 180's 200 held-out queries and SHA as the evaluation set.
- Create or stage a separate deterministic training-query set with its own
  prefix and SHA when a policy uses query observations.
- No source row or query may cross the declared train/evaluation boundary.
- Geometry-only policies must record that they used no queries.
- Candidate selection uses evaluation results only after policy definitions,
  caps, and tie-breaks are checked into the suite config.

## Phase 1: existing-head coverage audit

On the real 100k corpus, three exact/disjoint physical owners, graph degree 32,
RaBitQ codes, BW4/H100, cap 4096, width32/seeds32, and the Task 180 held-out
queries:

1. reproduce the production persisted-head and owner-oracle cells;
2. emit the required overlap, membership, score-gap, and region histograms;
3. verify exact scoring of the unchanged head again only as a same-generation
   membership control; and
4. publish a quantitative loss decomposition before building new policies.

The phase must answer whether misses are broadly distributed, concentrated in
specific geometric regions, or dominated by a small recurring set of absent
landmarks. Owner balance alone is not a coverage finding.

## Phase 2: fixed-cap landmark screen

Implement benchmark-only, deterministic builders for the following policy
families. Freeze the exact algorithm, seed, training inputs, and tie-breaks in
code/config before measuring it:

1. `current_sample`: unchanged production control;
2. `geometry_landmarks`: a scalable diversity/covering policy such as
   hierarchical farthest-point selection or coarse-cluster medoids;
3. `graph_landmarks`: query-independent landmarks chosen from bounded graph
   coverage/centrality statistics; and
4. `training_landmarks`: a diagnostic policy that greedily covers/frequency-
   ranks owner-oracle seeds from the disjoint training queries.

Test all policies at cap 4096 with exact landmark scoring, 32 returned seeds,
and otherwise identical graph/traversal/codecs. This isolates landmark
membership. Report builder peak memory, elapsed time, deterministic digest,
head bytes, cached bytes, and evaluation-query diagnostics.

Only after a policy wins under exact landmark scoring may its persisted
head-graph search be measured at the same cap and seed count. Do not tune head
width or seed count again unless a policy first establishes a statistically
meaningful membership-recall improvement over `current_sample`.

## Phase 3: bounded hierarchy trigger

If no fixed cap-4096 policy reaches distinct recall@10 `>= 0.9900` at 100k,
test one pre-registered two-level bounded hierarchy rather than another linear
cap sweep. It must have explicit caps on:

- first-level representatives scored;
- second-level regions opened;
- total second-level landmarks scored;
- returned seeds; and
- memory/storage per level.

Hold BW4/H100 and RaBitQ traversal fixed. The hierarchy may partition by
geometry or graph coverage, not by hash owner. It must not issue per-query
owner scans, grow work with N without a declared cap, or fetch an uncapped
remote seed set.

If a fixed-cap policy reaches `>= 0.9900`, skip the hierarchy and continue with
that simpler candidate.

## Phase 4: residual traversal attribution

Run same-seed RaBitQ versus exact-neighbor traversal only when the best bounded
candidate is within `0.0050` distinct recall of the same-run owner oracle but
remains below NFR-017's `0.9990` floor. The two arms must use byte-identical
seed IDs and the same graph/BW/H/top-k. This is the only phase allowed to infer
a residual neighbor-code contribution.

Do not introduce OPQ, a new quantizer, or a different graph before this trigger
fires. Task 180 already showed that the primary gap precedes neighbor
traversal.

## Phase 5: full-scale confirmation

Promote exactly three arms to 10k/50k/100k on one generation per scale:

1. unchanged production `current_sample`;
2. benchmark-only owner oracle; and
3. the best bounded fixed-cap or hierarchical candidate.

Use at least 200 held-out queries / 2,000 distinct top-10 trials and 50 warm
latency measurements after 10 warmups at concurrency 1. Record recall and
Wilson interval, p50/p95/p99/max, build/publish time, total physical/control/
source/single-index bytes, per-level head bytes/cache, topology, remote
engagement, and unanimous installed release provenance.

Selection order is: highest evaluation distinct recall, then lowest warm p50
among overlapping recall intervals, then lowest cached-head bytes, then lowest
construction time. Training diagnostics never break an evaluation tie.

## Decision gate

Issue GO to Task 182 only if one precisely identified bounded candidate:

1. performs no O(N) query-time scan and declares all query-work caps;
2. reaches distinct recall@10 `>= 0.9990` at 10k, 50k, and 100k;
3. reaches 100k warm p50 `<= 37.6 ms` and p95 `<= 3x` its own p50;
4. passes exact/disjoint topology and remote-engagement gates at every scale;
5. reports all head levels' storage, cache, and construction costs;
6. reproduces under a clean release build with per-node unanimous provenance;
   and
7. has a frozen deterministic policy contract that Task 182 can implement
   without post-hoc algorithm selection.

If no candidate passes, close Task 181 with a reviewed NO-GO. Do not weaken
NFR-017, promote training-query leakage, or substitute the owner oracle.

## Evidence and packet rules

Required review packets:

1. `reviews/task-181/001-coverage-landmark-plan/`: task/suite/diagnostic design;
2. `reviews/task-181/002-existing-head-coverage/`: Phase 1 evidence;
3. `reviews/task-181/003-fixed-cap-policy-screen/`: Phase 2 and any hierarchy
   trigger decision;
4. `reviews/task-181/004-residual-attribution/`: conditional Phase 4 evidence or
   a documented non-trigger; and
5. `reviews/task-181/005-full-scale-decision/`: 10k/50k/100k and GO/NO-GO.

All matrices use checked-in `SuiteConfig` files and retain manifests, results,
reports, compact cited logs, checksums, and packet-local artifact manifests.
Corpus/query TSVs, truth caches, node logs, polling exhaust, and run directories
remain banned from commits.

## Non-goals

- Production implementation or default changes (conditional Task 182).
- Another width/seed sweep over the unchanged Task 180 sample.
- Linear cap growth as the primary strategy.
- OPQ, new quantizers, codec promotion, graph replacement, or Task 167 DML.
- Task 172 throughput, telemetry-overhead, injected-RTT, or capacity work.

## References

- Task 180 packets 002 and 003: attribution screen and full-scale NO-GO.
- Task 179 packet 048: owner scan versus persisted head.
- FR-080: bounded coordinator head-index behavior.
- FR-081: distributed hop-round orchestration.
- NFR-017: distinct-recall and matched-latency release gate.
- NFR-018: physical storage accounting.

