# Task 118 Final Closeout Audit Template

Use this template when the Intel 10k/50k/100k artifacts have landed in
`reviews/task-118/006-final-attribution-matrix/artifacts/`.

The closeout claim is valid only when every item below has packet-local evidence
and no item relies on terminal scrollback, uncommitted files, truth caches, raw
per-query JSONL, or AMD-only timing.

Current-head note: packet 014 corrected the HNSW frontier diagnostic semantics.
Any final `truth@10 in frontier`, `truth@100 in frontier`, and `frontier_*`
claims must come from artifacts generated after commit
`df7ff2a0324929bd385e710ed97807be971773df`. Packet 016 provides AMD-local
current-head 10k frontier and score-correlation evidence, but final closeout
should use Intel artifacts for all three required scales: 10k, 50k, and 100k.

## Artifact Presence

Required final Intel artifacts:

- `cargo-pgrx-test-pg18-score-sanity-intel.log`
- `suite-manifest-10k-intel.json`
- `results-10k-intel.jsonl`
- `suite-run-10k-intel.log`
- `suite-manifest-50k-intel.json`
- `results-50k-intel.jsonl`
- `suite-run-50k-intel.log`
- `suite-manifest-100k-intel.json`
- `results-100k-intel.jsonl`
- `suite-run-100k-intel.log`
- per-step logs cited by packet 006 `request.md` and `artifacts/manifest.md`

Presence check:

```bash
for path in \
  reviews/task-118/006-final-attribution-matrix/artifacts/cargo-pgrx-test-pg18-score-sanity-intel.log \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-10k-intel.json \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-10k-intel.log \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-50k-intel.json \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-50k-intel.log \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-100k-intel.json \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-100k-intel.log
do
  test -s "$path" && printf 'present\t%s\n' "$path" || printf 'MISSING\t%s\n' "$path"
done
```

Score-sanity runtime check:

```bash
rg -n "test result: ok|1 passed|0 failed" \
  reviews/task-118/006-final-attribution-matrix/artifacts/cargo-pgrx-test-pg18-score-sanity-intel.log
```

## Selected-Step Status

Both Intel manifests must show every selected step succeeded.

```bash
for manifest in \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-10k-intel.json \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-50k-intel.json \
  reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-100k-intel.json
do
  echo "$manifest"
  jq -r '[.steps[] | select(.selected)] | group_by(.status)[] | [.[0].status, length] | @tsv' "$manifest"
done
```

Expected shape per scale with the current suite config:

- `36` selected steps total;
- `6` load;
- `6` recall;
- `6` hnsw-frontier;
- `6` hnsw-score-correlation;
- `6` latency;
- `6` storage.

Across the 10k, 50k, and 100k Intel manifests together, this is `108` selected
steps: `18` of each selected step kind.

## Result Row Completeness

Each Intel results file must contain the required result kinds.

```bash
for results in \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl
do
  echo "$results"
  jq -r '.kind + "\t" + .metric' "$results" | sort | uniq -c
done
```

Expected minimum per scale:

- `36` `recall	recall` rows;
- `6` `hnsw-frontier	hnsw_frontier` rows;
- `6` `hnsw-score-correlation	hnsw_score_correlation` rows;
- `36` `latency	latency` rows;
- at least `6` total storage rows for `metric=="storage_field"` and
  `values.field=="total"`;
- load timing rows for all six source/compressed format lanes.

## Acceptance Criteria Audit

### 1. Candidate Containment Diagnostic

Evidence must show, for each scale, format, and build path:

- `truth@10 in frontier`;
- `truth@100 in frontier`;
- final emitted row count;
- visited count / final visited count.

Extraction:

```bash
jq -r 'select(.kind=="hnsw-frontier") |
  [.step, .values.storage_format, .values.prefix, .values.ef_search,
   .values["truth@10 in frontier"], .values["truth@100 in frontier"],
   .values["visited final"], .values.emitted] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl
```

Completion standard:

- rows exist for TurboQuant, PqFastScan, and RaBitQ;
- rows exist for source-build and compressed-build lanes;
- rows exist at 10k, 50k, and 100k;
- every row is `ef_search=200`;
- the final decision table compares recall@10 with `truth@10 in frontier`.

### 2. Rerank Boundary Counters

Evidence must show exact-reranked candidate count and dropped-before-exact count.

Extraction:

```bash
jq -r 'select(.kind=="hnsw-frontier") |
  [.step, .values.storage_format, .values.prefix, .values.ef_search,
   .values["exact rerank"], .values["quantized rerank"],
   .values["dropped before exact"], .values.emitted] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl
```

Completion standard:

- every required lane has a row;
- the packet explicitly states whether any candidate was dropped before exact
  rerank;
- the dominant-loss classification does not blame final rerank/output unless
  these counters support it.

### 3. Source-F32 Build Vs Compressed-Build A/B

Evidence must compare source-build and compressed-build lanes for the same scale
and format.

Extraction:

```bash
jq -r 'select(.kind=="recall" and .values.ef_search=="200") |
  [.step, .values.storage_format, .values.prefix,
   .values["recall@k"], .values["mean q-time"]] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl
```

Completion standard:

- every required format has both source-build and compressed-build rows;
- the final packet states whether compressed-build changes recall, containment,
  latency, or storage;
- if source-vs-compressed cannot be expressed for any lane, packet 006 must
  point to a narrow blocker instead of declaring a result.

### 4. Approx-Score Correlation Evidence

Evidence must exist for TurboQuant, PqFastScan, and RaBitQ at 10k, 50k, and
100k.

The synthetic known-order score-correlation fixture must also pass on the final
PG18 host. This guards against wrong-sign, missing-comparison, or badly
misordered diagnostic behavior before interpreting large-scale score rows.

Extraction:

```bash
jq -r 'select(.kind=="hnsw-score-correlation") |
  [.step, .values.storage_format, .values.prefix, .values.ef_search,
   .values["mean spearman"], .values["mean |rank shift|"],
   .values["max |rank shift|"], .values["mean |score delta|"],
   .values["missing cmp"]] | @tsv' \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl
```

Completion standard:

- every required format/scale/build path has a row;
- missing comparison count is interpreted;
- the final decision table distinguishes scorer-ordering loss from candidate
  containment loss.

### 5. Final Decision Packet

Packet 006 must include a final table with one row per
`format x scale x build path` at `ef_search=200`.

Generate the final table skeleton with packet 018's extractor:

```bash
jq -sr -f reviews/task-118/018-final-table-extractor/artifacts/task118-final-table.jq \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl \
  > reviews/task-118/006-final-attribution-matrix/artifacts/final-decision-table-intel.tsv
```

Extractor output check:

```bash
awk -F '\t' 'NR==1 {print "header_columns", NF; next} {rows++; if (NF != 15) bad++} END {print "data_rows", rows; print "bad_width_rows", bad + 0}' \
  reviews/task-118/006-final-attribution-matrix/artifacts/final-decision-table-intel.tsv
```

Expected output:

- `header_columns 15`
- `data_rows 18`
- `bad_width_rows 0`

Required columns:

| Scale | Format | Build path | Recall@10 | Mean q-time | Truth@10 in frontier | Truth@100 in frontier | Exact rerank | Dropped before exact | Mean Spearman | Mean rank shift | Total storage | Total storage bytes | Dominant loss stage | Next action |
| --- | --- | --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | --- | --- |

Allowed dominant-loss labels:

- `graph build quality`
- `traversal scorer quality`
- `frontier width`
- `rerank boundary`
- `visibility/output behavior`
- `benchmark harness issue`
- `no implementation follow-up justified`

The final packet should state whether RaBitQ remains worth pursuing for HNSW,
whether TurboQuant/PqFastScan need HNSW-specific follow-up, and whether a
follow-up task should focus on graph construction, traversal scoring, or wider
frontier/rerank behavior.

## Commit Checklist

Before pushing final closeout:

- `git status --short` has no staged truth caches, raw diagnostic JSONL, corpus
  TSV/TSV.GZ, scratch logs, or tunnel/SSM exhaust.
- packet 006 `request.md` cites only committed packet-local artifacts.
- packet 006 `artifacts/manifest.md` records head SHA, command, timestamp, lane,
  scale, storage format, build path, key result lines, and isolated one-index
  surface for each cited artifact set.
- all commits are pushed to
  `task-118-hnsw-quantized-recall-attribution`.
