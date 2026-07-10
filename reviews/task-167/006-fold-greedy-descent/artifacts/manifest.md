# Packet 167/006 — M5 fold redesign: greedy graph descent for insert candidates

- task bucket / packet: reviews/task-167/006-fold-greedy-descent
- surface: single-node ec_distann, debug `.so` (code change under review), PG18
  (port 28818), DB `distann_t165`, staged real DBpedia (`ec_real_{10k,50k}`).
- method: same A/B as packet 167/005 (build ~95%, INSERT ~5% → delta buffer, fold,
  recall vs brute-force GT). Baselines: old fold = packet 167/005; full rebuild =
  packet 026.

## The fix (code)

Packet 167/005 proved the incremental fold was not recall-viable because
`graph_insert_record` chose the new node's forward neighbors from ONLY the fixed
`head_index_cap` (4096) head-sample region — too small a fraction of a large
graph, so folded nodes were weakly integrated.

`graph_insert_record` now runs a **greedy best-first graph descent**
(`greedy_insert_candidates`) over the PERSISTED graph:
- seeds the beam from the head samples, then walks node records' neighbor edges,
  scoring neighbors by their EMBEDDED codes (the FR-081 scan metric,
  `DistannPreparedQuery::score_dists_batch`) — **zero heap reads during the walk**;
- reserves exact co-placed-vector reads (`fetch_heap_source_vector`) for only the
  final `build_list_size` candidate pool handed to `robust_prune`;
- the back-edge reprune's `source_map` now draws from the descent's visited pool
  (a superset of the old head-sample-only map).

A first implementation that read exact heap vectors for every neighbor during the
walk took **46 min for 500 inserts** (unusable) and was replaced by the
code-based traversal above (fast).

## Result — recall@10, greedy fold vs old fold vs full rebuild

| scale | sweep | old fold (167/005) | **greedy fold** | full rebuild (026) |
| ----- | ----- | ------------------ | --------------- | ------------------ |
| 10k | 16  | 0.7950 | **0.8610** | 0.9935 |
| 10k | 32  | 0.8250 | **0.9095** | 0.9990 |
| 10k | 64  | 0.8470 | **0.9735** | 0.9995 |
| 10k | 100 | 0.8570 | **0.9820** | 1.0000 |
| 10k | 200 | 0.8780 | **0.9930** | 1.0000 |
| 50k | 16  | 0.6260 | 0.6540 | 0.9150 |
| 50k | 32  | 0.6520 | 0.6920 | 0.9545 |
| 50k | 64  | 0.6700 | 0.7395 | 0.9840 |
| 50k | 100 | 0.6810 | 0.7790 | 0.9880 |
| 50k | 200 | 0.6910 | **0.8280** | 0.9950 |

(50k greedy = scaled visit budget, commit `c574c1fa8`; fixed-budget 50k was
0.7605 at tk=200 — the scaled budget raised it to 0.828.)

## Reading — 10k parity, 50k improved-not-parity (honest)

- **10k: the greedy descent closes the fold-recall gap.** recall@10 at tk=200
  rose 0.878 → **0.993** (gap to full rebuild 0.122 → 0.007); tk=64 0.847 →
  0.974. At 10k the head-sample seed covers ~40% of the graph, so the descent
  reaches the new node's true neighborhood. Directly resolves 167-005 / 004-P2 at
  10k.
- **50k: a large improvement, but NOT full parity.** recall@10 at tk=200 rose
  0.691 → **0.828** (and scales with the visit budget: 100 visits → 0.76, ~400 →
  0.828). At 50k the head samples cover only ~8% of the graph; even with a scaled
  budget the greedy best-first walk does not consistently reach the true
  neighborhood, so a ~0.17 gap to full rebuild (0.995) remains.
- **Verdict:** the greedy descent is a real, substantial fix (10k parity; 50k
  major improvement, budget-tunable) but the incremental fold is **not yet
  recall-parity at 50k+**. Closing the scale gap needs a convergence-based descent
  (the scan's beam + early-exit, seeded from the medoid) rather than a single
  best-first walk, OR REINDEX (epoch rebuild) remains the parity mechanism for
  large incremental loads. This is the honest remaining M5 fold work — the fold is
  now much more usable, not fully solved at scale.

## Known follow-ups

- A fold run inside a SINGLE multi-statement transaction (CREATE INDEX + INSERT +
  fold in one txn) errors with the new descent (snapshot/heap-open in an
  uncommitted same-txn context); the normal path (aminsert commits, operator
  `fold` is a separate statement) works. Worth hardening.
- The descent still rebuilds the full sorted directory per folded row (004-P2
  O(N) per-insert work) — a separate perf item.
