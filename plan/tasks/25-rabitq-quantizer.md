# Task 25: RaBitQ Quantizer — Symphony Stage 1

Status: proposed — **research gate for ADR-045**. Ship standalone, publish a recall
study, decide on Symphony Stages 2–3 from data.

Executes **ADR-045 Stage 1**. Supersedes ADR-031 in scope (RaBitQ graduates from
prefilter to standalone distance).

## Scope

Graduate the existing ADR-031 binary-prefilter work into a first-class
RaBitQ quantizer alongside `prod.rs` and `grouped_pq.rs`, per the ADR-045
Stage 1 framing (supersedes ADR-031). Net-new surface: a rotation front-end
seam, a proper unbiased distance estimator with error-bound API, and
`Quantizer` / `QueryScorer` trait registration. The sidecar encode, the
cached runtime path, and the SIMD-accelerated Hamming / signed-POPCNT
scoring already exist on `main` and are the starting point, not a rewrite.

### Prior work on main (do not reinvent)

- `d662a72` — persisted binary sidecars on bulk build
  (`src/am/build.rs`, `page.rs`, `insert.rs`).
- `e1b0912` — cached ADR-031 binary runtime path in scan.
- `552f4d1` — grouped binary traversal score mode (`src/am/scan.rs`,
  `src/lib.rs`).
- `df8fd04`, `eeb814a` — ADR-030 binary traversal docs + bench surfaces.
- Review packets `279` (sign-binary study), `281` (cached prefilter
  runtime), `285`/`286` (persisted sidecar feasibility + A/B), `359`
  (binary traversal score mode), `360` (window-64 operating point).

Task 25 lifts this from an in-scan prefilter into a standalone quantizer
module and runs the Stage-1 recall gate on it. It does **not** redo the
sidecar plumbing, the cache path, or the scalar/SIMD scoring kernel —
those are inputs.

This is the **only research risk that can kill SymphonyQG.** If RaBitQ cannot
hold recall within 1pp of exact at the bit budget that matches PQ4 storage,
Stage 2 (quantized-graph build) and Stage 3 (no-rerank query path) do not
start. The task is scoped so that shelving after Stage 1 is a clean outcome
with no stranded code in the graph/build path.

## Why now

- SymphonyQG (ADR-045) is the single largest latency-per-recall win on the
  roadmap, but it is gated on RaBitQ as primary distance. We cannot design
  Stages 2–3 without first knowing whether the recall budget exists.
- RaBitQ standalone is independently useful: it fits as a first-class
  quantizer under ADR-032's coexisting formats, and the scoring kernel
  (~8 ns/candidate) composes with DiskANN and SPANN downstream.
- Cheap to ship, cheap to shelve. The quantizer module is self-contained and
  can be exercised entirely through the existing offline feasibility harness
  (same pattern as task 22) before any AM wiring exists.

## Design outline

See ADR-045 §"Stage 1" and ADR-031 for the scoring-kernel properties inherited
from the prefilter design. Summary:

- **Front-end rotation.** Random rotation (fast Hadamard) to decorrelate the
  distribution before binarization. Reuse existing SRHT; leave a seam for
  ADR-036 OPQ to replace it.
- **Encoding.** Sign-bit of each rotated coordinate packed into `D/8` bytes
  per vector (192 B at 1536d). Store a single f32 scalar (the L2 norm of
  the rotated vector, or equivalently the RaBitQ normalization constant)
  per vector alongside the code.
- **Distance estimator.** Unbiased estimator from the RaBitQ paper;
  implement both the plain form and the error-bound form. The error bound
  is what Stage 3 will use to size the candidate pool.
- **Scoring kernel.** XOR + POPCNT over packed codes, issued in full-width
  SIMD batches (AVX2 baseline, AVX-512 / SVE specializations gated on
  runtime dispatch — same scheme as task 21).
- **API shape.** Implement the existing `Quantizer` / `QueryScorer` traits
  so the quantizer is drop-in for offline eval; no AM integration required
  for Stage 1.

## Subtasks

### Phase 1 — graduate to a first-class quantizer module

- [ ] **Lift sidecar + scorer to `src/quant/rabitq.rs`.** Move the binary
  encode / cache-path / scoring surface currently living in `src/am/` into a
  standalone quantizer module. AM code calls the module; does not embed it.
- [ ] **Rotation front-end.** Reuse SRHT; wire the rotation in as a
  pluggable seam so ADR-036 OPQ or a learned rotation can replace it
  without touching the scorer.
- [ ] **Unbiased distance estimator + error bound.** Upgrade the current
  POPCNT score into the ADR-045 estimator form with a usable error-bound
  API. This is what Stage 3 (task 27) will consume to eliminate rerank.
- [ ] **Scalar norm storage.** Per-vector f32; decide on layout
  (interleaved with code vs. side table) based on the sidecar's current
  cache profile (packet 285/286 measurements are the baseline).
- [ ] **Register on the `Quantizer` / `QueryScorer` traits.** Becomes
  drop-in for offline eval and (later) for AM selection under ADR-032's
  reloption seam.
- [ ] **Deprecate the in-scan prefilter entry points.** Once the module
  is consumed through traits, delete the ad-hoc hooks in `src/am/scan.rs`
  and `src/am/build.rs` rather than leaving both paths live (per the
  "deprecate = delete" project rule).

### Phase 2 — offline recall study

- [ ] **Feasibility binary.** `src/bin/rabitq_feasibility.rs` — standalone,
  same shape as `aq_feasibility` (task 22). Trains / encodes on the 50k and
  1M real seams; runs the query seam with an offline scorer; emits a recall
  curve vs. exact and vs. PQ4.
- [ ] **Bit-budget sweep.** Plot recall@10 at the RaBitQ bit budget required
  to match PQ4 storage (192 B vs. 768 B — RaBitQ should come in cheaper at
  equal recall, or equal recall at cheaper storage).
- [ ] **Error-bound calibration.** Measure the distribution of the distance
  estimator's error on the real seams. The tail of this distribution is what
  Stage 3 will use to size the safe candidate pool.

**Decision gate (ADR-045 research gate):** RaBitQ recall@10 within **1pp of
exact** at the bit budget required to match PQ4 storage on both the 50k and
1M seams.

- **Pass** → publish the recall study as a review packet, unblock task 26's
  successor "Symphony Stage 2" (quantized-graph build).
- **Marginal** (within 1–2pp, promising on one seam only) → keep the module
  as a non-rerank-eliminating quantizer under ADR-032; do not commit to
  Stages 2–3 yet; return to OPQ (task 20) to close the gap via a learned
  rotation.
- **Fail** (>2pp gap at PQ4 storage parity) → shelve Stages 2–3 of ADR-045,
  keep the module as the ADR-031 prefilter successor, record the null result.

### Phase 3 — quantizer-seam integration (gated on pass)

- [ ] **Register under `Quantizer` trait.** Becomes selectable via the
  same reloption seam that ADR-032 defines for PQ4 vs. grouped PQ.
- [ ] **Benchmark packet.** Scan kernel, memory footprint, build time
  against task 15 PqFastScan baseline on the 50k and 1M real seams.
- [ ] **Handoff contract.** Document the RaBitQ API surface that
  Symphony Stage 2 (quantized-graph build) will consume — rotation, scorer,
  error bound — and freeze it before Stage 2 starts.

## Owns

- ADR-045 Stage 1 (this task is the gate)
- `src/quant/rabitq.rs` (new)
- `src/bin/rabitq_feasibility.rs` (new)
- Supersedes the ADR-031 prefilter design in scope; ADR-031 remains as a
  fallback posture if the gate fails

## Dependencies

- Existing SRHT rotation (reusable as-is).
- Existing `Quantizer` / `QueryScorer` trait seams (already landed for
  PqFastScan on the native-build lane).
- 50k and 1M real-corpus seams (tasks 10054 / 12) — needed for the
  recall study. Not blocked on them being *complete*, only *queryable*.

## Unblocks

- If the gate passes: Symphony Stage 2 (quantized-graph build under
  `src/am/symphony/`), which unblocks Stage 3 (no-rerank query path) and
  the headline 2–4× QPS win over `ec_hnsw` at equal recall.
- If the gate fails cleanly: a first-class binary quantizer module for
  ADR-031-style prefiltering, usable under `ec_hnsw` without any of the
  graph-layout risk. Not the headline win, but not wasted work.

## Out of scope

- Symphony AM (`src/am/symphony/`) — Stage 2, separate task.
- Quantization-aware edge selection — Stage 2.
- No-rerank query path — Stage 3.
- Learned rotation (OPQ) — task 20; the RaBitQ rotation seam is
  pluggable so OPQ slots in later.
- DiskANN integration of RaBitQ — ADR-034 notes RaBitQ is a candidate for
  the in-memory tier, but that sits behind Symphony Stage 1 + DiskANN
  stabilization.

## Notes

- **Ship the null result if it fails.** A clean write-up of "RaBitQ at
  PQ4-parity storage loses 3pp recall on 1M real corpus" is a real
  contribution and frees reviewer attention. Do not hide a marginal result
  by promoting it.
- **Freeze the scorer API before Stage 2 starts.** Symphony's
  quantization-aware edge selection will call the RaBitQ scorer in its hot
  path; a mid-Stage-2 scorer refactor is the most expensive kind of churn.
- **Error-bound data is load-bearing for Stage 3.** The quality of the
  Stage 3 no-rerank decision depends directly on how tight the error-bound
  distribution is. Measure it carefully in Phase 2; it is not just a
  checkbox.
