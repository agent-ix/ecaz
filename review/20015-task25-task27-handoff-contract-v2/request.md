# Review Request: Task 25 Slice 16 — Task 27 Handoff Contract v2 (Authoritative)

Scope: documentation. Supersedes
`review/20005-task25-task27-handoff-contract/request.md` in full.
Captures the `src/quant/rabitq.rs` public surface as it actually
ships after slices 9 (paper-faithful estimator), 12 (q-bit
encoder), 13 (seed plumbing), and 15 (centered API). Task 27
(Symphony Stages 2 & 3) consumes this document, not the v1
contract.

Task: `plan/tasks/25-rabitq-quantizer.md` (slice 16 — addresses
reviewer feedback from
`review/20014-task25-centered-api/feedback/2026-04-23-01-reviewer.md`).

Branch: `task25-rabitq-stage1-phase0` (slice 16 builds on `d2429f5`).

## Why a v2 contract

The v1 contract (`20005`) froze the slice-4 surface: a single-
scalar `α_c = mean(|c_i|)` estimator with a Cauchy-Schwarz bound,
no centered path, no q-bit knob. Every one of those pieces changed
in subsequent slices:

- Slice 9 replaced the estimator with the paper-faithful form
  (`o_dot` scalar, ε-concentration bound).
- Slice 12 added `bits_per_dim ∈ {1, 2, 4, 8}` and a third stored
  scalar `||x_dec||`.
- Slice 13 decoupled `SrhtRotation` from `ProdQuantizer` via
  `with_seed(dim, seed)`.
- Slice 15 added the centered-encode / centered-score API
  (`CenterContext`, `CenteredScorer`) — the actual Symphony
  prerequisite.

The v1 contract is now actively misleading. This packet freezes
the current (v2) surface as the task-27 handoff document.

## Authoritative public surface (as exposed via `bench_api`)

```rust
// Traits (unchanged since slice 2; remain the primary API for
// non-Symphony consumers — DiskANN in-memory tier, ADR-031
// successor, offline eval).
pub trait Quantizer {
    fn encode_code(&self, v: &[f32]) -> Box<[u8]>;
    fn prepare_scorer(&self, q: &[f32]) -> Box<dyn QueryScorer + Send + Sync + '_>;
    fn code_len(&self) -> usize;
    fn wire_format_version(&self) -> u32;
}

pub trait QueryScorer {
    fn score(&self, code: &[u8]) -> f32;
}

// Rotation seam (slice 3 + 13).
pub trait Rotation: Send + Sync {
    fn dimensions(&self) -> usize;
    fn apply(&self, v: &[f32]) -> Vec<f32>;
}

pub struct SrhtRotation { /* opaque */ }
impl SrhtRotation {
    pub fn with_seed(dim: usize, seed: u64) -> Self;       // preferred for prod
    pub fn new(dim: usize, prod: Arc<ProdQuantizer>) -> Self; // ADR-031 compat path
    pub fn prod(&self) -> Option<&Arc<ProdQuantizer>>;     // Some iff built via ::new
    pub fn seed(&self) -> Option<u64>;                     // Some iff built via ::with_seed
}
impl Rotation for SrhtRotation { ... }

// Quantizer (slices 3 + 12 + 13 + 15).
pub struct RaBitQQuantizer { /* opaque */ }
impl RaBitQQuantizer {
    // Construction.
    pub fn new(rotation: Arc<dyn Rotation>) -> Self;                    // q = 1 default
    pub fn with_bits(rotation: Arc<dyn Rotation>, bits: u8) -> Result<Self, String>;
    pub fn with_srht(dim: usize, prod: Arc<ProdQuantizer>) -> Self;
    pub fn with_srht_bits(dim, prod, bits) -> Result<Self, String>;
    pub fn with_seeded_srht_bits(dim, seed, bits) -> Result<Self, String>;

    // Shape.
    pub fn dimensions(&self) -> usize;
    pub fn bits_per_dim(&self) -> u8;
    pub fn packed_bytes(&self) -> usize;     // ⌈D·bits/8⌉
    pub fn sign_bytes(&self) -> usize;       // alias of packed_bytes() for q=1 callers

    // Absolute path with error bound (slice 9).
    pub fn prepare_estimator(&self, q: &[f32]) -> PreparedEstimator;
    pub fn estimate_ip(&self, prepared: &PreparedEstimator, code: &[u8]) -> DistanceEstimate;

    // Centered path (slice 15) — Symphony Stage 2 prerequisite,
    // q = 1 only. Asserts otherwise.
    pub fn prepare_center(&self, center: &[f32]) -> CenterContext;
    pub fn encode_code_centered(&self, v: &[f32], c: &CenterContext) -> Box<[u8]>;
    pub fn prepare_scorer_centered(&self, q: &[f32]) -> CenteredScorer;
    pub fn centered_residual_magnitude(&self, code: &[u8]) -> f32;
}
impl Quantizer for RaBitQQuantizer { ... }   // absolute, c = 0 implicit

// Prepared query state, both paths.
pub struct PreparedEstimator { /* opaque */ }
impl PreparedEstimator {
    pub fn dimensions(&self) -> usize;
    pub fn query_norm(&self) -> f32;
}

pub struct RaBitQScorer { /* opaque */ }
impl QueryScorer for RaBitQScorer { ... }

pub struct CenterContext { /* opaque */ }
impl CenterContext { pub fn raw(&self) -> &[f32]; }

pub struct CenteredScorer { /* opaque */ }
impl CenteredScorer {
    pub fn score_at(&self, code: &[u8], c: &CenterContext) -> DistanceEstimate;
}

// Distance + bound.
#[derive(Debug, Clone, Copy)]
pub struct DistanceEstimate {
    pub estimate: f32,
    pub bound:    f32,
}

// ADR-031 PQ-derived sidecar helpers (slice 2 graduation; still
// exposed for the ec_hnsw prefilter-successor pipeline).
pub fn derive_persisted_sidecar_words(quantizer: &ProdQuantizer, code: &[u8]) -> Vec<u64>;
pub fn persisted_sidecar_word_count(dim: u16, bits: u8, seed: u64) -> usize;

// Constants.
pub const RABITQ_NORM_LEN:           usize = 4;
pub const RABITQ_UNIT_DOT_LEN:       usize = 4;
pub const RABITQ_XNORM_LEN:          usize = 4;   // stores ||x_dec||
pub const RABITQ_SCALAR_LEN:         usize = 12;  // = sum of the three above
pub const RABITQ_SUPPORTED_BITS:     [u8; 4] = [1, 2, 4, 8];
pub const RABITQ_BOUND_CONFIDENCE:   f32 = 2.5;
```

## Code layouts (on-disk / in-memory)

### Absolute encoding (trait path, any `bits_per_dim`)

```
offset 0                 packed_bytes()          +4             +8             +12
+------------------------+--------------------+--------------+--------------+
| levels (LSB packed q   |  ||o||  (f32 LE)   |  o_dot       |  ||x_dec||   |
|  bits per coord)       |                    |  (f32 LE)    |  (f32 LE)    |
+------------------------+--------------------+--------------+--------------+
```

- `packed_bytes = ⌈D · bits / 8⌉`.
- `o_dot = ⟨o_unit, x_dec / ||x_dec||⟩`.
- At `bits = 1`, levels are sign bits and `||x_dec|| = √D` (stored
  for layout uniformity).

### Centered encoding (`encode_code_centered`, q = 1 only)

```
offset 0                 ⌈D/8⌉              +4                 +8                 +12
+------------------------+------------------+------------------+------------------+
| sign bits of (v − c)   | ||v − c||        | o_dot on unit    | ⟨x̄, c_tilde⟩    |
| rotated                | (f32 LE)         | residual (f32 LE)| (f32 LE)         |
+------------------------+------------------+------------------+------------------+
```

Same byte length as absolute at `q = 1`, **different scalar
meanings**. Absolute and centered codes are NOT interchangeable:
task 27's AM must keep them in separate containers.

## Estimator formulae (authoritative)

**Absolute path** (any q):

```
estimate = ||o|| · Σ q_i · dequant(level_i) / (o_dot · ||x_dec||)
ε²(o)   = (1 − o_dot²) / (D · o_dot²)
bound    = RABITQ_BOUND_CONFIDENCE · ||q|| · ||o|| · ε(o)
```

**Centered path** (paper §3.1 eq. 5–6, q = 1):

```
⟨x̄, q_tilde⟩  = (1/√D) · Σ q_tilde_i · sign(v_tilde_i − c_tilde_i)
unit_ip      = (⟨x̄, q_tilde⟩ − center_dot) / (||q − c|| · o_dot)
bound        = RABITQ_BOUND_CONFIDENCE · ε(o)    (||q||·||o|| factors = 1 on unit residuals)
```

The AM combines the unit-residual IP with `||q − c||` (computed
inside `score_at`) and the stored `||v − c||` (via
`centered_residual_magnitude`) through paper equation (2) to
recover any distance metric it wants.

## Invariants task 27 may rely on

- `bound ≥ 0`; `bound = 0` ⇔ `o_dot = 1` ⇔ the code is exact.
- `bound = +∞` ⇔ degenerate candidate (|o_dot| < 1e-6, non-finite
  scalars, zero-magnitude residual). Task 27 should filter via
  `bound.is_finite()` before using the estimate.
- The ε-bound is **probabilistic** at ~99% one-sided confidence
  (`C = 2.5`), not worst-case. Realized tail violation rate matches
  that nominal tail probability.
- `RaBitQScorer::score` returns `estimate_ip(...).estimate` for
  the same `(query, code)` pair — no scoring divergence between
  the trait seam and the inherent `estimate_ip` method.
- `bits_per_dim` is set at construction and cannot change for the
  lifetime of a quantizer handle. Codes produced at different
  `bits_per_dim` settings are NOT interchangeable.
- Rotation is `Send + Sync`. Multiple threads may share one
  `Arc<RaBitQQuantizer>` for encode / score. Scorers and estimator
  state are NOT `Send` — each thread prepares its own.

## What task 27 still has to do (not part of this contract)

- Graph build + quantization-aware α-pruning (`src/am/symphony/build.rs`).
- Padded-adjacency page layout + `INDEX_FORMAT_*` constant.
- Multi-visit beam-search dynamics.
- FastScan / signed-POPCNT kernel for the per-neighbor
  `⟨x̄, q_tilde⟩` hot loop (slice 15 ships the scalar reference;
  task 27 swaps in the batched kernel at its own data layout).
- End-to-end Symphony recall gate (Stage 2 and Stage 3 targets
  from ADR-045 — not retested at the quantizer level).

## What is still NOT frozen

- `wire_format_version()` currently returns `0` on `RaBitQQuantizer`.
  A dedicated `INDEX_FORMAT_RABITQ` (and
  `INDEX_FORMAT_RABITQ_CENTERED`) constant lands when task 27 first
  persists centered codes on disk.
- `RABITQ_BOUND_CONFIDENCE = 2.5` is a crate-level constant. Task 27
  may request a per-instance knob if Stage 3's candidate-pool sizer
  wants to tune confidence per index.
- SIMD. The absolute-path estimator and centered `score_at` are
  scalar references; task 27's hot loop is expected to go through
  its own batched kernel.
- Extended RaBitQ math at q > 1. The slice-12 q-bit extension uses
  uniform binning and the q=1 bound formula; Extended RaBitQ's
  Lloyd-Max codebook and q-aware bound are parked as ADR-045
  open follow-ups. Off Symphony's critical path.

## Validation

- `cargo check --lib` — clean.
- `cargo test --lib` — 549 pass (14 RaBitQ-specific tests exercise
  the surface claimed above).
- `cargo build --release -p ecaz-cli` — clean.
- `src/lib.rs` `bench_api` — exports every type / constant listed
  in the surface above, including the sidecar helpers and
  `RaBitQScorer` (reviewer flagged these as missing from v1;
  fixed in slice 16).

## Closing

Task 27 starts against this v2 contract. When it lands, the
additions it needs (if any) get recorded as a follow-up packet
amending this document — not as in-tree drift against a frozen
surface.
