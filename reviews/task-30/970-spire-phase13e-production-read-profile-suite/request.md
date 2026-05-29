# Review Request: Production Read Profile Suite Capture

## Summary

This checkpoint adds the missing benchmark evidence surface for Phase 13e. `ecaz bench spire-pipeline` can now call `ec_spire_remote_search_production_read_profile(...)` for each sampled query and render a `Production read profile` table with the production read-path counters needed for AWS profiling.

`ecaz bench suite` now has a first-class `spire-pipeline` step, so local and AWS read-profile rows can be driven through the canonical suite runner instead of ad hoc shell glue. The packet includes a dry-run suite config proving the new step expands to `bench spire-pipeline` with `--include-production-read-profile`.

## Scope

- Adds `--include-production-read-profile` to `ecaz bench spire-pipeline`.
- Aggregates status, result source, selected/remote PID counts, dispatch/socket counts, connect p50/p95, candidate p50/p95, heap p50/p95, merge p50/p95, total p50/p95, candidate/heap query counts, payload bytes, timeout/cancel counts, degraded skip count, and returned candidate count.
- Adds `kind: spire-pipeline` to `ecaz bench suite` configs.
- Adds suite expansion, validation, expected artifact handling, and result-row parsing for spire-pipeline output.
- Marks the Phase 13e production read-profile capture checklist item complete.

## Validation

Artifacts are under `artifacts/` and summarized in `artifacts/manifest.md`.

- `cargo test -p ecaz-cli spire_pipeline`: pass, 18 focused tests.
- `cargo check -p ecaz-cli`: pass, with existing dead-code warning.
- `cargo fmt --all --check`: pass, with existing stable-rustfmt warnings.
- `git diff --check`: pass.
- `ecaz bench suite run --dry-run` through `cargo run -p ecaz-cli`: pass; manifest includes `kind: spire-pipeline` and `--include-production-read-profile`.

## Remaining Gaps

This implements the evidence capture path. It does not itself run AWS correctness/performance matrices, prove slow/fast remote overlap in the Phase 13e packet series, or justify connection pooling. Those remain open until the corrected placement/query flow is measured at AWS scale.
