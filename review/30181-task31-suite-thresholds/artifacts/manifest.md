# Artifact Manifest

## Packet

- head SHA: `bb47dade`
- packet/topic: `review/30181-task31-suite-thresholds`
- lane: `ecaz bench suite threshold smoke`
- fixture: `crates/ecaz-cli/suites/task31-m5-ivf-100k.json`
- storage format: `pq_fastscan` in expanded Task 31 IVF load step
- rerank mode: `heap_f32` in expanded Task 31 IVF load step
- isolation/shared-table surface: shared-table corpus surface; dry-run/report smoke only
- timestamp: 2026-05-03

## Artifacts

### `candidate_threshold_dry_run_manifest.json`

- command: `cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --only-tag candidate --manifest-output review/30181-task31-suite-thresholds/artifacts/candidate_threshold_dry_run_manifest.json`
- key result: selected candidate-tagged recall, latency, and explain steps.

### `threshold_fixture_manifest.json`

- command: hand-authored fixture manifest for threshold report validation.
- key content: one succeeded recall step and one passing threshold result.

### `threshold_report.log`

- command: `cargo run -p ecaz-cli -- --log-file review/30181-task31-suite-thresholds/artifacts/threshold_report.log bench suite report --manifest review/30181-task31-suite-thresholds/artifacts/threshold_fixture_manifest.json --results-output review/30181-task31-suite-thresholds/artifacts/threshold_results.jsonl`
- key line: `recall-fixture-floor | pass | 0.998 | Gte 0.995`

### `threshold_results.jsonl`

- command: produced by the threshold report command.
- key result: normalized recall row extracted from the fixture artifact.
