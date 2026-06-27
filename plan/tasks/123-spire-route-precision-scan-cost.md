# Task 123: SPIRE Route Precision vs. Scan Cost — Floor, Granularity, Soft-Routing

Status: **Phase A closeout requested** (2026-06-27;
`reviews/task-123/001-phase-a-latency-floor-decomposition/`). Phase A local
evidence fails the high-recall flat-floor gate and names the scan/candidate
path as the binding wall; Phase B/C are deferred unless reviewer acceptance
explicitly overrides the gate.
Owner: coder. Worked on the `task-121-spire-coarse-routing-recall-doe` branch
(shared with the closed-out Task 121 DOE).
Priority: P1 follow-up to Task 121. **Local-only** until a promotion candidate
exists.

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
