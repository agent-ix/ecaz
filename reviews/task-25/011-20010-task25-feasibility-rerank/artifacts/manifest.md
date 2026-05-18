# Artifacts — Packet 20010 (--rerank-k flag + two-stage verdict)

Head SHA at capture: `a707395`. Lane: `task25-rabitq-stage1-phase0`.

## `run-dbpedia-10k-rerank-100.txt`

Command:
```
./target/release/ecaz quant feasibility \
    --quantizer rabitq \
    --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv \
    --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv \
    --dim 1536 --top-k 10 --rerank-k 100
```

Timestamp: 2026-04-23.

Key cited result lines:
- `recall@10 (no rerank) mean: 0.8975`   (Symphony Stage-3-aligned gate)
- `recall@10 (rerank K'=100) mean: 1.0000`   (non-Symphony rerank pipeline)
- `bound  mean=0.050`
- `error  mean=0.010`
- `tightness (error / bound) mean: 0.211`
- `GATE (no-rerank / Symphony Stage-3): FAIL (recall gap 10.250 pp > 2.0 pp)`

Demonstrates that the paper-faithful estimator lands every true
top-10 inside the estimator's top-100 on every query — matching
the RaBitQ paper's 1-bit + exact-rerank published numbers.
