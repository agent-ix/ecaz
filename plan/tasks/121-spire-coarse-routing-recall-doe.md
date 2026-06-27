# Task 121 - SPIRE Coarse-Routing Recall Broad Exploration (DOE)

Status: **reopened / amended (2026-06-27) — re-scoped to the contained
multi-instance substrate and a dual latency+recall mandate (see Amendment
below).** The prior single-instance closeout
(2026-06-26; `reviews/task-121/026-phase4-final-pareto-verdict/`, reviewer
sign-off `.../feedback/2026-06-26-01-reviewer.md`) stands as record: the
recall / route-containment findings are topology-independent and retained, and
no SPIRE default was promoted. What reopens is (a) measurement on the local
multi-instance lane and (b) latency as a co-equal objective to recall.

## Amendment (2026-06-27): multi-instance substrate + dual latency/recall mandate

This task — together with Task 123 — is reopened under a sharper mandate. Two
**co-equal** research paths, with the **routing algorithm as the shared lever for
both**:

1. **Improved recall** — better route/leaf selection so truth is contained at
   lower nprobe (this task's original DOE; those findings are retained).
2. **Improved latency** — make the post-route path cheap: efficient **scoring,
   traversal, planning, and communications** (tuple materialize / encode /
   transport), while holding or improving recall.

The routing algorithm drives both: better routing raises containment (recall)
*and* shrinks how much must be scanned/shipped to reach a recall target
(latency).

**Substrate: the contained local multi-instance lane** (coordinator + N worker PG
instances on one host — the Phase 0 lane) **wherever a measurement is
topology-sensitive.** SPIRE is a distributed AM; single-instance runs do not
faithfully exercise its executor (coordinator-routed scan, remote leaf
materialization, libpq tuple transport). Same-machine multi-instance has ~0
network RTT, so it is a *clean* efficiency instrument: it runs the real
distributed path with the network term removed, leaving per-stage engine cost
(scoring / traversal / planning / communications) as genuine, attributable,
improvable work. Recall is topology-independent and may stay single-instance;
**all latency / scan-cost / transport measurement moves to the multi-instance
lane where possible.** AWS stays out of scope until a contained multi-instance
candidate exists.

## Why

Task 120 ran a six-phase SPIRE coarse-rerank measurement program and concluded
"do not promote." The post-mortem (see `reviews/task-120/` feedback and the
final-recommendation packet) found the conclusion was sound *for what it
measured*, but the investigation was narrow and partly mis-aimed:

- Phase 1's stage-containment funnel localized essentially **all** recall loss
  to coarse **route/leaf selection**. At 100k only 77% of truth reaches the
  candidate frontier at nprobe 8 and 92% at nprobe 32; the block, candidate,
  and exact-rerank stages lose nothing (stages 4/5/6 carry identical
  contained/missing counts).
- But the task's founding premise (from Tasks 79-85) was the opposite - that
  loss was in selected-leaf block / candidate surface, "rather than pure
  top-graph routing." The pre-written phases were **not re-planned** when
  Phase 1 disproved that premise. Phases 2-3 went on to measure leaf/block and
  candidate/rerank-width levers that Phase 1 had already exonerated (both came
  back dead/neutral, as they had to). Phase 4 was constrained to route
  overfetch and a Non-Goal explicitly forbade improving the router.
- Net: the **one axis that matters - routing precision - was never tuned.**
  `boundary_replica_count` was 0 (off), `top_graph_search_list_size` was pinned
  at 96 with recall still rising there (knee never found), `nlists=128` was
  *coarser* than auto (sqrt(100k) ~= 316, so leaves held ~780 vectors each, a
  poor centroid proxy), and k-means trained on a 10k sample at every scale.

The "failure" was not of coarse-rerank; it was a failure to follow the evidence
the program's own first phase produced. This task does the broad routing
exploration that should have followed Phase 1.

## Goal

Broadly and decisively map the SPIRE coarse-routing design space, **locally**,
to answer: can route-stage truth containment be recovered to a competitive
recall/cost point by improving routing precision (and candidate quantization),
**without** brute-force overfetch - and if not, where exactly is the wall?

The output must be **per-lever attribution** ("lever X moved route-stage
containment by N points at equal nprobe; lever Y did nothing"), not an
aggregate final-recall verdict. No wishy-washy answers.

Primary metric: **route-stage truth containment** from the stage-containment
funnel (the instrument validated in Task 120), measured at fixed nprobe so
routing quality is isolated from overfetch. Secondary: final recall@10,
candidate volume, latency, storage.

## Method - Design of Experiments (screen -> drill -> Pareto)

Decisive by construction: a one-factor-at-a-time (OFAT) screen finds which
levers matter; a factorial drill quantifies the winners to saturation; a Pareto
selects a promotion candidate or proves the wall.

### Phase 0 - Tooling: local multi-node as a first-class lane

- Make **local multi-node SPIRE** (1 coordinator + N worker PG instances on one
  host) a first-class `ecaz bench suite` lane / step type, so distributed
  correctness and recall can be measured locally with **no bespoke harness**.
  Task 120 packet 017 needed a hand-rolled "phase13e" harness for exactly this,
  and a coder flagged that churn. Extend the suite runner, not a script
  (per CLAUDE.md "Benchmark Runner: ecaz bench suite only").
- Confirm the existing Load-step reloption-matrix path (`index_name` +
  `reloptions`) builds per-variant `ec_spire` indexes from one config; add any
  small convenience needed to drive an OFAT / factorial matrix.
- No benchmarks execute in this phase; it lands as its own commit(s).

### Phase 1 - OFAT routing + storage screen @ 100k

Baseline = the Task 120 config: `nlists=128, recursive_fanout=8,
top_graph_enabled=1, top_graph_degree=32, top_graph_build_list_size=100,
top_graph_search_list_size=96, boundary_replica_count=0,
training_sample_rows=10000, storage_format=rabitq`.

Build one index per single-lever change (each its own isolated table/index) and
measure route-stage containment + recall across nprobe `[8,16,24,32,48,64,96]`
via the funnel. Levers and screen values:

| Lever | Baseline | Screen values | Hypothesis |
| --- | --- | --- | --- |
| `top_graph_search_list_size` | 96 | 200, 400, 800 | beam clipped; widen to saturation |
| `boundary_replica_count` | 0 | 1, 2, 4 | classic IVF boundary-loss fix, never tried |
| `nlists` | 128 | 316, 512, 1024 | finer leaves -> tighter centroid proxy |
| `recursive_fanout` | 8 | 0, 16, 32 | hierarchy depth vs precision |
| `top_graph_degree` | 32 | 48, 64 | richer graph connectivity |
| `training_sample_rows` | 10000 | 50000, 100000 | centroid quality plateaus past 10k sample |
| `storage_format` | rabitq | turboquant | candidate-stage recall vs scan cost (rabitq + TQ only; PQ out of scope) |

Output: a per-lever delta(route-stage containment) table at 100k, every number
tracing to funnel JSONL, and the **significant-lever set** to carry forward.

### Phase 2 - Factorial drill @ 10k / 50k / 100k

On the levers that screened significant, run a factorial grid (including
interactions - e.g. finer leaves x boundary replication) across all three
scales, **each lever swept to saturation** (capture every knee; no arbitrary
ceiling like the 96 that clipped Task 120).

### Phase 3 - Scan efficiency for the winners

Only after a recall-recovering config is identified: make it cheap. Implement
and measure the scan-efficiency levers the winner needs - block pruning with
**recall-aware recovery** (the softer policy Task 120 never tried; its hard
`l2` cap collapsed recall 0.93 -> 0.51 at 100k), early termination across
leaves, and default-format (TurboQuant) block summaries (block pruning is
currently RaBitQ-only and off). A/B each at 10k/50k/100k.

### Phase 4 - Cost/quality Pareto + verdict

Pareto of recall vs latency vs storage vs candidate volume with funnel
attribution. Output: a named promotion candidate with 10k/50k/100k evidence,
**or** a decisive, evidence-backed "local routing precision tops out at X -
here is the wall and why."

## Non-Goals

- **No AWS.** Local-only until a promotion-worthy candidate exists. (Task 120
  burned 8+ hours on AWS setup churn; the local multi-node lane in Phase 0
  replaces AWS for distributed correctness/recall checks.)
- No on-disk format promoted from final recall alone; candidate/route
  containment must justify it.
- No bespoke bench sweepers or per-packet shell glue; extend `ecaz bench suite`.
- **Do not pre-exclude any routing lever.** (Explicit reversal of Task 120's
  "do not improve topology routing" stance, which blinded it to the fix.)

## Process Guardrails (from the Task 120 post-mortem)

- **Config-baseline section mandatory** in every measurement packet: which
  capabilities are on/off (`boundary_replica_count`, `top_graph_enabled`, beam,
  fanout, training sample) and why.
- **Re-plan gate:** the Phase 2 grid is chosen *from* the Phase 1 screen
  results with reviewer sign-off. The downstream plan adapts to evidence; it is
  not pre-committed. Any phase whose result contradicts a prior premise stops
  and re-scopes before proceeding.
- **Program-level review** at each phase boundary, distinct from packet review:
  "do the tested levers address the stage the funnel says is lossy?"
- Every measurement A/B-isolated, per-variant index on its own table, release
  build, evidence per NFR-007.

## Acceptance Criteria

1. **Phase 0:** local multi-node is a first-class suite lane and a single
   SuiteConfig builds per-variant `ec_spire` indexes via reloptions - no
   hand-rolled harness.
2. **Phase 1:** per-lever delta(route-stage containment) screen table at 100k,
   every cited number tracing to funnel JSONL; significant-lever set named.
3. **Phase 2:** factorial drill on the significant levers at 10k/50k/100k, each
   swept to its knee (saturation demonstrated, not an arbitrary ceiling).
4. **Phase 3:** scan-efficiency A/B (recall + latency + storage) at
   10k/50k/100k for the recall-recovering config.
5. **Phase 4:** a Pareto plus a decisive verdict - a named promotion candidate
   with 10k/50k/100k evidence, or an evidence-backed wall. No aggregate
   hand-waving.
6. **Finding-tied AC:** the final recommendation must demonstrate that the
   tested levers address the route-stage loss the Phase 1 funnel localized.

## Closeout (2026-06-26)

Task 121 closed as an evidence-backed no-promote result. Packet
`reviews/task-121/026-phase4-final-pareto-verdict/` synthesizes the Phase 1-4
evidence, and the outside reviewer signed off on closeout in
`reviews/task-121/026-phase4-final-pareto-verdict/feedback/2026-06-26-01-reviewer.md`.

The final finding is narrow and evidence-backed:

- route-stage containment equals final recall in every measured run, so the
  lossy stage is route/leaf selection rather than candidate scoring, block
  pruning, or exact rerank;
- boundary replication is the primary recovery lever, with
  `boundary_replica_count=4`, `training_sample_rows=50000`, and
  `recursive_fanout=8` as the practical local follow-up candidate;
- the b4 candidate is not a default: at 100k it reaches high recall only with
  high low-nprobe latency and about `392 MiB` SPIRE index storage, while b8
  proves saturation but is a storage/latency wall;
- retuned sampled block pruning is recall-neutral and useful only at high
  nprobe; it does not move the low/mid operating point and does not reduce
  object bytes in the local pipeline counters.

No code or on-disk format change is promoted by this task. Carry `b4/tr50/f8`
only as a named non-default research candidate if a future task introduces a
cheaper route-precision mechanism than boundary replication.

Reviewer loose thread for any future pruning-as-I/O work: packets 019/022/024
show object bytes unchanged while candidates drop. Before claiming block
pruning saves scan I/O, a follow-up must prove whether pruning is structurally
post-read or whether the current local single-node scan path simply cannot
surface read-byte reduction.
