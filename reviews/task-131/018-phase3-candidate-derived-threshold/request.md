# Task 131 Review Request: Phase 3 Candidate-Derived Threshold

Head: `393b92548fba88aad6adcca987db3db48a72b864`

## Scope

This checkpoint removes the manual scalar from the production threshold-profile diagnostic:

- adds `ec_spire_remote_search_production_candidate_threshold_profile(index_oid, query, top_k)`
- derives `threshold_score` from the merged global compact candidate frontier
- includes local compact candidates and remote compact candidate batches in the same validated merge path
- uses the kth compact candidate score only when the merged frontier has at least `top_k` rows
- returns zero profile rows when there is no kth frontier, rather than inventing an unsafe threshold
- reuses packet 017's production threshold-profile fanout to ask each worker how many selected blocks/rows have sound upper bounds below that derived scalar

This remains diagnostic-only. It does not push threshold updates during scan, does not early-stop worker scanning, and does not claim a latency win.

## Validation

Artifacts are listed in `artifacts/manifest.md`.

- `cargo check --lib`
- `cargo test --lib global_compact_candidate_threshold_score_requires_full_top_k_frontier`

Both passed.

## Next Work

Use this derived threshold in the local multi-instance fixture to measure boundability from real compact candidate frontiers, then move the scalar from post-candidate diagnostics toward scan-time update/early-stop accounting.
