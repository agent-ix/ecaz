# Recall Methodology

## Dataset

**Primary corpus:** OpenAI `text-embedding-3-large` 1536-dimensional embeddings of DBpedia entity descriptions, published by Qdrant on Hugging Face.

| Field | Value |
| --- | --- |
| Source | Qdrant `dbpedia-entities-openai3-text-embedding-3-large-1536-1M` |
| Total rows | 1,000,000 |
| Dimensionality | 1536 |
| Element type | float32 |
| Distance | inner product (cosine on unit-normalized vectors) |

## Working Subsets

| Subset | Corpus rows | Queries | Purpose |
| --- | --- | --- | --- |
| `tqhnsw_real_50k` | 50,000 | 1,000 | NFR-003 headline shape |
| `tqhnsw_real_10k` | 10,000 | 200 | Fast iteration |

### Selection Rule

Subsets are deterministic, not random:

1. Sort the full dataset by `_id` ascending (lexicographic)
2. `tqhnsw_real_50k` corpus: rows [0, 49,999], queries: rows [50,000, 50,999]
3. `tqhnsw_real_10k` corpus: rows [0, 9,999], queries: rows [10,000, 10,199]

The canonical conversion script is `scripts/qdrant_dbpedia_to_tsv.py`.

## File Format

Corpus and query files are tab-separated:

- Two columns: `id` (int8) and `embedding_json` (JSON float array)
- No header row, UTF-8, Unix line endings

## Loading

```bash
python3 scripts/load_real_corpus.py
```

The loader is idempotent — it skips tables that already exist.

## Reproducing Results

```bash
# Load the dataset
python3 scripts/load_real_corpus.py

# Run the SQL recall benchmark
make bench-recall-sql
```

## Further Reading

- [Benchmarks](benchmarks.md) — measured results
- [RECALL_REAL_CORPUS.md](RECALL_REAL_CORPUS.md) — full dataset contract (selection rules, manifest format, SHA-256 validation)
- [RECALL_ANN_BENCHMARKS_ANCHOR.md](RECALL_ANN_BENCHMARKS_ANCHOR.md) — external recall validation anchor
