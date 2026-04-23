# Review Request: Task 25 Slice 6 — Task 27 Handoff Contract (RaBitQ Stage-1 API Freeze)

> **SUPERSEDED** by `review/20015-task25-task27-handoff-contract-v2/request.md`.
> The API described below (single-scalar `α_c` estimator, no centered
> path) was replaced during slices 9, 12, 13, and 15. Task 27 consumes
> the v2 contract. This packet is kept only for historical context; do
> not use its signatures as ground truth.


Scope: documentation only. No code changes in this slice. This
packet freezes the public API surface that task 27 (Symphony
Stages 2 & 3) will consume from `src/quant/rabitq.rs` and its
`bench_api` re-exports.

Task: `plan/tasks/25-rabitq-quantizer.md` (Phase 3, slice 6 of 6).
Consumer: `plan/tasks/27-symphony-access-method.md`.

Branch: `task25-rabitq-stage1-phase0` (slice 6 builds on `b20ed02`).

## Purpose

Task 27 Stage 2 is the Symphony quantized-graph build; Stage 3 is
the no-rerank query path whose candidate-pool sizing depends on
the slice-4 estimator's error bound. The task doc calls out
mid-stage scorer refactors as the most expensive kind of churn in
that lane, so the RaBitQ API that Stage 2 calls into has to be
nailed down *before* Stage 2 starts, even if the Phase 2 recall
gate has not been run yet.

This packet is what gets reviewed for that freeze. Task 27 must
not start until this packet has reviewer sign-off and the Phase 2
recall gate run has a PASS verdict in a sibling packet.

## Frozen surface (authoritative)

All items below are stable against task-27 consumers. Any change
requires (a) amending this contract and (b) a sign-off that the
change is compatible with whatever task-27 code already depends
on it.

### Types

```rust
// src/quant/rabitq.rs — re-exported via ecaz::bench_api::*

pub const RABITQ_NORM_LEN:   usize = 4;  // ||c||
pub const RABITQ_ALPHA_LEN:  usize = 4;  // α_c = mean|c_i|
pub const RABITQ_SCALAR_LEN: usize = 8;  // norm + alpha

pub trait Rotation: Send + Sync {
    fn dimensions(&self) -> usize;
    fn apply(&self, v: &[f32]) -> Vec<f32>;
}

pub struct SrhtRotation { /* opaque */ }
impl SrhtRotation {
    pub fn new(dimensions: usize, prod: Arc<ProdQuantizer>) -> Self;
    pub fn prod(&self) -> &Arc<ProdQuantizer>;
}
impl Rotation for SrhtRotation { ... }

pub struct RaBitQQuantizer { /* opaque */ }
impl RaBitQQuantizer {
    pub fn new(rotation: Arc<dyn Rotation>) -> Self;
    pub fn with_srht(dim: usize, prod: Arc<ProdQuantizer>) -> Self;
    pub fn dimensions(&self) -> usize;
    pub fn sign_bytes(&self) -> usize;     // dim.div_ceil(8)
    pub fn prepare_estimator(&self, query: &[f32]) -> PreparedEstimator;
    pub fn estimate_ip(&self, prepared: &PreparedEstimator, code: &[u8])
        -> DistanceEstimate;
}
impl Quantizer for RaBitQQuantizer { ... }    // encode_code / prepare_scorer / code_len / wire_format_version

pub struct PreparedEstimator { /* opaque */ }
impl PreparedEstimator {
    pub fn dimensions(&self) -> usize;
    pub fn query_norm(&self) -> f32;
}

pub struct RaBitQScorer { /* opaque */ }
impl QueryScorer for RaBitQScorer { fn score(&self, code: &[u8]) -> f32; }

#[derive(Debug, Clone, Copy)]
pub struct DistanceEstimate {
    pub estimate: f32,
    pub bound:    f32,
}
```

### Free functions

```rust
pub fn derive_persisted_sidecar_words(
    quantizer: &ProdQuantizer,
    code: &[u8],
) -> Vec<u64>;

pub fn persisted_sidecar_word_count(
    dimensions: u16,
    bits: u8,
    seed: u64,
) -> usize;
```

(These are the ADR-031 PQ-derived sidecar helpers. Task 27 does
not need to call them directly — they are AM-side build-path
helpers — but they are part of the frozen surface because
removing or renaming them would ripple through the AM code task
27 sits alongside.)

### On-disk code layout (per vector)

```
offset 0                 ⌈D/8⌉                 ⌈D/8⌉+4         ⌈D/8⌉+8
+------------------------+------------------+------------------+
| sign bits (LSB-first)  |  ||c|| (f32 LE)  |  α_c  (f32 LE)   |
+------------------------+------------------+------------------+
```

- Sign bit for coordinate `i` lives at bit `i % 8` of byte
  `i / 8` of the sign section. `1` = `c_i ≥ 0`.
- `||c||` and `α_c` are **little-endian** `f32`. They are
  contiguous; task 27 can read them as
  `code[sign_bytes..sign_bytes+8]` and `try_into::<[u8;8]>()`
  without the two-field split if it prefers.
- `α_c = Σ|c_i| / D` — the least-squares coefficient of the
  sign-vector approximation `c ≈ α_c · sign(c)`.

### Estimator semantics

```
⟨q, c⟩ ≈ α_c · Σ_i q_i · sign(c_i)             [DistanceEstimate.estimate]
|⟨q, c⟩ − estimate| ≤ ||q|| · ||r_c||           [DistanceEstimate.bound]
||r_c||² = max(0, ||c||² − α_c² · D)
```

Invariants that Stage 3 may rely on:

- `bound ≥ 0` always.
- `bound = 0` ⇔ `||r_c|| = 0` ⇔ `c` is a sign-aligned vector
  (every coordinate has magnitude `α_c`). In that case the
  estimator is exact.
- The bound is **Cauchy-Schwarz** (worst-case envelope), not a
  confidence interval. Realized error is ≪ bound on average;
  the empirical `tightness = mean(error) / mean(bound)`
  reported by the Phase 2 harness is the calibration number
  Stage 3 uses when sizing candidate pools. A target of
  `pool_size = k + slack · max(bound in pool)` is appropriate.

### Rotation contract

- `Rotation::apply(v)` MUST return exactly `Rotation::dimensions()`
  coordinates. RaBitQ's code layout does not carry rotation
  padding.
- `Rotation` is `Send + Sync`; the quantizer stores it as
  `Arc<dyn Rotation>` and shares it between build and scan paths.
- `SrhtRotation` is the default during ADR-045 Stage 1. OPQ
  (task 20) will land as a second `impl Rotation`. Task 27 must
  not assume SRHT; it sees only `dyn Rotation` through the
  quantizer.

### Trait dispatch

Task 27 has two choices for scoring:

1. **Scalar estimate only** — hold `&dyn QueryScorer`, call
   `score(code)`. Returns `DistanceEstimate::estimate`. Use this
   at the rerank seam if Stage 2's build phase is what matters.
2. **Estimate + bound** — hold `&RaBitQQuantizer` concretely, call
   `estimate_ip(&prepared, code)`. Use this in the Stage 3 query
   path where the bound gates candidate-pool early-exit.

Both funnel into the same private estimator body; both produce
identical `estimate` values.

## What is NOT frozen

These can still change before / during task 27 without amending
the contract:

- `Quantizer::wire_format_version()` currently returns `0`. A
  dedicated `INDEX_FORMAT_RABITQ` constant lands when the AM
  (task 27's Symphony AM or a backport to `ec_hnsw`) first
  persists RaBitQ codes on-disk. Until that reader exists, the
  value is not load-bearing.
- The feasibility binary's CLI surface (`src/bin/rabitq_feasibility.rs`).
- The in-module unit tests. Task 27 should write its own.
- Internal helpers (`l2_norm`, `mean_abs`, `sign_words_from_byte_slice`,
  `estimate_ip_impl`). These are `pub(crate)`-scoped or private;
  task 27 must go through the public surface above.
- SIMD acceleration of the estimator inner loop. Slice 4 is
  scalar; SIMD lands in a follow-up slice after the gate clears.

## Consumers task 27 is expected to add

- **Stage 2 (quantized-graph build).** Calls `encode_code` on
  every indexed vector, persists the code + the sign words it
  needs for quantization-aware edge selection. May also call
  `derive_persisted_sidecar_words` if it wants the ADR-031
  PQ-derived path for speed.
- **Stage 3 (no-rerank query).** Calls `prepare_estimator` per
  query, then `estimate_ip` in the per-candidate inner loop. Uses
  `DistanceEstimate.bound` to decide whether the top-K is
  defensible without a rerank pass; if `bound` allows an
  ambiguous ordering within the top-K candidates, falls back to
  the rerank pass or widens the pool.

## Open questions for reviewer

1. Does the contract need a guarantee that `estimate` is unbiased
   over the space of rotations Stage 2 might use? The scalar
   estimator is unbiased for a *given* rotation (this is how
   Cauchy-Schwarz gives a bound); across OPQ vs. SRHT
   rotations the scalar `estimate` distribution differs.
   I do not think Stage 3 needs cross-rotation unbiasedness —
   the bound is pointwise — but flagging for confirmation.
2. `SrhtRotation::prod()` is public on the struct. If task 27
   only consumes `dyn Rotation`, the accessor is irrelevant to
   it; it is there for the ADR-031 PQ-derived sidecar helpers.
   Keep, or narrow to `pub(crate)`?
3. Phase 2 recall study status: slice 5 shipped the binary but
   the gate-decision run against the 50k / 1M seams has not
   happened yet (I do not have the TSV seams in this
   environment). Is this contract sign-off blocked on that run,
   or can it land with a note that task 27 kickoff is separately
   gated on a `review/20006-*` packet containing the real
   verdict?
