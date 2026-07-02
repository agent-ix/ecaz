# Task 131 Phase 3 Increment A A/B Pre-Registration

- Head SHA: c0202541b98c6f249b0e102b28b7f29cc4b8194d
- Task bucket: `reviews/task-131/027-phase3-increment-a-ab`
- Suite config: `artifacts/task131-phase3-increment-a-ab-suite.json`
- Dry-run manifest: `artifacts/dryrun-manifest.json`
- Status: pre-registered before running the A/B suite; dry-run expansion completed only.

## Planned Matrix

- Runner: `ecaz bench suite`
- Dry-run validation command: `target/debug/ecaz bench suite --config reviews/task-131/027-phase3-increment-a-ab/artifacts/task131-phase3-increment-a-ab-suite.json --dry-run --manifest-output reviews/task-131/027-phase3-increment-a-ab/artifacts/dryrun-manifest.json`
- Steps: local multi-instance PG18 SPIRE production-read A/B.
- Scale/index: `10k n128/b4` and `50k n128/b4`.
- Storage: `rabitq`.
- Summaries: `ec_spire.leaf_block_rows=64` in `pgoptions` and `load_session_gucs`.
- Sweep/top-k: `nprobe=96`, `k=10`.
- Query sets: full prepared query sets, 200 queries for 10k and 1000 queries for 50k.
- Variants:
  - `threshold-off`: `ec_spire.remote_search_global_pre_heap_merge=off`, `ec_spire.remote_search_initial_threshold_early_stop=off`, timeline payload disabled.
  - `threshold-on`: `ec_spire.remote_search_global_pre_heap_merge=off`, `ec_spire.remote_search_initial_threshold_early_stop=on`, timeline payload disabled.
- Fault drills: on; `skip_fault_drills` is intentionally omitted.
- Rowcap arm: skipped; `skip_bench_rowcap=true`.

## Pre-Registered Decision Rule

Promote beyond increment A only if `threshold-on` matches `threshold-off` returned ID lists for every query at both scales and beats `threshold-off` p50 and p95 latency by more than observed run-to-run/noise variance at both 10k and 50k. The evidence must also show rows or leaf blocks avoided through the production threshold profile.

If returned IDs diverge, recall drops, no rows/blocks are skipped, or latency is flat/regressed at either scale, the acceptable conclusion is to shelve this Phase 3 path with these numbers.
