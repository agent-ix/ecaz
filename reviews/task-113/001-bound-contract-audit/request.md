# Task 113 / 001 — Phase 1: RaBitQ bound contract audit + prune-cutoff soundness tests

Branch: `task-113-ivf-bound-aware-candidate-pruning`
Code commit: `274b02743`
Phase: 1 (Bound API Audit) — satisfies Acceptance Criterion 1.

## Summary

Phase 1 is the honest audit gate. I audited the existing RaBitQ candidate
scoring APIs and documented, in code, exactly which surface yields a sound
bound and which does not. **The stop condition does NOT fire**: RaBitQ already
carries a sound, deterministic prune bound, and it is already threaded into the
IVF scan paths. So Phase 1 closes with a green light to proceed (not "no scan
behavior change").

## The two RaBitQ bound surfaces

RaBitQ exposes two completely different "bounds"; conflating them would be a
recall bug:

1. **Cauchy-Schwarz prune cutoff** — `RaBitQQuantizer::try_estimate_ip_scalar`
   and `try_estimate_ip`, both taking `min_ip_to_keep`. The cutoff value is
   `max_estimate = ||o|| · ||q|| / |o_dot|`, a **deterministic upper bound** on
   the asymmetric estimate (`src/quant/rabitq.rs:1160-1168`). Because the true
   estimate provably cannot exceed it, pruning a candidate whose
   `max_estimate < min_ip_to_keep` is **recall-safe** — that candidate can never
   reach the running top-k frontier. Direction: upper bound on a *higher-is-
   better* IP, so it prunes from below the cutoff. Monotone in the threshold.
   **This is the sound prune surface.**

2. **ε-concentration envelope** — `DistanceEstimate.bound`, the symmetric
   `2.5 · ||q|| · ||c|| · √ε²` Gaussian-tail term (`rabitq.rs:4200-4202`,
   `RABITQ_BOUND_CONFIDENCE = 2.5`). This is a **probabilistic ~99% confidence**
   bound, *not* deterministic — by construction a candidate's true error can
   exceed it. Using it as a hard skip threshold would drop recall. That is the
   explicit Task 113 Non-Goal, so it is **not** used to prune. The existing
   `estimator_bound_dominates_error_on_random_vectors` test only asserts it holds
   *empirically* over fixed seeds — it is a calibration check, not a soundness
   proof.

## Where the sound cutoff is already wired (Phase 2/3 status)

The `min_ip_to_keep` cutoff is already threaded into all three IVF row/scalar
posting paths, deriving the threshold from the running top-k worst score and
counting prunes via `record_posting_pruned_by_bound`:

- Row posting path: `scan.rs:1589-1602` (and the no-SoA visitor at `1807-1820`).
- Dense posting block direct-scan path: `scan.rs:1997-2012`.
- SoA scalar fallback: `scan.rs:1806-1820`.

The implication for Task 113: **Phase 2 (row path) is already landed** and
recall-safe by construction (the cutoff is a sound upper bound). Phase 3's
**batch/dense-block** kernels (TurboQuant / bits1 / grouped-PQ batch) score the
whole contiguous slab and then retain — they do *not* pre-prune per element.
That is the deliberate 111 direct-scan/contiguous-copy advantage; per-element
bound checks inside the batch kernel would break the contiguous SIMD pass.
Evidence-backed Phase 3 position (acceptance criterion 3 allows this): batch
kernels keep full-slab scoring; only the scalar/row fallback prunes pre-score.
I will state this in the Phase 2/3 packet with counter evidence.

## What this slice adds

The cutoff surface (the actual IVF prune surface) had **zero** unit tests. Added
three to `src/quant/rabitq.rs`:

- `try_estimate_scalar_cutoff_never_prunes_a_keepable_candidate` — soundness:
  over a 64-code × 81-cutoff grid, any pruned candidate's *unpruned* estimate is
  strictly below the cutoff (so the drop was provably safe), and any kept
  estimate matches the unpruned value.
- `try_estimate_scalar_cutoff_is_monotone_in_threshold` — raising the cutoff
  only ever prunes more, never un-prunes.
- `try_estimate_bound_carrying_cutoff_agrees_with_scalar` — the bound-carrying
  `try_estimate_ip` prunes on the same cutoff as the scalar fast path.

Also recorded the two-surface contract as a doc comment on
`try_estimate_ip_scalar` (Acceptance Criterion 1: bound contract documented in
code).

## Evidence

- `artifacts/phase1-cutoff-tests.log` — 3 passed.
- Validation: `cargo clippy --lib --no-default-features --features pg18 -D warnings` clean.

## Next

Phase 4 (slice 002): a calibrated **sound** `LazyRerankBound` for RaBitQ derived
from surface (1), plus the two seam fixes the Task-112 reviewer flagged
(trait monotonicity precondition; harden `drive_lazy_rerank` on
`worst_kept.is_finite()` + true exact-score feeding; k-cap or documented-
remaining). See `reviews/task-112/001-.../feedback/2026-06-19-01-reviewer.md`.
