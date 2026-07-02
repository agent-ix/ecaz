# Task 131 Review Request: Phase 3 Threshold Bound Diagnostic

Head: `d3b284516739021376a6f5e668411390f12810fd`

## Scope

This packet pivots away from the Phase 0/1 heap-side path per reviewer feedback in packet 009. It adds a scan-time diagnostic for Phase 3 boundability:

- new local SQL endpoint: `ec_spire_remote_search_coordinator_local_threshold_profile(...)`
- new scan helper: `collect_quantized_selected_leaf_threshold_profile(...)`
- reports selected/evaluated PID counts, threshold score/IP, sound upper-bound availability/missing counts, block and row selected/skipped counts, and summary scoring time
- uses RaBitQ leaf block summaries with radius weight `1.0` so counted skips require a full-radius upper bound below the coordinator-supplied threshold
- diagnostic only: it does not prune result-producing scan work and does not claim latency improvement

The endpoint is local-only for this checkpoint. If selected PIDs fan out to remote targets, it errors rather than silently pretending to profile distributed workers. The next useful slice is production/libpq fanout or a streaming threshold prototype that pushes scalar thresholds during candidate production.

## Validation

Artifacts are listed in `artifacts/manifest.md`.

- `cargo test --lib collect_quantized_selected_leaf_threshold_profile_reports_safe_skips`
- `cargo check --lib`

Both passed.

## Reviewer Notes

This is intended to answer the reviewer directive's missing Phase 0 scan-time field: whether selected lists/blocks have a sound upper bound that could be used by Phase 3. It deliberately does not extend the global-preheap matrix or the explicit-subset heap path.
