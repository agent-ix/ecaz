# Artifact Manifest

## Packet

- head SHA: `76863071`
- packet/topic: `review/30182-task31-suite-filtered-thresholds-resume`
- lane: `ecaz bench suite filtered-threshold/resume smoke`
- fixture: `crates/ecaz-cli/suites/task31-m5-ivf-100k.json`
- storage format: `pq_fastscan` in expanded Task 31 IVF load step
- rerank mode: `heap_f32` in expanded Task 31 IVF load step
- isolation/shared-table surface: shared-table corpus surface; dry-run only
- timestamp: 2026-05-03

## Artifacts

### `candidate_filter_dry_run_manifest.json`

- command: `cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --only-tag candidate --manifest-output review/30182-task31-suite-filtered-thresholds-resume/artifacts/candidate_filter_dry_run_manifest.json`
- key result: selected candidate-tagged recall, latency, and explain steps after adding filtered suite thresholds.
