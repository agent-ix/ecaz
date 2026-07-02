# Task 131 Phase 3 Increment A A/B Pre-Registration

This packet pre-registers the A/B suite before running it.

The planned run is `ecaz bench suite` over 10k and 50k `n128/b4` local multi-instance SPIRE with summaries enabled via `ec_spire.leaf_block_rows=64`. It compares the default-off initial-threshold worker early-stop gate against the same fixture with `ec_spire.remote_search_initial_threshold_early_stop=on`.

Success requires all of the following:

- `threshold-on` and `threshold-off` return identical ID lists for every query.
- Recall remains matched.
- `threshold-on` beats `threshold-off` p50 and p95 latency by more than run-to-run/noise variance at both 10k and 50k.
- Production threshold profile rows/blocks show scan work avoided.

Null/shelve criteria: any identity mismatch, recall drop, zero scan work avoided, or flat/regressed latency at either scale is sufficient to shelve this Phase 3 path with the resulting evidence.

Artifacts:

- `artifacts/task131-phase3-increment-a-ab-suite.json`
- `artifacts/manifest.md`
- `artifacts/dryrun-manifest.json`
