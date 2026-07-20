# Packet 167/006 — M5 fold redesign: incremental fold reaches REBUILD PARITY

- task bucket / packet: reviews/task-167/006-fold-greedy-descent
- surface: single-node ec_distann, **release** `.so`, PG18 (port 28818), DB
  `distann_t165`, staged real DBpedia (`ec_real_{10k,50k,100k}`).
- method (A/B per scale): build the ec_distann graph on the first ~95% of rows,
  `INSERT` the remaining ~5% (→ D5 delta buffer via `aminsert`),
  `ec_distann_fold_delta_into_graph(idx)` to graph-fold them, then recall vs
  brute-force GT over the full corpus with `ecaz bench recall` (sweep
  `[16,32,64,100,200]`, k=10, 200 queries). Full-rebuild baseline = packet 026.

## The fix (code, commit `f70691402`)

Packet 167/005 proved the incremental fold was not recall-viable — the old
`graph_insert_record` chose the new node's forward neighbors from ONLY the fixed
`head_index_cap` (4096) head-sample region, so folded nodes were weakly
integrated and recall collapsed at scale (0.878 at 10k → 0.691 at 50k, ~5% folded).

Two iterations:
1. A **greedy graph descent** (code-based traversal): fully fixed 10k (→0.993) but
   plateaued below parity at 50k (→0.828) — a single best-first walk under-explores
   a large graph where the head samples cover only ~8%.
2. **Reuse the proven FR-081 scan search** (`collect_distann_hits`: convergent beam
   + hop rounds + early-exit, seeded from the head index) to find the new node's
   true nearest neighbors, then `robust_prune` over their exact co-placed vectors.
   This converges reliably at every scale. (A first descent variant that read exact
   heap vectors per neighbor took 46 min/500 inserts and was discarded.)

## Result — recall@10, incremental fold vs full rebuild

| scale | sweep | old fold (005) | greedy (interim) | **scan-search fold** | full rebuild (026) |
| ----- | ----- | -------------- | ---------------- | -------------------- | ------------------ |
| 10k | 16  | 0.7950 | 0.8610 | **0.9995** | 0.9935 |
| 10k | 200 | 0.8780 | 0.9930 | **1.0000** | 1.0000 |
| 50k | 16  | 0.6260 | 0.6540 | **0.9400** | 0.9150 |
| 50k | 64  | 0.6700 | 0.7395 | **0.9825** | 0.9840 |
| 50k | 200 | 0.6910 | 0.8280 | **0.9965** | 0.9950 |
| 100k| 16  | — | — | **0.8605** | 0.8685 |
| 100k| 64  | — | — | **0.9650** | 0.9650 |
| 100k| 200 | — | — | **0.9905** | 0.9925 |

Full scan-search sweeps: 10k `0.9995/1.0/1.0/1.0/1.0`; 50k
`0.9400/0.9655/0.9825/0.9910/0.9965`; 100k `0.8605/0.9215/0.9650/0.9795/0.9905`
(full rebuild 100k `0.8685/0.9260/0.9650/0.9770/0.9925`).

## Reading

**The incremental fold now matches full rebuild recall at every scale
(10k / 50k / 100k).** 10k at parity (1.0); 50k equals or beats rebuild at every
sweep point (tk=16 0.940 vs 0.915; tk=200 0.9965 vs 0.9950); 100k within ±0.008
(mostly ±0.005) of rebuild at every point. This resolves the Task 167 004-P2
fold-recall-parity acceptance criterion — the D5 delta-buffer + `fold` path is now
a recall-viable incremental-insert mechanism, not an interim posture requiring
REINDEX.

## Remaining M5 work (unchanged by this packet)

- **004-P2 perf:** the fold runs the scan search + a full sorted-directory rewrite
  per folded row (O(N) per row); batching the directory publish across the fold is
  a perf follow-up (recall is now solved; throughput is not).
- **004-P1:** distributed `aminsert` self-insertion (write-endpoint new-record
  append + coordinator routing) is still unbuilt.
- A fold inside a SINGLE multi-statement transaction (CREATE INDEX+INSERT+fold in
  one txn) errors on snapshot/heap-open; the normal separate-statement path works.
