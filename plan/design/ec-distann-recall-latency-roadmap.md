# ec_distann Recall and Latency Optimization Roadmap

Status: active planning ledger (2026-07-20). This document records the option
space after Tasks 180--183. It is not an ADR and does not authorize a production
default, format, protocol, or placement change. Canonical execution scope lives
in `plan/tasks/`; accepted architectural decisions live in an ADR.

## Purpose

Keep the complete optimization search space durable without turning it into one
unreviewable task. Candidate IDs in this document remain stable across tasks,
negative results, and superseding designs. A task may import only a narrow set
of IDs, must pre-register its experiment, and may advance at most one candidate
unless its task definition explicitly says otherwise.

Candidate status vocabulary:

- **active**: assigned to the next executable task;
- **unmeasured**: plausible but not yet assigned;
- **conditional**: runs only after its recorded trigger;
- **deferred**: deliberately below nearer measured work;
- **rejected**: measured or contractually invalid; do not repeat unchanged;
- **superseded**: retained for history but replaced by a more precise candidate.

## Immutable starting evidence

The retained production candidate is Task 182's explicit, default-off
`training_landmarks_exact` policy: cap 4,096, exact scoring of all landmarks,
32 returned seeds, BW4/H100, graph degree 32, normal RaBitQ neighbor scoring,
and exact final ranking.

| Scale | Current-sample recall | Trained recall | Delta | Current p50 | Trained p50 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 0.9990 | 0.9990 | 0.0000 | 34.2 ms | 38.5 ms |
| 50k | 0.9545 | 0.9685 | +0.0140 | 44.1 ms | 39.3 ms |
| 100k | 0.9275 | 0.9625 | +0.0350 | 40.7 ms | 41.4 ms |

Task 183's fresh 100k retained-policy profile measured 0.9625 recall and
40.20 ms warm mean latency:

| Stage | Mean | Wall share |
| --- | ---: | ---: |
| Remote payload materialization | 26.955 ms | 67.05% |
| Traversal | 7.918 ms | 19.70% |
| Head scoring | 2.272 ms | 5.65% |
| Executor/client residual | 2.170 ms | 5.40% |
| Other CustomScan setup | 0.702 ms | 1.75% |
| Seed selection | 0.101 ms | 0.25% |
| Output merge | 0.053 ms | 0.13% |
| Query preparation | 0.028 ms | 0.07% |

The same-generation owner oracle reached 0.9970 recall with the same graph,
BW/H, and RaBitQ traversal but used a forbidden O(N) seed scan. Same-seed exact
neighbor traversal reached only 0.9605 recall and raised p50 from 43.8 to
113.1 ms. Therefore entry coverage is the primary measured recall direction,
remote materialization is the primary measured latency direction, and a broad
neighbor-codec replacement is not currently justified.

Task 184 then selected executor-driven payload materialization in fixed global
ranked windows of 10. On matched physical generations it preserved distinct
recall and reduced warm latency as follows:

| Scale | Recall eager / lazy10 | Mean ms eager / lazy10 | p95 ms eager / lazy10 | Remote materialize ms eager / lazy10 |
| --- | --- | ---: | ---: | ---: |
| 10k | 0.9990 / 0.9990 | 34.10 / 20.70 | 40.10 / 23.80 | 22.497 / 9.901 |
| 50k | 0.9685 / 0.9685 | 36.00 / 22.20 | 42.60 / 24.90 | 24.244 / 10.037 |
| 100k | 0.9625 / 0.9625 | 38.30 / 22.40 | 48.40 / 25.60 | 25.596 / 10.179 |

The candidate also reduced remote payload bytes by 72–75%, passed the
adversarial projection/qual/null/toast/mixed-owner/outage matrix, and changed no
storage or construction cost. ADR-085 D12 records the selected semantics;
Task 191 subsequently made this the production default and release-validated
the retained baseline. Its matched A/B reproduced recall identity and improved
warm mean by 36.2%/38.5%/39.2% and p95 by 36.8%/40.7%/44.7% at
10k/50k/100k. At 100k the retained production point is 0.9625 recall and
23.70 ms mean; traversal remains 7.849 ms (33.1%), so Task 187 is unblocked.
Outside review accepted Task 184 on 2026-07-20 and carried four non-blocking
hardening requirements into Task 191: a proven external-TOAST fixture,
scan-local stable-prefix payload reuse across deepening, unambiguous stage
accounting, and pre-output runner provenance capture. Task 191 closed all four.

NFR-017's `0.999` and `37.6 ms` values are aspirational comparison references,
not hard acceptance thresholds. Decisions use complete relative Pareto evidence
while topology, correctness, failure semantics, and bounded work remain hard
validity requirements.

## Program shape and dependency order

| Task | Workstream | Entry condition | Output |
| --- | --- | --- | --- |
| 184 | Remote payload materialization | complete — PROMOTE | fixed batch-10 winner; productionization in Task 191 |
| 185 | Fixed-cap gateway landmarks | executable now, independent of 184 | at most one fixed-cap recall candidate |
| 186 | Larger compressed/hierarchical head | after Task 185 disposition | at most one bounded routing/capacity candidate |
| 187 | Traversal transport | complete — STOP, no candidate | fresh 100k attribution; nine-way contract inherited by Task 194 |
| 188 | Graph/search residual recall | after Tasks 185/186 establish the remaining entry gap | at most one graph or adaptive-work candidate |
| 189 | Hybrid codec/distance | only after same-seed evidence identifies a codec opportunity | at most one codec candidate |
| 190 | Architectural escalation | only if narrower tasks leave a material gap | reviewed architecture decision and follow-up, not a bundled rewrite |
| 191 | Lazy payload productionization | complete — PROMOTE | production lazy10; release A/B and feature isolation passed |
| 192 | Owner endpoint validation amortization | complete — PROMOTE | measured generation-keyed row-schema cache; productionization in Task 195 |
| 193 | Owner payload batch fetch | complete — STOP | prepared-plan candidate was not useful end to end; Task 196 owns independent duplicate follow-up |
| 194 | Traversal transport attribution | complete — STOP | reconciled nine-way TRAV-01 counters; fixed-work wider/fewer-round candidate was not useful end to end |
| 195 | Owner schema cache productionization | Task 192 PROMOTE complete | normal release path, benchmark selector removed, release A/B |

Tasks 184, 191, 187, and 192--194 are complete. Task 192 promoted its bounded
generation-keyed row-schema cache after identical recall/storage and warm mean
wins of 21.9% / 15.7% / 16.9% at 10k / 50k / 100k; Task 195 owns the normal
release change. Task 187 closed STOP on a fresh
byte-identical 100k generation: traversal 7.468 ms of a 22.40 ms warm mean
(remote expansion 6.174 ms, local 1.230 ms, derived remainder 0.065 ms), with
no per-owner traversal transport decomposition to attribute a candidate. Review
re-reading of the same run's materialization counters showed the 10.018 ms
materialization stage is ~90% owner-side endpoint work
(`owner_open_validate_work` 6.722 ms/scan, `owner_payload_sql_work`
8.340 ms/scan for ~6.64 rows, wire/encode/decode ~1.68 ms/scan), which
satisfies the recorded triggers for MAT-37/MAT-38 and motivates Tasks
192–194 in that order. Tasks 185 recall work proceeds independently. Tasks
186, 188--190 remain gated by their recorded prerequisites to prevent
expensive architecture or codec work from outrunning the measured
bottlenecks. Task 172 retains broad throughput, injected-RTT, and capacity
characterization; Task 167 retains physical DML.

Every production-affecting winner receives a separately numbered
productionization task. A benchmark winner is not a default change.

## Candidate ledger: remote payload materialization

Task 184 completed attribution and selected one isolated family. The measured
control already grouped by owner, drove owners concurrently, pooled
connections, prepared the outer statement, and sent projection attnums; those
remain controls rather than new candidates.

| ID | Candidate | Status / trigger |
| --- | --- | --- |
| MAT-01 | Lazy first payload batch followed by deterministic deepening | **production** via Task 191; fixed window 10 |
| MAT-02 | Executor cursor-driven remote payload fetch | **production mechanism**; adversarial qual/failure matrix passed |
| MAT-03 | Adaptive payload batch size from observed qual rejection | deferred; fixed 10 already matched consumed remote rows and won end to end |
| MAT-04 | Fixed 10/20/40 bounded batches | **fixed 10 selected**; 20/40 not advanced |
| MAT-05 | `k + margin` fast path for provably unfiltered queries | conditional on planner proof |
| MAT-06 | No-qual fast path with fail-closed fallback | conditional on projection/qual audit |
| MAT-07 | Pipeline next payload batch while executor consumes current rows | conditional on lazy batching winner |
| MAT-08 | Start materialization during the final traversal round | conditional on stable-finalist evidence |
| MAT-09 | Speculative payload prefetch for likely final candidates | deferred; account for wasted work |
| MAT-10 | Cancel speculative work when global ranking changes | conditional on MAT-09 |
| MAT-11 | Move decoded payloads into output rows instead of cloning | deferred after fixed-10 winner |
| MAT-12 | Rank-indexed payload storage instead of `HashMap<vec_id, payload>` | deferred after fixed-10 winner |
| MAT-13 | Preserve request order and eliminate result-map lookup | deferred after fixed-10 winner |
| MAT-14 | Remove the second nested `Vec<Vec<u8>>` copy | deferred after fixed-10 winner |
| MAT-15 | Packed payload buffer with offsets and null bitmap | conditional on decode/copy share |
| MAT-16 | Avoid PostgreSQL array construction for each payload row | conditional on wire/decode share |
| MAT-17 | Cache resolved row schema per published generation | **measured winner Task 192; productionization Task 195** |
| MAT-18 | Cache attnum-to-send-function resolution | included in Task 192's resolved immutable schema entry; productionization Task 195 |
| MAT-19 | Cache the owner-side inner SPI plan | measured STOP in Task 193 packet 005: 100k warm mean 23.60→23.50 ms; payload SQL 8.747→8.600 ms/scan |
| MAT-20 | Cache projection-specific SQL by generation/projection fingerprint | measured as the bounded MAT-19 refinement; same STOP result in Task 193 packet 005 |
| MAT-21 | Replace textual `ctid` formatting with typed/binary locators | deferred after fixed-10 winner |
| MAT-22 | Return row-tier locator with expanded candidates | conditional; changes expansion wire payload |
| MAT-23 | Direct batched `vec_id -> row-tier TID` lookup | production mechanism confirmed by Task 193 packet-001 audit |
| MAT-24 | `unnest(vec_ids) WITH ORDINALITY` join to directory/row tier | production mechanism confirmed by Task 193 packet-001 audit |
| MAT-25 | Heap-block/TID-sorted fetch followed by rank restoration | conditional on heap locality counters |
| MAT-26 | Batch detoast/binary-send work by physical block | conditional on varlena/heap share |
| MAT-27 | Covering row-tier layout for common scalar projections | deferred; format/storage decision |
| MAT-28 | Exclude large/toasted columns unless planner proof requires them | deferred; Task 184 preserved existing planner projection proof |
| MAT-29 | Strengthen minimal projection derivation | deferred; current endpoint already accepts attnums |
| MAT-30 | Generation-scoped coordinator payload cache | conditional on cross-query hit-rate evidence |
| MAT-31 | Bounded hot cache keyed by generation, vec_id, and projection | conditional on MAT-30 |
| MAT-32 | Bounded coordinator hot-payload replica | deferred; storage/lifecycle decision |
| MAT-33 | Compress wide varlena payloads | conditional on wire-byte dominance |
| MAT-34 | Streaming binary response instead of row/array results | deferred; protocol change |
| MAT-35 | Combine final exact ranking and materialization in one owner endpoint | conditional on redundant owner work |
| MAT-36 | Piggyback likely-winner payloads on final expansion | deferred; couples traversal and materialization |
| MAT-37 | Cache safe frozen-generation lookup state owner-side | **PROMOTE Task 195**; Task 192 reduced 100k warm mean 23.70 -> 19.70 ms |
| MAT-38 | Avoid repeated attested-generation validation on a hot connection | **PROMOTE Task 195**; packet-006 epoch/reclaim fencing passed |
| MAT-39 | Owner-side parallel heap fetch | conditional on owner CPU/IO dominance |
| MAT-40 | Projection-shape payload cache/prepared portal | conditional on repeated projection shapes |

## Candidate ledger: bounded entry coverage

Task 185 owns fixed-cap gateway objectives. Task 186 owns larger or hierarchical
heads. Candidate training never uses evaluation queries.

| ID | Candidate | Status / trigger |
| --- | --- | --- |
| HEAD-01 | Trained cap 8,192 exact scan | conditional Task 186 capacity control |
| HEAD-02 | Trained cap 16,384 exact scan | conditional after HEAD-01 signal |
| HEAD-03 | Larger compressed head plus bounded exact shortlist rerank | conditional Task 186 |
| HEAD-04 | Two-level trained representatives and landmark groups | conditional Task 186 |
| HEAD-05 | IVF-style centroid routing over trained landmarks | conditional Task 186 |
| HEAD-06 | HNSW/Vamana navigation over a larger trained landmark set | conditional Task 186 |
| HEAD-07 | Query-conditioned bounded trained-region routing | conditional Task 186 |
| HEAD-08 | Multiple complementary heads under one total scoring cap | conditional Task 186 |
| HEAD-09 | Query-selected head ensemble | conditional after HEAD-08 diagnostic |
| HEAD-10 | Score all representatives, open only the best groups | conditional Task 186 |
| HEAD-11 | Bounded per-owner heads merged at coordinator | unmeasured; owner is load balance, not semantic region |
| HEAD-12 | Coordinator-resident compact summary of every owner head | deferred format/cache candidate |
| HEAD-13 | Diversity-aware selection instead of nearest 32 landmarks | active Task 185 |
| HEAD-14 | Penalize seeds sharing the same traversal basin | active Task 185 |
| HEAD-15 | Maximal-marginal-relevance distance/graph seed selection | active Task 185 diagnostic |
| HEAD-16 | Force seed coverage across disjoint training-query regions | unmeasured |
| HEAD-17 | Select landmarks by marginal bounded-traversal recall gain | active Task 185 primary candidate |
| HEAD-18 | Train gateway nodes that lead traversal to truth neighbors | active Task 185 primary candidate |
| HEAD-19 | Submodular cover over successful seed-to-result basins | active Task 185 alternative |
| HEAD-20 | Hard-query mining on a separate validation slice | active Task 185 input discipline |
| HEAD-21 | Allocate capacity to low-recall training-query clusters | unmeasured |
| HEAD-22 | Lightweight query-to-region classifier | conditional Task 186 |
| HEAD-23 | LSH/binary-code landmark-group routing | conditional Task 186 |
| HEAD-24 | Query-residual routing against coarse centroids | conditional Task 186 |
| HEAD-25 | Learned query-to-seed predictor with normal graph traversal | deferred; training/runtime complexity |
| HEAD-26 | Deterministic learned predictor plus conservative fallback | deferred; follows HEAD-25 |
| HEAD-27 | Landmark-to-region shortcut edges | conditional Task 188 graph work |
| HEAD-28 | Navigational overlay between landmark gateways | conditional Task 188/190 |
| HEAD-29 | Disjoint bounded multi-start seed groups | conditional Task 186 |
| HEAD-30 | Second seed group only for low-confidence traversals | conditional Task 186/188 |
| HEAD-31 | Adaptive 16/32/64 seeds from score gaps | low priority; unchanged-head seed widening was flat |
| HEAD-32 | Reachability-aware rather than nearest-distance seed ranking | active Task 185 |
| HEAD-33 | End-to-end traversal-success objective instead of oracle overlap | active Task 185 governing hypothesis |
| HEAD-34 | Repeated/near-query result cache | deferred workload optimization, not corpus recall |

## Candidate ledger: traversal and remote transport

Task 187 begins only after Task 184 refreshes the residual profile.

| ID | Candidate | Status / trigger |
| --- | --- | --- |
| TRAV-01 | Split owner execution, transport, decode, frontier, and graph-read timers | **complete Task 194 packet 008**; 34 stage / 26 work rows, remote/traversal reconciliation errors 1.17% / 1.32% |
| TRAV-02 | Coordinator cache of immutable decoded graph records | conditional on repeat-read evidence |
| TRAV-03 | Bounded per-generation remote-node cache | conditional on TRAV-02 |
| TRAV-04 | Owner cache of decoded graph pages/nodes | conditional on owner decode share |
| TRAV-05 | Packed expansion response instead of row/array structures | conditional on wire/decode share |
| TRAV-06 | Delta/compressed neighbor IDs | conditional on wire-byte share |
| TRAV-07 | Contiguous packed neighbor codes and scores | conditional on decode/scoring share |
| TRAV-08 | Bounded two-hop expansion in one RPC | conditional on RTT/round dominance |
| TRAV-09 | Bounded neighbor prefetch data in expansion response | conditional on TRAV-08 |
| TRAV-10 | Speculative next-owner expansion | deferred; wasted-work accounting required |
| TRAV-11 | Pipeline consecutive hop rounds | conditional on RTT/round dominance |
| TRAV-12 | Bounded owner-local subsearch per RPC | conditional on hop-RTT dominance |
| TRAV-13 | Baton-passing owner orchestration | deferred until ADR-085 RTT reopen trigger |
| TRAV-14 | More nodes per round at fixed BW x H work | measured STOP Task 194 packet 007: nodes 40.0→47.04; mean 24.30→24.20 ms; p95 regressed |
| TRAV-15 | Wider rounds with fewer hops | measured STOP Task 194 packet 007: hops 10.0→5.88 and transport wait -0.744 ms, but no useful e2e win |
| TRAV-16 | Confidence-based early termination for easy queries | conditional Task 188/187 |
| TRAV-17 | Extra rounds for hard queries under a fixed maximum | conditional Task 188 |
| TRAV-18 | Frontier-stability/score-gap adaptive work | conditional Task 188 |
| TRAV-19 | Conservative second traversal on low confidence | conditional Task 188 |
| TRAV-20 | Block-local owner expansion order | conditional on graph-read locality |
| TRAV-21 | Reuse traversal/frontier scratch between queries | conditional on allocation profile |
| TRAV-22 | Bounded bitset visited/frontier representation | conditional on mapping feasibility |
| TRAV-23 | Borrow graph/code bytes instead of allocating decode buffers | conditional on allocation profile |
| TRAV-24 | SIMD-batch all RaBitQ scoring returned per round | conditional on flush-width profile |
| TRAV-25 | Reduce async runtime/context transitions | conditional on client/runtime share |
| TRAV-26 | Persist owner query-preparation state | unmeasured; query-digest reuse already exists |
| TRAV-27 | Straggler-aware owner scheduling and tail accounting | active diagnostic |
| TRAV-28 | Replicated coordinator top-layer graph | deferred Task 190 architecture |
| TRAV-29 | Replicated frequently traversed bridge nodes | deferred Task 190 architecture |
| TRAV-30 | Routing-only gateway copies without full graph replication | deferred Task 190 architecture |

## Candidate ledger: graph construction and adaptive search

Task 188 owns this family only after bounded entry work quantifies the residual.

| ID | Candidate | Status / trigger |
| --- | --- | --- |
| GRAPH-01 | BW sweep with H fixed | conditional Task 188 control |
| GRAPH-02 | H sweep with BW fixed | conditional Task 188 control |
| GRAPH-03 | Larger candidate/list-size frontier | conditional on residual traversal misses |
| GRAPH-04 | Larger exact final-rerank width | conditional on rerank attribution |
| GRAPH-05 | Adaptive exact rerank width from score margins | conditional after GRAPH-04 |
| GRAPH-06 | Graph degree R48/R64 | conditional build/storage A/B |
| GRAPH-07 | Higher build search list size | conditional build-quality A/B |
| GRAPH-08 | Vamana alpha tuning | conditional build-quality A/B |
| GRAPH-09 | Closure/stitch parameter tuning | conditional if shard stitching is implicated |
| GRAPH-10 | Connectivity and reachability audit | active Task 188 prerequisite |
| GRAPH-11 | Reverse-edge repair for low-indegree nodes | conditional on GRAPH-10 |
| GRAPH-12 | Bridge edges between weak regions | conditional on GRAPH-10 |
| GRAPH-13 | Seed-aware landmark-to-region shortcuts | conditional on Task 185 gateway evidence |
| GRAPH-14 | Alternate deterministic graph-build seeds | unmeasured stability diagnostic |
| GRAPH-15 | Bounded second-graph ensemble | deferred storage/build candidate |
| GRAPH-16 | Training-query-aware gateway augmentation | conditional on Task 185 |
| GRAPH-17 | Query-difficulty adaptive search budget | conditional after confidence diagnostics |
| GRAPH-18 | Attribute owner-oracle residual to graph, BW/H, or rerank | active Task 188 decision requirement |

## Candidate ledger: codec and distance estimation

Task 189 is conditional. The unchanged full exact-neighbor arm is rejected.

| ID | Candidate | Status / trigger |
| --- | --- | --- |
| CODEC-01 | Higher-bit RaBitQ neighbors | conditional Task 189 |
| CODEC-02 | RaBitQ residual correction | conditional Task 189 |
| CODEC-03 | Exact score only for ambiguous frontier comparisons | conditional Task 189 preferred hybrid |
| CODEC-04 | Exact score only for final expansion candidates | conditional Task 189 |
| CODEC-05 | Two deterministic RaBitQ rotations with bounded union | conditional Task 189 |
| CODEC-06 | Rotation-seed stability/selection audit | unmeasured diagnostic |
| CODEC-07 | OPQ plus PQ neighbor codes | deferred; requires same-seed justification |
| CODEC-08 | OPQ for a large compressed head only | conditional Task 186, not neighbor codec |
| CODEC-09 | TurboQuant neighbor codes | deferred |
| CODEC-10 | F16 residual vectors for selective correction | conditional Task 189 |
| CODEC-11 | Learned/LSQ-refined codebooks | deferred |
| CODEC-12 | Exact vectors for bounded gateway nodes only | conditional Task 189/188 |
| CODEC-13 | Approximate route plus exact final ranking | retained control; current path already exact-ranks finals |

## Candidate ledger: architectural escalation

Task 190 may compare architectures but may not implement several together.

| ID | Candidate | Status / trigger |
| --- | --- | --- |
| ARCH-01 | Compact global seed index resident at coordinator | deferred Task 190 |
| ARCH-02 | Replicated global routing layer over sharded lower graph | deferred Task 190 |
| ARCH-03 | Graph/community-aware placement instead of hash placement | deferred; FR-078/ADR-085 replacement |
| ARCH-04 | Hash ownership plus replicated boundary nodes | deferred Task 190 |
| ARCH-05 | Columnar/packed immutable row tier | deferred; storage format decision |
| ARCH-06 | Covering payload sidecars for common projections | deferred; workload/storage decision |
| ARCH-07 | Dedicated binary RPC instead of SQL-function transport | deferred protocol decision |
| ARCH-08 | Shared-memory or Unix-domain same-host transport | deferred topology-specific path |
| ARCH-09 | GPU/SIMD exhaustive compact-head scoring | deferred accelerator path |
| ARCH-10 | GPU-batched head and traversal scoring | deferred throughput architecture |
| ARCH-11 | Cross-query batching to amortize transport/scoring | deferred throughput task |
| ARCH-12 | Publish-time cache/prepared-state prewarming | unmeasured operational candidate |
| ARCH-13 | Query-routed coordinator colocated with likely result owner | deferred topology decision |
| ARCH-14 | Per-query coordinator selection under one logical index | deferred lifecycle/routing decision |
| ARCH-15 | Workload-aware payload replication | deferred storage/lifecycle decision |

## Measured and contractual negative-result ledger

| ID | Result | Disposition |
| --- | --- | --- |
| NEG-01 | Width64/seeds64 was recall-flat and not faster than width32/seeds32 | reject unchanged width/seed tuning |
| NEG-02 | Exact scoring of the unchanged cap-4,096 sample did not recover recall | membership, not head-score precision, was limiting |
| NEG-03 | Random cap-16,384 reached only 0.9440 at 100k | do not treat random linear cap growth as validation |
| NEG-04 | Owner scan reached 0.9970 but cost roughly 2.45 s at 100k | diagnostic only; O(N) production scan forbidden |
| NEG-05 | Exact-neighbor traversal reached 0.9605 versus RaBitQ 0.9625 and was 2.58x slower | reject unchanged full exact-neighbor traversal |
| NEG-06 | Region-balanced and facility cap-4,096 heads had distinct persisted digests but identical top-32 seeds and 0.9625 recall | reject those unchanged objectives |
| NEG-07 | Head scoring is 5.65% and seed selection 0.25% of current wall mean | do not lead latency work with head micro-optimization |
| NEG-08 | Automatic small-corpus policy substitution lacks a production threshold/profile and the trained policy is opt-in | reject unchanged automatic 10k bypass |
| NEG-09 | DiskANN graph prefetch was a measured loss in Task 168 | caution only; DistANN remote prefetch requires its own attributable evidence |
| NEG-10 | A full-scale matrix with no selected candidate provides no promotion evidence | conditional skips remain valid closeout outcomes |

Rejected candidates may be reopened only when the new task names the changed
premise and does not repeat the same experiment unchanged.

## Task import and experiment rules

1. A task names the candidate IDs it imports before measurement.
2. Each behavior change is isolated in its own A/B; no stacked candidate
   families in one attribution cell.
3. Initial screens use a fresh 100k physical generation and Task 182's retained
   policy unless the task explains a different control.
4. Only a useful isolated candidate proceeds to 10k/50k/100k confirmation.
5. All matrices and sweeps use checked-in `ecaz bench suite` configs.
6. Recall, mean/p50/p95/p99/max latency, storage, construction, work/bytes,
   topology, remote engagement, query separation, and release provenance are
   reported wherever applicable.
7. Training, validation, and evaluation inputs are disjoint and independently
   attested. Evaluation data never selects a policy.
8. Work remains explicitly bounded; no silent owner scan, unbounded fetch,
   uncapped fanout, or partial-result success.
9. A stage-local win must move end-to-end behavior and preserve semantics.
10. Stop is a valid outcome. Negative evidence updates this ledger.

## ADR triggers

Do not create an ADR merely to list candidates. Add or amend an ADR only after
evidence selects a durable decision involving one or more of:

- persisted head, graph, row-tier, or payload format;
- lazy/incremental materialization and executor semantics;
- new wire protocol or failure model;
- placement, replication, or coordinator routing;
- production codec/default change;
- upgrade, rebuild, rollback, or lifecycle behavior; or
- accelerator/runtime dependency.

Small internal changes such as removing copies, caching attested schema state,
or replacing a map do not require an ADR unless they alter a durable contract.

## Evidence references

- Task 179 packets 048 and 059--060: physical path, owner oracle, lifecycle,
  and accepted 10k/50k/100k baseline.
- Task 180 packets 002--003: bounded-head attribution and width/seed negative.
- Task 181 packets 002--006: landmark diagnostics, fixed-cap screen, and GO.
- Task 182 packets 004--008: production trained head, A/B, closeout, and
  NFR-017 reconciliation.
- Task 183 packets 002--006: codec attribution, alternative heads, stage
  profile, and STOP.
- Task 184 packets 001--004: materialization attribution, fixed-window
  candidate, adversarial semantics/failure matrix, full-scale PROMOTE.
- NFR-007 and NFR-017 through NFR-020: evidence, comparison, storage, bounded
  work, and failure contracts.
