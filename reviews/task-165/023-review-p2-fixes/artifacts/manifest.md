# Manifest — packet 023 (reviewer P2 + 019-P1 fixes)

- head SHA (base): 3ed8e7fa20746d3f428107db96ffc5d6c20b67f8; branch: task-165-ec-distann-m3; date: 2026-07-09
- surface: real 3x PG18 fixture, release .so; command: ecaz dev distann-multicluster local-multinode-pg18 --nodes 3 --rows 2000 --dim 16 --queries 50 --top-k 10

## Fixes
- 021-P2 disjoint signature now over (id, EXACT DISTANCE) in canonical (dist,id) order — proves distances match (recall), not just the id set; deterministic (tie-order-independent).
- 019-P1 suite recall gate now FAILS the fixture on a real regression (pass=false bails; SKIPPED/INCONCLUSIVE are env, non-fatal).
- 021-P2 ec_distann_list_directory + resolve_owned_rows now validate the node-record tag/length (decode_node_heap_identity) before slicing offsets — important since list_directory drives destructive DELETE pruning.

## Key result lines
- disjoint_shard identical_after_prune=true per_node_rows[n1:2000->647 n2:2000->639 n3:2000->714]
- suite_recall_gate delta=0.0000 pass=true; qual_correctness mismatch=0 pass=true; recovery clean
