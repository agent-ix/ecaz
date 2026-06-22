# Task 118 Intel Closeout Runbook

This runbook is for the Intel desktop that owns final Task 118 measurement.
The AMD host can produce relative checks, but final closeout evidence should be
captured on Intel.

Current-head note: packet 014 corrects the frontier containment diagnostic so
`frontier_*` rows describe the AM's ef_search-sized candidate pool before SQL
LIMIT truncation. Packet 016 regenerated 10k frontier and score-correlation
diagnostics on the slower AMD host as a current-head preview, but final Task
118 closeout should use Intel evidence for all three required scales:
10k, 50k, and 100k. Do not treat packet 016's AMD rows as final host-class
performance evidence.

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

## Score-Sanity Runtime Test

Packet 009 added the synthetic known-order score-correlation fixture, but the
AMD-local runtime attempts remained inconclusive during compile. Packet 020
repeated that current-head AMD attempt with the same result. Run the focused
runtime test on the Intel/normal PG18 host before final closeout:

```bash
cargo pgrx test pg18 test_ech_score_correlation_synthetic_known_ordering \
  > reviews/task-118/006-final-attribution-matrix/artifacts/cargo-pgrx-test-pg18-score-sanity-intel.log 2>&1
```

Commit the log if it passes. If it fails, treat that as a Task 118 scorer
sanity blocker rather than closing the task from benchmark rows alone.

## 10k Suite

Run the full 10k suite on Intel so the final closeout has Intel recall,
latency, storage, frontier-containment, rerank-counter, and score-correlation
rows at the smallest required scale:

```bash
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database tqvector_bench \
  --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-10k-intel.log \
  bench suite run \
  --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json \
  --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts \
  --manifest-output reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-10k-intel.json \
  --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl \
  --only-tag ec_real_10k \
  --continue-on-error \
  --allow-debug-backend
```

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
  --manifest reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-10k-intel.json

cargo run -p ecaz-cli -- bench suite status \
  --manifest reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-50k-intel.json

cargo run -p ecaz-cli -- bench suite status \
  --manifest reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-100k-intel.json
```

If a results file needs to be regenerated from an existing manifest and logs:

```bash
cargo run -p ecaz-cli -- bench suite report \
  --manifest reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-10k-intel.json \
  --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl

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
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-10k-intel.json

jq -r '[.steps[] | select(.selected)] | group_by(.status)[] | [.[0].status, length] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-50k-intel.json

jq -r '[.steps[] | select(.selected)] | group_by(.status)[] | [.[0].status, length] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-100k-intel.json
```

Each scale should include source-build and compressed-build rows for all three
formats. With the current suite shape, expect per scale:

- 6 `hnsw-frontier` rows per scale, all at `ef_search=200`;
- 6 `hnsw-score-correlation` rows per scale, all at `ef_search=200`;
- 36 recall rows per scale, because recall keeps the six-value sweep across 6
  source/compressed format lanes;
- 36 latency rows per scale, because latency keeps the same six-value sweep;
- storage rows for all 6 source/compressed format lanes.

Row-kind check:

```bash
jq -r '.kind + "\t" + .metric' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl \
  | sort | uniq -c

jq -r '.kind + "\t" + .metric' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  | sort | uniq -c

jq -r '.kind + "\t" + .metric' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl \
  | sort | uniq -c
```

Extract the final Task 118 decision table skeleton:

```bash
jq -sr -f reviews/task-118/018-final-table-extractor/artifacts/task118-final-table.jq \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl \
  > reviews/task-118/006-final-attribution-matrix/artifacts/final-decision-table-intel.tsv
```

The extractor output should have 19 lines: one header plus 18 data rows
(`3 scales x 3 formats x 2 build paths`). Fill the blank
`Dominant loss stage` and `Next action` columns manually in packet 006 after
interpreting the rows.

## Commit Scope

Commit only decision-grade artifacts:

- `suite-manifest-10k-intel.json`
- `cargo-pgrx-test-pg18-score-sanity-intel.log`
- `results-10k-intel.jsonl`
- `suite-run-10k-intel.log`
- per-step 10k logs cited by the request
- `suite-manifest-50k-intel.json`
- `results-50k-intel.jsonl`
- `suite-run-50k-intel.log`
- per-step 50k logs cited by the request
- `suite-manifest-100k-intel.json`
- `results-100k-intel.jsonl`
- `suite-run-100k-intel.log`
- per-step 100k logs cited by the request
- `final-decision-table-intel.tsv`
- updated `reviews/task-118/006-final-attribution-matrix/artifacts/manifest.md`
- updated `reviews/task-118/006-final-attribution-matrix/request.md`

Do not commit truth caches, raw per-query JSONL, staged corpus TSV files, or
temporary scratch/diagnostic exhaust.

## Final Decision Packet

After the Intel rows land, update packet 006 with a final classification table:

Use `final-decision-table-intel.tsv` from the extractor as the table skeleton.
Packet 006 may convert it to Markdown, but it must preserve the generated
recall, frontier, rerank, score-correlation, and storage values.

The final Task 118 closeout should explicitly classify TurboQuant, PqFastScan,
and RaBitQ as graph-build quality, traversal scorer quality, frontier width,
rerank boundary, visibility/output behavior, benchmark harness issue, or no
follow-up justified.
