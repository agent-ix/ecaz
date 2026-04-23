# Review Request: Task 25 Slice 4 — Unbiased Distance Estimator + Error-Bound API

Scope:
- `src/quant/rabitq.rs`:
  - New public constants `RABITQ_ALPHA_LEN = 4` and `RABITQ_SCALAR_LEN = 8`
    (= `RABITQ_NORM_LEN + RABITQ_ALPHA_LEN`). `code_len()` grows by 4 B:
    at D=1536 the code is now 200 B (192 sign + 4 norm + 4 alpha) vs.
    slice-2's 196 B. PQ4 parity remains 768 B — margin is fine.
  - `encode_code` now writes a second scalar `α_c = mean(|c_i|)` after
    `||c||`. `prepare_scorer` keeps the full-precision rotated query
    (not just sign words).
  - New public `PreparedEstimator` struct + new `RaBitQQuantizer`
    methods `prepare_estimator(query)` and
    `estimate_ip(&prepared, code) -> DistanceEstimate`. The trait
    `QueryScorer::score` now returns `estimate.estimate`; the
    slice-2 Hamming surrogate is gone.
  - `DistanceEstimate` is now load-bearing (slice 1 had declared it
    as a typed stub; slice 4 fills in the fields with real values).
  - Two new unit tests: `estimator_recovers_self_ip_on_sign_aligned_vector`
    (exactness on inputs the binary approximation captures perfectly)
    and `estimator_bound_dominates_error_on_random_vectors` (the
    Cauchy-Schwarz envelope holds across five deterministic Gaussian
    seeds at D=256). A small `deterministic_gaussian` helper lives
    in the test module — splitmix-seeded Box-Muller, no new deps.

Task: `plan/tasks/25-rabitq-quantizer.md` (Phase 1, slice 4 of 6).
This is the load-bearing new surface for task 27 Stage 3.

Branch: `task25-rabitq-stage1-phase0` (slice 4 builds on `0e5d2a4`).

## Problem

Slice 2's `RaBitQScorer::score` was a Hamming-weighted surrogate —
just enough to exercise the round-trip. ADR-045 Stage 1 actually
asks for an unbiased inner-product estimator with a usable error
bound, because Stage 3 (task 27) eliminates the rerank pass by
sizing its candidate pool from the bound distribution. Shipping the
module without the estimator would freeze the wrong public API for
the handoff contract.

## Approach

### The estimator

The candidate vector `c` (rotated, full f32) is approximated by
`α_c · sign(c)`, where `α_c` is the least-squares coefficient:

```
α_c = argmin_α ||c − α·sign(c)||²
    = ⟨c, sign(c)⟩ / D
    = mean(|c_i|)
```

Given a query `q` (rotated, kept in full f32 at scoring time — the
asymmetric half of the estimator), the inner-product estimator is

```
⟨q, c⟩ ≈ α_c · Σ_i q_i · sign(c_i)
```

and the exact error is `⟨q, c − α_c·sign(c)⟩ = ⟨q, r_c⟩`, where
`r_c` is the residual. Cauchy-Schwarz gives

```
|⟨q, c⟩ − estimate| ≤ ||q|| · ||r_c||,
||r_c||² = ||c||² − α_c²·D.
```

The bound is tight only when `q` is aligned with `r_c`; on average
the realized error is ≪ bound. That's the gap Phase 2 measures and
Stage 3 exploits.

### What gets stored per vector

- `⌈D/8⌉` bytes of sign bits (unchanged)
- 4 B `||c||` (unchanged from slice 2)
- 4 B `α_c` (new)

At D=1536: 200 B total. `code_len()` and the scalar offsets are
derived from `RABITQ_NORM_LEN` / `RABITQ_ALPHA_LEN` constants so
layout changes propagate through one place.

### API shape — two seams, one body

- `QueryScorer::score(code) -> f32` — the scalar IP estimate only.
  This is the trait path that AM rerank sites will call through
  `&dyn QueryScorer`.
- `RaBitQQuantizer::estimate_ip(&prepared, code) -> DistanceEstimate`
  — the bound-carrying path. Stage 3's candidate pool sizer needs
  both `estimate` and `bound`, so it will reach through this
  inherent method rather than the trait.

Both funnel into the private `estimate_ip_impl`, so there is a
single place where the layout + math live. `PreparedEstimator` and
`RaBitQScorer` are intentionally separate types: the first exposes
accessors (`dimensions`, `query_norm`) that the Stage 3 caller
wants; the second is trait-bag-friendly (no lifetime/borrow
surface beyond the trait).

### Why Box-Muller in-test rather than a rand dep

The existing crate already uses `rand` / `rand_chacha` in
production code, so adding `rand` to `[dev-dependencies]` would be
an option. I kept the test helper in-file because (a) the quality
bar for a 5-seed sanity check is low, and (b) keeping the test
self-contained avoids a new dev-dep in a slice whose point is the
estimator math, not the test harness. Easy to replace with
`ChaCha8Rng + StandardNormal` in slice 5 when the feasibility
binary uses it for the recall study.

## Verification

- `cargo check --lib` clean.
- `cargo test --lib` — **542 passed** (539 baseline + 3 slice-4
  additions; the slice-3 custom-rotation test graduated into the
  estimator tests as a helper shape). 0 failures, 4 ignored (all
  pre-existing pg_test cases).
- `estimator_recovers_self_ip_on_sign_aligned_vector`: on a vector
  with coordinates `±α`, the residual is zero, so the estimator is
  exact and the bound collapses. Asserts both.
- `estimator_bound_dominates_error_on_random_vectors`: five
  deterministic Gaussian pairs at D=256. For each, asserts
  `|estimate − truth| ≤ bound + 1e-4` (the epsilon absorbs f32
  roundoff in the `||c||² − α²·D` computation).

## What this slice does NOT do

- No statistical calibration of the bound's tail. That is Phase 2
  (slice 5) on the 50k/1M real seams — the bound is mathematically
  valid here; the question for the gate is whether it is *tight
  enough* at PQ4-parity storage to hit recall@10 within 1pp.
- No SIMD for the inner-product kernel. The slice-4 body iterates
  one coordinate at a time (`q_i · sign(c_i)` accumulated). SIMD
  lands when the feasibility study confirms this estimator is the
  one going into Stage 2, so we SIMD a frozen formula, not a moving
  target.
- No persistent per-vector α in the AM sidecar. ADR-031's persisted
  sidecar is sign-only; slice 4's α lives in the RaBitQ code, not
  the ADR-031 sidecar. Phase 3 will decide whether to extend the
  sidecar format (new INDEX_FORMAT_* constant) or keep two separate
  per-vector tables.

## Open questions for reviewer

1. `DistanceEstimate.bound` semantics: I named it `bound` (plain
   Cauchy-Schwarz envelope). Alternatives: `epsilon` (matches the
   RaBitQ paper) or `residual_norm_times_query_norm` (descriptive,
   ugly). Freezing the name before slice 6 (the handoff contract)
   matters because task 27's code will read this struct directly.
2. The asymmetric choice: query full-precision, candidate binary.
   Symmetric RaBitQ (both binary) is ~2× faster but drops a bit of
   recall. Happy to keep the asymmetric estimator in Stage 1 and
   swap to symmetric in Stage 3 under a reloption if Phase 2 data
   says it's safe.
3. `α_c = mean(|c_i|)`: computed in f32 directly. For very high D
   (our 1536 case) this is fine; below D=32 the two-pass algorithm
   would be worth it. Not relevant for our corpus seams.
