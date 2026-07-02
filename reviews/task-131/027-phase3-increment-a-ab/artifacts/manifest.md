# Task 131 Phase 3 Increment A A/B Result

- Pre-registration head SHA: c0202541b98c6f249b0e102b28b7f29cc4b8194d
- Result head SHA: b277cd9f6b90c6446b01b8e303d6948d0e28a451
- Task bucket: `reviews/task-131/027-phase3-increment-a-ab`
- Suite config: `artifacts/task131-phase3-increment-a-ab-suite.json`
- Dry-run manifest: `artifacts/dryrun-manifest.json`
- Result summary: `artifacts/ab-result-summary.md`
- Status: completed A/B suite; result is shelve/reject for this initial-threshold early-stop path.

## Matrix

- Runner: `ecaz bench suite`
- Dry-run validation command: `target/debug/ecaz bench suite --config reviews/task-131/027-phase3-increment-a-ab/artifacts/task131-phase3-increment-a-ab-suite.json --dry-run --manifest-output reviews/task-131/027-phase3-increment-a-ab/artifacts/dryrun-manifest.json`
- Run command: `target/debug/ecaz bench suite run --config reviews/task-131/027-phase3-increment-a-ab/artifacts/task131-phase3-increment-a-ab-suite.json`
- Run completed: 2026-07-02T07:55:00-07:00
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

## Artifacts

All cited benchmark artifacts are packet-local. Generated distributed-correctness
TSV files are intentionally not committed.

- `artifacts/ab-result-summary.md`
- `artifacts/suite-manifest.json`
- `artifacts/10k-n128-b4/bench-suite/local-real-production-read-suite.json`
- `artifacts/10k-n128-b4/bench-suite/suite-manifest.json`
- `artifacts/10k-n128-b4/bench-suite/suite-run.log`
- `artifacts/10k-n128-b4/bench-suite/results.jsonl`
- `artifacts/10k-n128-b4/bench-suite/storage.log`
- `artifacts/10k-n128-b4/bench-suite/production-read-k10-threshold-off-default.log`
- `artifacts/10k-n128-b4/bench-suite/production-read-k10-threshold-off-default-identity.jsonl`
- `artifacts/10k-n128-b4/bench-suite/production-read-k10-threshold-on-default.log`
- `artifacts/10k-n128-b4/bench-suite/production-read-k10-threshold-on-default-identity.jsonl`
- `artifacts/50k-n128-b4/bench-suite/local-real-production-read-suite.json`
- `artifacts/50k-n128-b4/bench-suite/suite-manifest.json`
- `artifacts/50k-n128-b4/bench-suite/suite-run.log`
- `artifacts/50k-n128-b4/bench-suite/results.jsonl`
- `artifacts/50k-n128-b4/bench-suite/storage.log`
- `artifacts/50k-n128-b4/bench-suite/production-read-k10-threshold-off-default.log`
- `artifacts/50k-n128-b4/bench-suite/production-read-k10-threshold-off-default-identity.jsonl`
- `artifacts/50k-n128-b4/bench-suite/production-read-k10-threshold-on-default.log`
- `artifacts/50k-n128-b4/bench-suite/production-read-k10-threshold-on-default-identity.jsonl`

## Key Results

- Identity: `cmp -s` passed for off/on identity JSONL at both 10k and 50k.
- 10k recall/latency: off `recall@k=0.9985`, `latency_p50=609.243 ms`, `latency_p95=686.941 ms`; on `recall@k=0.9985`, `latency_p50=613.294 ms`, `latency_p95=728.343 ms`. Recall is matched under current duplicate-tolerant metrics; returned top-10 IDs are not guaranteed distinct.
- 50k recall/latency: off `recall@k=1.0000`, `latency_p50=2645.864 ms`, `latency_p95=3287.777 ms`; on `recall@k=1.0000`, `latency_p50=2659.226 ms`, `latency_p95=3191.039 ms`. Recall is matched under current duplicate-tolerant metrics; returned top-10 IDs are not guaranteed distinct.
- Production profile totals: 10k off `total_p50=570.000 ms`, `total_p95=655.000 ms`; 10k on `total_p50=576.000 ms`, `total_p95=653.000 ms`; 50k off `total_p50=2605.000 ms`, `total_p95=3090.000 ms`; 50k on `total_p50=2620.000 ms`, `total_p95=3214.000 ms`.
- Actual scan avoidance: every scan profile row at both scales and both variants reports `leaf_block_skipped_sum=0`.
- Diagnostic threshold profile: potential skipped rows/blocks are nonzero but identical on/off, so they are not production scan avoidance.
- Duplicate-ID defect filed as `plan/tasks/132-spire-distributed-result-deduplication.md`: 10k threshold-off has 183/200 duplicate-containing top-10 results; 50k threshold-off has 1000/1000.
