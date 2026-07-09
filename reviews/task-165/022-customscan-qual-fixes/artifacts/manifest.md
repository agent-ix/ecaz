# Manifest — packet 022 (CustomScan qual/LIMIT/index-binding correctness fixes)

- head SHA (pre-commit base): c423b9a626c371f94c0341eb7d2e6198669b47b1
- branch: task-165-ec-distann-m3; date: 2026-07-09
- surface: real 3x PG18 fixture (ecaz dev distann-multicluster), release .so; 110 pg_tests (debug)
- command: ecaz dev distann-multicluster local-multinode-pg18 --nodes 3 --rows 2000 --dim 16 --queries 50 --top-k 10

## Fixes (reviewer 011/020)
- P1 payload ships ALL non-dropped columns (custom_scan.rs build_payload_metadata) — quals on non-projected columns no longer see NULL.
- P1 over-fetch cursor (run_search_and_build_outputs + deepen-on-demand in custom_scan_access) — LIMIT applies after quals, not before.
- P2 planner binds ORDER BY Var attno to the ec_distann index (custom_scan_candidate_index + orderby_var_attno + index_first_key_attno).

## Key result lines
- qual_correctness single_n=10 multi_n=10 mismatch=0 pass=true  (WHERE source[1]>0 ... LIMIT 10, source NOT projected)
- RECALL_RESULT identical; suite_recall_gate delta=0 pass=true; disjoint_shard identical; recovery clean
- 110 distann pg_tests pass; clippy clean
