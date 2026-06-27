# Task 122 Packet 006 Artifact Manifest

- head SHA: `6ff749156bdfcfd13fa63ea359e2fdb459442b00`
- task bucket: `reviews/task-122/006-spire-recall-width-sweep`
- timestamp: `2026-06-27T14:20:39Z`
- runner: `ecaz bench suite`
- backend: PG18, `/Users/peter/.pgrx`, port `28818`, database `tqvector_bench`
- build profile: `release`
- fixture: `data/staged-current/ec_real_{10k,50k,100k}_corpus.tsv`
- profile: `ec_spire`
- storage formats: `turboquant` and `rabitq`
- bits: `4`
- query count: `100`
- k: `10`
- sweep axes: scale `10k/50k/100k`, format `turboquant/rabitq`, rerank width `25/50/100/200`, nprobe `24/48/96/192`
- table isolation: isolated one-prefix-per-scale-per-format tables, with shared recall steps over the same loaded prefix for each width
- runner status: `audit passed: 30 steps`; final `suite-manifest.json` records `30` succeeded steps

## Commands

Audit:

```sh
/Users/peter/.cargo/bin/ecaz bench suite audit \
  --config reviews/task-122/006-spire-recall-width-sweep/artifacts/task122-spire-recall-width-sweep.json \
  --log-file reviews/task-122/006-spire-recall-width-sweep/artifacts/suite-audit.log
```

Dry run:

```sh
/Users/peter/.cargo/bin/ecaz bench suite run \
  --config reviews/task-122/006-spire-recall-width-sweep/artifacts/task122-spire-recall-width-sweep.json \
  --dry-run \
  --log-file reviews/task-122/006-spire-recall-width-sweep/artifacts/suite-dry-run.log
```

Backend/GUC check:

```sh
/Users/peter/.cargo/bin/ecaz dev sql \
  --pg 18 \
  --db tqvector_bench \
  --socket-dir /Users/peter/.pgrx \
  --raw \
  --sql "SELECT ecaz_build_profile(); SELECT current_setting('ec_spire.pre_materialization_prune') AS pre_materialization_prune;" \
  --log-output reviews/task-122/006-spire-recall-width-sweep/artifacts/guc-check.log
```

Run:

```sh
/Users/peter/.cargo/bin/ecaz bench suite run \
  --config reviews/task-122/006-spire-recall-width-sweep/artifacts/task122-spire-recall-width-sweep.json \
  --host /Users/peter/.pgrx \
  --port 28818 \
  --log-file reviews/task-122/006-spire-recall-width-sweep/artifacts/suite-run.log
```

## Artifacts

- `task122-spire-recall-width-sweep.json`: checked-in suite config.
- `suite-audit.log`: suite audit output.
- `suite-dry-run.log`: dry-run command trace.
- `guc-check.log`: release backend and prune-GUC confirmation.
- `suite-run.log`: full suite command trace.
- `suite/suite-manifest.json`: structured suite manifest and step statuses.
- `suite/results.jsonl`: structured load and recall results.
- `suite/load-*.log`: six load logs for fresh TQ/RaBitQ 10k/50k/100k prefixes.
- `suite/recall-*.log`: twenty-four recall logs for the width and format matrix.

Generated truth caches under `suite/truth-*-k10.json` were intentionally not
committed; they are regenerable corpus-derived data and are covered by the
review-packet ban on truth/cache artifacts.

## Key Results

At 10k, both formats saturated for every tested width and nprobe:

```text
turboquant/rabitq, width 25/50/100/200, nprobe 24/48/96/192: recall@10 1.0000, ndcg@10 1.0000
```

At 50k, width did not change recall or NDCG for either format:

```text
nprobe 24:  recall@10 0.9450, ndcg@10 0.9969
nprobe 48:  recall@10 0.9760, ndcg@10 0.9993
nprobe 96:  recall@10 0.9940, ndcg@10 0.9999
nprobe 192: recall@10 1.0000, ndcg@10 1.0000
```

At 100k, width again did not change recall or NDCG for either format:

```text
nprobe 24:  recall@10 0.8940, ndcg@10 0.9893
nprobe 48:  recall@10 0.9430, ndcg@10 0.9948
nprobe 96:  recall@10 0.9860, ndcg@10 0.9981
nprobe 192: recall@10 0.9980, ndcg@10 0.9997
```

The release run therefore shows this SPIRE lane is nprobe-limited, not
rerank-width-limited, across candidate budgets `25` through `200`. TQ and
RaBitQ are recall-equivalent in this matrix.
