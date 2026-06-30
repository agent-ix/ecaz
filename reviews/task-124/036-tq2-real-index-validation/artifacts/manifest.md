# Task 124 Packet 036 Artifact Manifest

- head SHA: `62117e1ae876ed58aec4544d968d47d0696266f5`
- task bucket: `reviews/task-124/036-tq2-real-index-validation/`
- lane: local arm64 PG18, `tqvector_bench`, host `/Users/peter/.pgrx`, port `28818`
- fixture: staged `ec_real_10k`, `ec_real_50k`, `ec_real_100k`
- storage/index: `ec_ivf`, `storage_format=coarse_rerank`,
  `coarse_format=rabitq`, `coarse_bits=1`, `rerank_format=turboquant2`,
  `rerank_width=100`, `stage2_final_rerank_width=15`
- runner: `ecaz bench suite`

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `task124-tq2-post-simd-suite.json` | copied from packet 008 TQ2 suite config | 12-step 10k/50k/100k TQ2 matrix |
| `suite-audit.log` | `./target/release/ecaz bench suite audit --config reviews/task-124/036-tq2-real-index-validation/artifacts/task124-tq2-post-simd-suite.json --log-file reviews/task-124/036-tq2-real-index-validation/artifacts/suite-audit.log` | audit passed: 12 steps |
| `suite-run.log` | `./target/release/ecaz bench suite run --config reviews/task-124/036-tq2-real-index-validation/artifacts/task124-tq2-post-simd-suite.json --artifact-dir reviews/task-124/036-tq2-real-index-validation/artifacts/tq2-post-simd-suite --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/036-tq2-real-index-validation/artifacts/suite-run.log` | 12 succeeded / 0 failed |
| `suite-status.log` | `./target/release/ecaz bench suite status --manifest reviews/task-124/036-tq2-real-index-validation/artifacts/tq2-post-simd-suite/suite-manifest.json --log-file reviews/task-124/036-tq2-real-index-validation/artifacts/suite-status.log` | completed=12 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0 |
| `suite-report.log` | `./target/release/ecaz bench suite report --manifest reviews/task-124/036-tq2-real-index-validation/artifacts/tq2-post-simd-suite/suite-manifest.json --log-file reviews/task-124/036-tq2-real-index-validation/artifacts/suite-report.log` | report generated from `results.jsonl` |
| `tq2-post-simd-suite/results.jsonl` | suite structured output | TQ2 recall unchanged from packet 008; real latency now includes `quant=turboquant_qjl` rows |
| `tq2-post-simd-suite/recall-*-tq2-g100-w100-final15.log` | suite recall steps | 10k 0.9770, 50k 0.8050, 100k 0.7490/0.7550 recall@10 |
| `tq2-post-simd-suite/latency-*-tq2-g100-w100-final15.log` | suite latency steps with `--task87-candidate-batch-counters` | TQ2 rows: 9,600 SIMD candidates plus 400 scalar-tail candidates per 100-query run |
| `tq2-post-simd-suite/storage-*-tq2-g100-w100-final15.log` | suite storage steps | storage recorded for provenance, not used as a closeout rationale |

Truth cache files emitted by the suite are regenerable and are intentionally not
part of the committed evidence set.
