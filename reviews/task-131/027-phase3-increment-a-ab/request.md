# Task 131 Phase 3 Increment A A/B Result

This packet records the pre-registered A/B suite and completed local
multi-instance result for Task 131 Phase 3 increment A.

The run used `ecaz bench suite` over 10k and 50k `n128/b4` local multi-instance
SPIRE with summaries enabled via `ec_spire.leaf_block_rows=64`. It compared the
default-off initial-threshold worker early-stop gate against the same fixture
with `ec_spire.remote_search_initial_threshold_early_stop=on`.

## Pre-Registered Success Criteria

- `threshold-on` and `threshold-off` return identical ID lists for every query.
- Recall remains matched.
- `threshold-on` beats `threshold-off` p50 and p95 latency by more than run-to-run/noise variance at both 10k and 50k.
- Production threshold profile rows/blocks show scan work avoided.

Null/shelve criteria: any identity mismatch, recall drop, zero scan work avoided, or flat/regressed latency at either scale is sufficient to shelve this Phase 3 path with the resulting evidence.

## Result

Result: shelve/reject this initial-threshold early-stop path.

The correctness criteria passed: returned ID files are byte-identical for
`threshold-off` vs `threshold-on` at both scales, and recall is matched.

The performance/work-avoidance criteria failed:

- 10k: recall matched at `0.9985`, but latency did not improve
  (`609.243/686.941 ms` p50/p95 off vs `613.294/728.343 ms` on).
- 50k: recall matched at `1.0000`, but p50 did not improve
  (`2645.864/3287.777 ms` p50/p95 off vs `2659.226/3191.039 ms` on);
  profile total latency also regressed (`2605/3090 ms` off vs
  `2620/3214 ms` on).
- Actual scan profile rows show zero leaf blocks skipped at both scales and
  both variants (`leaf_block_skipped_sum=0` on every remote node). The
  threshold-profile diagnostic rows are nonzero but identical on/off, so they
  are not production scan work avoided.

## Artifacts

- `artifacts/task131-phase3-increment-a-ab-suite.json`
- `artifacts/manifest.md`
- `artifacts/dryrun-manifest.json`
- `artifacts/ab-result-summary.md`
- `artifacts/10k-n128-b4/bench-suite/results.jsonl`
- `artifacts/10k-n128-b4/bench-suite/production-read-k10-threshold-{off,on}-default.log`
- `artifacts/10k-n128-b4/bench-suite/production-read-k10-threshold-{off,on}-default-identity.jsonl`
- `artifacts/50k-n128-b4/bench-suite/results.jsonl`
- `artifacts/50k-n128-b4/bench-suite/production-read-k10-threshold-{off,on}-default.log`
- `artifacts/50k-n128-b4/bench-suite/production-read-k10-threshold-{off,on}-default-identity.jsonl`
