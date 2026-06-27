# Artifact Manifest: Task 111h Packet 045

Head SHA: `a49c369f2a0332652a3d1cd778cf0ab8238bd084`

Task bucket: `reviews/task-111h/`

Packet path: `reviews/task-111h/045-corrected-compact-50k-v9/`

Timestamp: 2026-06-20 America/Los_Angeles

Lane / fixture / storage format / rerank mode: local PG18 corrected compact
v9 warm-cache sweep over `ec_real_50k`, `dim=1536`, `k=10`, 200 queries,
`rerank_width=64`, nprobe sweep `8,16,32,64,128,200`, `coarse_rerank`
storage format. Formats covered: source f32, index f16, index RaBitQ-4
estimator/exact-dequant clips 2/3/4, index RaBitQ-8 estimator/exact-dequant
clips 2/3/4, and index TurboQuant default/exact-dequant.

Surface isolation: isolated one-prefix/one-table/one-index surfaces per cell
inside fresh database `task111h_corrected_50k_v9`; this is not the shared-table
1M lane.

Corpus provenance:

- `data/staged-current/ec_real_50k_manifest.json`
- corpus rows: 50000, SHA256
  `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133`
- query rows: 1000, suite used `queries_limit=200`, SHA256
  `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`
- generated truth cache `artifacts/suite/truth-50k-k10.json` is intentionally
  not committed per repo packet rules.

## Commands

```sh
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -c 'DROP DATABASE IF EXISTS task111h_corrected_50k_v9 WITH (FORCE)'
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -c 'CREATE DATABASE task111h_corrected_50k_v9'
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d task111h_corrected_50k_v9 -c 'CREATE EXTENSION ecaz'

target/release/ecaz bench suite audit --config reviews/task-111h/045-corrected-compact-50k-v9/artifacts/task111h-50k-corrected-compact-v9-suite.json
target/release/ecaz bench suite run --dry-run --config reviews/task-111h/045-corrected-compact-50k-v9/artifacts/task111h-50k-corrected-compact-v9-suite.json --database task111h_corrected_50k_v9 --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111h/045-corrected-compact-50k-v9/artifacts/suite-dry-run-manifest.json
target/release/ecaz bench suite run --config reviews/task-111h/045-corrected-compact-50k-v9/artifacts/task111h-50k-corrected-compact-v9-suite.json --database task111h_corrected_50k_v9 --host /home/peter/.pgrx --port 28818 --continue-on-error --manifest-output reviews/task-111h/045-corrected-compact-50k-v9/artifacts/suite-manifest.json --results-output reviews/task-111h/045-corrected-compact-50k-v9/artifacts/results.jsonl --log-file reviews/task-111h/045-corrected-compact-50k-v9/artifacts/suite-run.log
target/release/ecaz bench suite status --manifest reviews/task-111h/045-corrected-compact-50k-v9/artifacts/suite-manifest.json
target/release/ecaz bench suite report --manifest reviews/task-111h/045-corrected-compact-50k-v9/artifacts/suite-manifest.json --results-output reviews/task-111h/045-corrected-compact-50k-v9/artifacts/results.jsonl
```

## Artifact Inventory

| Artifact | Purpose | Key result |
| --- | --- | --- |
| `task111h-50k-corrected-compact-v9-suite.json` | Checked-in `ecaz bench suite` config for the corrected 50k compact matrix. | 65 configured steps. |
| `drop-db.log`, `create-db.log`, `create-extension.log` | Fresh database setup. | database recreated; extension version `0.1.1`. |
| `suite-audit.log` | Suite audit. | `audit passed: 65 steps`. |
| `suite-dry-run-manifest.json`, `suite-dry-run.log` | Dry-run command expansion. | all planned commands rendered. |
| `suite-manifest.json` | Executed suite manifest. | 65 selected steps succeeded. |
| `suite-status.log` | Post-run suite status. | `completed=65 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`. |
| `results.jsonl` | Structured benchmark results. | 602 result rows. |
| `suite-report.md` | Generated suite report. | 65 completed, 0 failed. |
| `summary.md` | Parsed compact result summary. | Main recall/latency/storage tables and immediate readouts. |
| `suite/*.log` | Per-step load, recall, latency, storage, and host precheck logs. | Source logs for every cited result. |

## Key Result Lines Cited

At nprobe 64:

| cell | recall@10 | mean ms | p95 ms | ec_ivf index MiB | total MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| source f32 | 0.9820 | 5.54 | 6.10 | 13.80 | 808.70 |
| index f16 | 0.9820 | 5.57 | 6.86 | 166.70 | 961.50 |
| best RaBitQ-4, `rq4 est c3` | 0.9475 | 4.00 | 4.43 | 56.60 | 851.50 |
| best RaBitQ-8, `rq8 est c4` | 0.9770 | 4.24 | 5.08 | 93.40 | 888.30 |
| TurboQuant default | 0.9475 | 4.41 | 5.83 | 56.60 | 851.50 |
| TurboQuant exact-dequant | 0.9475 | 6.82 | 7.92 | 56.60 | 851.50 |

Threshold readout:

- Source f32 and index f16 first hit recall@10 >= 0.99 at nprobe 128.
- RaBitQ-4 clips 2/3/4 did not hit recall@10 >= 0.97 or >= 0.99 in this
  50k/w64 slice.
- RaBitQ-8 clip 4 hit recall@10 >= 0.99 at nprobe 128; clip 3 hit >= 0.97
  but not >= 0.99.
- TurboQuant default and exact-dequant did not hit recall@10 >= 0.97 or >=
  0.99.
- Exact-dequant did not improve recall for RaBitQ-8 clip 4 or TurboQuant in
  this slice.

## Notes

- Load logs warn that the CLI `ec_ivf` profile does not yet list the new
  `rabitq_rerank_least_squares`, `rabitq_rerank_clip`, and
  `rerank_exact_dequant` reloptions as known profile options. The loader passed
  them through verbatim, the extension accepted them, and the cells completed.
  This is a CLI profile hygiene follow-up, not a run failure.
- This packet is a corrected 50k sweep only. It does not close 111h; the
  reopened task still requires corrected 100k/final-scale sweeps and final
  matched-recall decisions before any final 1M run.
