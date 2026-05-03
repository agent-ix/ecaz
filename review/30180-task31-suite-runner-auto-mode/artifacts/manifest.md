# Artifact Manifest

## Packet

- head SHA: `707ab566`
- packet/topic: `review/30180-task31-suite-runner-auto-mode`
- lane: `ecaz bench suite auto-mode smoke`
- fixture: `crates/ecaz-cli/suites/task31-m5-ivf-100k.json`
- storage format: `pq_fastscan` in expanded Task 31 IVF load step
- rerank mode: `heap_f32` in expanded Task 31 IVF load step
- isolation/shared-table surface: shared-table corpus surface; no benchmark steps executed in this packet
- timestamp: 2026-05-03

## Artifacts

### `only_tag_manifest.json`

- command: `cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --only-tag recall --manifest-output review/30180-task31-suite-runner-auto-mode/artifacts/only_tag_manifest.json`
- key line: selected 3 recall-tagged dry-run steps and skipped 5 other steps.

### `empty_results.jsonl`

- command: `cargo run -p ecaz-cli -- bench suite report --manifest review/30180-task31-suite-runner-auto-mode/artifacts/only_tag_manifest.json --results-output review/30180-task31-suite-runner-auto-mode/artifacts/empty_results.jsonl`
- key result: empty file, expected because dry-run steps are not parsed as completed results.

### `result_fixture_manifest.json`

- command: hand-authored fixture manifest for parser validation.
- key content: 4 succeeded steps pointing at existing Task 31 load, recall, latency, and storage logs.

### `result_report.log`

- command: `cargo run -p ecaz-cli -- --log-file review/30180-task31-suite-runner-auto-mode/artifacts/result_report.log bench suite report --manifest review/30180-task31-suite-runner-auto-mode/artifacts/result_fixture_manifest.json --results-output review/30180-task31-suite-runner-auto-mode/artifacts/results.jsonl`
- key line: `steps: completed 4, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0`
- key result: report includes parsed load timings, recall row, latency row, storage field rows, and storage index rows.

### `results.jsonl`

- command: produced by the report command above.
- key result: 18 normalized result rows.
