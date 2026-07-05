# Artifacts — Packet 20011 (q-bit recall sweep)

Head SHA at capture: `7edc1e8`. Lane: `task25-rabitq-stage1-phase0`.

## `sweep-dbpedia-10k.txt`

Command (shell loop):
```
for BITS in 1 2 4 8; do
    ./target/release/ecaz quant feasibility \
        --quantizer rabitq \
        --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv \
        --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv \
        --dim 1536 --top-k 10 --bits $BITS
done
```

Timestamp: 2026-04-23.

Key cited result lines (per bits setting):

| bits | code size | recall@10 | gap       | verdict  |
|------|-----------|-----------|-----------|----------|
|  1   |   204 B   |  0.8975   | 10.25 pp  | FAIL     |
|  2   |   396 B   |  0.9430   |  5.70 pp  | FAIL     |
|  4   |   780 B   |  0.9790   |  2.10 pp  | FAIL (0.1 pp from MARGINAL) |
|  8   |  1548 B   |  0.9865   |  1.35 pp  | MARGINAL |

q=4 ≈ PQ4-parity storage (780 B vs. 768 B). Demonstrates smooth
recall/storage tradeoff and flips the verdict from FAIL to
MARGINAL at q=8 — the "keep module + run OPQ" branch of the task
rubric. Off Symphony's critical path (Symphony uses q=1).
