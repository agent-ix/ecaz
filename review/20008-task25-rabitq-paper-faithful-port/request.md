# Review Request: Task 25 Slice 9 — Paper-Faithful RaBitQ Estimator Port

Scope:
- `src/quant/rabitq.rs`:
  - The per-vector scalar stored alongside the sign bits changes
    from `α_c = mean(|c_i|)` (slice-4 "least-squares linear fit of
    α·sign(c)") to `o_dot = ⟨o_unit, sign(o)/√D⟩` — the cosine
    between the unit-normalized rotated vector and its normalized
    sign vector. This is what the RaBitQ paper stores. Byte layout
    unchanged (still 4 B), so `code_len()` is stable.
  - `estimate_ip_impl` now computes the paper's asymmetric
    estimator:
    ```
    estimate = ||o|| · Σ_i q_i · sign(o_i) / (o_dot · √D)
    ```
    The division by `o_dot` is the key cancellation that the
    slice-4 form was missing; it absorbs the variance introduced
    by the binary approximation.
  - `DistanceEstimate.bound` now carries the ε-concentration
    bound from the paper (§"Error bound"):
    ```
    ε²(o) = (1 − o_dot²) / (D · o_dot²)
    bound = C · ||q|| · ||o|| · ε(o)      with C = RABITQ_BOUND_CONFIDENCE
    ```
    Replaces slice-4's Cauchy-Schwarz worst-case envelope. New
    public constant `RABITQ_BOUND_CONFIDENCE = 2.5` (≈ 99% one-
    sided Gaussian-tail confidence).
  - Degenerate `o_dot` (within `1e-6` of zero, or non-finite)
    returns `estimate = 0`, `bound = +∞`. Stage 3 can treat those
    as "unscorable" candidates and fall back to exact rerank.
  - Constant rename: `RABITQ_ALPHA_LEN` → `RABITQ_UNIT_DOT_LEN`.
  - `mean_abs` helper deleted (no longer used).
  - Module-level doc comment updated to describe the new scalar
    semantics.
- `src/lib.rs` `bench_api` — re-export updated to the new
  constant name + the new confidence constant.
- Two in-module unit tests updated:
  - `estimator_recovers_self_ip_on_sign_aligned_vector` — still
    exact because on an `±α` vector, `o_dot = 1`, `ε = 0`, and
    the paper's formula collapses to `||o||² = ⟨v, v⟩`.
  - `estimator_bound_dominates_error_on_random_vectors` — same
    five deterministic Gaussian seeds; the ε-bound now holds.

Task: `plan/tasks/25-rabitq-quantizer.md` (slice 9 of 10 — added
after the slice-8 FAIL verdict flagged that the slice-4 estimator
was not paper-faithful).

Branch: `task25-rabitq-stage1-phase0` (slice 9 builds on `e15b37c`).

## Problem

The slice-4 estimator used `α_c = mean(|c_i|)` — the least-squares
coefficient for approximating `c ≈ α·sign(c)`. That coefficient
is *correct* as a linear fit; it is *not* what RaBitQ's
concentration proof uses. The paper stores the cosine
`⟨o_unit, sign(o)/√D⟩` and divides by it in the estimator to cancel
the sign-bit quantization variance. Without that cancellation, the
estimator has extra variance that shows up as error at scoring
time — which is most of what we measured as the 10.65 pp recall
gap on DBpedia-10k in slice 8.

The slice-6 handoff contract to task 27 documented the slice-4
semantics. That was premature — it locked in a non-paper-faithful
shape. Slice 9 rectifies the math before anyone consumes the
contract (task 27 has not started).

## Approach

### Why the rename

`RABITQ_ALPHA_LEN` → `RABITQ_UNIT_DOT_LEN`. The old name meant
"the α in `α·sign(c)`". The new name means "the dot product with
the normalized sign vector" — `o_dot`. Same 4 B, different scalar,
different math. Renaming the constant makes mis-reads at call
sites harder.

### Why `RABITQ_BOUND_CONFIDENCE = 2.5`

The paper's concentration proof gives `P(|err| ≤ ε) ≥ 1 − δ`
where `ε` scales with a coefficient tied to the desired `δ`.
Practical RaBitQ implementations use constants between 1.9 (≈95%)
and 3.0 (≈99.7%) depending on how conservative the downstream
candidate-pool sizer wants to be. `2.5` ≈ 99% one-sided is the
middle-ground default; a future slice can add
`RaBitQQuantizer::with_bound_confidence(c)` if Stage 3 wants to
tune per-query.

### Why `O_DOT_FLOOR = 1e-6` and `bound = +∞` for degenerate vectors

`o_dot = 0` would mean the rotated vector is perfectly
orthogonal to its own sign vector — mathematically impossible
for nonzero `o` (since `o_i · sign(o_i) ≥ 0` for all `i`), but
f32 roundoff or exactly-zero inputs can hit it. Returning
`+∞` for the bound lets downstream callers filter those cases
cleanly with a single `bound.is_finite()` check rather than
re-checking `o_dot` at every score site.

### What the on-disk code looks like now

Byte offsets unchanged:

```
offset 0                 ⌈D/8⌉                 ⌈D/8⌉+4         ⌈D/8⌉+8
+------------------------+------------------+------------------+
| sign bits (LSB-first)  |  ||o|| (f32 LE)  | o_dot (f32 LE)   |
+------------------------+------------------+------------------+
```

Same layout as slice 4. Old codes produced under slice 4 would
be *readable* but would give wrong estimates (the scorer would
treat `mean(|c_i|)` as `o_dot` and divide by the wrong scalar).
Because no AM path has ever written RaBitQ codes to disk (the
ADR-031 prefilter uses only the sign bits, no scalars), there
are no codes in the wild to migrate.

## What this slice does NOT do

- No SIMD in the estimator inner loop. Same scalar loop as slice 4.
- No persistent sidecar format change in the AM. ADR-031's
  binary sidecar stores only sign words; it does not carry the
  RaBitQ scalars. Task 27 Stage 2 will decide whether to extend
  the sidecar (new `INDEX_FORMAT_*`) or keep RaBitQ codes in a
  separate per-vector table.
- No amendment to `review/20005-task25-task27-handoff-contract/request.md`.
  The contract still reads the old semantics; a follow-up packet
  (the same one that carries the slice-10 verdict) will amend it.
  Deferring the amendment lets the reviewer see what changed
  without diffing a stale document.

## Verification

- `cargo check --lib` clean; no warnings.
- `cargo test --lib` — **542 passed, 0 failed**. Seven
  `quant::rabitq::tests` exercises pass against the new math:
  - `code_len_matches_dimension` — layout constants resolve.
  - `sign_words_from_rotated_matches_manual_pack` — bit pack invariant.
  - `encode_then_score_same_vector_is_nonnegative` — round-trip smoke.
  - `hamming_similarity_identity_equals_dim` — ADR-031 primitive invariant.
  - `custom_rotation_plugs_into_seam` — trait seam regression.
  - `estimator_recovers_self_ip_on_sign_aligned_vector` — paper-
    faithful exact case (`o_dot = 1` → `ε = 0` → estimate = truth).
  - `estimator_bound_dominates_error_on_random_vectors` — five
    deterministic Gaussian seeds, `err ≤ bound` on every one at
    `RABITQ_BOUND_CONFIDENCE = 2.5`.

The slice-10 packet will carry the real-corpus verdict — the
number that actually matters.

## Containment audit

Per the conversation earlier: paper-faithful math is contained to
`src/quant/rabitq.rs`. Audited:

- **ADR-031 production prefilter (`ec_hnsw`)**: untouched. Uses
  `sign_words_from_packed_4bit` + `hamming_similarity` only — no
  scalars, no estimator. All 539 AM-related tests pass unchanged.
- **`ProdQuantizer::binary_sign_*` methods**: untouched (they
  delegate into the Hamming primitives, not the estimator).
- **`src/am/**`: no file changed in this slice.
- **`bench_api` re-exports**: one constant renamed, one added.
  Zero external consumers today.
- **`ecaz quant feasibility`** (slice 7): CLI signature unchanged;
  sees new numbers through the same `estimate_ip` call.

## Open questions for reviewer

1. `RABITQ_BOUND_CONFIDENCE` exposed as a crate-level constant.
   Alternative: fold it into a `RaBitQQuantizer::with_bound_confidence(c)`
   constructor so Stage 3 can tune per-instance. Happy to do the
   constructor form if you'd rather not carry a knob at module
   scope.
2. Degenerate-`o_dot` behavior is `bound = +∞`. Stage 3 can
   equivalently treat it as "candidate always survives the bound
   check" or "candidate is unscorable". I chose the latter; if
   you want the former, change the sentinel to
   `bound = query_norm * candidate_norm * LARGE` instead.
3. The ε-concentration bound is symmetric (`|err| ≤ bound`); the
   paper occasionally uses an asymmetric tail. For Stage 3's
   candidate-pool sizing the symmetric form is more conservative
   and therefore safer; I left it symmetric.
