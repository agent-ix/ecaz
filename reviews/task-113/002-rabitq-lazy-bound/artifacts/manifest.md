# Manifest — Task 113 / 002 RaBitQ lazy bound + posting-prune A/B switch

- Head SHA: `5a58bdf7a` (code commits `5385de836` + `5a58bdf7a` on
  `task-113-ivf-bound-aware-candidate-pruning`)
- Task bucket: `reviews/task-113/`
- Packet path: `reviews/task-113/002-rabitq-lazy-bound/`
- Type: code review + deferred bench configs. No benchmark run (env-blocked).
- Storage format / quant: RaBitQ (prune equivalence test + prune A/B);
  coarse_rerank/heap_f32/f32/table (lazy A/B).
- Rerank mode: heap_f32 (lazy A/B), n/a for the posting-prune path.
- Isolation: one-index-per-table; the deferred A/B configs build one matched
  index per scale and A/B by session GUC, holding the index fixed.

## Artifacts

- `lazy-unit-tests.log` — `cargo test --lib --no-default-features --features pg18 lazy::`
  → `11 passed`.
- `lazy-pg18-tests.log` — `cargo pgrx test pg18 lazy_rerank` (`7 passed`) and
  `cargo pgrx test pg18 posting_bound_prune` (`1 passed`).
- `cargo-clippy.log` — `cargo clippy --all-targets --no-default-features
  --features pg18 -- -D warnings` clean.
- `task-113-posting-prune-ab.intel-local.json` — DEFERRED Phase 5 A/B
  (posting_bound_prune on/off), env-blocked here, ready for the bench host.
- `task-113-lazy-rerank-ab.intel-local.json` — DEFERRED Phase 5 joint 112+113
  lazy A/B (lazy_heap_rerank on/off); supersedes the 112 packet's config.

## Key result lines cited by request.md

- `test am::ec_ivf::lazy::tests::rabitq_default_bound_matches_nobound ... ok`
- `test am::ec_ivf::lazy::tests::finite_floor_gate_blocks_spurious_stop_on_neg_inf_exact_scores ... ok`
- `test am::ec_ivf::lazy::tests::rabitq_bound_is_monotone_non_decreasing ... ok`
- `test tests::pg_test_ec_ivf_lazy_rerank_equals_fixed_width ... ok`
- `test tests::pg_test_ec_ivf_posting_bound_prune_equals_unpruned ... ok`
- `test result: ok. 7 passed` (lazy pg18), `test result: ok. 1 passed` (prune pg18)

## Code references

- `RaBitQLazyBound` + trait monotonicity precondition + finite-floor gate:
  `src/am/ec_ivf/lazy.rs`.
- Live bound seam (RaBitQLazyBound::default() replaces NoBound; true exact-score
  feeding): `src/am/ec_ivf/scan.rs:2175-2180`.
- `ec_ivf.posting_bound_prune` GUC: `src/am/ec_ivf/options.rs`.
- `posting_prune_cutoff()` + three prune sites + snapshot field:
  `src/am/ec_ivf/scan.rs`.
