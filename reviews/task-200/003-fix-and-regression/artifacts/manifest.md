# Task 200 fix/regression artifacts

- Packet: `reviews/task-200/003-fix-and-regression/`
- Code head: `fa84ff3b0` (pushed).
- Fixture: `/home/peter/.ecaz/clusters/task200-counters-off-100k`; reused for
  every attribution and regression run. No corpus/index rebuild occurred after
  the source fix; only the extension was rebuilt/installed when source changed.
- Clean provenance build: detached worktree at `fa84ff3b0`, using
  `CARGO_TARGET_DIR=/home/peter/.cargo-target`:
  `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo
  pgrx install --release --pg-config
  /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features
  --features 'pg18 distann-head-attribution-benchmark'`. Runtime verification
  is in `clean-extension-provenance.log`.
- Clean final regression command: `BEGIN; SELECT count(*) FROM (SELECT
  repeat_no, q.id, c.* FROM generate_series(1,2) AS repeats(repeat_no) CROSS
  JOIN task179_physical_100k_queries q CROSS JOIN LATERAL
  ec_distann_physical_seed_coverage_benchmark('dm_idx', q.source, 32, 32) c
  ORDER BY repeat_no, q.id LIMIT 300) coverage; SELECT
  pg_log_backend_memory_contexts(pg_backend_pid()); COMMIT;`.
- Clean final result: 300 rows; RSS series in `final-rss-series-clean.log`
  (401820–402648 KB, fitted slope +1.42 KB/s); final context in
  `clean-final-node1-postgres.log` at 11:00:53:
  `TopTransactionContext: 142606336 total` and `Grand total: 144745408`.
- Clean production A1 command: `ecaz bench latency --prefix
  task179_physical_100k --profile ec_distann --iterations 300
  --hold-transaction --sample-backend-memory`, with the same reused fixture
  and clean extension. Its series is under `clean-held-tx-a1/`; RSS rose from
  251892 to 260780 KB during setup and then plateaued for the remainder.
- Historical unfixed result: `../002-attribution/artifacts/owner-seed-20.log`
  plus `../002-attribution/artifacts/attribution-node1-postgres.log`, showing
  `TopTransactionContext: 5595201536 total`.
- Attribution evidence in packet 002 was collected from the prior dirty build
  and is labeled there; the closeout regression and production A1 above were
  rerun from the clean committed extension.
- Because the clean A1 shows no unbounded retention and the production read path
  is unchanged, the conditional 10/50/100k recall/latency/storage matrix waiver
  applies. If a future change reaches the production read path, the waiver
  lapses.
- Task 188's `benchmark_backend_batch_size=5` remains only in its historical
  packet artifact; the active default is zero. The clean A1 and regression both
  used one backend without reconnects, so no live reconnect workaround remains
  to mask this defect.
- No corpus, query TSV, PGDATA, or cluster directory is committed.
