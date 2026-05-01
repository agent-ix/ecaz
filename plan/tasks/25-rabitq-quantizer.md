# Task 25: RaBitQ Quantizer

Status: landed on `main` for the first-class RaBitQ quantizer and IVF
integration. Symphony is shelved and is no longer the active consumer for this
task.

Current landed surface:

- `src/quant/rabitq.rs` implements RaBitQ as a first-class quantizer with the
  absolute-path `Quantizer` / `QueryScorer` trait surface.
- `ec_ivf` supports `storage_format = 'rabitq'` and the `quantizer = 'rabitq'`
  alias, with build/scan/insert/vacuum coverage.
- `ecaz quant feasibility` owns the offline RaBitQ recall/error-bound study
  surface.
- Benchmark docs include local IVF RaBitQ rows at 10K and 25K.
- The centered API that was built for possible Symphony work remains in
  `RaBitQQuantizer`, but there is no active Symphony AM lane.

This task superseded ADR-031's narrow prefilter framing by graduating RaBitQ
from a beam-search prefilter into a reusable quantizer/profile option.

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

This task originally served as the **research gate for SymphonyQG**. That gate
is now historical: Symphony is shelved, while the reusable RaBitQ quantizer and
IVF integration remain landed.

## Why this landed

- RaBitQ standalone is independently useful: it fits as a first-class
  quantizer under ADR-032's coexisting formats, and the scoring kernel
  composes with IVF and possible future DiskANN in-memory-tier work.
- It gives IVF a compact alternative to TurboQuant/PQ-FastScan for local
  quantizer/storage comparisons.
- The quantizer module is self-contained and can be exercised through the
  offline feasibility harness before any new AM work exists.

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

## Historical Subtasks

The checklist below records the original execution plan. The authoritative
current state is the landed-surface summary at the top of this file; not every
landed slice maps one-to-one to the original checkbox wording.

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

**Decision gate (ADR-045 research gate, superseded for Symphony):**
Originally defined as RaBitQ recall@10 within **1pp of exact** at the bit
budget required to match PQ4 storage on both the 50k and 1M seams, using
absolute (non-centered) 1-bit encoding.

Reading the Symphony paper (§3.1.1, §3.1.2) during implementation flipped
this gate's relevance: Symphony does **not** use absolute RaBitQ encoding.
It quantizes per-vertex residuals `(n − v) / ||n − v||` and exploits
multi-visit beam search to absorb estimator error. The absolute-encoding
1pp gate is therefore not the right blocker for starting Stage 2.

Actual outcomes recorded in review packets:

- **Absolute, 1 bit/dim, no rerank** (slice 10, packet 20009):
  recall@10 = 0.8975 on DBpedia-10k (10.25 pp gap). FAIL by the
  original rubric. **This configuration is not what Symphony uses**;
  the result stands as a characterisation of standalone 1-bit RaBitQ
  only.
- **Absolute, 1 bit/dim, rerank K'=100** (slice 11, packet 20010):
  recall@10 = 1.0000 on DBpedia-10k. PASS for rerank-pipeline
  consumers (DiskANN in-memory tier, ADR-031 successor).
- **Absolute, q-bit sweep, no rerank** (slice 12, packet 20011):
  recall climbs 0.8975 → 0.9865 at q ∈ {1, 2, 4, 8}. MARGINAL at
  q = 8. Off Symphony's critical path.
- **Symphony (centered, 1 bit/dim, multi-visit beam)**: gate lives
  at task 27's end-to-end test, not here. Task 25 is responsible
  for exposing the centered API primitives (slice 15); the recall
  gate itself is Stage-2 / Stage-3 territory.

Historical decision record:

- Task 27 (Symphony Stages 2, 3) was technically unblocked by the centered API,
  but is now shelved by roadmap decision.
- `src/quant/rabitq.rs` ships and remains usable by non-Symphony
  consumers via the absolute-path traits and the `--rerank-k` flag.
- Higher-bit quantization (Extended RaBitQ) stays parked per ADR-045
  "Open follow-ups"; no current consumer requires it.

### Phase 3 — quantizer-seam integration

- [x] **Register under `Quantizer` trait.** Becomes selectable via the
  same reloption seam that ADR-032 defines for PQ4 vs. grouped PQ.
- [x] **IVF integration.** `ec_ivf` accepts `storage_format = 'rabitq'` /
  `quantizer = 'rabitq'`, persists RaBitQ posting-list payloads, and scans
  through the RaBitQ estimator path.
- [x] **Benchmark packet.** Local IVF RaBitQ benchmark rows are recorded in
  the benchmark docs for 10K and 25K.
- [x] **Handoff contract.** The RaBitQ API surface was documented for the
  now-shelved Symphony lane; it remains useful as code documentation, not as
  an active dependency.

## Owns

- `src/quant/rabitq.rs`
- `ec_ivf` RaBitQ quantizer/profile support
- `ecaz quant feasibility`
- Supersedes the ADR-031 prefilter-only design in scope

## Dependencies

- Existing SRHT rotation (reusable as-is).
- Existing `Quantizer` / `QueryScorer` trait seams (already landed for
  PqFastScan on the native-build lane).
- 50k and 1M real-corpus seams (tasks 10054 / 12) — needed for the
  recall study. Not blocked on them being *complete*, only *queryable*.

## Follow-ups

- Product-class RaBitQ claims remain gated on controlled benchmark hardware.
- Extended RaBitQ remains parked unless a non-Symphony consumer needs better
  recall at PQ4-parity storage.
- DiskANN integration is optional future work, not an active blocker.
- Symphony is shelved; do not treat this task as unblocking Task 27.

## Out of scope

- Symphony AM (`src/am/symphony/`) — shelved.
- Quantization-aware edge selection — shelved with Symphony.
- No-rerank query path — shelved with Symphony.
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
