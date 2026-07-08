# Manifest — Task 165 M3 exit gate: 50k multinode recall

- head SHA: this commit (task-165-ec-distann-m3)
- task bucket / packet: reviews/task-165/006-multinode-50k-recall
- lane / fixture: Intel local, ec_distann, real DBpedia 50k (dim 1536)
- index: m1_50k_mono_idx (monolithic build, 50000 rows), one-index-per-table
  (m1_50k_mono_corpus), REINDEXed with the current .so (format v2)
- build profile: release (ecaz_build_profile = release, verified)
- storage: isolated one-index-per-table
- command: `ecaz dev sql --db ec_distann_bench --file reviews/task-165/006-multinode-50k-recall/recall-compare.sql`
- timestamp: 2026-07-08
- key result (artifacts/recall-compare.log): 51 queries, 2-node loopback top-10
  vs single-node top-10 — identical_queries=51/51, total_mismatched_ids=0.
  Byte-identical => multinode recall == single-node (delta 0, >= single-node
  - 0.001, the M3 gate).

## Method note

The gate spec asks distinct_recall(multinode) >= distinct_recall(single-node)
- 0.001. This packet proves the stronger property: the 2-node top-k is
id-for-id identical to the single-node top-k across 51 queries, so the recall
delta is exactly 0 for any ground truth. Loopback substrate (ADR-085 D2): both
roster nodes address the same instance; the coordinator materialises remote
hits' heap TIDs from its full local directory (slice 005).

## Reproduction caveat

ec_distann_bench predates the M2 endpoints; its extension SQL was refreshed by
applying the ec_distann_{expand_nodes,apply_record_writes,epoch_fingerprint,
debug_*} CREATE FUNCTION statements from the installed ecaz--0.1.1.sql (with
MODULE_PATHNAME -> $libdir/ecaz). A fresh DB via CREATE EXTENSION carries them
directly.
