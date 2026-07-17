# Task 183: ec_distann Residual Recall and Latency Optimization

Status: **active — Phase 2 fixed-budget coverage** (2026-07-17). Phase 1 proved
byte-identical trained seeds and measured exact-neighbor traversal at 0.9605
recall / 113.1 ms p50 versus RaBitQ at 0.9625 / 43.8 ms, so codec replacement
is a NO-GO and fixed-cap entry coverage is next. Task 182 recorded its
production-path 10k/50k/100k A/B and promoted the explicit trained policy as
this task's frozen bounded-head baseline. Priority: P1 measurement-first
follow-up.

## Why

Task 182 showed that deterministic, disjoint-training landmark selection is a
real improvement on the normal production path:

| Scale | Production recall | Trained-landmark recall | Recall delta | Production p50 | Trained-landmark p50 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 0.9990 | 0.9990 | 0.0000 | 34.2 ms | 38.5 ms |
| 50k | 0.9545 | 0.9685 | +0.0140 | 44.1 ms | 39.3 ms |
| 100k | 0.9275 | 0.9625 | +0.0350 | 40.7 ms | 41.4 ms |

The same-generation owner oracle reached 0.9970 at 50k and 100k using the same
graph, BW4/H100, and RaBitQ neighbor scoring, but required an O(N) scan. At
100k the trained head represented 0.5503 of the owner oracle's top-32 seeds and
reduced queries with zero represented owner seeds from 0.435 to 0.015. This
identifies better bounded entry coverage as the primary remaining recall
direction while leaving a measurable residual traversal/codec uncertainty.

Task 182 productionized and validated the first trained 4,096-landmark
exact-scoring head. This task starts from Task 182's measured production result,
including its explicit training relation and persisted policy/count/digests.

## Goal

Decompose the recall remaining between the Task 182 bounded production head and
the same-generation owner oracle, test better entry coverage at explicitly
bounded query work, and improve latency only through separately attributable
A/B changes. Select at most one benchmark candidate for a later production
implementation task.

This task is measurement-first. Benchmark-only builders, routing policies,
oracles, counters, and suite surfaces are allowed. Do not change production
defaults, persisted formats, graph construction, traversal budgets, or neighbor
codecs in this task.

## Required baseline

Before experiments, copy no result files and rerun no stale Task 181 fixture.
Reference Task 182's immutable production A/B packet and freeze:

- the production landmark policy and deterministic digest;
- corpus, disjoint training-query, and held-out evaluation-query identities;
- head cap, scoring mode, returned seeds, BW/H, graph degree, and codec;
- physical topology, installed release provenance, and storage layout; and
- recall/CI, warm latency distribution, build/publish cost, physical bytes,
  cached-head bytes, and owner-oracle result at each scale.

If Task 182 does not reproduce a useful relative improvement, Task 183 must use
the retained production control as baseline and explicitly record why the
trained-head branch was not inherited.

## Phase 1: residual traversal and codec attribution

At 100k, run byte-identical bounded seed IDs through:

1. normal RaBitQ-neighbor traversal; and
2. benchmark-only exact-neighbor traversal.

Hold graph, BW4/H100, top-k, hop order, remote ownership, and coordinator merge
identical. Emit seed digests and fail the comparison if they differ. This A/B
runs regardless of an arbitrary distance-to-oracle threshold: its purpose is
to measure the residual RaBitQ contribution, not to presume it.

Also retain the O(N) owner oracle using normal RaBitQ traversal. It remains a
diagnostic upper reference and can never be selected.

## Phase 2: fixed-budget trained coverage

At cap 4,096 with exact head scoring and 32 returned seeds, compare the Task 182
policy with pre-registered deterministic alternatives that use only the
disjoint training set:

1. owner-seed frequency/coverage control inherited from Task 182;
2. region-balanced coverage that prevents a frequent region from consuming the
   landmark budget; and
3. clustered or facility-location coverage that rewards representing distinct
   query-relevant seed neighborhoods.

Freeze algorithms, seeds, region definitions, training inputs, and tie-breaks
before evaluation. Record owner-seed membership/overlap, zero-representation
rate, score-gap/rank histograms, landmark frequency, region balance, builder
memory/time, deterministic digest, head bytes, and cached bytes.

Select by held-out recall first; for overlapping quality results use warm p50,
cached bytes, and build time in that order. Training-set diagnostics never
break an evaluation tie.

## Phase 3: bounded capacity and query-conditioned routing

Only after Phase 2 identifies a fixed-cap winner may the task test each of the
following as an isolated A/B:

1. trained cap 4,096 versus 8,192, holding policy and returned seeds fixed; and
2. one pre-registered query-conditioned routing design over trained landmark
   groups.

The routing design must cap representatives scored, groups opened, total
landmarks scored, returned seeds, remote requests, bytes fetched, and cached
bytes. It may use geometry/training regions but not hash ownership as a semantic
cluster. It may not fall back to an owner scan or make query work grow without
a declared bound.

The earlier random-sample 16,384 head and geometry hierarchy remain measured
negative controls. Do not repeat them or infer that random cap growth validates
trained cap growth.

## Phase 4: isolated latency work

Profile the Task 182 production head and the best recall candidate into bounded
head scoring, seed preparation, local traversal, remote expansion/materialize,
merge, and executor overhead. Pursue a latency change only when the profile
identifies its target.

Eligible isolated changes include:

- contiguous/vectorized exact landmark scoring with scalar equivalence;
- a bounded coarse shortlist followed by exact scoring, with seed-identity and
  recall A/B evidence; and
- a small-corpus bypass only if Task 182 reproduces the 10k latency regression
  without a recall gain.

Measure one latency change at a time against byte-identical index/query inputs.
Do not combine a coverage policy, capacity change, routing design, and scoring
optimization into one unattributed benchmark cell.

## Full-scale confirmation

Promote no more than one bounded recall candidate and one independently
attributed latency variant to a checked-in `ecaz bench suite` A/B at
10k/50k/100k. Use at least 200 held-out queries / 2,000 distinct top-10 trials
and 50 warm latency samples after 10 warmups at concurrency 1. Record recall and
Wilson interval, p50/p95/p99/max, build/publish time, physical/control/source/
single-index bytes, all head/group bytes and cache, topology, remote engagement,
and unanimous installed release provenance.

Run 1m after the 100k candidate demonstrates a useful relative improvement and
the staged 1m corpus satisfies the same provenance/topology requirements. The
1m result is a scaling confirmation, not a substitute for 10k/50k/100k.

All matrices, sweeps, and multi-step runs use checked-in `ecaz bench suite`
configs. If the runner lacks a required arm or metric, extend it in a separate
checkpoint before measuring. Do not add packet-local sweep scripts.

## Decision

Advance a candidate to a separately numbered production task only if it:

1. is bounded and deterministic, with no owner-wide query scan;
2. demonstrates a useful, repeatable recall improvement over Task 182 at the
   deficient scales without a recall regression at another measured scale;
3. reports the complete matched latency, storage, cache, and construction
   tradeoff, without hiding a cost that negates the recall gain;
4. passes physical topology, remote engagement, query separation, and release
   provenance checks at every measured scale; and
5. has no unresolved algorithm, parameter, format, or work-cap choice.

The proposed NFR-017 recall target and IVF latency anchor are informational
comparison points, not hard task gates. The decision is based on the complete
relative Pareto result. A promising benchmark arm is not itself a production
default change.

## Required review packets

1. `reviews/task-183/001-residual-plan/`: frozen Task 182 baseline, suite design,
   attribution contract, and pre-registered policies;
2. `reviews/task-183/002-codec-attribution/`: same-seed RaBitQ/exact-neighbor and
   owner-oracle evidence;
3. `reviews/task-183/003-fixed-budget-coverage/`: cap-4,096 trained policy A/B;
4. `reviews/task-183/004-bounded-routing-capacity/`: conditional trained-cap and
   query-conditioned routing evidence;
5. `reviews/task-183/005-latency-attribution/`: profile and isolated latency A/B;
6. `reviews/task-183/006-full-scale-decision/`: 10k/50k/100k, conditional 1m,
   and advance/iterate/stop decision.

Every measurement packet follows NFR-007 and contains a packet-local artifact
manifest, suite config, suite manifest, results JSONL, report, and only the
compact cited logs. Corpus/query TSVs, truth caches, node logs, polling exhaust,
and regenerable run directories remain banned from commits.

## Non-goals

- Production implementation or default promotion.
- An O(N) owner scan, uncapped remote seeding, or hash-owner clustering.
- Evaluation-query training or post-hoc policy/parameter selection.
- Random head-cap sweeps, another unchanged width/seed sweep, or repetition of
  the measured-negative geometry hierarchy.
- Graph replacement, OPQ, or a new production neighbor codec before the
  same-seed attribution establishes that direction.
- Task 167 incremental DML or Task 172 throughput/capacity/RTT work.

## References

- Task 181 packet 006: corrected landmark-selection GO.
- Task 181 packet 005: immutable 10k/50k/100k measurements.
- Task 182 production-path A/B and closeout (required baseline).
- NFR-007: benchmark provenance.
- NFR-017: proposed recall/latency comparison targets.
- NFR-018 through NFR-020: storage, bounded work, and failure semantics.
