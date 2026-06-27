# Task 123: SPIRE Route Precision vs. Scan Cost — Floor, Granularity, Soft-Routing

Status: **reopened / amended (2026-06-27) — the cost verdict was single-INSTANCE;
re-scoped to the contained multi-instance substrate and a dual latency+recall
mandate (see Amendment below); contained multi-instance Phase A baseline is
under review in `reviews/task-123/009-multi-instance-phase-a-baseline/`.** The prior single-instance closeout
(2026-06-27; completion record `reviews/task-123/008-completion-record/`, reviewer
sign-offs `.../feedback/2026-06-27-01..03-reviewer.md`) stands as record: the
recall / route-containment findings are topology-independent and retained, and no
single-instance promotion candidate landed. What reopens is the **efficiency /
latency** verdict — it was measured on the single-instance code path, which does
not faithfully exercise SPIRE's distributed executor, and must be re-measured on
the multi-instance lane.
Owner: coder. Worked on the `task-121-spire-coarse-routing-recall-doe` branch.
Priority: P1. **Local-only** (single host) — single-instance for recall,
contained multi-instance for latency/efficiency — until a candidate exists.

## Amendment (2026-06-27): multi-instance substrate + dual latency/recall mandate

This task — together with Task 121 — is reopened under a sharper mandate. Two
**co-equal** paths, with the **routing algorithm as the shared lever for both**:

1. **Improved latency** — make the post-route path cheap on the real distributed
   executor.
2. **Improved recall** — better routing so truth is contained at lower nprobe.

The original single-instance Phase A/B evidence stands as record but does **not**
settle the efficiency question: SPIRE is a distributed AM, and the
`flat-dominates` / `scan-path-is-the-wall` verdict was taken on the
single-instance path. Same-machine multi-instance has ~0 network RTT, so it runs
the real distributed executor with the network term removed — the correct, clean,
pre-AWS efficiency instrument. **Re-measure on the contained multi-instance lane.**

### Efficiency dimensions to measure and improve (the latency path)

On the multi-instance lane, attribute and improve each per-stage cost while
holding/improving recall:

- **Scoring** — candidate distance/quant scoring cost and volume.
- **Traversal** — route/leaf selection and graph/beam walk (the routing
  algorithm itself; shared with the recall path).
- **Planning** — coordinator query planning / leaf-set selection / fan-out
  decisioning.
- **Communications** — coordinator↔worker tuple transport: serialization /
  materialization (hex-text vs typed `bytea[]`), projection width, object bytes
  shipped per worker. (Network RTT ≈ 0 on one host, so this isolates the
  encode/materialize CPU surface — see
  `121-spire-distributed-read-transport-efficiency`.)

The Phase A latency floor + per-stage decomposition and the Phase B
`nlists × boundary` granularity work are re-run on the multi-instance lane where
topology-sensitive; recall containment may remain single-instance. AWS stays out
of scope until a contained multi-instance candidate exists.

## Why

Task 121 (SPIRE coarse-routing recall DOE) closed as an evidence-backed
**no-promote**. It proved two things decisively:

- All recall loss is at **route/leaf selection** (route-stage containment ==
  final recall in every run); the block/candidate/rerank stages lose nothing.
- The loss is **recoverable in principle** (boundary `b8` reaches recall
  1.0000), but the only recovery lever tested — **boundary replication** — costs
  too much: `b4` buys back route recall at ~4x index storage (392 MiB at 100k)
  and high low-nprobe latency, and Phase-3 block pruning is neutral at the
  operating point (it only trims compute at saturated high nprobe, never I/O).

Task 121 deliberately scoped itself to routing *recall* and left two things
unexamined that now gate any promotion:

1. **Absolute latency / scan cost.** Clean p50 is ~1-5 s/query at 100k *even at
   `b0`* (b4/tr50: 955 ms @ nprobe 8 → 4730 ms @ nprobe 96). Pipeline counters
   show ~383k candidates/query and ~158 MB object-bytes/query at nprobe 96 —
   more than the entire ~77 MB 4-bit corpus footprint, per query. This suggests
   SPIRE may be ~100-1000x off a flat-scan latency floor. If so, recovering
   route recall is moot: the index is uncompetitive on speed regardless of
   recall.
2. **Leaf granularity (`nlists`).** Tasks 120 and 121 ran `nlists=128`
   throughout (~780 vectors/leaf at 100k). The Task 121 verdict flagged
   `nlists=316` (≈ sqrt(100k), the textbook default) as an "interaction/cost
   axis only — never drilled." Finer leaves should *simultaneously* tighten the
   centroid proxy (better routing precision → less need for boundary
   replication) and reduce per-leaf scan volume (lower latency). It is the one
   lever that could move **both** walls and was never properly tested.

## Goal

Determine whether SPIRE route precision can be recovered at **competitive
absolute cost**, or prove that the **scan path itself** (not routing) is the
binding wall. Output must be per-lever attribution on **both** route-stage
truth containment **and** absolute latency / candidate volume, measured against
a flat-scan floor — not an aggregate verdict.

Primary metrics: route-stage truth containment (the Task 120/121 funnel
instrument) at fixed nprobe, **and** clean cache-warm p50/p95 latency with the
flat-scan floor as the reference. Secondary: candidate volume, object-bytes
read, storage.

## Phases

### Phase A — Latency floor + per-stage decomposition (gate; do first)

- Flat / exact baseline (`ec_flat` or pgvector exact) at 10k/50k/100k → the
  recall-1.0 latency floor SPIRE must be judged against.
- Per-stage SPIRE latency breakdown at the operating point (route / leaf-read /
  candidate-score / rerank-heap), using the funnel timers already instrumented
  in `ecaz bench spire-pipeline`.
- **Gate:** if SPIRE is not within ~5-10x of the flat floor, the binding wall is
  the **scan path**, not routing. Re-scope toward the existing IVF
  scan-efficiency line (Tasks 111-117) / the SPIRE distributed-transport task
  (121-distributed-read-transport-efficiency), naming the specific dominant
  stage. This phase is cheap and decides whether Phases B/C are worth running.

### Phase B — Leaf-granularity drill

- Factorial `nlists {316, 512, 1024}` x `boundary_replica_count {0, 1, 2}` at
  10k/50k/100k, each its own isolated table/index, baseline otherwise = the
  Task 121 `b4/tr50/f8` family minus the lever under test.
- Measure route-stage containment **and** clean latency **and** candidate
  volume **and** storage per cell.
- Hypothesis to confirm or kill: finer leaves deliver `b4`-level route recall at
  ≈`b0` storage and materially lower latency — i.e. the "cheaper route
  precision" the Task 121 verdict called for.

### Phase C — Query-time multiprobe / soft routing (stretch; only if A+B promise)

- Visit neighbor leaves at search time (soft assignment / multiprobe routing) to
  capture the boundary-replica recall benefit **without** replica storage.
- A/B against the Phase-B winner at 10k/50k/100k on recall + latency + storage.

## Non-Goals

- **No AWS** until a local promotion candidate exists (same posture as Task 121).
- **No full quant bit-width sweep.** Candidate-stage scoring is lossless at
  4-bit (funnel shows zero candidate-stage loss); quant is orthogonal to the
  route-precision and scan-cost questions here.
- No bespoke bench sweepers; extend / drive `ecaz bench suite` (FR-038) only.
- Do not re-litigate boundary replication as a *default* — Task 121 settled that
  it is too expensive; it appears here only as a co-tuned axis against `nlists`.

## Acceptance Criteria

1. **Phase A:** flat-scan latency floor + SPIRE per-stage latency decomposition
   at 10k/50k/100k, with the within-Nx gate explicitly evaluated and the binding
   wall named (routing vs scan path).
2. **Phase B:** `nlists x boundary` factorial at 10k/50k/100k with per-cell
   attribution on route-stage containment **and** latency/candidate-volume/
   storage; the `nlists` knee demonstrated (not an arbitrary ceiling).
3. **Decisive verdict:** a named config that recovers route recall at
   competitive latency/storage versus the flat floor, **or** evidence-backed
   proof that the scan path is the binding wall — pointing at the specific stage
   and the owning task. No aggregate hand-waving.
4. **Finding-tied:** every recommendation traces to the route-stage funnel and
   the flat-floor latency comparison; evidence per
   `spec/non-functional/NFR-007-benchmark-provenance.md`.

## Phase A Gate Status (2026-06-27)

Packet `reviews/task-123/001-phase-a-latency-floor-decomposition/` ran the
required flat exact floor and SPIRE decomposition at 10k / 50k / 100k against
the existing Task 121 Phase 3 `b4/tr50/f8` surfaces.

The recall-1.0 nprobe 96 path is outside the task's 5-10x flat-floor gate:

| Scale | Flat exact p50 | SPIRE nprobe 96 p50 | Ratio | Recall@10 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 29.4 ms | 496.2 ms | 16.9x | 1.0000 |
| 50k | 80.2 ms | 2159.5 ms | 26.9x | 1.0000 |
| 100k | 223.3 ms | 5483.0 ms | 24.6x | 1.0000 |

Route-stage containment equals final recall in every Phase A row. The
high-recall 100k path scans about 379k candidates/query and reads about
303.7 MiB/query from local-store objects, so the binding wall is the
post-route local scan/candidate path rather than candidate scoring/rerank
loss or route precision alone.

Per the Phase A gate, the next owner should re-scope toward the IVF/SPIRE
scan-efficiency line instead of running the full Phase B factorial by default.
The closest existing owners are the dense IVF scan/candidate tasks
(`111`/`111e`) for candidate-frontier and scan-locality architecture, plus a
SPIRE-specific local-store/transport-efficiency follow-up if SPIRE continues
as a local-store index after this no-go.

## Contained Multi-Instance Phase A Baseline Status (2026-06-27)

Reviewer feedback in
`reviews/task-123/008-completion-record/feedback/2026-06-27-03-reviewer.md`
correctly scoped the prior cost verdict as **single-instance**. Packet
`reviews/task-123/009-multi-instance-phase-a-baseline/` reruns the requested
100k contained local multi-instance baseline for `n128 b4/tr50/f8` and
`n1024 b2/tr50/f8`.

Initial read, pending review: the real distributed executor path does not show
the multi-second single-instance scan wall. `n1024 b2/tr50/f8` reaches recall
1.0000 at nprobe 64 with p50 87.323 ms / p95 90.365 ms and a 246.1 MiB
coordinator index, while `n128 b4/tr50/f8` reaches recall 1.0000 at nprobe 96
with p50 337.096 ms / p95 479.785 ms and a 392.2 MiB coordinator index.

Residual instrumentation gap: the current local multi-instance production-read
profile exposes candidate/heap/endpoint/total timings, remote dispatch/candidate
counts, and projected payload rows/bytes. It does not yet expose per-worker
object bytes shipped or the full requested leaf-read /
materialize+transport-encode / candidate-score / heap split.

## Phase B Spot-Check Status (2026-06-27)

Reviewer feedback in
`reviews/task-123/003-final-closeout-request/feedback/2026-06-27-01-reviewer.md`
requested a cheap 100k `nlists=1024` spot-check before accepting closeout.
Packet `reviews/task-123/004-phase-b-100k-nlists-spotcheck/` ran that check for
`boundary_replica_count in {0,1}` at nprobe 8 / 16 / 32.

The spot-check separates scan cost from route precision:

| Config | nprobe | Clean p50 | Route containment / recall |
| --- | ---: | ---: | ---: |
| n1024 b0 | 8 | 75.5 ms | 223 / 320 = 0.6969 |
| n1024 b0 | 16 | 95.1 ms | 256 / 320 = 0.8000 |
| n1024 b0 | 32 | 153.8 ms | 280 / 320 = 0.8750 |
| n1024 b1 | 8 | 102.3 ms | 251 / 320 = 0.7844 |
| n1024 b1 | 16 | 143.8 ms | 282 / 320 = 0.8812 |
| n1024 b1 | 32 | 236.1 ms | 298 / 320 = 0.9313 |

The repeated 100k flat exact p50 was 203.8 ms. Finer leaves dramatically reduce
candidate volume and latency compared with Phase A `nlists=128,b4,np8`, but the
best tested spot-check cell (`n1024,b1,np32`) remains below Phase A
`n128,b4,np8` recall (`300 / 320 = 0.9375`) and far from the reviewer's
viability target of approximately 0.99 recall at low nprobe.

Packet `reviews/task-123/006-phase-b-100k-n1024-b2-followup/` then added the
obvious missing `boundary_replica_count=2` cell at `nlists=1024`, extending the
sweep to nprobe 64:

| Config | nprobe | Clean p50 | Route containment / recall |
| --- | ---: | ---: | ---: |
| n1024 b2 | 8 | 120.1 ms | 268 / 320 = 0.8375 |
| n1024 b2 | 16 | 179.6 ms | 292 / 320 = 0.9125 |
| n1024 b2 | 32 | 312.3 ms | 302 / 320 = 0.9438 |
| n1024 b2 | 64 | 526.0 ms | 309 / 320 = 0.9656 |

The b2 SPIRE index is `246.0 MiB`; by comparison, packet 004 measured
`167.9 MiB` for b1 and `89.8 MiB` for b0. At nprobe 32, b2 adds only four
recalled truths over b1 (`302/320` vs `298/320`) while clean p50 rises from
`236.1 ms` to `312.3 ms`. At nprobe 64, recall still remains below the
reviewer's viability target of approximately 0.99.

Route containment equals final recall in every b0/b1/b2 spot-check row. The
closeout result is therefore no-go / re-scope: `nlists=128` can recover recall
only at high scan cost, while `nlists=1024` keeps scan cost lower but does not
recover enough route containment before latency and storage costs become
unattractive.

## Completion Record (2026-06-27)

Task 123 is complete as an evidence-backed local 100k no-go / re-scope result.
The operator explicitly directed that development should not stop waiting on
intermediate review timing after the Phase B follow-up packet, and the outside
reviewer signed off on packet 008 in
`reviews/task-123/008-completion-record/feedback/2026-06-27-01-reviewer.md`.

The sharpest completion finding is flat-exact dominance in the local 100k
regime: same-run flat exact returns recall 1.0 at `161-204 ms`, while the best
tested SPIRE spot-check row (`n1024,b2,np64`) is slower and less accurate at
`309 / 320 = 0.9656` recall and `526.0 ms` p50. Chasing approximately 0.99
recall would only move SPIRE farther beyond the flat exact latency envelope, so
no further local 100k spot-check runs are warranted.

Completion evidence:

- Packet `reviews/task-123/001-phase-a-latency-floor-decomposition/` satisfies
  AC1 and names the binding wall: the high-recall SPIRE path is outside the
  task's 5-10x flat-floor gate at every measured scale, with 100k nprobe 96 at
  `5483.0 ms` versus flat `223.3 ms` (`24.6x`) and about
  `303.7 MiB/query` of local-store object reads.
- Packet `reviews/task-123/004-phase-b-100k-nlists-spotcheck/` addresses the
  reviewer-requested cheap `nlists=1024` check for boundary 0/1. Finer leaves
  are fast, but the best b1 row reaches only `298 / 320 = 0.9313`.
- Packet `reviews/task-123/006-phase-b-100k-n1024-b2-followup/` covers the
  remaining obvious boundary-2 spot-check. b2 reaches only
  `309 / 320 = 0.9656` at nprobe 64 with p50 `526.0 ms` and a `246.0 MiB`
  SPIRE index.
- Route containment equals final recall in Phase A and every b0/b1/b2
  spot-check row, so the recommendation is tied to both the route-stage funnel
  and the flat-floor latency comparison.

No Phase C work starts from Task 123 because Phase A and the Phase B spot-check
do not produce a promising local 100k promotion candidate. This does not claim
SPIRE is globally dead: the no-go is scoped to the local single-node 100k regime
where flat exact is feasible. SPIRE's intended opportunity remains larger
distributed / disk-resident regimes where flat exact is not the comparator to
beat.

Owning follow-ups:

- IVF/SPIRE scan-efficiency line: Tasks `111` and `111e`, especially
  candidate-frontier and scan-locality work.
- SPIRE distributed read/transport line:
  `121-spire-distributed-read-transport-efficiency`, where SPIRE should prove
  value in its intended distributed regime.
