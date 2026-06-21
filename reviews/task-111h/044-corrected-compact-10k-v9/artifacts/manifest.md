# Artifact Manifest: Task 111h Packet 044

Head SHA: `92f0d95f9e4802d8e2886ea82a2c5ceac049a5b3`

Task bucket: `reviews/task-111h/`

Packet path: `reviews/task-111h/044-corrected-compact-10k-v9/`

Timestamp: 2026-06-20 America/Los_Angeles

Lane / fixture / storage format / rerank mode: local PG18 corrected compact
v9 warm-cache sweep over `ec_real_10k`, `dim=1536`, `k=10`, 200 queries,
`rerank_width=64`, nprobe sweep `8,16,32,64,128,200`, `coarse_rerank`
storage format. Formats covered: source f32, index f16, index RaBitQ-4
estimator/exact-dequant clips 2/3/4, index RaBitQ-8 estimator/exact-dequant
clips 2/3/4, and index TurboQuant default/exact-dequant.

Surface isolation: isolated one-prefix/one-table/one-index surfaces per cell
inside fresh database `task111h_corrected_10k_v9`; this is not the shared-table
1M lane.

Corpus provenance:

- `data/staged-current/ec_real_10k_manifest.json`
- corpus rows: 10000, SHA256
  `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`
- query rows: 200, SHA256
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- generated truth cache `artifacts/suite/truth-10k-k10.json` is intentionally
  not committed per repo packet rules.

## Commands

```sh
CARGO_INCREMENTAL=0 cargo build --release --no-default-features --features pg18
CARGO_INCREMENTAL=0 cargo build --release -p ecaz-cli
cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18

/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -c 'DROP DATABASE IF EXISTS task111h_corrected_10k_v9'
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -c 'CREATE DATABASE task111h_corrected_10k_v9'
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d task111h_corrected_10k_v9 -c 'CREATE EXTENSION ecaz'

target/release/ecaz bench suite audit --config reviews/task-111h/044-corrected-compact-10k-v9/artifacts/task111h-10k-corrected-compact-v9-suite.json
target/release/ecaz bench suite run --dry-run --config reviews/task-111h/044-corrected-compact-10k-v9/artifacts/task111h-10k-corrected-compact-v9-suite.json --database task111h_corrected_10k_v9 --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111h/044-corrected-compact-10k-v9/artifacts/suite-dry-run-manifest.json
target/release/ecaz bench suite run --config reviews/task-111h/044-corrected-compact-10k-v9/artifacts/task111h-10k-corrected-compact-v9-suite.json --database task111h_corrected_10k_v9 --host /home/peter/.pgrx --port 28818 --continue-on-error --manifest-output reviews/task-111h/044-corrected-compact-10k-v9/artifacts/suite-manifest.json --results-output reviews/task-111h/044-corrected-compact-10k-v9/artifacts/results.jsonl --log-file reviews/task-111h/044-corrected-compact-10k-v9/artifacts/suite-run.log
target/release/ecaz bench suite status --manifest reviews/task-111h/044-corrected-compact-10k-v9/artifacts/suite-manifest.json
target/release/ecaz bench suite report --manifest reviews/task-111h/044-corrected-compact-10k-v9/artifacts/suite-manifest.json --results-output reviews/task-111h/044-corrected-compact-10k-v9/artifacts/results.jsonl
```

## Artifact Inventory

| Artifact | Purpose | Key result |
| --- | --- | --- |
| `task111h-10k-corrected-compact-v9-suite.json` | Checked-in `ecaz bench suite` config for the corrected 10k compact matrix. | 65 configured steps. |
| `build-extension-release.log` | Release extension build. | finished successfully in 6m27s. |
| `build-cli-release.log` | Release CLI build. | finished successfully in 9m10s; one pre-existing dead-code warning in `corpus/load.rs`. |
| `install-pgrx-pg18.log` | Install extension into PG18. | installed `ecaz` 0.1.1. |
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
| source f32 | 1.0000 | 3.97 | 4.35 | 5.10 | 164.20 |
| index f16 | 0.9990 | 2.57 | 3.33 | 36.00 | 195.00 |
| best RaBitQ-4, `rq4 est c3` | 0.9835 | 2.40 | 2.82 | 13.90 | 173.00 |
| best RaBitQ-8, `rq8 est c4` | 0.9990 | 2.14 | 2.55 | 21.30 | 180.40 |
| TurboQuant default | 0.9815 | 2.16 | 2.63 | 13.90 | 172.90 |
| TurboQuant exact-dequant | 0.9815 | 3.83 | 4.68 | 13.90 | 172.90 |

Threshold readout:

- Source f32 and index f16 first hit recall@10 >= 0.99 at nprobe 16.
- RaBitQ-4 clips 2/3/4 did not hit recall@10 >= 0.99 in this 10k/w64 slice.
- RaBitQ-8 clip 3 and clip 4 hit recall@10 >= 0.99 at nprobe 16.
- TurboQuant default and exact-dequant did not hit recall@10 >= 0.99.
- TurboQuant exact-dequant did not improve recall over default on this slice.

## Notes

- Load logs warn that the CLI `ec_ivf` profile does not yet list the new
  `rabitq_rerank_least_squares`, `rabitq_rerank_clip`, and
  `rerank_exact_dequant` reloptions as known profile options. The loader passed
  them through verbatim, the extension accepted them, and the cells completed.
  This is a CLI profile hygiene follow-up, not a run failure.
- This packet is a corrected 10k smoke/sweep only. It does not close 111h; the
  reopened task still requires corrected 50k/100k sweeps and final matched
  recall decisions before any final 1M run.
