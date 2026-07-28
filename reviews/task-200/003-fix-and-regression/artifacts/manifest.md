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
- Executable gate config: `task200-coverage-memory-regression-suite.json`.
  The suite run used the existing corpus under
  `data/task106_full_sweep_100k/`, a one-time bootstrap fixture build, and
  then `--reuse-fixture --coverage-memory-regression-iterations 300` with
  `--coverage-memory-regression-max-slope-kb-per-s 1024`. The final generated
  `executable-regression-run/suite-manifest.json` records the exact command,
  config SHA, and successful step.
- Bootstrap artifact: `fixture-bootstrap-run/distann-local-multinode.log`
  records release extension `fa84ff3b0`, 100,000 source rows, three Published
  owners, `physical_ms=1023268`, and `publish_ms=1156605`. The bootstrap used
  the existing corpus TSVs; it did not regenerate corpus data.
- Final regression result from
  `executable-regression-run/distann-local-multinode.log`:
  `coverage_invocations=300 rows_returned=300 samples=16609
  rss_first_kb=19556 rss_last_kb=396632 rss_delta_kb=377076
  rss_slope_kb_per_s=5.82 max_slope_kb_per_s=1024.00 pass=true`.
  The RSS series is `coverage-memory-regression.series.log`; the normalized
  result is in `results.jsonl`. The large first-to-last ramp is startup/working
  set acquisition; the gate is slope-based as required and shows no
  per-invocation unbounded growth.
- Measurement surface: one shared three-owner physical `dm_idx` generation;
  one coordinator backend held one explicit transaction for all 300 coverage
  calls. No one-index-per-table control was used in this diagnostic.
- After the cited artifacts were captured, the stopped 6.7G fixture under
  `/home/peter/.ecaz/clusters/task200-counters-off-100k` and raw operational
  logs were removed. No corpus or PGDATA is committed.
- Sibling conversion audit: `sibling-conversion-audit.md`.
