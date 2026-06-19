//! Task 112: lazy heap-f32 exact-rerank frontier driver and its correctness
//! contract.
//!
//! # What this module is
//!
//! The `ec_ivf` `rerank = 'heap_f32'` stage takes the approximate candidate
//! frontier (the top-`rerank_width` heap TIDs by *quantized* score), fetches
//! each one's full-precision heap source vector, exact-scores it, and returns
//! the frontier sorted by exact score. `scan.rs` then streams those candidates
//! out of `amgettuple` in exact-score order; the executor pulls `k` of them
//! (`k <= rerank_width`) to satisfy the query `LIMIT`.
//!
//! The fixed-width path fetches and exact-scores **every** candidate in the
//! frontier before emitting any. This module adds a *lazy* driver that
//! exact-scores candidates in approximate-best-first order and can stop early
//! once it can **prove** the un-reranked candidates cannot change the result
//! the executor will pull. The driver is gated so the bench can A/B it against
//! the fixed-width path deterministically.
//!
//! # The correctness contract (Phase 2)
//!
//! Scores here are **negative inner product**: *lower is better* (this matches
//! `candidate_cmp` in `scan.rs`, which orders ascending by score). For a
//! candidate `i` let `a_i` be its approximate (quantized) frontier score and
//! `e_i` its exact heap-f32 score.
//!
//! The AM is an *ordered index scan*: it must be able to hand the executor
//! candidates in ascending `e` order, and it does **not** know `k`. So the set
//! the AM is responsible for is the entire `rerank_width` frontier reordered by
//! exact score — in the worst case the executor pulls all of it.
//!
//! A lazy stop is **recall-safe** only if every candidate we skip (do not
//! exact-score) is provably worse, by exact score, than every candidate the
//! executor could pull. Concretely, process candidates in ascending-`a` order
//! and maintain the set `R` of already-exact-scored candidates. Let
//! `worst_kept` be the largest exact score among the candidates that could
//! still be emitted. We may stop before exact-scoring the remaining suffix
//! `S` (the not-yet-fetched candidates) **iff**
//!
//! ```text
//!     for every j in S:  lower_bound(e_j) >= worst_kept
//! ```
//!
//! i.e. even in the best case each skipped candidate's exact score is no better
//! than something we already keep, so it can never enter the executor's
//! top-`k` for any `k`. `lower_bound(e_j)` must be a **sound** lower bound on
//! the exact score — never larger than the true `e_j`. The
//! [`LazyRerankBound`] trait supplies it.
//!
//! ## Why a *lower* bound, and why today's frontier cannot provide one
//!
//! Because lower exact score is better, "candidate `j` cannot beat `worst_kept`"
//! is `e_j >= worst_kept`, which we can only assert from below: we need
//! `lower_bound(e_j) >= worst_kept`. The approximate score `a_j` is **not** a
//! sound lower bound on `e_j`: quantization error is two-sided, so `a_j` may be
//! either above or below `e_j`. Using `a_j` (or `a_j` plus a guessed slack) as
//! the stop signal would drop recall whenever a candidate's true exact score
//! came in better than its quantized estimate suggested. That is the explicit
//! Non-Goal ("no recall-risky heuristic early stops").
//!
//! A *calibrated* lower bound on `e_j` — e.g. RaBitQ's bound-capable scoring
//! surface — is the missing ingredient. Producing it is **Task 113**
//! (`113-ivf-bound-aware-candidate-pruning.md`, Phase 4 "Rerank Frontier
//! Integration"). Task 113 lands [`RaBitQLazyBound`], the RaBitQ-derived
//! implementor wired into `scan.rs` in place of `NoBound`.
//!
//! ## Task 113 status: bound seam live; the latency skip still gated on two
//! remaining levers
//!
//! Even with [`RaBitQLazyBound`] wired, **no skip fires today**, by design and
//! for two independent, honest reasons:
//!
//! 1. **The k-cap is not built.** This AM is an ordered index scan with no `k`
//!    pushdown, so the executor may pull the full `rerank_width` frontier and
//!    the sound `min_kept` floor equals the width: `floor == considered`, so the
//!    stop predicate is never reached. Lowering `min_kept` below the width
//!    requires recall-safe **incremental / on-demand emission of the skipped
//!    suffix across `amgettuple` calls** — fetch the mandatory floor, evaluate
//!    the stop, and only fetch + exact-score a suffix candidate if the executor
//!    actually pulls past the floor. That is invasive (it restructures the
//!    rerank stage from a single tid-sorted batch into a resumable fetch) and is
//!    the documented remaining Phase 4 work.
//! 2. **A *tight, sound* `f(approx)` bound is not available from RaBitQ.**
//!    [`RaBitQLazyBound`] is affine (`a - slack`); a sound `slack` must be a
//!    *deterministic* worst-case error budget, but RaBitQ's only *calibrated*
//!    (tight) envelope is **probabilistic** (~99%), which is recall-risky and
//!    rejected (Task 113 Non-Goal). The deterministic worst-case `slack` is too
//!    loose to fire skips. The genuinely tight + sound path is to **carry the
//!    per-candidate Cauchy-Schwarz `ip` upper bound on the frontier candidate**
//!    (computed for free during posting scan) rather than re-derive it from the
//!    scalar approx score — a small frontier-format change tracked as remaining
//!    Phase 4 work. See [`RaBitQLazyBound`] for the full soundness analysis.
//!
//! `RaBitQLazyBound::default()` carries `slack = +inf`, so its `lower_bound` is
//! `-inf` — identical to `NoBound`. The lazy path is therefore still
//! **byte-for-byte identical** to the fixed-width path (proven by the
//! equivalence test); the bound seam is live and the stop logic is unchanged
//! when a finite bound and the k-cap land.
//!
//! See [`NoBound`] for the always-safe default, [`RaBitQLazyBound`] for the
//! RaBitQ implementor, and [`LazyRerankBound`] for the seam.

/// Sound lower bound on the exact negative-inner-product score of a candidate,
/// computed from whatever the approximate frontier carries.
///
/// # Preconditions implementors MUST uphold
///
/// 1. **Soundness.** `lower_bound(approx_score)` MUST be `<=` the candidate's
///    true exact score for every candidate whose approximate frontier score is
///    `approx_score`. A bound that can exceed the true exact score is a recall
///    bug (it would let the lazy driver skip a candidate that should have been
///    emitted).
/// 2. **Monotonicity.** `lower_bound` MUST be **non-decreasing in
///    `approx_score`**: `a1 <= a2  =>  lower_bound(a1) <= lower_bound(a2)`.
///    [`drive_lazy_rerank`] processes the suffix in ascending approx order and
///    checks the suffix *head* only; that head-only check is sound **only**
///    because monotonicity guarantees the head carries the smallest lower
///    bound in the suffix. An implementor that breaks monotonicity would make
///    the head-only stop unsound (the driver would have to scan all of the
///    suffix). `NoBound` (constant), `SlackBound` (affine), and
///    [`RaBitQLazyBound`] (affine) all satisfy this.
///
/// Today the IVF frontier carries only the two-sided quantized score, which is
/// not a sound lower bound (see module docs), so [`NoBound`] is the always-safe
/// default. Task 113 adds [`RaBitQLazyBound`], the RaBitQ-derived implementor.
pub(super) trait LazyRerankBound {
    /// A sound lower bound on the exact score of the candidate whose
    /// approximate frontier score is `approx_score`. Must never exceed the true
    /// exact score, and must be non-decreasing in `approx_score` (see the trait
    /// preconditions).
    fn lower_bound(&self, approx_score: f32) -> f32;
}

/// The sound default used until Task 113 supplies a calibrated bound: every
/// not-yet-reranked candidate might be arbitrarily good, so its lower bound is
/// `-inf`. Under this bound the lazy stop predicate provably never fires early,
/// so the lazy driver reranks the full frontier and returns results identical
/// to the fixed-width path.
pub(super) struct NoBound;

impl LazyRerankBound for NoBound {
    #[inline]
    fn lower_bound(&self, _approx_score: f32) -> f32 {
        f32::NEG_INFINITY
    }
}

/// Task 113 Phase 4: the RaBitQ-derived calibrated lower bound on the exact
/// neg-IP score, as a function of the approximate (quantized neg-IP) frontier
/// score `a`.
///
/// Scores here are neg-IP (lower is better), so `a = -ip_approx` and the exact
/// score is `e = -ip_exact`. We need `lower_bound(a) <= e`, i.e. an **upper
/// bound on `ip_exact`** expressed against `a`. RaBitQ's *sound* surface is the
/// Cauchy-Schwarz cutoff `|ip_exact| <= ||o|| * ||q|| / |o_dot|`
/// (`RaBitQQuantizer::try_estimate_ip_scalar`, audited in Task 113 Phase 1).
///
/// The frontier candidate, however, carries only the scalar approximate score
/// `a` — the per-candidate scalars (`||o||`, `o_dot`) needed for a *tight*
/// per-candidate Cauchy-Schwarz bound are dropped after scoring. So a bound
/// expressible as `f(a)` can only be the affine envelope
///
/// ```text
///     lower_bound(a) = a - slack
/// ```
///
/// which is sound **iff** `slack >= sup |e - a|` over the candidates. `slack`
/// is therefore a *deterministic worst-case quantization-error budget*. This is
/// monotone non-decreasing in `a` (affine, slope 1), satisfying the trait
/// precondition.
///
/// # Soundness vs. tightness — the honest Phase 4 finding
///
/// RaBitQ does **not** expose a deterministic, tight `f(a)` error budget:
///
/// - Its only *calibrated* (tight) per-candidate envelope
///   ([`crate::quant::rabitq::DistanceEstimate::bound`]) is **probabilistic**
///   (~99% Gaussian-tail). Using it as `slack` would be recall-risky — the
///   explicit Task 113 Non-Goal — so it is rejected.
/// - A *deterministic* worst-case `slack` from `a` alone is the codebook clip
///   range scaled by the dimension; it is sound but very loose, so it fires no
///   skips on the real IVF frontier.
///
/// The genuinely tight + sound win is to carry the per-candidate Cauchy-Schwarz
/// `ip_exact` upper bound on the frontier candidate itself (computed for free
/// during posting scan, where the scalars are in hand) rather than re-deriving
/// it from `a`. That is a frontier-format change tracked as remaining Phase 4
/// work (see module docs / the packet); it is not faked here.
///
/// Default: `slack = +inf` reproduces [`NoBound`] (the stop never fires), so
/// wiring `RaBitQLazyBound::default()` in place of `NoBound` is byte-identical
/// and recall-safe. A finite `slack` only ever activates skips that the
/// soundness precondition above guarantees are correct.
#[derive(Debug, Clone, Copy)]
pub(super) struct RaBitQLazyBound {
    /// Deterministic worst-case `sup |e - a|` budget. Must be `>= 0`. `+inf`
    /// (the default) disables early stops (equivalent to [`NoBound`]).
    slack: f32,
}

impl Default for RaBitQLazyBound {
    fn default() -> Self {
        Self {
            slack: f32::INFINITY,
        }
    }
}

impl RaBitQLazyBound {
    /// Build a bound with an explicit deterministic error budget. `slack` MUST
    /// be a sound upper bound on `sup |e - a|` for the candidates being
    /// reranked, or the lazy stop becomes a recall bug. Non-finite or negative
    /// `slack` collapses to the always-safe `NoBound` behavior.
    #[cfg(test)]
    pub(super) fn with_slack(slack: f32) -> Self {
        let slack = if slack.is_finite() && slack >= 0.0 {
            slack
        } else {
            f32::INFINITY
        };
        Self { slack }
    }
}

impl LazyRerankBound for RaBitQLazyBound {
    #[inline]
    fn lower_bound(&self, approx_score: f32) -> f32 {
        // a - inf == -inf (NoBound); a - finite_slack is affine, monotone.
        approx_score - self.slack
    }
}

/// Outcome of planning a lazy rerank pass over the approximate frontier.
///
/// `reranked_prefix_len` is how many of the approx-sorted candidates must be
/// exact-scored; the remaining `skipped` candidates are provably unable to
/// enter the executor's result and keep their approximate score (they will sort
/// after every reranked candidate and are never emitted for any `k` that the
/// fixed-width path would have emitted).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LazyRerankPlan {
    /// Total candidates in the frontier considered for rerank.
    pub considered: usize,
    /// Number of candidates (a prefix of the approx-sorted frontier) that must
    /// be exact-scored to guarantee a correct emit order.
    pub reranked_prefix_len: usize,
}

impl LazyRerankPlan {
    /// Candidates skipped by the lazy stop (heap fetch + exact score avoided).
    pub(super) fn skipped(&self) -> usize {
        self.considered.saturating_sub(self.reranked_prefix_len)
    }
}

/// Drive a lazy exact-rerank pass best-approx-first, exact-scoring candidates
/// incrementally and stopping as soon as the remaining suffix is provably
/// unable to enter the result.
///
/// `approx_scores_sorted` are the frontier candidates' approximate scores in
/// **ascending** order (best first) — the order `collect_ranked_probe_
/// candidates` produces. `fetch_exact(i)` returns the exact heap-f32 score for
/// the candidate at sorted position `i` (the heap fetch + rescore); the driver
/// calls it exactly once per reranked candidate, in ascending `i`. `bound`
/// supplies a **sound lower bound** on each candidate's exact score, derived
/// from its approximate score.
///
/// # The stop predicate (the safety proof, in code)
///
/// We keep `worst_kept` = the largest *exact* score among candidates already
/// reranked (the worst-ranked candidate we are committed to keeping). After
/// fetching at least `min_kept` candidates we may stop before reranking the
/// suffix `S = [p..]` iff
///
/// ```text
///     for every j in S:  bound.lower_bound(approx[j]) >= worst_kept
/// ```
///
/// Then every skipped candidate's exact score is `>= lower_bound >= worst_kept`,
/// so it cannot rank better than something we already keep and can never enter
/// the executor's top-`k` for any `k <= reranked_prefix_len`. Because the suffix
/// is in ascending approx order and a sound bound used here is non-decreasing in
/// approx score, the cheapest sufficient check is on the suffix head only:
/// `bound.lower_bound(approx[p]) >= worst_kept`.
///
/// # Soundness with `NoBound`
///
/// With [`NoBound`], `lower_bound(_) == -inf`, and `worst_kept` is a finite
/// exact score once anything has been fetched, so `-inf >= worst_kept` is always
/// false: the driver never stops early and reranks the full width. That makes
/// the lazy path byte-identical to the fixed-width path until Task 113 supplies
/// a calibrated finite bound — at which point the same predicate begins firing
/// with no change to this logic.
///
/// `min_kept` is the largest number of candidates the executor could pull. This
/// AM is an ordered index scan with no `k` pushdown, so callers pass the full
/// frontier width; the floor then equals the width and no skip is possible even
/// with a tight bound. A future `k`-cap (or on-demand fetch of the skipped
/// suffix) can lower `min_kept` to unlock skips — that is orthogonal to the
/// bound and is the second prerequisite documented at the module level.
///
/// Returns the [`LazyRerankPlan`] describing how many candidates were reranked.
pub(super) fn drive_lazy_rerank<B, F>(
    approx_scores_sorted: &[f32],
    bound: &B,
    min_kept: usize,
    mut fetch_exact: F,
) -> LazyRerankPlan
where
    B: LazyRerankBound,
    F: FnMut(usize) -> f32,
{
    let considered = approx_scores_sorted.len();
    let floor = min_kept.min(considered);

    let mut worst_kept = f32::NEG_INFINITY;
    let mut reranked_prefix_len = considered;
    // `i` indexes both `approx_scores_sorted` and the caller's fetch closure, so
    // the range loop is the clearest form here.
    #[allow(clippy::needless_range_loop)]
    for i in 0..considered {
        // Before fetching candidate `i`, check whether the whole suffix
        // starting at `i` can be skipped. Only allowed once the mandatory floor
        // is met.
        if i >= floor && worst_kept.is_finite() {
            // Gate on a *finite* kept floor. Two reasons:
            //   1. Until something real has been exact-scored, `worst_kept` is
            //      `-inf`, and `lower_bound(_) >= -inf` is trivially true for
            //      any finite (or even `-inf`) bound — that would let a caller
            //      that mis-feeds the floor (e.g. the `NoBound` placeholder fed
            //      `-inf` exact scores) stop spuriously and skip live
            //      candidates. Requiring finiteness makes the driver robust to
            //      that landmine (Task-112 review finding 2).
            //   2. With `NoBound` the suffix-head lower bound is `-inf`, so the
            //      predicate `-inf >= worst_kept` is false for any finite
            //      `worst_kept` — the stop still never fires, preserving the
            //      byte-identical full-width rerank.
            let suffix_head_lb = bound.lower_bound(approx_scores_sorted[i]);
            if suffix_head_lb >= worst_kept {
                reranked_prefix_len = i;
                break;
            }
        }
        let exact = fetch_exact(i);
        if exact > worst_kept {
            worst_kept = exact;
        }
    }

    LazyRerankPlan {
        considered,
        reranked_prefix_len,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test-only calibrated bound: a sound lower bound that is the approximate
    /// score minus a fixed non-negative slack. (For neg-IP, a sound lower bound
    /// is `a - slack` only if the true error is bounded by `slack`; this models
    /// what Task 113 would provide and lets us exercise the early stop.)
    struct SlackBound {
        slack: f32,
    }
    impl LazyRerankBound for SlackBound {
        fn lower_bound(&self, approx_score: f32) -> f32 {
            approx_score - self.slack
        }
    }

    /// Drive the rerank where the exact score equals the approx score (a perfect
    /// quantizer) and record which positions were actually fetched.
    fn drive_recording(
        approx: &[f32],
        bound: &impl LazyRerankBound,
        min_kept: usize,
    ) -> (LazyRerankPlan, Vec<usize>) {
        let mut fetched = Vec::new();
        let plan = drive_lazy_rerank(approx, bound, min_kept, |i| {
            fetched.push(i);
            approx[i]
        });
        (plan, fetched)
    }

    #[test]
    fn no_bound_never_stops_early() {
        let approx = [-9.0_f32, -8.0, -7.0, -6.0, -5.0];
        let (plan, fetched) = drive_recording(&approx, &NoBound, 2);
        assert_eq!(plan.considered, 5);
        assert_eq!(
            plan.reranked_prefix_len, 5,
            "NoBound must rerank full width"
        );
        assert_eq!(plan.skipped(), 0);
        assert_eq!(fetched, vec![0, 1, 2, 3, 4], "every candidate is fetched");
    }

    #[test]
    fn no_bound_empty_frontier_is_a_noop() {
        let (plan, fetched) = drive_recording(&[], &NoBound, 10);
        assert_eq!(plan.considered, 0);
        assert_eq!(plan.reranked_prefix_len, 0);
        assert_eq!(plan.skipped(), 0);
        assert!(fetched.is_empty());
    }

    #[test]
    fn min_kept_floors_the_reranked_prefix() {
        // Even a tight bound cannot skip below the mandatory kept floor.
        let approx = [-9.0_f32, -8.0, -7.0, -6.0];
        let (plan, fetched) = drive_recording(&approx, &SlackBound { slack: 0.0 }, 3);
        assert!(plan.reranked_prefix_len >= 3);
        assert!(fetched.len() >= 3);
    }

    #[test]
    fn calibrated_bound_can_stop_early() {
        // Exact == approx (zero slack). After the floor of 2, the suffix head's
        // lower bound (-7.0) is >= worst_kept (-8.0, the worst of the kept
        // prefix), so the driver stops and skips the rest.
        let approx = [-9.0_f32, -8.0, -7.0, -6.0, -5.0];
        let (plan, fetched) = drive_recording(&approx, &SlackBound { slack: 0.0 }, 2);
        assert_eq!(plan.considered, 5);
        assert_eq!(
            plan.reranked_prefix_len, 2,
            "zero-slack bound stops at floor"
        );
        assert_eq!(plan.skipped(), 3);
        assert_eq!(fetched, vec![0, 1], "only the kept prefix is fetched");
    }

    #[test]
    fn looser_bound_never_skips_more_than_a_tighter_one() {
        // A looser (larger-slack) bound forces at least as many candidates to be
        // reranked before a safe stop — soundness is preserved as it loosens.
        let approx = [-9.0_f32, -8.0, -7.0, -6.0, -5.0];
        let (tight, _) = drive_recording(&approx, &SlackBound { slack: 0.0 }, 2);
        let (loose, _) = drive_recording(&approx, &SlackBound { slack: 1.5 }, 2);
        assert!(
            loose.reranked_prefix_len >= tight.reranked_prefix_len,
            "looser bound must not skip more than a tighter one"
        );
    }

    #[test]
    fn rabitq_default_bound_matches_nobound() {
        // RaBitQLazyBound::default() (slack = +inf) must reproduce NoBound:
        // full-width rerank, no skips, byte-identical.
        let approx = [-9.0_f32, -8.0, -7.0, -6.0, -5.0];
        let (plan, fetched) = drive_recording(&approx, &RaBitQLazyBound::default(), 2);
        assert_eq!(plan.reranked_prefix_len, 5, "default bound reranks full width");
        assert_eq!(plan.skipped(), 0);
        assert_eq!(fetched, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn rabitq_finite_slack_can_stop_early() {
        // A finite, sound slack lets the affine bound fire the same early stop
        // SlackBound does — RaBitQLazyBound is the production shape of that.
        let approx = [-9.0_f32, -8.0, -7.0, -6.0, -5.0];
        let (plan, _) = drive_recording(&approx, &RaBitQLazyBound::with_slack(0.0), 2);
        assert_eq!(plan.reranked_prefix_len, 2, "zero-slack RaBitQ bound stops at floor");
        assert_eq!(plan.skipped(), 3);
    }

    #[test]
    fn rabitq_bound_is_monotone_non_decreasing() {
        // Trait precondition: lower_bound is non-decreasing in approx score.
        let bound = RaBitQLazyBound::with_slack(0.7);
        let mut prev = f32::NEG_INFINITY;
        for bits in -50i32..=50 {
            let a = bits as f32 * 0.3;
            let lb = bound.lower_bound(a);
            assert!(lb >= prev, "non-monotone at a={a}: {lb} < {prev}");
            assert!(lb <= a, "bound {lb} exceeded approx {a} (slack must be >= 0)");
            prev = lb;
        }
    }

    #[test]
    fn rabitq_negative_or_nonfinite_slack_collapses_to_nobound() {
        for bad in [-1.0_f32, f32::NAN, f32::INFINITY] {
            let bound = RaBitQLazyBound::with_slack(bad);
            assert_eq!(
                bound.lower_bound(-3.0),
                f32::NEG_INFINITY,
                "bad slack {bad} must collapse to NoBound semantics",
            );
        }
    }

    #[test]
    fn finite_floor_gate_blocks_spurious_stop_on_neg_inf_exact_scores() {
        // Reproduces the Task-112 review landmine: if a caller feeds -inf exact
        // scores (the old placeholder) AND a finite bound, the stop must NOT
        // fire while worst_kept is still -inf. With the is_finite() gate the
        // driver reranks the full width instead of skipping live candidates.
        let approx = [-9.0_f32, -8.0, -7.0, -6.0, -5.0];
        let mut fetched = Vec::new();
        let plan = drive_lazy_rerank(&approx, &RaBitQLazyBound::with_slack(0.0), 2, |i| {
            fetched.push(i);
            f32::NEG_INFINITY // every exact score is -inf (the placeholder)
        });
        assert_eq!(
            plan.reranked_prefix_len, 5,
            "with -inf exact scores the finite-floor gate must keep reranking",
        );
        assert_eq!(plan.skipped(), 0);
        assert_eq!(fetched, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn skipped_candidates_are_provably_not_better_than_kept() {
        // Property check of the contract: with exact == approx, every skipped
        // candidate's exact score is >= the worst kept exact score.
        let approx = [-9.0_f32, -8.5, -8.0, -1.0, -0.5];
        let (plan, fetched) = drive_recording(&approx, &SlackBound { slack: 0.0 }, 1);
        let worst_kept = fetched
            .iter()
            .map(|&i| approx[i])
            .fold(f32::NEG_INFINITY, f32::max);
        for (skipped, &score) in approx
            .iter()
            .enumerate()
            .take(plan.considered)
            .skip(plan.reranked_prefix_len)
        {
            assert!(
                score >= worst_kept,
                "skipped candidate {skipped} ({score}) was better than worst kept ({worst_kept})"
            );
        }
    }
}
