# Manifest — Task 165 004-P1: transport fault drills

- head SHA: 17f4ca8cc1b76d05a92cc0e6616b54c7fa400895
- packet: reviews/task-165/008-transport-fault-drills
- lane / fixture: Intel local, ec_distann, real DBpedia 10k subset (3000 rows, v3 p3_idx), dim 1536
- build profile: release
- command: ecaz dev sql --db ec_distann_m2 --file reviews/task-165/008-transport-fault-drills/fault-drills.sql
- timestamp: 2026-07-08
- key result (artifacts/fault-drills.log): the real AM scan path is fail-closed
  under transport faults —
  - DRILL 0 baseline (good 2-node roster): 10 rows
  - DRILL 1 connection_reset (node-1 unreachable port): ERROR [EC_INTERNAL]
  - DRILL 2 missing_remote_target (node-1 nonexistent db): ERROR [EC_INTERNAL]
  - DRILL 3 no-false-reject (session epoch bump, same content): 10 rows
  - DRILL 4 recovery (good roster restored): 10 rows
  Every fault is an ERROR (never a wrong/partial result); baseline, no-false-
  reject, and recovery are identical to the single-node baseline.
