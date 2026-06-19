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
//! Integration"). Until 113 lands such a bound for the IVF candidate frontier,
//! the only *sound* lower bound this module can assert is
//! `lower_bound(e_j) = -inf` — i.e. "a skipped candidate might be arbitrarily
//! good" — under which the stop predicate provably never fires and the lazy
//! driver exact-scores the full width. That makes the lazy path **byte-for-byte
//! identical** to the fixed-width path today (proven by the equivalence test),
//! while the bound seam is ready for 113 to switch the early stop on with no
//! further changes to the stop logic.
//!
//! See [`NoBound`] for that sound default and [`LazyRerankBound`] for the seam.

/// Sound lower bound on the exact negative-inner-product score of a candidate,
/// computed from whatever the approximate frontier carries.
///
/// The contract is **soundness**: `lower_bound(candidate)` MUST be `<=` the
/// candidate's true exact score for every candidate. A bound that can exceed
/// the true exact score is a recall bug (it would let the lazy driver skip a
/// candidate that should have been emitted).
///
/// Today the IVF frontier carries only the two-sided quantized score, which is
/// not a sound lower bound (see module docs), so the only correct implementor
/// is [`NoBound`]. Task 113 will add a calibrated implementor that derives a
/// real lower bound from a bound-capable quantizer (RaBitQ first).
pub(super) trait LazyRerankBound {
    /// A sound lower bound on the exact score of the candidate whose
    /// approximate frontier score is `approx_score`. Must never exceed the true
    /// exact score.
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
        if i >= floor {
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
