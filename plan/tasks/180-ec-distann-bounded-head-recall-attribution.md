# Task 180: ec_distann Bounded-Head Recall Attribution Benchmark

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: completed — measured negative for width/seed tuning (corrected
2026-07-15). The selected width64/seeds64 arm was statistically flat in recall
and slower than unchanged production at 100k. The original rationale also
cited proposed NFR-017 targets as hard gates; those targets were not
stakeholder-approved. Depends on the completed Task 179 physical hash-shard
lane and feeds Task 181's completed landmark-selection follow-up.

## Why

Task 179 deliberately accepted a large, measured recall/latency tradeoff when
it replaced the per-query full-owner seed scan with the bounded persisted
coordinator head:

| Scale | owner-scan recall@10 | persisted-head recall@10 | persisted-head warm p95 |
| --- | ---: | ---: | ---: |
| 10k | 1.0000 | 1.0000 | 55.0 ms |
| 50k | 1.0000 | 0.9800 | 74.4 ms |
| 100k | 0.9950 | 0.9500 | 69.1 ms |

The owner scan and persisted head used the same graph, query budget, and
RaBitQ neighbor codes. Their 100k difference therefore establishes seeding as
the primary source of the 4.5-point loss; it does not implicate a codec change.
The separate cap sweep found only a 0.005 improvement from cap 256 to 4096 at
100k. What remains unknown is whether bounded recall is lost because the
persisted sample omits useful entry regions, because approximate search of the
sample fails to find its best entries, or because the fixed 32 returned seeds
are too narrow.

Task 179 is closed and SHALL NOT be reopened for this quality investigation.
Task 172 owns the broader throughput, full-telemetry, and capacity program;
this task supplies the narrower attribution and Pareto decision needed before
any bounded-head implementation change is justified.

## Goal

Use first-class `ecaz bench suite` arms to decompose the bounded-head recall
loss, identify the best bounded recall/latency point, and issue a reviewed
GO/NO-GO decision for a later production implementation task.

Task 180 is measurement-only. It may add benchmark-only diagnostics and suite
configuration surfaces, but it SHALL NOT change the production seed strategy,
default reloptions/GUCs, persisted format, or neighbor codec. A proven winner
must be implemented and re-measured in a separately numbered task.

## Required benchmark surfaces

Extend the `distann-local-multinode` suite step rather than adding packet-local
scripts. Every normalized result row must identify the seed mode, head cap,
head-search width, returned seed count, beam width, hop rounds, neighbor-score
mode, installed extension SHA/profile on every node, and corpus/query identity.

The runner must support these diagnostic arms:

1. `persisted_head`: the unchanged production path.
2. `head_sample_exact`: exact inner-product scoring of every vector in the
   already-persisted bounded sample, returning the requested number of best
   seeds. This isolates sample coverage from approximate head-graph search.
3. `owner_scan`: the existing benchmark-only O(N) full-owner oracle. It is a
   reference, never a promotable candidate.
4. `exact_neighbor`: a benchmark-only, fixed-seed traversal oracle that scores
   neighbor vectors exactly. This arm is conditional under the trigger below
   and must remain unavailable in normal production builds.

Decouple head-search width and returned seed count from
`ec_distann.beam_width`; varying either must not silently change BW, H, top-k,
head cap, graph degree, codec, or corpus. Emit the persisted sample count and
the separately accounted head-sample/head-graph bytes so cap growth is not
mistaken for storage neutrality.

## Phase 1: 100k attribution screen

Use the real 100k staged corpus, three physical owners, graph degree 32,
RaBitQ neighbor codes, BW4/H100, the same held-out query set, and exact/disjoint
topology preflight for every arm. Change one axis per A/B.

1. Reproduce production cap-4096 `persisted_head` and the `owner_scan` oracle.
2. Compare production head-graph search with `head_sample_exact` at cap 4096,
   head-search width 32, and 32 returned seeds.
3. At cap 4096 and 32 returned seeds, sweep head-search widths
   `32, 64, 128, 256`.
4. Using the best width from step 3, sweep returned seed counts
   `32, 64, 128`.
5. If exact cap-4096 sample recall remains below `0.9900`, build exact-sample
   arms at caps `8192` and `16384`, holding width/seed behavior fixed. These
   arms determine whether bounded sample coverage, rather than search, is the
   limiting factor.
6. Run `exact_neighbor` only when the best bounded seeding arm is within
   `0.0050` of the same-run owner-scan oracle but still below NFR-017's
   proposed `0.9990` recall target. Use the exact same seeds in RaBitQ and
   exact-neighbor arms so the result attributes only traversal scoring.

The screen uses at least 200 held-out queries / 2,000 top-10 membership trials.
Latency arms use the exact NFR-017/Task-146 cache and concurrency protocol;
each arm includes the standard warmup and enough measured queries to report
p50/p95/p99 without reusing recall-workload means as latency percentiles.

## Phase 2: full-scale confirmation

Promote exactly three arms from the screen into a 10k/50k/100k matrix:

- unchanged production `persisted_head`;
- benchmark-only `owner_scan` oracle; and
- the best bounded candidate selected by the pre-registered order below.

Select the bounded candidate by: (1) highest distinct recall@10, then
(2) lowest warm p50 among cells whose recall confidence intervals overlap,
then (3) lowest total head bytes. Do not choose a cell from a single favorable
latency sample or stack unmeasured settings.

At every scale record:

- distinct recall@10 and the membership-recall metric retained for continuity;
- warm p50/p95/p99/max under the NFR-017 protocol;
- physical-generation, control-index, head-sample, head-graph, source, and
  same-run single-index bytes;
- build and publish time, measured sample count, and estimated cached-head
  memory;
- exact owner coverage, placement balance, zero residue/orphans, and remote
  expansion/materialization engagement; and
- per-node installed extension SHA/profile with unanimity enforced before
  measurements are accepted.

All matrices and sweeps must be driven by checked-in `SuiteConfig` files and
retain `suite-manifest.json`, `results.jsonl`, compact cited raw logs, audit,
report, checksums, and a packet-local artifact manifest. Corpus TSVs, truth
caches, node logs, polling exhaust, and regenerable run directories remain
banned from commits.

## Decision gate

Issue GO for a later bounded-head implementation task only if the selected
bounded candidate:

1. performs no O(N) per-query owner scan and has an explicit cap on all
   query-time head work;
2. demonstrates a reproducible recall improvement over unchanged production
   where bounded-head coverage is deficient without regressing another scale;
3. reports matched 10k/50k/100k warm latency and storage, with no material cost
   that negates the recall improvement;
4. passes the physical topology and remote-engagement gates at every scale;
5. reports rather than hides head storage/memory and build-time growth; and
6. reproduces under a clean release build with machine-attested extension
   provenance.

The proposed NFR-017 values (`0.9990` recall and the `37.6 ms` IVF anchor) are
aspirational comparison points, not stakeholder-approved hard task gates. If
no bounded arm provides a useful relative improvement, close Task 180 with a
reviewed NO-GO and the measured Pareto frontier. A NO-GO is a valid completion
outcome; do not promote owner-scan behavior or change production defaults to
manufacture a winner.

## Stop conditions

- Do not run cap-growth arms when exact cap-4096 scoring already reaches
  `0.9900`; in that case the sample has enough measured coverage and work
  stays on bounded search/seed selection.
- Do not build the exact-neighbor oracle unless Phase 1 step 6's trigger fires.
- Stop any arm whose topology or extension-provenance preflight fails; all
  downstream measurements from that arm are invalid.
- Stop a proposed production direction that requires O(N) per-query work,
  uncapped remote seed collection, or a persisted-format change. Record it as
  diagnostic evidence and split any format design into its own task.

## Non-goals

- Implementing or promoting the winning production strategy.
- OPQ, a new quantizer, or another GroupedPQ/TurboQuant comparison; Task 162
  already selected RaBitQ from measured evidence.
- Task 167 physical INSERT/UPDATE/DELETE work.
- Task 172 throughput, full distributed telemetry, instrumentation-overhead,
  injected-RTT, or 1m/10m capacity modeling.
- Cloud deployment or a new corpus.

## Required review packets

1. `reviews/task-180/001-bounded-head-recall-plan/`: task definition and
   suite-surface design review.
2. `reviews/task-180/002-100k-attribution-screen/`: Phase 1 implementation,
   exact per-axis A/B evidence, and the pre-registered Phase 2 candidate.
3. `reviews/task-180/003-full-scale-decision/`: 10k/50k/100k confirmation,
   proposed NFR-017 comparison, and GO/NO-GO closeout.
4. `reviews/task-180/004-decision-rationale-correction/`: correction separating
   the valid relative width/seed NO-GO from unapproved proposed NFR targets.

## References

- Task 179 packet 038: cap 64/256/4096 sensitivity.
- Task 179 packet 048: owner scan versus persisted head, including the
  100k `0.9950 -> 0.9500` recall attribution.
- Task 179 packet 066: BW16/H25 fixed-product negative latency result.
- Task 179 packet 072: final signed-off physical baseline and provenance.
- FR-080: coordinator head-index behavior and bounded-memory contract.
- FR-081: distributed hop-round orchestration.
- NFR-017: distinct-recall and matched-latency release gate.
- NFR-018: physical storage accounting.
