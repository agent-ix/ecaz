# Task 118 Current-Head Diagnostic Regeneration Supplement

Packet 014 corrects the HNSW frontier containment diagnostic semantics:
`frontier_*` rows and truth-containment counters now use the AM's
ef_search-sized candidate pool before SQL LIMIT truncation.

Because packet 006's existing 10k frontier logs were generated before that
change, they are not final Task 118 containment evidence. The final packet must
regenerate 10k frontier diagnostics on the current branch head before making
any closeout claim that compares recall with `truth@10 in frontier`.

## Required Current-Head Rule

Before final closeout:

- install the current branch head with `pg_test` diagnostics enabled;
- regenerate the 10k `hnsw-frontier` rows for all three formats and both
  source-build / compressed-build paths;
- run the 50k and 100k Intel suites from the same current branch head;
- cite only regenerated/current-head frontier rows in the final decision table.

The old 10k recall, latency, storage, and score-correlation rows can remain
useful context when their code paths are unchanged, but the final packet must
not cite pre-`6ff2d1d3d8aa04edced517497d940c65ea3d6bca` 10k frontier rows as
proof of candidate containment. Prefer citing artifacts generated at or after
`df7ff2a0324929bd385e710ed97807be971773df`.

## 10k Frontier Regeneration Command

Run this on the Intel benchmark desktop after `git pull --ff-only` and after
reinstalling the extension with `--features 'pg18 pg_test'`:

```bash
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database tqvector_bench \
  --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-10k-frontier-current-head.log \
  bench suite run \
  --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json \
  --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts \
  --manifest-output reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-10k-frontier-current-head.json \
  --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-frontier-current-head.jsonl \
  --only frontier-10k-hnsw-turboquant \
  --only frontier-10k-hnsw-pq-fastscan \
  --only frontier-10k-hnsw-rabitq \
  --only frontier-10k-hnsw-turboquant-compressed-build \
  --only frontier-10k-hnsw-pq-fastscan-compressed-build \
  --only frontier-10k-hnsw-rabitq-compressed-build \
  --continue-on-error \
  --allow-debug-backend
```

The suite will also write per-query frontier JSONL files because the diagnostic
command exposes them. Keep those as local scratch unless a reviewer explicitly
asks for them; the final packet should cite the summarized per-step logs and
`results-10k-frontier-current-head.jsonl`.

## Expected Shape

The dry-run artifact in this packet proves the current suite selects exactly
six 10k frontier steps:

- `frontier-10k-hnsw-turboquant`
- `frontier-10k-hnsw-pq-fastscan`
- `frontier-10k-hnsw-rabitq`
- `frontier-10k-hnsw-turboquant-compressed-build`
- `frontier-10k-hnsw-pq-fastscan-compressed-build`
- `frontier-10k-hnsw-rabitq-compressed-build`

Packet 015 narrows these 10k diagnostic-only steps to `ef_search=200`, matching
the final decision table and the already-narrowed 50k/100k diagnostic shape.
Recall and latency steps still keep their broader sweep where needed.

## Commit Scope

Commit these regenerated 10k artifacts into packet 006:

- `suite-manifest-10k-frontier-current-head.json`
- `results-10k-frontier-current-head.jsonl`
- `suite-run-10k-frontier-current-head.log`
- the six regenerated per-step 10k frontier `.log` files cited by packet 006
- updated packet 006 `request.md` and `artifacts/manifest.md`

Do not commit the raw per-query frontier JSONL files, truth caches, staged
corpus TSV files, scratch logs, or operational exhaust.
