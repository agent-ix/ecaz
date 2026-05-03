# Artifact Manifest

## Packet

- head SHA: `c70fc0b9`
- packet/topic: `review/30179-task31-suite-runner-execution`
- lane: `ecaz bench suite execution smoke`
- fixture: `crates/ecaz-cli/suites/task31-m5-ivf-100k.json`
- storage format: `pq_fastscan` in expanded Task 31 IVF load step
- rerank mode: `heap_f32` in expanded Task 31 IVF load step
- isolation/shared-table surface: shared-table corpus surface; dry-run only
- timestamp: 2026-05-03

## Artifacts

### `audit.log`

- command: `cargo run -p ecaz-cli -- --log-file review/30179-task31-suite-runner-execution/artifacts/audit.log bench suite audit --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json`
- key line: `[suite:task31-m5-ivf-100k] audit passed: 8 steps`

### `dry_run.log`

- command: `cargo run -p ecaz-cli -- --log-file review/30179-task31-suite-runner-execution/artifacts/dry_run.log --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --manifest-output review/30179-task31-suite-runner-execution/artifacts/suite-manifest.json`
- key line: `[suite:task31-m5-ivf-100k] wrote review/30179-task31-suite-runner-execution/artifacts/suite-manifest.json`
- key line: expanded all 8 suite steps

### `status.log`

- command: `cargo run -p ecaz-cli -- --log-file review/30179-task31-suite-runner-execution/artifacts/status.log bench suite status --manifest review/30179-task31-suite-runner-execution/artifacts/suite-manifest.json`
- key line: `[suite:task31-m5-ivf-100k] completed=0 failed=0 skipped=0 dry_run=8 missing_artifacts=0 stale=0`

### `report.log`

- command: `cargo run -p ecaz-cli -- --log-file review/30179-task31-suite-runner-execution/artifacts/report.log bench suite report --manifest review/30179-task31-suite-runner-execution/artifacts/suite-manifest.json`
- key line: `# Suite Report: task31-m5-ivf-100k`
- key line: `steps: completed 0, failed 0, skipped 0, dry-run 8, missing artifacts 0, stale 0`

### `suite-manifest.json`

- command: produced by the dry-run command above.
- key content: config SHA256, redacted connection metadata, 8 selected dry-run steps, expanded child commands, and expected artifact paths.

### `legacy-suite-manifest.json`

- command: `cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 bench suite --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --only storage-real100k-n128 --manifest-output review/30179-task31-suite-runner-execution/artifacts/legacy-suite-manifest.json`
- key content: legacy alias selected only `storage-real100k-n128`; other steps are skipped.
