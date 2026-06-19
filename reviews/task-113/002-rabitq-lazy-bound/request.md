# Task 113 / 002 — Phase 4 RaBitQ lazy bound + seam fixes; Phase 2/5 prune A/B switch

Branch: `task-113-ivf-bound-aware-candidate-pruning`
Code commits: `5385de836` (lazy bound + seam fixes), `5a58bdf7a` (prune GUC + proof)
Phases: 4 (Rerank Frontier Integration), plus the Phase 2/5 posting-prune A/B
switch and recall-safety proof.

## Summary

This slice (a) lands the RaBitQ-derived lazy rerank bound seam and the two seam
fixes the Task-112 review flagged, (b) adds the `ec_ivf.posting_bound_prune` A/B
switch with a pruned==unpruned recall-safety proof, and (c) ships the two
deferred Phase-5 A/B configs. It also gives the honest soundness finding on the
lazy latency win.

## Phase 4 — the RaBitQ lazy bound (the real new work)

### What landed

- **`RaBitQLazyBound`** (`lazy.rs`): an affine `lower_bound(a) = a - slack`,
  monotone non-decreasing, default `slack = +inf` (== `NoBound`). Wired into
  `scan.rs` in place of `NoBound`. Because the default reproduces `NoBound`, the
  lazy path stays byte-identical and recall-safe — proven by the unchanged
  `test_ec_ivf_lazy_rerank_equals_fixed_width` pg18 test still passing.
- **Seam fix 1 — monotonicity as a trait precondition.** `LazyRerankBound` now
  documents soundness AND monotonicity as required preconditions; the suffix-
  head-only stop check in `drive_lazy_rerank` is sound only under monotonicity.
- **Seam fix 2 — the `-inf >= -inf` landmine.** `drive_lazy_rerank` now gates
  the early stop on `worst_kept.is_finite()`, so a caller that mis-feeds `-inf`
  exact scores can never trigger a spurious skip. And `scan.rs` no longer feeds
  the `|_| NEG_INFINITY` placeholder — it feeds the candidates' true approximate
  frontier scores, so `worst_kept` becomes a finite floor immediately. New unit
  test `finite_floor_gate_blocks_spurious_stop_on_neg_inf_exact_scores` pins
  that the gate prevents the recall collapse the reviewer warned about.

### The honest soundness finding (the crux)

**No skip fires today, for two independent reasons — and faking either would be
a recall bug:**

1. **No k-cap.** This AM is an ordered index scan with no `k` pushdown, so the
   executor can pull the full `rerank_width`. The sound `min_kept` floor equals
   the width (`floor == considered`), so the stop predicate is never reached.
   Lowering it needs recall-safe **incremental / on-demand emission of the
   skipped suffix across `amgettuple` calls** — restructuring the rerank stage
   from one tid-sorted batch into a resumable fetch. That is invasive and is the
   documented **remaining Phase 4 work** (not built here, as the 112 review
   anticipated).
2. **No tight, sound `f(approx)` bound from RaBitQ.** `RaBitQLazyBound` is
   affine in the scalar approx score `a`; a sound `slack` must be a
   *deterministic* worst-case error budget. RaBitQ's only *calibrated* (tight)
   per-candidate envelope (`DistanceEstimate.bound`) is **probabilistic** (~99%
   Gaussian-tail) and is rejected as recall-risky (Task 113 Non-Goal). The
   deterministic worst-case `slack` is too loose to fire skips.

The genuinely tight + sound win is to **carry the per-candidate Cauchy-Schwarz
`ip` upper bound on the frontier candidate** (computed for free during posting
scan, where the per-candidate scalars are in hand) rather than re-derive it from
`a` — a small frontier-format change. That, plus the k-cap, are the documented
remaining levers; both are recorded in `lazy.rs` module docs and the deferred
lazy A/B config comment.

## Phase 2/5 — posting bound-prune A/B switch + recall-safety proof

- **`ec_ivf.posting_bound_prune` GUC** (default on, `options.rs`) gates the
  running-top-k cutoff threaded into all three posting scoring sites (row, SoA
  scalar fallback, dense-block direct scan), via a shared `posting_prune_cutoff()`
  helper. Off → `None` cutoff → unpruned scan, for a clean A/B.
- **`test_ec_ivf_posting_bound_prune_equals_unpruned`** (pg18): on a RaBitQ IVF
  index the pruned scan returns **byte-identical** `(tid, tid, score)` outputs
  in identical order to the unpruned scan; prune-off records zero pruned-by-bound
  and prune-on prunes `>=` that. This is the row/dense-path recall-safety proof
  the task requires (pruned == unpruned).
- Exposed `postings_pruned_by_bound` in the debug counter snapshot.

### Phase 2/3 status recap (from packet 001 audit)

Phases 2 (row path) and the dense-block direct-scan path already thread the sound
cutoff and count `pruned-by-bound` (pre-existing, audited in packet 001). The
**batch kernels** (TurboQuant / bits1 / grouped-PQ) deliberately do NOT pre-prune
per element — they score the contiguous slab in one SIMD pass (the Task 111
direct-scan advantage) and retain after; per-element bound checks inside the
batch would break the contiguous pass. That is the evidence-backed Phase 3
"not-pruned in batch" decision (acceptance criterion 3 allows it). The
prune happens **before full score** on the scalar/row paths; on batch paths the
only available lever is at retention (already applied via `would_reject_score`).

## Task 112 acceptance-criteria advanced (for the joint closeout)

This slice advances Task 112 toward joint closure:
- 112 AC "calibrated lower bound supplied for IVF frontier": the bound seam
  (`RaBitQLazyBound`) is live, with the honest finding that a *materially-firing*
  bound needs per-candidate carriage + k-cap (documented-remaining, not faked).
- 112 review findings 1 & 2 (monotonicity precondition; placeholder + `is_finite`
  gate): **both carried and fixed** here.
- 112 AC 4 (bench evidence): the deferred joint lazy A/B config is updated and
  supersedes the 112 packet's; still recall-neutral by construction until the
  k-cap lands. Do not close 112 on this slice — close jointly when the bench
  host runs the A/B post-k-cap.

## Phase 5 — deferred bench configs (env-blocked, NFR-007: no fabricated numbers)

- `artifacts/task-113-posting-prune-ab.intel-local.json` — posting prune on/off.
- `artifacts/task-113-lazy-rerank-ab.intel-local.json` — joint 112+113 lazy A/B.

Both enumerate every Evidence Requirements field, use the standard
`[8,16,24,32,48,64]` nprobe sweep, and state the non-standard-config reason in
their `comment`. Run on the Intel bench desktop after staging `ec_real_100k`.

## Evidence

- `artifacts/lazy-unit-tests.log` — 11 passed.
- `artifacts/lazy-pg18-tests.log` — lazy 7 passed; posting_bound_prune 1 passed.
- `artifacts/cargo-clippy.log` — clean.

## Tested green

- `lazy::` unit suite (11), incl. the 5 new RaBitQ/gate tests.
- `cargo pgrx test pg18 lazy_rerank` (7) — equivalence + counters unchanged
  under the new seam.
- `cargo pgrx test pg18 posting_bound_prune` (1) — pruned == unpruned.
- The three Phase-1 cutoff tests (packet 001).

Note: pg_tests install a debug `.so`; no latency bench was run on this box, so
that is not a concern here.

## Deferred to the bench host

- Run both A/B configs (Phase 5 promotion evidence).
- The k-cap (incremental cross-`amgettuple` suffix emission) + per-candidate
  sound-bound carriage — the two remaining Phase 4 levers that turn the seam
  into an actual skip. Documented precisely in `lazy.rs`.
