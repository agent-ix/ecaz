# Task 200 fix/regression artifacts

- Packet: `reviews/task-200/003-fix-and-regression/`
- Code head: `fa84ff3b0` (pushed).
- Fixture: `/home/peter/.ecaz/clusters/task200-counters-off-100k`; reused for
  every attribution and regression run. No corpus/index rebuild occurred after
  the source fix; only the extension was rebuilt/installed when source changed.
- Final extension command: `cargo pgrx install --release --pg-config
  /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features
  --features 'pg18 distann-head-attribution-benchmark'`.
- Final regression command: `BEGIN; SELECT count(*) FROM (SELECT q.id, c.*
  FROM task179_physical_100k_queries q CROSS JOIN LATERAL
  ec_distann_physical_seed_coverage_benchmark('dm_idx', q.source, 32, 32) c
  ORDER BY q.id LIMIT 200) coverage; SELECT
  pg_log_backend_memory_contexts(pg_backend_pid()); COMMIT;`.
- Final result: 200 rows; RSS series in `final-rss-series.log` (402780–403300
  KB); final context in `final-node1-postgres.log`:
  `TopTransactionContext: 142606336 total` and `Grand total: 145793984`.
- Historical unfixed result: `../002-attribution/artifacts/owner-seed-20.log`
  plus `../002-attribution/artifacts/attribution-node1-postgres.log`, showing
  `TopTransactionContext: 5595201536 total`.
- Production A1 evidence remains in packet 002; its held transaction is flat,
  so the task’s conditional 10/50/100k matrix waiver applies.
- Provenance: final source checkpoint is committed, but the host checkout had
  approximately 30 unrelated dirty `src/am` files from other agents during
  the release build; this is disclosed rather than presented as a clean SHA.
- No corpus, query TSV, PGDATA, or cluster directory is committed.
