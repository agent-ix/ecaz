# Artifact Manifest: Task 79 Packet 008

- head SHA: `f1babe9f745affa14953a08a69aad509db759d4b`
- task bucket: `reviews/task-79/008-row-budget-geometry-refinement/`
- timestamp: `2026-06-01T15:18:48-07:00`
- lane: local PG18, `task79_spire_candidate_surface`
- fixture: `task79_surface_100k`, 100k corpus, 200-query benchmark slice
- storage format: RaBitQ
- rerank mode: width 25
- surface isolation: shared task79 benchmark table/index, rebuilt per nlists bracket
- suite config: `suite-rabitq-row-budget-geometry-refinement.json`
- suite config sha256: `1151cb1b245c822fc1da7e51c8ed7ecdd04478113030741867d9145af54ad4a4`

## Artifacts

- `suite-rabitq-row-budget-geometry-refinement.json`: checked-in `ecaz bench suite` config.
- `install-current-ecaz-pg18.log`: installed current extension into PG18.
- `pg18-restart.log`: PG18 restart log after installing the extension.
- `suite-audit.log`: suite audit output.
- `suite-dry-run.log` and `suite-dry-run-manifest.json`: dry-run outputs.
- `suite-run.log`: full suite execution log.
- `suite-status.log`: status output, `completed=9 failed=0 skipped=0`.
- `suite-report.log`: generated suite report.
- `suite-manifest.json`: structured suite manifest.
- `results.jsonl` and `report-results.jsonl`: structured result rows.
- `precheck-existing-task79-surface.log`: corpus/query/extension precheck.
- `rebuild-100k-rabitq-n384-f16-b0-tg256.log` and `rebuild-100k-rabitq-n448-f16-b0-tg256.log`: index rebuild logs.
- `pipeline-...log` and matching `funnel-...jsonl` files: row-budget geometry refinement benchmark runs.

## Key Result Lines

| nlists | row budget | nprobe | candidates | routes | object bytes | p50 ms | recall |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 384 | 24k | 192 | 4,836,456 | 18,790 | 3,944,894,520 | 53.343 | 0.9910 |
| 384 | 24k | 224 | 4,836,456 | 18,790 | 3,944,894,520 | 57.000 | 0.9955 |
| 384 | 24k | 256 | 4,836,456 | 18,790 | 3,944,894,520 | 63.022 | 0.9975 |
| 384 | 25k | 192 | 5,036,739 | 19,531 | 4,108,251,780 | 50.814 | 0.9910 |
| 384 | 25k | 224 | 5,036,739 | 19,531 | 4,108,251,780 | 57.164 | 0.9955 |
| 384 | 25k | 256 | 5,036,739 | 19,531 | 4,108,251,780 | 63.290 | 0.9975 |
| 384 | 26k | 192 | 5,236,559 | 20,282 | 4,271,232,464 | 50.802 | 0.9910 |
| 384 | 26k | 224 | 5,236,559 | 20,282 | 4,271,232,464 | 56.739 | 0.9955 |
| 384 | 26k | 256 | 5,236,559 | 20,282 | 4,271,232,464 | 62.737 | 0.9975 |
| 448 | 24k | 192 | 4,833,877 | 20,972 | 3,943,026,160 | 48.898 | 0.9870 |
| 448 | 24k | 224 | 4,833,877 | 20,972 | 3,943,026,160 | 54.662 | 0.9890 |
| 448 | 24k | 256 | 4,833,877 | 20,972 | 3,943,026,160 | 62.649 | 0.9935 |
| 448 | 25k | 192 | 5,036,640 | 21,820 | 4,108,417,200 | 48.958 | 0.9870 |
| 448 | 25k | 224 | 5,036,640 | 21,820 | 4,108,417,200 | 53.674 | 0.9890 |
| 448 | 25k | 256 | 5,036,640 | 21,820 | 4,108,417,200 | 58.515 | 0.9935 |
| 448 | 26k | 192 | 5,233,192 | 22,611 | 4,268,737,804 | 49.711 | 0.9870 |
| 448 | 26k | 224 | 5,233,192 | 22,611 | 4,268,737,804 | 57.483 | 0.9890 |
| 448 | 26k | 256 | 5,233,192 | 22,611 | 4,268,737,804 | 60.388 | 0.9935 |
