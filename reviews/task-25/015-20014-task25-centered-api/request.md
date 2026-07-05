# Review Request: Task 25 Slice 15 — Centered Encode + Score API (Symphony Prerequisite)

Scope:
- `src/quant/rabitq.rs` — four new inherent methods on
  `RaBitQQuantizer` plus two new public types, implementing the
  Symphony paper's §3.1 per-vertex centered RaBitQ path:

  - `struct CenterContext { rotated, raw }` — pre-computed per-
    vertex state (rotated + raw center vector).
  - `RaBitQQuantizer::prepare_center(center) -> CenterContext`.
  - `RaBitQQuantizer::encode_code_centered(v, center) -> Box<[u8]>`
    — encodes the unit-normalized rotated residual against the
    center. Asserts `bits_per_dim = 1`.
  - `RaBitQQuantizer::prepare_scorer_centered(query) -> CenteredScorer`.
  - `struct CenteredScorer` with `score_at(code, center) ->
    DistanceEstimate`.
  - `RaBitQQuantizer::centered_residual_magnitude(code) -> f32`
    accessor for the AM to read `||v − c||` without re-parsing
    the code.

- `src/lib.rs` `bench_api` — re-exports `CenterContext` and
  `CenteredScorer`.

- `spec/adr/ADR-045-...md` — "Per-center RaBitQ API (Symphony
  Stage 2 prerequisite)" open-follow-up upgraded from "proposed"
  to "landed" with the actual method signatures.

Task: `plan/tasks/25-rabitq-quantizer.md` (slice 15; closes the
Symphony prerequisite).

Branch: `task25-rabitq-stage1-phase0` (slice 15 builds on `147b52e`).

## What the centered path implements

Paper eq (6):

```
⟨x̄, P⁻¹q⟩ = (1/||q_r − c||) · (⟨x̄, P⁻¹q_r⟩ − ⟨x̄, P⁻¹c⟩)
```

Amortization at build / query time:

| scalar                | when computed          | where stored      |
|-----------------------|------------------------|-------------------|
| `c_rotated`           | `prepare_center`       | `CenterContext`   |
| `⟨x̄, c_tilde⟩`       | `encode_code_centered` | code tail (4 B)   |
| `q_rotated`           | `prepare_scorer_centered` | `CenteredScorer` |
| `⟨x̄, q_tilde⟩`       | `score_at` (per-code)  | —                 |
| `||q_r − c||`         | `score_at` (per-visit) | —                 |
| `||v − c||`           | `encode_code_centered` | code tail (4 B)   |

Code layout at `bits = 1` (Symphony's config):
```
[sign bits: ⌈D/8⌉ B][||v − c||: 4 B][o_dot: 4 B][⟨x̄, c_tilde⟩: 4 B]
```
Same byte length as the absolute-encoded code at q=1 — the
`center_dot` scalar lives in the slot that `||x_dec||` occupies
in the absolute path. The two code shapes are **not** byte-
compatible (different meaning of the third scalar), so task 27's
AM layer must keep absolute-encoded and centered-encoded codes
in separate containers. A future `INDEX_FORMAT_*` constant bump
in `src/am/page.rs` will disambiguate on-disk.

## Design choices

### Centered path is q=1 only

Symphony uses q=1 exclusively per the paper §2.2 equation (3).
`encode_code_centered` and `prepare_scorer_centered` assert
`bits_per_dim = 1`; q>1 centered is a follow-up if/when a non-
Symphony consumer requests it.

### `score_at` returns unit-residual IP, not L2

The AM composes the unit IP with `||q − c||` (per-visit) and
`||v − c||` (per-code, read via `centered_residual_magnitude`)
to recover any distance metric it wants (L2, cosine, IP).
Keeping the quantizer metric-agnostic means the same centered
path serves Symphony's L2 search and any future IP-flavored
consumer.

### `score_at` computes `||q − c||` internally

`CenteredScorer` stashes the raw query vector; `score_at`
computes `||q − c||` on the fly per visit. Cost is O(D), small
relative to the per-neighbor `⟨x̄, q_tilde⟩` loop, and it avoids
forcing the AM to feed the same number into every score_at call.
Symphony's hot loop will still call score_at once per neighbor
and amortize the `||q − c||` over all neighbors of one vertex —
future optimization can cache the value on `CenteredScorer` if
profiling says so.

### `bound` collapses `||q||·||o||` to 1

Both query residual and data residual are unit vectors in the
centered path, so the ε-bound on `DistanceEstimate.bound`
reduces to `C · ε(o_dot)` with `C = RABITQ_BOUND_CONFIDENCE`.
The `||q|| · ||o||` factors in the absolute-path bound drop out.

### Scalar inner loop, not FastScan

`score_at` computes `⟨x̄, q_tilde⟩` as a scalar sum. FastScan /
signed-POPCNT in the hot path is task-27 territory — it requires
batching across multiple neighbor codes packed together, which
is an AM-side data-layout decision. Slice 15 is the correctness
reference; the AM-side kernel plugs in alongside.

## Verification

- `cargo check --lib` clean.
- `cargo test --lib` — **549 passed** (546 baseline + 3 slice-15
  additions). 0 regressions.
- Three new tests in `quant::rabitq::tests`:
  - `centered_estimator_is_exact_on_sign_aligned_residual` —
    self-IP on a residual with all-equal |coord| hits `o_dot = 1`,
    estimator is exact, bound collapses.
  - `centered_estimator_bound_dominates_error_on_random_vectors`
    — five deterministic Gaussian seeds over (center, v, query);
    the ε-bound envelopes realized unit-residual-IP error on
    every seed.
  - `centered_api_rejects_qbit_bits` — constructing with
    `bits_per_dim = 4` and calling `encode_code_centered` panics
    with a clear message (guards against silent misuse).

## Containment audit

- **ADR-031 production prefilter (`ec_hnsw`)**: untouched. It uses
  only `sign_words_from_packed_4bit` + `hamming_similarity`;
  neither is centered-aware.
- **`ProdQuantizer`**: untouched.
- **`src/am/**`: no file changed.
- **Trait surface (`Quantizer`, `QueryScorer`)**: unchanged.
  Centered path is inherent-methods-only.
- **Absolute encode / estimate path**: byte-for-byte identical
  behavior, same 14 unit tests (+ the 3 new ones).
- **Feasibility harness (`ecaz quant feasibility`)**: unchanged.
  Its purpose — comparing against exact in the *absolute* frame —
  is orthogonal to Symphony's residual frame. Task 27's end-to-
  end test is the right home for centered-path recall numbers.

## What this slice does NOT do

- No FastScan / SIMD on the centered hot loop.
- No `bits_per_dim > 1` centered path.
- No new feasibility harness mode. A "centered + residual corpus"
  recall study does not make sense in isolation because the
  residual choice depends on graph structure; that is Symphony's
  end-to-end system test (task 27).
- No `INDEX_FORMAT_*` constant for on-disk centered codes. Lands
  with task 27's AM persistence work.
- No amendment to the slice-6 handoff contract. The contract's
  absolute-path API is still correct; the centered path is
  additive (inherent methods). Amendment comes when task 27
  freezes its own centered-path dependencies.

## Closing

Task 25 (RaBitQ Stage 1) now includes everything Symphony needs
as a quantizer primitive. Task 27 starts with:
1. A 1-bit paper-faithful RaBitQ (slice 9)
2. A rotation seam (slice 3) + seeded SRHT rotation (slice 13)
3. The centered encode / score API (this slice)

The remaining task-27 work is purely AM-level: graph build with
the quantization-aware pruning rule, the beam-search data
layout, multi-visit beam dynamics, and end-to-end recall
validation. All of that lives in `src/am/symphony/` — no further
quantizer-module changes needed.
