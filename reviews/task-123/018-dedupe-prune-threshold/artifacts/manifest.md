# Task 123 Packet 018 Artifact Manifest

- Head SHA: `d2ffbdaa91837adc2f38b667aab8fa31e039d501`
- Task bucket: `reviews/task-123/018-dedupe-prune-threshold`
- Timestamp: `2026-06-29`
- Scope: code review packet for enabling pre-materialization prune threshold engagement under `VecIdDedupeEnabled`.
- Code commit under review: `d2ffbdaa9 Enable SPIRE prune threshold for dedupe`
- Changed files:
  - `src/am/ec_spire/scan/candidates.rs`
  - `src/am/ec_spire/scan/tests/candidates.rs`

## Rationale

Reviewer feedback in packets 014 and 017 identified that the rehomed pre-materialization prune was inert for the task's b2/b4 candidate configs because `boundary_replica_count > 0` selects `VecIdDedupeEnabled`, and `pre_materialization_min_ip_to_keep()` previously returned `None` for all deduped scans.

The fix removes that early `None` and reuses the accumulator's existing `min_ip_to_keep()` logic. For bounded deduped scans, that logic uses `candidates_by_vec_id.len()` and `peek_live_worst_deduped()` to compute the current retained unique-vec-id worst candidate.

The prune condition remains strict (`ip < min_ip_to_keep`), so equal-score/tie-break candidates are still materialized and can participate in the existing deterministic ordering.

## Validation

Formatting:

```sh
script -q -e -c 'cargo fmt --check' \
  reviews/task-123/018-dedupe-prune-threshold/artifacts/cargo-fmt-check.log
```

Result: passed. The log contains the repository's existing stable-toolchain warnings about nightly-only rustfmt import options.

Focused regression:

```sh
script -q -e -c 'cargo test miri_pre_materialization_prune_threshold_engages_for_bounded_dedupe -- --nocapture' \
  reviews/task-123/018-dedupe-prune-threshold/artifacts/cargo-test-pre-materialization-dedupe.log
```

Result: passed, `1 passed; 0 failed`.

Nearby bounded-dedupe checks:

```sh
script -q -e -c 'cargo test bounded_dedupe -- --nocapture' \
  reviews/task-123/018-dedupe-prune-threshold/artifacts/cargo-test-bounded-dedupe.log
```

Result: passed, `2 passed; 0 failed`.

```sh
script -q -e -c 'cargo test miri_rank_routed_leaf_rows_by_ip_keeps_bounded_best_deduped_candidates -- --nocapture' \
  reviews/task-123/018-dedupe-prune-threshold/artifacts/cargo-test-bounded-best-deduped.log
```

Result: passed, `1 passed; 0 failed`.

## Follow-up Measurement

This packet is the code checkpoint that makes the prune arm engage for b2/b4 candidates. It is not a task closeout packet. Because it changes scan behavior, the next packet must rerun the candidate A/B measurement with prune on/off after this commit and store benchmark evidence under the task bucket before any closeout claim.
