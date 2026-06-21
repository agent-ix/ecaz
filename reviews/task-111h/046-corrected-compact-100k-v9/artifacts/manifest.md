# Artifact Manifest: Task 111h Packet 046

Head SHA: `a7273eca84dd6ee525a1348cac9fc440c460d809`

Task bucket: `reviews/task-111h/`

Packet path: `reviews/task-111h/046-corrected-compact-100k-v9/`

Timestamp: 2026-06-20 America/Los_Angeles

Lane / fixture / storage format / rerank mode: local PG18 corrected compact
v9 warm-cache sweep over `ec_real_100k`, `dim=1536`, `k=10`, 200 queries,
`rerank_width=64`, nprobe sweep `8,16,32,64,128,200`, `coarse_rerank`
storage format. Formats covered: source f32, index f16, index RaBitQ-4
estimator/exact-dequant clips 2/3/4, index RaBitQ-8 estimator/exact-dequant
clips 2/3/4, and index TurboQuant default/exact-dequant.

Surface isolation: isolated one-prefix/one-table/one-index surfaces per cell
inside fresh database `task111h_corrected_100k_v9`; this is not the shared-table
1M lane.

Corpus provenance:

- `data/staged-current/ec_real_100k_manifest.json`
- corpus rows: 100000, SHA256
  `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`
- query rows: 1000, suite used `queries_limit=200`, SHA256
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- generated truth cache `artifacts/suite/truth-100k-k10.json` is intentionally
  not committed per repo packet rules.

## Commands

```sh
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -c 'DROP DATABASE IF EXISTS task111h_corrected_100k_v9 WITH (FORCE)'
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -c 'CREATE DATABASE task111h_corrected_100k_v9'
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d task111h_corrected_100k_v9 -c 'CREATE EXTENSION ecaz'

target/release/ecaz bench suite audit --config reviews/task-111h/046-corrected-compact-100k-v9/artifacts/task111h-100k-corrected-compact-v9-suite.json
target/release/ecaz bench suite run --dry-run --config reviews/task-111h/046-corrected-compact-100k-v9/artifacts/task111h-100k-corrected-compact-v9-suite.json --database task111h_corrected_100k_v9 --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111h/046-corrected-compact-100k-v9/artifacts/suite-dry-run-manifest.json
target/release/ecaz bench suite run --config reviews/task-111h/046-corrected-compact-100k-v9/artifacts/task111h-100k-corrected-compact-v9-suite.json --database task111h_corrected_100k_v9 --host /home/peter/.pgrx --port 28818 --continue-on-error --manifest-output reviews/task-111h/046-corrected-compact-100k-v9/artifacts/suite-manifest.json --results-output reviews/task-111h/046-corrected-compact-100k-v9/artifacts/results.jsonl --log-file reviews/task-111h/046-corrected-compact-100k-v9/artifacts/suite-run.log
target/release/ecaz bench suite status --manifest reviews/task-111h/046-corrected-compact-100k-v9/artifacts/suite-manifest.json
target/release/ecaz bench suite report --manifest reviews/task-111h/046-corrected-compact-100k-v9/artifacts/suite-manifest.json --results-output reviews/task-111h/046-corrected-compact-100k-v9/artifacts/results.jsonl
```

## Artifact Inventory

| Artifact | Purpose | Key result |
| --- | --- | --- |
| `task111h-100k-corrected-compact-v9-suite.json` | Checked-in `ecaz bench suite` config for the corrected 100k compact matrix. | 65 configured steps. |
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

| cell | recall@10 | mean ms | p95 ms | ec_ivf index MiB | total |
| --- | ---: | ---: | ---: | ---: | ---: |
| source f32 | 0.9720 | 7.72 | 8.49 | 24.60 | 1.6 GiB |
| index f16 | 0.9710 | 8.16 | 9.68 | 330.10 | 1.9 GiB |
| best RaBitQ-4, `rq4 est c3` | 0.9360 | 6.13 | 6.89 | 110.20 | 1.7 GiB |
| best RaBitQ-8, `rq8 exact c4` | 0.9670 | 6.42 | 7.36 | 183.60 | 1.7 GiB |
| RaBitQ-8 estimator c4 | 0.9670 | 6.46 | 7.49 | 183.60 | 1.7 GiB |
| TurboQuant default | 0.9375 | 8.33 | 9.92 | 110.10 | 1.7 GiB |
| TurboQuant exact-dequant | 0.9375 | 9.06 | 10.70 | 110.10 | 1.7 GiB |

Threshold readout:

- Source f32 and index f16 first hit recall@10 >= 0.97 at nprobe 64 and
  recall@10 >= 0.99 at nprobe 128.
- RaBitQ-4 clips 2/3/4 did not hit recall@10 >= 0.97 or >= 0.99 in this
  100k/w64 slice.
- RaBitQ-8 clip 4 first hit recall@10 >= 0.97 at nprobe 128 and >= 0.99 at
  nprobe 200.
- TurboQuant default and exact-dequant did not hit recall@10 >= 0.97 or >=
  0.99.
- Exact-dequant did not improve TurboQuant recall; RaBitQ-8 clip 4
  exact-dequant improved best recall only from 0.9915 to 0.9920 while being
  slower at nprobe 200.

## Notes

- Load logs warn that the CLI `ec_ivf` profile does not yet list the new
  `rabitq_rerank_least_squares`, `rabitq_rerank_clip`, and
  `rerank_exact_dequant` reloptions as known profile options. The loader passed
  them through verbatim, the extension accepted them, and the cells completed.
  This is a CLI profile hygiene follow-up, not a run failure.
- Load logs also warn that the staged corpus manifest prefix is `ec_real_100k`
  while each suite cell uses an isolated task prefix. The suite intentionally
  passed `--allow-manifest-mismatch`; corpus/query SHA256 values above are the
  provenance source.
- This packet is a corrected 100k sweep only. It does not close 111h; the
  reopened task still requires final matched-recall decision work and any
  selected final-scale locked run.
