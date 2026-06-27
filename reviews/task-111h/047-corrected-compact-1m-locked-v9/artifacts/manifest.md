# Artifact Manifest: Task 111h Packet 047

Head SHA: `452b8065cb2d1bd6f9019884a13972a2196e90be`

Task bucket: `reviews/task-111h/`

Packet path: `reviews/task-111h/047-corrected-compact-1m-locked-v9/`

Timestamp: 2026-06-21 UTC

Lane / fixture / storage format / rerank mode: local PG18 corrected compact
1M locked v9 warm-cache sweep over the staged DBPedia OpenAI3 1M fixture,
`dim=1536`, `k=10`, 100 measured queries, `rerank_width=64`, nprobe sweep
`8,16,32,64,128,200`, `coarse_rerank` storage format. Formats covered:
source f32, index f16, index RaBitQ-4 estimator clip 3, index RaBitQ-8
estimator clip 4, index RaBitQ-8 exact-dequant clip 4, index TurboQuant
default, and index TurboQuant exact-dequant.

Surface isolation: shared-table, one-index-at-a-time surface in fresh database
`task111h_corrected_1m_locked_v9`. The first cell loads the shared corpus and
queries; later cells skip already-loaded chunks, build exactly one ec_ivf index,
run recall, run warm latency, measure storage, and drop that index before the
next cell.

Corpus provenance:

- manifest:
  `data/benchmark-profile-inputs/dbpedia-openai3-1m-staged/ec_real_ann_benchmarks_anchor_manifest.json`
- corpus rows: 990000
- query rows: 10000
- suite measured 100 queries with `queries_limit=100`
- generated truth cache
  `artifacts/suite/truth-1m-k10.json` is intentionally not committed per repo
  packet rules.

## Commands

```sh
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -c 'DROP DATABASE IF EXISTS task111h_corrected_1m_locked_v9 WITH (FORCE)'
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -c 'CREATE DATABASE task111h_corrected_1m_locked_v9'
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d task111h_corrected_1m_locked_v9 -c 'CREATE EXTENSION ecaz'

target/release/ecaz bench suite audit --config reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/task111h-1m-corrected-compact-locked-v9-suite.json
target/release/ecaz bench suite run --dry-run --config reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/task111h-1m-corrected-compact-locked-v9-suite.json --database task111h_corrected_1m_locked_v9 --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/suite-dry-run-manifest.json
target/release/ecaz bench suite run --config reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/task111h-1m-corrected-compact-locked-v9-suite.json --database task111h_corrected_1m_locked_v9 --host /home/peter/.pgrx --port 28818 --continue-on-error --manifest-output reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/suite-manifest.json --results-output reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/results.jsonl --log-file reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/suite-run.log
target/release/ecaz bench suite status --manifest reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/suite-manifest.json --log-file reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/suite-status.log
target/release/ecaz bench suite report --manifest reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/suite-manifest.json --results-output reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/suite-report-results.jsonl --log-file reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/suite-report.md
```

## Artifact Inventory

| Artifact | Purpose | Key result |
| --- | --- | --- |
| `task111h-1m-corrected-compact-locked-v9-suite.json` | Checked-in `ecaz bench suite` config for the locked 1M compact matrix. | 44 configured steps. |
| `drop-db.log`, `create-db.log`, `create-extension.log` | Fresh database setup. | database recreated; extension installed. |
| `suite-audit.log` | Suite audit. | `audit passed: 44 steps`. |
| `suite-dry-run-manifest.json`, `suite-dry-run.log` | Dry-run command expansion. | all planned commands rendered. |
| `suite-manifest.json` | Executed suite manifest. | 44 selected steps succeeded. |
| `suite-status.log` | Post-run suite status. | `completed=44 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`. |
| `results.jsonl` | Structured benchmark results from the suite run. | 242 result rows. |
| `suite-report-results.jsonl` | Structured result rows emitted by `ecaz bench suite report`. | 242 result rows. |
| `suite-report.md` | Generated suite report. | 44 completed, 0 failed. |
| `summary.md` | Parsed compact result summary. | Main recall/latency/storage tables and immediate readouts. |
| `suite/*.log` | Per-step load, recall, latency, storage, and host precheck logs. | Source logs for every cited result; 44 committed log files. |

## Key Result Lines Cited

At nprobe 64:

| cell | recall@10 | latency mean ms | latency p95 ms | ec_ivf index |
| --- | ---: | ---: | ---: | ---: |
| source f32 | 0.9770 | 18.7 | 21.3 | 226.8 MiB |
| index f16 | 0.9770 | 21.4 | 26.8 | 3.2 GiB |
| index rq4 est c3 | 0.9290 | 18.6 | 22.7 | 1.0 GiB |
| index rq8 est c4 | 0.9730 | 18.1 | 20.9 | 1.8 GiB |
| index rq8 exact c4 | 0.9730 | 17.9 | 20.5 | 1.8 GiB |
| index tq default | 0.9400 | 18.1 | 20.8 | 1.0 GiB |
| index tq exact | 0.9390 | 18.0 | 21.1 | 1.0 GiB |

Threshold readout:

- Source f32, index f16, and RaBitQ-8 clip 4 first hit recall@10 >= 0.97 at
  nprobe 64.
- No measured cell hit recall@10 >= 0.99 by nprobe 200 in this 1M/w64 run.
- Index f16 matched source f32 recall but was slower and much larger.
- RaBitQ-8 exact-dequant did not improve recall over the estimator.
- RQ4 and TurboQuant did not hit recall@10 >= 0.97.

## Notes

- Load logs warn that the CLI `ec_ivf` profile does not list
  `rabitq_rerank_least_squares`, `rabitq_rerank_clip`, and/or
  `rerank_exact_dequant` as known profile options. The loader passed them
  through verbatim, the extension accepted them, and the suite completed. This
  remains CLI profile hygiene, not a run failure.
- Load logs warn that the staged corpus manifest prefix is
  `ec_real_ann_benchmarks_anchor` while the suite uses task prefix
  `task111h047_1m_shared`. The suite intentionally passed
  `--allow-manifest-mismatch`; the staged manifest above is the corpus
  provenance source.
- This packet is a locked 1M measurement packet for the reopened Task 111h
  compact decision. It does not include new code changes.
