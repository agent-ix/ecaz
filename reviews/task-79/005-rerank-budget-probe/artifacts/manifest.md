# Artifact Manifest: Task 79 Packet 005

- head SHA: `f1babe9f745affa14953a08a69aad509db759d4b`
- task bucket: `reviews/task-79/005-rerank-budget-probe/`
- timestamp: `2026-06-01T15:18:48-07:00`
- lane: local PG18, `task79_spire_candidate_surface`
- fixture: `task79_surface_100k`, 100k corpus, 200-query benchmark slice
- storage format: RaBitQ
- rerank mode: width 50 and 100 probe
- surface isolation: shared task79 benchmark table/index, rebuilt per suite step
- suite config: `suite-rabitq-rerank-budget-probe.json`
- suite config sha256: `a27289dbc4299a1fbf0ec2caa26fcd212b4593368cea5af78fe83b0078eb65d3`

## Artifacts

- `suite-rabitq-rerank-budget-probe.json`: checked-in `ecaz bench suite` config.
- `suite-audit.log`: suite audit output.
- `suite-dry-run.log`: dry-run output.
- `suite-run-with-pg-target.log`: full suite execution log.
- `suite-status.log`: status output, `completed=6 failed=0 skipped=0`.
- `suite-report.log`: generated suite report.
- `suite-manifest.json`: structured suite manifest.
- `results.jsonl`: structured result rows parsed by the suite runner.
- `precheck-existing-task79-surface.log`: corpus/query/extension precheck.
- `rebuild-100k-rabitq-n512-f16-b0-tg256.log`: n512 RaBitQ index rebuild log.
- `pipeline-100k-rabitq-n512-f16-b0-tg256-row24k-rerank50.log` and matching `funnel-...jsonl`: row24k / rerank50 benchmark.
- `pipeline-100k-rabitq-n512-f16-b0-tg256-row24k-rerank100.log` and matching `funnel-...jsonl`: row24k / rerank100 benchmark.
- `pipeline-100k-rabitq-n512-f16-b0-tg256-row25k-rerank50.log` and matching `funnel-...jsonl`: row25k / rerank50 benchmark.
- `pipeline-100k-rabitq-n512-f16-b0-tg256-row25k-rerank100.log` and matching `funnel-...jsonl`: row25k / rerank100 benchmark.

## Key Result Lines

| step | nprobe | candidates | routes | object bytes | p50 ms | recall |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| n512 row24k rerank50 | 160 | 4,831,812 | 23,222 | 3,941,595,240 | 45.279 | 0.9755 |
| n512 row24k rerank50 | 192 | 4,831,812 | 23,222 | 3,941,595,240 | 51.349 | 0.9840 |
| n512 row24k rerank100 | 160 | 4,831,812 | 23,222 | 3,941,595,240 | 46.550 | 0.9755 |
| n512 row24k rerank100 | 192 | 4,831,812 | 23,222 | 3,941,595,240 | 51.072 | 0.9840 |
| n512 row25k rerank50 | 160 | 5,029,652 | 24,160 | 4,102,984,304 | 43.643 | 0.9755 |
| n512 row25k rerank50 | 192 | 5,029,652 | 24,160 | 4,102,984,304 | 50.188 | 0.9840 |
| n512 row25k rerank100 | 160 | 5,029,652 | 24,160 | 4,102,984,304 | 45.586 | 0.9755 |
| n512 row25k rerank100 | 192 | 5,029,652 | 24,160 | 4,102,984,304 | 50.036 | 0.9840 |
