# Task 123 Review Request: Dedupe-Aware Prune Threshold

## Scope

This packet addresses the reviewer blocker that `ec_spire.pre_materialization_prune` was inert for the task's candidate configs (`boundary_replica_count=2/4`) because those configs use `VecIdDedupeEnabled`.

The code change is commit `d2ffbdaa9 Enable SPIRE prune threshold for dedupe`.

## Change

`SpireScoredCandidateAccumulator::pre_materialization_min_ip_to_keep()` now delegates to the existing `min_ip_to_keep()` implementation for all dedupe modes instead of returning `None` for `VecIdDedupeEnabled`.

That existing deduped path computes the threshold from the live retained unique-vec-id set:

- bounded by `candidates_by_vec_id.len()`
- worst candidate selected by `peek_live_worst_deduped()`
- strict prune condition remains `ip < min_ip_to_keep`

The strict comparison preserves equal-score/tie-break candidates for materialization and existing deterministic ordering.

## Validation

Artifacts:

- `artifacts/manifest.md`
- `artifacts/cargo-fmt-check.log`
- `artifacts/cargo-test-pre-materialization-dedupe.log`
- `artifacts/cargo-test-bounded-dedupe.log`
- `artifacts/cargo-test-bounded-best-deduped.log`

Commands/results:

- `cargo fmt --check`: passed, with existing stable-toolchain rustfmt warnings.
- `cargo test miri_pre_materialization_prune_threshold_engages_for_bounded_dedupe -- --nocapture`: passed.
- `cargo test bounded_dedupe -- --nocapture`: passed.
- `cargo test miri_rank_routed_leaf_rows_by_ip_keeps_bounded_best_deduped_candidates -- --nocapture`: passed.

## Review Notes

This is the code checkpoint that should make the b2/b4 prune A/B meaningful. It does not close Task 123 by itself. The next step is to rerun the local multi-instance prune on/off measurement after this commit and compare against packet 017, now that the prune guard can engage under `VecIdDedupeEnabled`.
