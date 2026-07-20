# Manifest — Task 167 M5 packet 004: fold-recall A/B

- head SHA: this commit (task-165-ec-distann-m3)
- packet: reviews/task-167/004-fold-recall
- lane / fixture: Intel local, ec_distann, real DBpedia (2001-row subset of the
  ec_distann_m2 10k corpus), dim 1536, graph_degree=32, 50 real queries
- build profile: release (rebuilt/installed with the M5 fold fn this cycle)
- storage: isolated one-index-per-table (ab_corpus / ab_fold)
- command: `ecaz dev sql --db ec_distann_m2 --file reviews/task-167/004-fold-recall/fold-recall.sql`
- timestamp: 2026-07-08
- key result (artifacts/fold-recall.log): recall@10 vs brute-force truth —
  A_full (full build) = 0.9560; B_fold (1801 built + 200 inserted+folded) =
  0.8900. Both indexes cover the identical 2001 rows.

## Finding

The M5 fold is **functionally correct** (folded rows are found via graph
traversal — pg_test `test_ec_distann_fold_delta_into_graph`) but is **not yet
recall-parity with a full rebuild**: ~0.066 recall@10 lower with ~21% of the
index folded in one batch. Root causes, both in the noted follow-up set:

1. **Append-if-free backlinks** — a forward neighbor already at graph_degree
   skips the back-edge, so folded nodes are under-linked *into* the graph.
2. **Head-sample-only candidate search** — folded nodes are not head samples,
   so a batch of folds does not interconnect (each connects only to the
   originally-built nodes).

This is a "measure, don't assume" result: the plausible "fold is recall-neutral"
claim is false. Closing the gap needs the full-reprune-backlink and
head-sample-refresh follow-ups. Until then, a REINDEX (epoch build) restores
full recall, and the delta buffer + fold keep inserted rows correct and
queryable in the interim.
