# Task 166 M4 — ec_distann benchmark gate verdict

**4-way comparison** (ec_distann vs ec_hnsw / ec_ivf / ec_spire), real DBpedia
10k/50k/100k (dim 1536), Intel local, **release** build, one-index-per-table,
via `ecaz bench suite` (FR-038). ec_distann numbers: packet 002; comparators:
packet 003. Best-recall sweep point per AM; latency **p50** (the `mean` column
is inflated by the cold first query — see note).

## Recall@10 (best of sweep)

| scale | ec_distann | ec_hnsw | ec_ivf | ec_spire |
|-------|-----------:|--------:|-------:|---------:|
| 10k   | **1.0000** | 0.9950  | 0.9740 | 1.0000   |
| 50k   | **0.9950** | 0.9735  | 0.9380 | 0.9625   |
| 100k  | **0.9925** | 0.9630  | 0.9255 | 0.9205   |

## Latency p50 (ms, at the best-recall sweep point)

| scale | ec_distann | ec_hnsw | ec_ivf | ec_spire |
|-------|-----------:|--------:|-------:|---------:|
| 10k   | 10.3       | 6.3     | **2.9**| 122.2    |
| 50k   | 13.9       | 7.4     | **5.8**| 271.6    |
| 100k  | 14.0       | 12.3    | **8.4**| 421.9    |

## Index size

| scale | ec_distann | ec_hnsw | ec_ivf | ec_spire |
|-------|-----------:|--------:|-------:|---------:|
| 10k   | 110.3 MiB  | 13.0    | 9.0    | **8.9**  |
| 50k   | 423.6 MiB  | 65.1    | 41.6   | **41.4** |
| 100k  | 815.2 MiB  | 130.2   | 81.7   | **81.4** |

## Verdict

- **Recall: ec_distann is the top AM at every scale** — ties ec_spire at 10k and
  leads all four at 50k/100k by +0.019–0.072 over the next-best (ec_hnsw). The
  recall advantage widens with scale, exactly where distributed ANN matters.
- **Latency: competitive.** ec_distann p50 is 10–14 ms — 2–5× ec_ivf/ec_hnsw but
  the same order of magnitude, and far below ec_spire's single-node p50 here
  (122–422 ms; that ec_spire figure looks config-bound at nprobe=32 single-node,
  not a floor — reported as-measured). ec_distann's p50 stays ~flat (10→14 ms)
  from 10k→100k while recall stays ≥ 0.9925.
- **Storage: ec_distann is the cost axis** — ~6–10× the comparators (co-placed
  full-precision heap for exact rerank + rich FR-076 node records: coarse code +
  graph_degree adjacency + embedded neighbor codes). This is the deliberate
  design trade: spend bytes to buy the recall lead and node-local exact rerank.

**Gate outcome: PASS.** ec_distann clears the M4 bar — it is recall-competitive-
to-leading against all three in-tree AMs across 10k/50k/100k at competitive p50
latency, with storage as the known, architecturally-motivated cost. The
distributed value proposition (scale-out past one node's memory) is what the
storage buys; the M3 gate already showed multi-node recall == single-node.

## Follow-ups (not gate blockers)

- ec_distann storage reduction (quantize the co-placed rerank vector / prune
  embedded neighbor codes) is a natural optimization task.
- Re-measure ec_spire single-node latency with a saner nprobe grid before citing
  it as a latency comparator; recall/storage for ec_spire stand.
- `mean` latency inflation: the suite's first warm query pays cold-cache cost;
  p50/p95 are the steady-state figures used here.
