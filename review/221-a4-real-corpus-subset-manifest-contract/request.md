# Review Request: A4 Real-Corpus Subset + Manifest Contract

## Context

Branch:
- `fix/a4-real-corpus-recall-lane`

Prior packets:
- `review/218-a4-real-corpus-recall-lane/request.md`
- `review/219-a4-real-corpus-loader-smoke/request.md`
- `review/220-a4-real-corpus-metric-contract-followup/request.md`

This slice addresses the remaining reproducibility blocker from reviews 218 and
219:
- pin the canonical DBpedia working subsets
- add a checked-in parquet -> TSV conversion recipe
- make the loader verify a manifest/hash contract before treating a staged
  subset as canonical

It does **not** claim that the first official DBpedia gate result has been
recorded yet. The actual parquet release is still staged out-of-band by the
user.

## What Landed

### 1. Canonical subset rule is now explicit in repo code and docs

`docs/RECALL_REAL_CORPUS.md` now pins the working subsets by rule rather than
by name only:

- sort the full dataset by `id` ascending
- `tqhnsw_real_50k` corpus: rows `[0, 49_999]`
- `tqhnsw_real_50k` queries: rows `[50_000, 50_999]`
- `tqhnsw_real_10k` corpus: rows `[0, 9_999]`
- `tqhnsw_real_10k` queries: rows `[10_000, 10_199]`

This is the selection rule the first official DBpedia number is expected to use.

Files:
- `docs/RECALL_REAL_CORPUS.md`
- `plan/tasks/12-real-corpus-recall.md`

### 2. Deterministic parquet -> TSV converter

Added:
- `scripts/qdrant_dbpedia_to_tsv.py`

The converter:
- takes the staged parquet file or directory
- uses canonical subset profiles (`tqhnsw_real_50k`, `tqhnsw_real_10k`)
- scans ids, sorts by `id` ascending, and selects the fixed corpus/query
  ranges above
- emits canonical `<prefix>_corpus.tsv` and `<prefix>_queries.tsv` files
- emits `<prefix>_manifest.json`

Design notes:
- `pyarrow` is required locally; the repo does not vendor parquet deps
- the script lazily imports `pyarrow`, so `--help` and clear missing-dependency
  errors work even when parquet support is not installed
- vector output is serialized deterministically using a stable numeric format
  (`.9g`) rather than raw Python `repr` of nested lists

### 3. Manifest/hash verification in the loader

`scripts/load_real_corpus.py` now supports:
- `--manifest-file`
- automatic sibling-manifest discovery when the input files follow the canonical
  `<basename>_{corpus,queries}.tsv` naming convention
- `--allow-manifest-mismatch` as the explicit override

Verification checks:
- manifest schema version
- fixture prefix
- dimensionality
- per-file basename
- per-file row count
- per-file SHA-256
- first/last ids

If any check fails, the loader aborts unless the operator passes
`--allow-manifest-mismatch`.

### 4. Task / contract docs updated

The real-corpus contract doc and Task 12 now both reflect the current state:
- infrastructure exists
- the subset/manifest contract now exists
- the first official DBpedia run still waits on the user staging the actual
  parquet release and committing the generated manifest with the benchmark
  result

## Evidence

### Converter help surface works without `pyarrow`

Command:

```bash
python3 scripts/qdrant_dbpedia_to_tsv.py --help
```

Result:
- shows the canonical profiles and arguments without requiring parquet deps

Missing dependency check:

```bash
python3 scripts/qdrant_dbpedia_to_tsv.py \
  --profile tqhnsw_real_50k \
  --parquet /tmp/does-not-matter \
  --output-dir /tmp/out
```

Result in the current environment:

```text
pyarrow is required for parquet conversion. Install it locally before running scripts/qdrant_dbpedia_to_tsv.py.
```

### Loader verifies a matching manifest

Created a tiny synthetic staged set in `/tmp`:
- `/tmp/tqhnsw_loader_manifest_smoke_corpus.tsv`
- `/tmp/tqhnsw_loader_manifest_smoke_queries.tsv`
- `/tmp/tqhnsw_loader_manifest_smoke_manifest.json`

Then ran:

```bash
./scripts/load_real_corpus_scratch.sh \
  --prefix tqhnsw_loader_manifest_smoke \
  --corpus-file /tmp/tqhnsw_loader_manifest_smoke_corpus.tsv \
  --queries-file /tmp/tqhnsw_loader_manifest_smoke_queries.tsv \
  --m 8
```

Observed output includes:

```text
[loader] verified manifest /tmp/tqhnsw_loader_manifest_smoke_manifest.json for prefix tqhnsw_loader_manifest_smoke
```

and the rest of the load/index-build completes normally.

### Loader rejects a mismatched manifest

Modified the synthetic manifest so `corpus.rows = 999`, then ran:

```bash
./scripts/load_real_corpus_scratch.sh \
  --prefix tqhnsw_loader_manifest_smoke \
  --corpus-file /tmp/tqhnsw_loader_manifest_smoke_corpus.tsv \
  --queries-file /tmp/tqhnsw_loader_manifest_smoke_queries.tsv \
  --manifest-file /tmp/tqhnsw_loader_manifest_smoke_bad_manifest.json \
  --m 8
```

Result:

```text
error: manifest verification failed for /tmp/tqhnsw_loader_manifest_smoke_bad_manifest.json: corpus.rows=999 (expected 24)
```

That is the intended guardrail for the canonical lane.

## What This Resolves

From review 218:
- item 1, canonical subset selection rule: addressed in docs + converter
- item 1, deterministic parquet bridge: addressed via
  `scripts/qdrant_dbpedia_to_tsv.py`
- item 1, manifest/hash contract: addressed in loader + docs, pending only the
  actual manifest file for the first staged DBpedia run

From review 219:
- the recommendation to make the first official DBpedia number reproducible is
  now concretely implemented

## What Still Remains

The remaining step is operational, not structural:
- stage the actual Qdrant DBpedia parquet
- run `scripts/qdrant_dbpedia_to_tsv.py`
- load the emitted canonical TSVs
- commit the generated real manifest alongside the first benchmark/report packet

Until that happens, the branch has the contract but not yet the first official
DBpedia result artifact.

## Files

- `scripts/qdrant_dbpedia_to_tsv.py`
- `scripts/load_real_corpus.py`
- `docs/RECALL_REAL_CORPUS.md`
- `plan/tasks/12-real-corpus-recall.md`
