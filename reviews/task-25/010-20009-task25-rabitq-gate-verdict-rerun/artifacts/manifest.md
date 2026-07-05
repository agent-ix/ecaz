# Artifacts — Packet 20009 (Gate verdict re-run, paper-faithful)

Head SHA at capture: `89f38c3`. Lane: `task25-rabitq-stage1-phase0`.

## `run-dbpedia-10k-paper-faithful.txt`

Command:
```
./target/release/ecaz quant feasibility \
    --quantizer rabitq \
    --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv \
    --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv \
    --dim 1536 --top-k 10
```

Timestamp: 2026-04-23.

Key cited result lines:
- `recall@10 mean: 0.8975`  (vs. slice 8 α-form: 0.8935 — within noise)
- `bound  mean=0.050  p50=0.050  p99=0.052` (12× tighter than slice 8)
- `error  mean=0.010  p50=0.009  p99=0.037` (27× smaller than slice 8)
- `tightness (error / bound) mean: 0.211`
- `GATE: FAIL (recall gap 10.250 pp > 2.0 pp)`

Binary under test: slice-9 paper-faithful estimator
(`o_dot = ⟨o_unit, sign(o)/√D⟩` scalar, ε-concentration bound).
Same seed / corpus / queries as packet 20007 — only the estimator
changed.
