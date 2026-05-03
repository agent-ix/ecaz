# Task 31 Suite Runner Threshold Slice

Reviewer: please review the threshold assertion slice for ADR-050.

## Scope

This adds config-driven suite thresholds:

- Optional top-level `thresholds` in suite JSON.
- Threshold fields: `name`, `step`, `metric`, `field`, `op`, and numeric `value`.
- Operators: `gt`, `gte`, `lt`, `lte`, `eq`.
- Thresholds evaluate against parsed result rows after an executed run.
- Threshold results are recorded in `suite-manifest.json`.
- A completed run fails if any configured threshold fails.
- `report` renders threshold results when present.

The Task 31 sample suite now includes candidate recall and p50 latency
threshold examples.

## Validation

Ran:

```text
cargo fmt --package ecaz-cli
cargo test -p ecaz-cli suite
cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --only-tag candidate --manifest-output review/30181-task31-suite-thresholds/artifacts/candidate_threshold_dry_run_manifest.json
cargo run -p ecaz-cli -- --log-file review/30181-task31-suite-thresholds/artifacts/threshold_report.log bench suite report --manifest review/30181-task31-suite-thresholds/artifacts/threshold_fixture_manifest.json --results-output review/30181-task31-suite-thresholds/artifacts/threshold_results.jsonl
```

`cargo test -p ecaz-cli suite` passed 12 tests. The build emitted the existing
PG18 server-header warnings from `csrc/pg18_pgstat_shim.c`; no suite-specific
warnings were introduced.

## Artifacts

- `artifacts/candidate_threshold_dry_run_manifest.json`
- `artifacts/threshold_fixture_manifest.json`
- `artifacts/threshold_report.log`
- `artifacts/threshold_results.jsonl`
- `artifacts/manifest.md`

No expensive benchmark steps were executed for this packet.

## Deferred

- Resume conflict checks against config hash and expanded command drift.
- Threshold row filters for multi-row sweeps; current behavior evaluates the
  first matching step/metric/field row.
