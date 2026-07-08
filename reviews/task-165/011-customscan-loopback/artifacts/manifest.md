# Manifest — Task 165 packet 011 (multi-node CustomScan, loopback-validated)

- **head SHA:** fff3b5f1d (code: 2102272ab CustomScan module; a8b8710e2 search-core refactor)
- **task bucket / packet:** reviews/task-165/011-customscan-loopback
- **branch:** task-165-ec-distann-m3
- **date:** 2026-07-08
- **surface:** committed loopback DB `ec_distann_cs` on the pgrx PG18 instance
  (socket `/home/peter/.pgrx`, port 28818), one-index-per-table. **Loopback**
  2-node roster (both entries → the same instance/db). Release `.so`.
- **fixture:** 400 synthetic rows, dim 8, `encode_to_ecvector(arr, 4, 42)`,
  `ec_distann (embedding ecvector_distann_ip_ops)`, default graph degree.
  Non-unit vectors (build warned) — irrelevant to the identity comparison, which
  pits multi-node vs single-node on the *same* data/distances.

## Commands

- build/install: `cargo pgrx install --release --no-default-features --features pg18 --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- setup: `ecaz dev sql --pg 18 --db ec_distann_cs --socket-dir /home/peter/.pgrx --raw --file artifacts/setup.sql`
- validate: `ecaz dev sql ... --file artifacts/validate.sql`

## Key result lines (see validate.log)

- `Custom Scan (EcDistannDistributedScan) on cs` — planner selects the CustomScan
  under a multi-node roster.
- `n_queries=20, identical_queries=20, total_mismatched_ids=0` — multi-node
  CustomScan top-10 == single-node baseline, id-for-id, all 20 queries.
- Dead-remote-port run: `ERROR ... [EC_INTERNAL] ... could not connect` — the
  remote path is genuinely exercised (fail-closed).

## Not in this packet (open)

- Real 3-PG-instance fixture (Slice A) — proves the shipping path across process
  boundaries with disjoint owned shards (include-mode global vec_ids).
- 3-worker `ecaz bench suite` distinct-recall exit gate (Slice D).
- TC-042 fault matrix + FR-082 lifecycle (Slice C).
