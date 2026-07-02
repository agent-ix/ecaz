# Task 131 Review Request: Phase 3 Production Threshold Profile

Head: `d3ee38b97ace209c78c460016de5137f6a762415`

## Scope

This checkpoint extends packet 016's local threshold-bound diagnostic to the production fanout shape:

- adds `ec_spire_remote_search_production_threshold_profile(index_oid, query, top_k, threshold_score)`
- selects the same production leaf set as `ec_spire_remote_search_production_scan_profile`
- evaluates local selected leaves directly
- dispatches the scalar `threshold_score` to remote workers through libpq using `ec_spire_remote_search_coordinator_local_threshold_profile`
- decodes one threshold-profile row per remote dispatch and remaps the worker's local node id to the coordinator node id, matching the existing scan-profile behavior

This is still diagnostic-only. It does not alter result-producing scans, does not early-stop workers, and does not claim a latency win. The value is that the coordinator can now ask every production worker, for a proposed global kth compact score, how many selected blocks/rows have sound full-radius upper bounds below that scalar threshold.

## Validation

Artifacts are listed in `artifacts/manifest.md`.

- `cargo check --lib`
- `cargo test --lib collect_quantized_selected_leaf_threshold_profile_reports_safe_skips`

Both passed.

## Next Work

Use this production fanout surface to derive a threshold from streamed/merged compact candidate scores and measure per-worker boundability in a multi-instance fixture. The actual Phase 3 prototype still needs threshold updates during scan and recall-safe early-stop accounting.
