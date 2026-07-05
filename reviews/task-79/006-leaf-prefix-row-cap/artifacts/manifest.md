# Artifact Manifest: Task 79 Packet 006

- head SHA: `f1babe9f745affa14953a08a69aad509db759d4b`
- task bucket: `reviews/task-79/006-leaf-prefix-row-cap/`
- timestamp: `2026-06-01T15:18:48-07:00`
- lane: local PG18, `task79_spire_candidate_surface`
- fixture: `task79_surface_100k`, 100k corpus, 200-query benchmark slice
- storage format: RaBitQ
- rerank mode: width 25
- surface isolation: shared task79 benchmark table/index, rebuilt per suite step
- suite config: `suite-rabitq-leaf-prefix-row-cap.json`
- suite config sha256: `bffdba100a85c691fb3fe46b8eb01e54549c5e92870c7071f90ab09404fab53f`

## Artifacts

- `suite-rabitq-leaf-prefix-row-cap.json`: checked-in `ecaz bench suite` config.
- `suite-audit.log` and `suite-audit-rerun.log`: suite audit outputs.
- `suite-dry-run.log`, `suite-dry-run-rerun.log`, `suite-dry-run-manifest.json`, and `suite-dry-run-manifest-rerun.json`: dry-run outputs.
- `suite-run.log` and `suite-run-rerun.log`: full suite execution logs.
- `suite-status.log`: status output, `completed=8 failed=0 skipped=0`.
- `suite-report.log`: generated suite report.
- `suite-manifest.json`: structured suite manifest.
- `results.jsonl` and `report-results.jsonl`: structured result rows.
- `install-current-ecaz-pg18.log` and `pg18-restart.log`: installed/restarted PG18 validation environment.
- `precheck-existing-task79-surface.log`, `precheck-load-new-guc.log`, `precheck-set-new-guc.log`, `precheck-show-existing-guc.log`: precheck and GUC validation logs.
- `rebuild-100k-rabitq-n512-f16-b0-tg256.log` and `rebuild-100k-rabitq-n256-f16-b0-tg256.log`: index rebuild logs.
- `pipeline-...log` and matching `funnel-...jsonl` files: fixed leaf-prefix row-cap benchmark runs.

## Key Result Lines

| step | nprobe | candidates | routes | object bytes | p50 ms | recall |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| n512 row26k leaf160 | 192 | 3,311,427 | 25,116 | 4,267,567,376 | 43.185 | 0.6730 |
| n512 row26k leaf160 | 256 | 3,311,427 | 25,116 | 4,267,567,376 | 50.545 | 0.6785 |
| n512 row26k leaf192 | 192 | 3,719,442 | 25,116 | 4,267,567,376 | 45.468 | 0.7315 |
| n512 row26k leaf192 | 256 | 3,719,442 | 25,116 | 4,267,567,376 | 53.225 | 0.7385 |
| n512 row30k leaf160 | 192 | 3,813,970 | 28,958 | 4,919,204,968 | 42.982 | 0.6730 |
| n512 row30k leaf160 | 256 | 3,813,970 | 28,958 | 4,919,204,968 | 50.081 | 0.6785 |
| n256 row26k leaf320 | 128 | 3,537,649 | 13,270 | 4,283,592,872 | 39.951 | 0.7570 |
| n256 row26k leaf320 | 192 | 3,537,649 | 13,270 | 4,283,592,872 | 53.726 | 0.7610 |
| n256 row26k leaf320 | 256 | 3,537,649 | 13,270 | 4,283,592,872 | 65.610 | 0.7620 |
| n256 row26k leaf384 | 128 | 3,981,136 | 13,270 | 4,283,592,872 | 40.651 | 0.8115 |
| n256 row26k leaf384 | 192 | 3,981,136 | 13,270 | 4,283,592,872 | 55.313 | 0.8165 |
| n256 row26k leaf384 | 256 | 3,981,136 | 13,270 | 4,283,592,872 | 69.934 | 0.8175 |
