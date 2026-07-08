# Manifest — 006-P3 transport session-setup cache validation

- head SHA: 891afb6a742360c115b7417fb9e4fe576d83b7aa
- packet: reviews/task-165/007-transport-session-cache
- lane / fixture: Intel local, ec_distann, real DBpedia 10k subset (3000 rows), dim 1536
- index: p3_idx (freshly built at format v3; the delta_count bump invalidated v2)
- build profile: release
- command: ecaz dev sql --db ec_distann_m2 --file reviews/task-165/007-transport-session-cache/validate.sql
- timestamp: 2026-07-08
- key result (artifacts/validate.log): 2-node loopback top-10 == single-node
  (base_minus_two=0, two_minus_base=0, order_identical=t) after moving remote
  session setup to per-connection (006-P3).
