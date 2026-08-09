# ec_distann Recall and Latency Optimization Roadmap

Status: active planning ledger (2026-07-29). This document records the option
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
10k/50k/100k. Task 195 then productionized the owner schema cache, retaining
0.9625 recall while moving the current 100k production point to 19.90 ms mean.
Traversal remains a separately attributed residual, so Task 187 is unblocked.
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
| 185 | Fixed-cap gateway landmarks | **complete — STOP** (accepted fixed-cap screen; reconciled 2026-08-07) | no production candidate; Task 186 capacity handoff |
| 186 | Larger compressed/hierarchical head | after Task 185 disposition | at most one bounded routing/capacity candidate |
| 187 | Traversal transport | complete — STOP, no candidate | fresh 100k attribution; nine-way contract inherited by Task 194 |
| 188 | Graph/search residual recall | after Tasks 185/186 establish the remaining entry gap | at most one graph or adaptive-work candidate |
| 189 | Hybrid codec/distance | only after same-seed evidence identifies a codec opportunity | at most one codec candidate |
| 190 | Architectural escalation | only if narrower tasks leave a material gap | reviewed architecture decision and follow-up, not a bundled rewrite |
| 191 | Lazy payload productionization | complete — PROMOTE | production lazy10; release A/B and feature isolation passed |
| 192 | Owner endpoint validation amortization | complete — PROMOTE | measured generation-keyed row-schema cache; productionization in Task 195 |
| 193 | Owner payload batch fetch | complete — STOP | prepared-plan candidate was not useful end to end; Task 196 owns independent duplicate follow-up |
| 194 | Traversal transport attribution | complete — STOP | reconciled nine-way TRAV-01 counters; fixed-work wider/fewer-round candidate was not useful end to end |
| 195 | Owner schema cache productionization | complete — outside-reviewed ACCEPT; PROMOTE | exact recall; warm mean -8.33%/-13.28%/-18.11%; selector removed |
| 196 | Lazy10 stable-prefix duplicate | complete — outside-reviewed ACCEPT; PROMOTE | exact-distance rank shift attributed; identity-keyed reuse passes nine semantic cases and exact-recall/work 10k/50k/100k A/B |
| 197 | Multinode release-profile preflight | complete — outside-reviewed ACCEPT; PROMOTE | pre-setup unanimous release/SHA gate, explicit suite diagnostic override, structured evidence |
| 201 | Post-replica latency residual | **SUPERSEDED by Tasks 205/206** (2026-07-29) — its frozen control was the inadmissible coordinator replica | no result; its Phase 1 attribution decomposition is reused by 205/206/216 |
| 202 | Cross-ISA ordered identity | proposed — portability gate; **not started** | canonical-generation x86_64/aarch64 result identity and release/upgrade verdict |
| 203 | Decision re-audit / paper conformance | **complete — review-closed** (2026-07-30) | four-drift audit of Tasks 161–202; NFR-021/NFR-022/NFR-018/StR-008 spec slice; follow-on program created as Tasks 204–210 |
| 204 | Storage-step arm fidelity | **complete — review-closed ACCEPT** (2026-07-30) | per-arm/per-node storage in `results.jsonl`; arms now differ (830 MB / 1.351 vs 2.490 GB / 4.052); growth row + held `≤2.0` gate carried |
| 205 | Expansion pushdown (Algorithm 1) | **complete — review-closed ACCEPT** (2026-08-06) | pushdown live and recall-neutral; response bytes −52.1/−60.8/−64.8%; request bytes unchanged; threshold-vs-limit split withdrawn, needs a separate instrumentation task |
| 206 | Traversal regime (wide beam) | **complete — review-closed 2026-08-05; recommendation REJECTED by Task 215** | BW64/H8 + 128 seeds recommended, then rejected on the normal release build; absolute rows are workload-specific (`top_k=200`/L200) and are not a release forecast |
| 207 | Head reconstruction (§2.2/§3) | **complete — review-closed, no promotion** (2026-08-05) | partition-union raises head membership (+5.3 pts @32) but end-to-end recall does not move (0.9486 → 0.9468); **membership is not the binding recall constraint** |
| 208 | NFR-021/NFR-022 conformance gates | **implementation complete; packet 001 ACCEPTed, packet 002 review-open** | normalized per-owned-record metric with 100k/10k ratio ≤ 2.0, `unavailable` ≠ pass, pre-registration validated; open P2 on the NFR-021 head clause |
| 209 | Bounded degraded completion (§4.2) | proposed — entry gate now satisfied by 205/206; **not started** | opt-in labeled straggler/hedge mode under NFR-020-AC-6 plus the degradation-curve reproduction |
| 210 | Distribution restoration | **complete — review-closed ACCEPT** (2026-08-08; independent concur `reviews/task-210/006-zero-byte-head/feedback/2026-08-08-02-fable.md`) — P0 | sharded head is the shipped default; zero-byte membership head (`coordinator_resident_unsharded_bytes=0`, `gap=none`); TRAV-30 owner; future hardening may bound control-class relation bytes |
| 211 | Head scaling law | **complete** (2026-08-03) | 0.02 law measured at 10k/50k/100k and selected as the implementation candidate; shipped default stays the fixed 4096 cap (no consistent all-scale win) |
| 212 | Crown cache | **complete** (2026-08-02) | pruning A/B activated but pruned zero shards and showed no latency win; 2048 entries selected as the opt-in capacity; defaults unchanged |
| 213 | Fused head hop | **complete** (2026-08-02) | fused consumer + activation counters + recall at 10k/50k/100k; defaults stay opt-in because every measured arm is `seed_set_change=true` |
| 214 | Spec remediation | **complete** (2026-08-01) | P0–P5 done: 78-finding drift inventory, elevation to `spec/functional/distann/`, FR-085–087 + ADR-087, six flow diagrams, clean review round |
| 215 | Wide-beam productionization | **complete — review-closed STOP** (2026-08-06) | normal-release A/B rejected BW64/H8: mean +20.2/+39.4/+47.7%, recall not equivalent (rose to 0.9815 at 100k); defaults restored by `01384502f`; **authoritative for the shipped top-k-10 default** |
| 216 | Owner expansion/serialization latency | **complete — review-closed negative STOP** (2026-08-07) | coordinator decode 0.076 ms of a 40.60 ms scan = 0.19% ceiling closes MAT-12/13/14/15; MAT-16/21/22 remain owner-side and are **not** retired; MAT-21 blocked on a same-generation lane |
| 217 | Same-generation A/B lane | **complete — review-closed ACCEPT** (2026-08-08) — P0 | epoch-fingerprint attestation per physical arm, fail-closed on generation change; 100k A/A byte-identical + runtime-switch A/B proof; extension-*binary*-swap arm still unexercised |
| 218 | Owner-side materialization latency | **complete — review-closed ACCEPT, MAT-21 STOP** (2026-08-08) | production lazy-10 denominator: owner endpoint 9.10 ms/scan of an 18.83 ms scan; typed `tid[]` locators neutral (payload SQL 8.555→8.455 ms/scan, byte-identical predictions); MAT-21 is retired by the negative, while MAT-16/MAT-22 remain open and are carried to a future owner-side task |
| 219 | Recall/latency Pareto default | proposed (2026-08-08) | decide the shipped operating point and whether recall-equivalence stays the acceptance clause for default changes |

Tasks 184, 191, 187, and 192--196 are complete. Task 195's implementation and
release matrix received an outside-reviewed ACCEPT/PROMOTE: exact recall held
at all three scales while warm mean improved by 8.33% / 13.28% / 18.11%, and
the normal release contains no benchmark selector or attribution surface. Task
196 also received an outside-reviewed ACCEPT/PROMOTE after its attributed
identity-keyed prefix reuse passed nine 100k semantic cases with zero duplicate
requests and the full exact-recall/work release matrix. Task 192
promoted its bounded generation-keyed row-schema cache after identical
recall/storage and warm mean wins of 21.9% / 15.7% / 16.9% at 10k / 50k /
100k; Task 195 owns the normal release change. Task 187 closed STOP on a fresh
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

## Post-199/200 handoff

> **CONFORMANCE CORRECTION (2026-07-29).** The replica arm is withdrawn as the
> program's latency control. The **owner-traversal arm — 18.3/20.4/19.9 ms at
> 10k/50k/100k — is the baseline.** See "Conformance correction" below.

Task 199 was outside-reviewed and promoted. Its release matrix recorded the
normal coordinator traversal replica reducing warm mean from 18.3/20.4/19.9 ms
to 15.3/16.4/16.2 ms at 10k/50k/100k with exact recall. That result is not
admissible as a control under [NFR-022](../../spec/non-functional/NFR-022-distann-control-validity.md):
the replica arm holds every owner's graph record and full-precision vector on
one coordinator (1.660 GB at 100k, linear in N), so it does not satisfy
[NFR-021](../../spec/non-functional/NFR-021-distann-distribution-invariant.md)
and is a comparison against a single-node index rather than against ec_distann.

The accompanying claim of "unchanged storage between the owner and replica arms"
is also not a measurement. The suite's storage step computes its scalars once,
before the arm loop (`crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:5153-5160`),
and reprints them inside it (`:5209-5212`), so the arms are byte-identical by
construction. The replica's 1,659,518,976 bytes appear only in a log-only metric
never parsed into `results.jsonl`, and `cluster_index_space_amplification` — the
NFR-018 ratio emitter that already exists at `distann_multicluster.rs:7419-7482`
and ran for Tasks 172 and 197 — was not run for 198/199.

Task 200 then fixed and regression-tested benchmark-only owner-seed detoast
retention; it did not alter the production read path and is unaffected.

The remaining work is split deliberately:

- ~~Task 201 owns fresh latency attribution and one isolated payload/executor or
  local-traversal optimization.~~ **Superseded 2026-07-29.** Its frozen control
  placed the Task 199 replica inside the control and forbade replica questions
  from entering the screen, so it could not surface this defect. The latency
  lane became Task 205 (pushdown) then Task 206 (regime), and the owner-side
  residual lane became **Task 216**, all controlled against the sharded
  owner-traversal arm.
- Task 202 owns the cross-ISA ordered-identity portability gate explicitly
  waived by Task 199. It is a correctness/release gate, not a latency tuning
  task, and it changes no production behavior by itself. **Not started.**

**State as of 2026-08-09** (see the program-shape table above for per-task
detail). Tasks 203--208 and 210--218 have all reported. The measured position:

- **Shipped default** is BW4/H100, L32, 32 seeds, with the sharded zero-byte
  membership head as the default (Task 210, merged). Task 215's normal-release
  A/B at `top_k=10` measures 100k at **0.9280 recall / 21.40 ms mean**.
- **Latency.** The coordinator-side family is closed on Task 216's 0.19%
  addressable ceiling, and traversal transport is a minority of the scan
  (~4.1 ms wait, 0.002 ms response encode). Task 218's production lazy-10
  attribution measured the owner endpoint at 9.10 ms/scan and the typed
  `tid[]` locator A/B was neutral end-to-end, so MAT-21 is retired. The
  remaining owner-side payload SQL stage is still open to MAT-16 and MAT-22,
  which are carried to a future owner-side task.
- **Recall.** Head construction (207) and head selection (185) are both closed
  without a candidate, and 207 showed membership is not the binding constraint.
  The one lever measured to move recall is search budget: Task 215's BW64/H8 arm
  reached **0.9815 at 100k** but cost +47.7% mean latency and was rejected under
  the recall-equivalence clause — a Pareto trade no task currently owns.
- **Open review requests:** `reviews/task-210/006-zero-byte-head/` (P0) and
  `reviews/task-208/002-retrospective-sweep/`, both with no feedback file.

Task 188's BW8 result remains a research candidate only: it was paired with an
experimental 16,384-landmark head, so it is not imported into Task 201 without
a new current-production-head validation. Note also that it was measured without
the Algorithm 1 pushdown (below), which bounds what it can say about beam width.

## Conformance correction (2026-07-29)

An audit of Tasks 161--202 against `DISTRIBUTEDANN` (arXiv:2509.06046) found the
program had drifted from the reference design on four axes. Task 203 owns the
per-task decision re-audit; this section records the ledger consequences.

**1. The traversal regime was never applied.** Task 162's G0 kill-check — the
measurement that unblocked the program — concluded that "wide beam, few rounds
is the only viable multinode shape ... multinode wants >=32", with BW=32/H=8
reaching 0.9940 recall at 20.3--28.3 ms projected, versus BW=4/H=64 at
77.6--141.6 ms ("far over"). The default was never changed. `mod.rs:253` is still
`BEAM_WIDTH = 4` and every distann suite from Task 179 onward is pinned at
BW=4/H=100. BW=4 was inherited from `ec_diskann`, whose Task 168 A/B tuned it to
fill 32-wide local SIMD kernels with no network in the loop.
`ECDISTANN_MAX_BEAM_WIDTH = 64` makes the paper's grid (BW 96--192) unreachable
even as a session GUC. BW >= 32 has never been run on the distributed path.

**2. The pushdown that makes wide beams affordable is absent.** Paper Algorithm 1
pushes threshold `t` and candidate limit `l` to each storage host, which prunes
before returning; §2.4 supplies `t = peek_worst(H_C)` per round. In ecaz
`code_threshold` is hardcoded `None` at the sole call site (`scan.rs:215`),
discarded by the production expander (`generation_read.rs:3146-3149`), `l` does
not exist, and owners return every neighbor unsorted and untruncated. FR-079
defaulted it off deliberately (FND-006) without recording that this removes the
mechanism the paper's beam depends on. `TRAV-14`/`TRAV-15` are void accordingly.

**3. The head diverges from §2.2/§3 on every axis.** Tasks 181 and 185 established
that head *membership* bounds recall (0.9625 vs the 0.9970 same-graph oracle) and
that three different 4,096-row objectives yielded identical top-32 seeds. Paper
§3 names the cause: the head must be built "from the union of the top layers of
each partition's graph, rather than the top layers of the stitched-together
graph". ecaz builds from the stitched global graph. The head is also not sharded
(§2.2) and not replicated (§4.1), and the promoted policy bypasses the persisted
Vamana graph in favour of a 4,096-point exact scan. `HEAD-11` and `HEAD-12` — the
paper's two structural remedies — are the live recall direction, not `HEAD-01`
capacity growth.

**4. The replica abandoned the distributed premise.** Covered above and in
`TRAV-28`/`TRAV-30`.

These interlock: BW=4 forces ~10 sequential rounds, which produce the transport
wait that motivated Task 190's architecture escalation, which produced the
replica, which removed transport wait by removing distribution. Each step was
locally reasonable; the chain begins at a parameter its own entry gate called
non-viable. Sequencing for the correction is **pushdown -> regime -> head**,
since neither the wide-beam nor the seed-width question can be answered without
the pushdown in place.

## Candidate ledger: remote payload materialization

Task 216's 0.19% ceiling is a coordinator-side screen: it retires only
MAT-12/13/14 and MAT-15's packed coordinator decode path. It must not be
applied to owner-side payload SQL or locator work (MAT-16/MAT-21), or to the
owner expansion wire path (MAT-22), without stage-specific evidence.

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
| MAT-12 | Rank-indexed payload storage instead of `HashMap<vec_id, payload>` | **STOP by coordinator ceiling screen — Task 216 isolated control**: coordinator decode is 0.076 ms against a 40.60 ms scan (0.19% maximum); no separate A/B is justified |
| MAT-13 | Preserve request order and eliminate result-map lookup | **STOP by coordinator ceiling screen — Task 216 isolated control**: coordinator map/association work is within the same sub-millisecond addressable region; no separate A/B is justified |
| MAT-14 | Remove the second nested `Vec<Vec<u8>>` copy | **STOP by coordinator ceiling screen — Task 216 isolated control**: coordinator-side copy/decode cannot exceed the measured 0.19% ceiling |
| MAT-15 | Packed payload buffer with offsets and null bitmap | **STOP — Task 216 isolated candidate**: coordinator decode is 0.076/40.60 ms (0.19%) and returned payload bytes are flat; the candidate's slower owner SQL is secondary implementation evidence, not the family-closing rationale |
| MAT-16 | Avoid PostgreSQL array construction for each payload row | **carried open to a future owner-side task** — Task 218 measured the production lazy-10 budget but advanced MAT-21 only; MAT-16 was not tested or retired |
| MAT-17 | Cache resolved row schema per published generation | **production behavior via accepted Task 195; exact-recall release A/B passed** |
| MAT-18 | Cache attnum-to-send-function resolution | production behavior via accepted Task 195 |
| MAT-19 | Cache the owner-side inner SPI plan | measured STOP in Task 193 packet 005: 100k warm mean 23.60→23.50 ms; payload SQL 8.747→8.600 ms/scan |
| MAT-20 | Cache projection-specific SQL by generation/projection fingerprint | measured as the bounded MAT-19 refinement; same STOP result in Task 193 packet 005 |
| MAT-21 | Replace textual `ctid` formatting with typed/binary locators | **STOP — Task 218 packet 002 review-closed ACCEPT**: same-generation production lazy-10 A/B was recall-identical and neutral end-to-end (payload SQL 8.555→8.455 ms/scan); the locator-representation hypothesis is retired, shipped defaults unchanged |
| MAT-22 | Return row-tier locator with expanded candidates | **carried open to a future owner-side task** — Task 218 measured the production lazy-10 budget but did not test the owner-expansion/wire candidate; not retired by MAT-21's negative |
| MAT-23 | Direct batched `vec_id -> row-tier TID` lookup | production mechanism confirmed by Task 193 packet-001 audit |
| MAT-24 | `unnest(vec_ids) WITH ORDINALITY` join to directory/row tier | production mechanism confirmed by Task 193 packet-001 audit |
| MAT-25 | Heap-block/TID-sorted fetch followed by rank restoration | conditional on heap locality counters |
| MAT-26 | Batch detoast/binary-send work by physical block | conditional on varlena/heap share |
| MAT-27 | Covering row-tier layout for common scalar projections | deferred; format/storage decision |
| MAT-28 | Exclude large/toasted columns unless planner proof requires them | deferred; Task 184 preserved existing planner projection proof |
| MAT-29 | Strengthen minimal projection derivation | deferred; current endpoint already accepts attnums |
| MAT-30 | Generation-scoped coordinator payload cache | conditional on cross-query hit-rate evidence; **NFR-021 screen required** — a generation-scoped cache is O(N) if unbounded and must carry an explicit fixed bound |
| MAT-31 | Bounded hot cache keyed by generation, vec_id, and projection | conditional on MAT-30; **NFR-021 screen required** — the bound must be a constant, not a fraction of N |
| MAT-32 | Bounded coordinator hot-payload replica | deferred; **NFR-021 screen required** — 'replica' here must remain bounded-in-N; the FR-084 precedent shows how a bounded-sounding entry becomes a full copy without the ledger changing |
| MAT-33 | Compress wide varlena payloads | conditional on wire-byte dominance |
| MAT-34 | Streaming binary response instead of row/array results | deferred; protocol change |
| MAT-35 | Combine final exact ranking and materialization in one owner endpoint | conditional on redundant owner work |
| MAT-36 | Piggyback likely-winner payloads on final expansion | deferred; couples traversal and materialization |
| MAT-37 | Cache safe frozen-generation lookup state owner-side | **production behavior via accepted Task 195**; release A/B reduced 100k warm mean 24.30 -> 19.90 ms |
| MAT-38 | Avoid repeated attested-generation validation on a hot connection | **production behavior via accepted Task 195**; epoch/reclaim fencing and release A/B passed |
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
| HEAD-13 | Diversity-aware selection instead of nearest 32 landmarks | measured STOP in Task 185 packet 003: recall-flat and materially slower basin-diversity candidate |
| HEAD-14 | Penalize seeds sharing the same traversal basin | measured STOP in Task 185 packet 003: recall-flat and materially slower basin-diversity candidate |
| HEAD-15 | Maximal-marginal-relevance distance/graph seed selection | measured STOP in Task 185 packet 003: no useful fixed-cap candidate |
| HEAD-16 | Force seed coverage across disjoint training-query regions | unmeasured |
| HEAD-17 | Select landmarks by marginal bounded-traversal recall gain | measured STOP in Task 185 packet 003: gateway membership was set-identical to control and recall-flat |
| HEAD-18 | Train gateway nodes that lead traversal to truth neighbors | measured STOP in Task 185 packet 003: isolated reachability did not transfer to the joint beam |
| HEAD-19 | Submodular cover over successful seed-to-result basins | measured STOP in Task 185 packet 003: no held-out recall improvement |
| HEAD-20 | Hard-query mining on a separate validation slice | measured STOP in Task 185 packet 003: input discipline retained; no candidate advanced |
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
| HEAD-32 | Reachability-aware rather than nearest-distance seed ranking | measured STOP in Task 185 packet 003: recall-flat at fixed cap 4,096 |
| HEAD-33 | End-to-end traversal-success objective instead of oracle overlap | measured STOP in Task 185 packet 003: isolated-budget signal did not transfer to the joint beam |
| HEAD-34 | Repeated/near-query result cache | deferred workload optimization, not corpus recall |

## Candidate ledger: traversal and remote transport

Task 187 begins only after Task 184 refreshes the residual profile.

| ID | Candidate | Status / trigger |
| --- | --- | --- |
| TRAV-01 | Split owner execution, transport, decode, frontier, and graph-read timers | **complete Task 194 packet 008**; 34 stage / 26 work rows, remote/traversal reconciliation errors 1.17% / 1.32% |
| TRAV-02 | Coordinator cache of immutable decoded graph records | conditional on repeat-read evidence |
| TRAV-03 | Bounded per-generation remote-node cache | conditional on TRAV-02 |
| TRAV-04 | Owner cache of decoded graph pages/nodes | conditional on owner decode share |
| TRAV-05 | Packed expansion response instead of row/array structures | **rejected — Task 216 packet 001 screen (2026-08-06)**: owner response encode measured 0.002 ms/scan at 100k; the named stage is not dominant in the conforming control |
| TRAV-06 | Delta/compressed neighbor IDs | conditional on wire-byte share |
| TRAV-07 | Contiguous packed neighbor codes and scores | **not selected — Task 216 packet 001 pre-registration (2026-08-06)** chose MAT-15/MAT-21/TRAV-05; the measured traversal stages (owner score 0.966 ms, graph read 1.221 ms/scan at 100k) are minor against materialization |
| TRAV-08 | Bounded two-hop expansion in one RPC | conditional on RTT/round dominance |
| TRAV-09 | Bounded neighbor prefetch data in expansion response | conditional on TRAV-08 |
| TRAV-10 | Speculative next-owner expansion | deferred; wasted-work accounting required |
| TRAV-11 | Pipeline consecutive hop rounds | conditional on RTT/round dominance |
| TRAV-12 | Bounded owner-local subsearch per RPC | conditional on hop-RTT dominance |
| TRAV-13 | Baton-passing owner orchestration | deferred until ADR-085 RTT reopen trigger |
| TRAV-14 | More nodes per round at fixed BW x H work | **STOP — Task 215 release A/B (2026-08-06)**. BW64/H8 (effective L=64, 128 seeds) was 20–48% slower than BW4/H100/L32 at 10k/50k/100k on the normal release build and not recall-equivalent (recall rose); defaults stay BW4/H100. The Task 215 reconciliation records that Task 206 used top_k=200/L200 while this normal-release run used top_k=10/effective L64, despite matching per-scale query hashes and warm-cache protocol; absolute latency rows are workload-specific. The higher-recall/slower-latency trade was explicitly rejected under the recall-equivalence clause. |
| TRAV-15 | Wider rounds with fewer hops | **STOP — Task 215 release A/B (2026-08-06)**, same run and caveats as TRAV-14. Residual latency remains owner compute/serialization; that lane is Task 216. |
| TRAV-16 | Confidence-based early termination for easy queries | conditional Task 188/187 |
| TRAV-17 | Extra rounds for hard queries under a fixed maximum | conditional Task 188 |
| TRAV-18 | Frontier-stability/score-gap adaptive work | conditional Task 188 |
| TRAV-19 | Conservative second traversal on low confidence | conditional Task 188 |
| TRAV-20 | Block-local owner expansion order | conditional on graph-read locality |
| TRAV-21 | Reuse traversal/frontier scratch between queries | conditional on allocation profile |
| TRAV-22 | Bounded bitset visited/frontier representation | conditional on mapping feasibility |
| TRAV-23 | Borrow graph/code bytes instead of allocating decode buffers | **not selected — Task 216 packet 001 pre-registration (2026-08-06)** chose MAT-15/MAT-21/TRAV-05; traversal decode stages measured minor at 100k, so allocation borrowing has no dominant stage to target |
| TRAV-24 | SIMD-batch all RaBitQ scoring returned per round | conditional on flush-width profile |
| TRAV-25 | Reduce async runtime/context transitions | conditional on client/runtime share |
| TRAV-26 | Persist owner query-preparation state | unmeasured; query-digest reuse already exists |
| TRAV-27 | Straggler-aware owner scheduling and tail accounting | active diagnostic |
| TRAV-28 | Replicated coordinator top-layer graph | **SCOPE DRIFT — entry not delivered as written.** Selected by Task 190, but Tasks 198/199 shipped a **full-graph** replica (every vec_id's graph record + full-precision vector, 1.660 GB at 100k, linear in N on one node), not the bounded top-layer structure this row describes. The delivered artifact violates NFR-021, NFR-018's per-node bound, NFR-017:38, and FR-078:492. A bounded top-layer candidate remains unbuilt and unmeasured. |
| TRAV-29 | Replicated frequently traversed bridge nodes | deferred Task 190 architecture |
| TRAV-30 | Routing-only gateway copies without full graph replication | **complete — review-closed ACCEPT in Task 210 packet 006** (2026-08-08). The NFR-021-conforming direction is shipped as part of the distribution-restoration task; the zero-byte membership-head gate is accepted, and no latency win is required for this conformance work. |

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
| GRAPH-13 | Seed-aware landmark-to-region shortcuts | conditional on Task 186 capacity evidence; Task 185 fixed-cap gateway result was negative |
| GRAPH-14 | Alternate deterministic graph-build seeds | unmeasured stability diagnostic |
| GRAPH-15 | Bounded second-graph ensemble | deferred storage/build candidate |
| GRAPH-16 | Training-query-aware gateway augmentation | conditional on Task 186 capacity evidence; Task 185 fixed-cap gateway result was negative |
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
| ARCH-02 | Replicated global routing layer over sharded lower graph | **measured useful by Task 198; promoted to Task 199 productionization**: exact recall, 14–17% warm-mean win, about 65–66% extra generation storage; production unchanged pending normal-build/operator gate |
| ARCH-03 | Graph/community-aware placement instead of hash placement | deferred; FR-078/ADR-085 replacement |
| ARCH-04 | Hash ownership plus replicated boundary nodes | deferred Task 190 |
| ARCH-05 | Columnar/packed immutable row tier | deferred; storage format decision |
| ARCH-06 | Covering payload sidecars for common projections | deferred; workload/storage decision |
| ARCH-07 | Dedicated binary RPC instead of SQL-function transport | rejected for this escalation: measured encode/decode/connection work is only 0.071 ms/scan and the ten sequential remote/backend boundaries remain; reopen only with an independently measured transport-service premise |
| ARCH-08 | Shared-memory or Unix-domain same-host transport | rejected for this escalation: it optimizes the same-host benchmark topology but neither serves genuinely remote owners nor removes sequential owner boundaries; reopen only for an explicitly same-host deployment product |
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
| NEG-01 | Width64/seeds64 was recall-flat and not faster than width32/seeds32 | **QUALIFIED — measured only at BW=4.** Task 180 screen B swept head_seed_count ∈ {32,64,128} with the beam popping 4 candidates per round, where additional seeds are structurally unusable. The negative is valid for BW=4 and says nothing about `k_head` at the paper's wide-beam regime (k_head=200 at BW=128). Re-test at wide beam before treating seed width as closed. |
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

0. **Conformance work is exempt from the candidate rules below** (added
   2026-07-30). Distribution, placement, and NFR-021 conformance work is
   delivered against the invariant, not screened against latency. It is never
   subject to "advance at most one candidate", never requires a measured
   end-to-end win to land, and a measured latency cost is reported rather than
   used as grounds to withhold the property. Task 210 owns this work. The rule
   exists because the ledger admits only winners, and sharding cannot win on
   latency — the withdrawn replica was *faster* than the sharded owner arm, which
   is precisely how distribution was traded away without any single gate
   objecting.
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
- Task 185 packets 003--004: fixed-cap 100k screen and accepted STOP decision;
  gateway membership Jaccard 1.0 with the control, recall 0.9625 tie, and
  basin-diversity warm-mean regression.
- Task 216 packets 001 and 002 correction: owner-stage attribution, the
  coordinator-only 0.19% MAT-15 ceiling, eager-control and feature-build
  provenance correction, and the same-generation prerequisite for any
  follow-up locator A/B.
- NFR-007 and NFR-017 through NFR-020: evidence, comparison, storage, bounded
  work, and failure contracts.

## Current post-206 handoff (2026-08-06)

The task index and older ledger text must distinguish review closure from
productionization. Task 206 is review-closed, but its BW64/H8 recommendation
is not the shipped default; Task 215 owns the normal-release decision. Task
205 is review-closed with the threshold-versus-limit split explicitly
withdrawn as unsupported by the available counters. Task 207 is review-closed
with no promotion; its union-construction result does not justify repeating
that lane unchanged.

Status update (2026-08-07): items 1, 2, and the attribution half
of item 4 are done. Task 185's fixed-cap screen is now **STOP**: gateway
set-cover tied the frequency control at 0.9625 with Jaccard-1.0 membership,
while basin diversification was materially slower; Task 186 is the next
capacity handoff. Task 205 is review-closed (accepted disposition in
`reviews/task-205/005-attribution-closeout/`). Task 215's release A/B ran and
recorded **STOP** — BW64/H8 was 20–48% slower on the normal release build and
not recall-equivalent; defaults remain BW4/H100. Its decision-account
follow-ups now reconcile the Task 206 work-surface gap, reject the
higher-recall/slower-latency Pareto trade, declare the skipped Task 208/210
entry gate, and point mechanism accounting to Task 216
(`reviews/task-215/003-release-matrix-and-decision/`). Task 216's 100k
attribution is accepted and selected MAT-15 (MAT-21 secondary, TRAV-05
rejected). The isolated MAT-15 screen is now **STOP**: the captured arm was
the explicit eager `materialization_batch_size=0` control and a release-profile
feature build, while the coordinator decode ceiling was only 0.19%. The
remaining execution order is to run Task 186's transparent cap-8,192 capacity
control, then advance at most one bounded larger/head-routing candidate. Do
not restart MAT-15; MAT-21 remains blocked until a same-generation lane exists.
attribution is accepted and selected MAT-15 (MAT-21 secondary, TRAV-05
rejected). The isolated MAT-15 screen is now **STOP**: the captured arm was
the explicit eager `materialization_batch_size=0` control and a release-profile
feature build, while the coordinator decode ceiling was only 0.19%. The two
arms were rebuilt independently, so future MAT-21 work carries a
generation-swap/pinned-input requirement and a maximum-win screen. Task 216's
closeout is review-closed. The remaining execution order is to run Task 186's
transparent cap-8,192 capacity control, then advance at most one bounded
larger/head-routing candidate. Do not restart MAT-15; MAT-21 remains blocked
until a same-generation production lazy-10 attribution lane exists.

Task 216 imports the owner-side residual implication from the corrected Task
205 and Task 206 evidence. Response-byte reduction alone is not sufficient:
Task 205 left request bytes unchanged and moved end-to-end latency only
modestly, while Task 206 attributed the larger residual to owner compute and
serialization. The first candidate screen therefore belongs to packed
expansion/neighbor representations, decode/copy allocation, or typed locator
work only when stage evidence selects it. A useful candidate receives its own
productionization task; a stage-local-only win is recorded as STOP.
