# Task 118 Intel Closeout Runbook

This runbook is for the Intel desktop that owns final Task 118 measurement.
The AMD host can produce relative checks, but final closeout evidence should be
captured on Intel.

Current-head note: packet 014 corrects the frontier containment diagnostic so
`frontier_*` rows describe the AM's ef_search-sized candidate pool before SQL
LIMIT truncation. Before closing Task 118, use the packet 013 supplement to
regenerate 10k frontier diagnostics on the current branch head in addition to
the 50k/100k Intel runs below.

## Branch And Build

Use branch:

```bash
git checkout task-118-hnsw-quantized-recall-attribution
git pull --ff-only
```

Install the extension with the pg_test diagnostics enabled, because the suite's
`hnsw-frontier` and `hnsw-score-correlation` steps call `tests.*` diagnostic
functions:

```bash
cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features
```

The benchmark database used by prior Task 118 packets is `tqvector_bench` on
PG18 port `28818` with socket directory `/home/peter/.pgrx`.

## 50k Suite

```bash
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database tqvector_bench \
  --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-50k-intel.log \
  bench suite run \
  --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json \
  --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts \
  --manifest-output reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-50k-intel.json \
  --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  --only-tag ec_real_50k \
  --continue-on-error \
  --allow-debug-backend
```

## 100k Suite

```bash
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database tqvector_bench \
  --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-100k-intel.log \
  bench suite run \
  --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json \
  --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts \
  --manifest-output reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-100k-intel.json \
  --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl \
  --only-tag ec_real_100k \
  --continue-on-error \
  --allow-debug-backend
```

## Status And Re-Extraction

Check selected-step completion for each manifest:

```bash
cargo run -p ecaz-cli -- bench suite status \
  --manifest reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-50k-intel.json

cargo run -p ecaz-cli -- bench suite status \
  --manifest reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-100k-intel.json
```

If a results file needs to be regenerated from an existing manifest and logs:

```bash
cargo run -p ecaz-cli -- bench suite report \
  --manifest reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-50k-intel.json \
  --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl

cargo run -p ecaz-cli -- bench suite report \
  --manifest reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-100k-intel.json \
  --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl
```

## Required Post-Run Checks

All selected steps should succeed:

```bash
jq -r '[.steps[] | select(.selected)] | group_by(.status)[] | [.[0].status, length] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-50k-intel.json

jq -r '[.steps[] | select(.selected)] | group_by(.status)[] | [.[0].status, length] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-100k-intel.json
```

Each scale should include source-build and compressed-build rows for all three
formats. With the current suite shape, expect at least:

- 6 `hnsw-frontier` rows per scale, all at `ef_search=200`;
- 6 `hnsw-score-correlation` rows per scale, all at `ef_search=200`;
- 36 recall rows per scale, because recall keeps the six-value sweep across 6
  source/compressed format lanes;
- 36 latency rows per scale, because latency keeps the same six-value sweep;
- storage rows for all 6 source/compressed format lanes.

Row-kind check:

```bash
jq -r '.kind + "\t" + .metric' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  | sort | uniq -c

jq -r '.kind + "\t" + .metric' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl \
  | sort | uniq -c
```

Extract the final Task 118 decision rows:

```bash
jq -r 'select(.kind=="recall" and .values.ef_search=="200") |
  [.step, .values.storage_format, .values["recall@k"], .values["mean q-time"]] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl

jq -r 'select(.kind=="hnsw-frontier") |
  [.step, .values.storage_format, .values.ef_search, .values["truth@10 in frontier"],
   .values["truth@100 in frontier"], .values["visited final"], .values.emitted,
   .values["exact rerank"], .values["dropped before exact"]] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl

jq -r 'select(.kind=="hnsw-score-correlation") |
  [.step, .values.storage_format, .values.ef_search, .values["mean spearman"],
   .values["mean |rank shift|"], .values["max |rank shift|"], .values["missing cmp"]] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl

jq -r 'select(.kind=="storage" and .metric=="storage_field" and .values.field=="total") |
  [.step, .values.storage_format, .values.value, .values.value_bytes] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl
```

## Commit Scope

Commit only decision-grade artifacts:

- `suite-manifest-50k-intel.json`
- `results-50k-intel.jsonl`
- `suite-run-50k-intel.log`
- per-step 50k logs cited by the request
- `suite-manifest-100k-intel.json`
- `results-100k-intel.jsonl`
- `suite-run-100k-intel.log`
- per-step 100k logs cited by the request
- updated `reviews/task-118/006-final-attribution-matrix/artifacts/manifest.md`
- updated `reviews/task-118/006-final-attribution-matrix/request.md`

Do not commit truth caches, raw per-query JSONL, staged corpus TSV files, or
temporary scratch/diagnostic exhaust.

## Final Decision Packet

After the Intel rows land, update packet 006 with a final classification table:

| Format | Scale | Build path | Recall@10 | Truth@10 in frontier | Exact rerank | Dropped before exact | Mean Spearman | Storage total | Dominant loss stage | Next action |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- | --- |

The final Task 118 closeout should explicitly classify TurboQuant, PqFastScan,
and RaBitQ as graph-build quality, traversal scorer quality, frontier width,
rerank boundary, visibility/output behavior, benchmark harness issue, or no
follow-up justified.
