# Artifacts — Packet 20007 (Task 25 first gate verdict)

Head SHA at capture: `e15b37c` (packet commit; see
`review/20007-task25-rabitq-gate-verdict/request.md` for context).
Lane: `task25-rabitq-stage1-phase0`.

## `run-dbpedia-10k.txt`

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
- `recall@10 mean: 0.8935`
- `bound  mean=0.612  p50=0.612  p99=0.630`
- `error  mean=0.268  p50=0.278  p99=0.352`
- `tightness (error / bound) mean: 0.437`
- `GATE: FAIL (recall gap 10.650 pp > 2.0 pp)`

Binary under test: slice-4 α = mean(|c_i|) estimator. Superseded
by slice 9 (paper-faithful form); see
`review/20009-task25-rabitq-gate-verdict-rerun/artifacts/run-dbpedia-10k-paper-faithful.txt`
for the re-run.
