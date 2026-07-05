# Review Request: Task 43 SPIRE And Vacuum Miri Prefixes

## Summary

This checkpoint promotes four existing pure tests into the `miri_` lane:

- DiskANN vacuum neighbor repair compaction and padding.
- DiskANN vacuum repair encoded-length preservation.
- SPIRE ranked routed leaf candidate bounded vec-id dedupe.
- SPIRE scan candidate cursor one-shot ranked emission.

No behavior changed. The code change is limited to adding the `miri_` prefix
to bounded unit tests that were already in the normal Rust test suite.

## Review Focus

- Confirm these tests are pure enough to be permanent Miri lane members.
- Confirm the selected SPIRE cases are representative for Task 43's top-k /
  candidate merge coverage goal.
- Confirm the selected DiskANN vacuum cases cover the highest-risk pure tuple
  repair contracts without pulling pgrx callbacks into Miri.

## Validation

Validation artifacts are in `artifacts/` and summarized by
`artifacts/manifest.md`.

- `cargo +nightly miri test --lib miri_vc_006_repair_neighbors_compacts_and_pads`
  passed.
- `cargo +nightly miri test --lib miri_vc_009_repair_preserves_encoded_length`
  passed.
- `cargo +nightly miri test --lib miri_rank_routed_leaf_rows_by_ip_keeps_bounded_best_deduped_candidates`
  passed.
- `cargo +nightly miri test --lib miri_scan_candidate_cursor_emits_ranked_candidates_once`
  passed.

No full Miri run is claimed in this packet.

