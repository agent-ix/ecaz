# Task 200 fix/regression artifacts

- Packet: `reviews/task-200/003-fix-and-regression/`
- Code heads: root fix `fa84ff3b0`; latest executable-gate implementation
  `9fffefffb`; clean committed rerun at `d845d8e43` (all pushed).
- Fixture: `/home/peter/.ecaz/clusters/task200-counters-off-100k` for the clean
  fixed run. The required pre-fix control reused the separately preserved
  `/home/peter/.ecaz/clusters/task200-counters-on-100k`. No corpus or index
  rebuild occurred for either regression run; only the extension was rebuilt
  when switching source versions.
- Clean provenance build: detached worktree at `fa84ff3b0`, using
  `CARGO_TARGET_DIR=/home/peter/.cargo-target`:
  `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo
  pgrx install --release --pg-config
  /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features
  --features 'pg18 distann-head-attribution-benchmark'`. Runtime verification
  is in `clean-extension-provenance.log`.
- Clean final regression command: `ecaz bench suite run --config
  reviews/task-200/003-fix-and-regression/artifacts/task200-coverage-memory-regression-suite.json
  --artifact-dir
  reviews/task-200/003-fix-and-regression/artifacts/post-warmup-regression-run
  --manifest-output
  reviews/task-200/003-fix-and-regression/artifacts/post-warmup-regression-run/suite-manifest.json`.
- Clean final result: the reused fixture emitted 300 rows and 16,569 RSS
  samples. After six warm-up invocations, a one-second settle, and 40 samples
  trimmed from each edge, the stable series had RSS p01=401,756 KB,
  p99=402,776 KB, p01-to-p99 delta=1,020 KB, and fitted slope=+1.02 KB/s.
  The committed limits are 4,096 KB and 100 KB/s; the gate passed. The full
  series is `post-warmup-regression-run/coverage-memory-regression.series.log`.
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
- No corpus, query TSV, PGDATA, or cluster directory is committed. The clean
  committed rerun performed one necessary fixture bootstrap from the existing
  staged corpus after the prior stopped fixture had been removed; all gate
  attempts after that bootstrap used `--reuse-fixture`.
- Executable gate config: `task200-coverage-memory-regression-suite.json`.
  The suite used the existing corpus under
  `data/task106_full_sweep_100k/`, performed one bootstrap fixture build, and
  then used `--reuse-fixture` for the regression run. The packet config sets
  `coverage_memory_regression_max_slope_kb_per_s=100.0` and
  `coverage_memory_regression_max_delta_kb=4096.0`. The final acceptance line
  is in `post-warmup-regression-run/distann-local-multinode.log`; it records
  `warmup_invocations=6`, `warmup_settle_ms=1000`, `stable_samples=16489`,
  `rss_p01_to_p99_kb=1020`, `rss_slope_kb_per_s=1.02`, and `pass=true`.
  The earlier 1,024 KB/s run is historical only and is not used to establish
  the current threshold.
- Bootstrap artifact: `fixture-bootstrap-postwarmup/distann-local-multinode.log`
  records 100,000 source rows, three Published owners, and release fixture
  construction followed by `--reuse-fixture` gate execution. The bootstrap
  used the existing corpus TSVs; it did not regenerate corpus data. Its suite
  manifest records the bootstrap runner and config provenance.
- Final regression result from
  `post-warmup-regression-run/distann-local-multinode.log`:
  `coverage_invocations=300 rows_returned=300 samples=16569
  stable_samples=16489 rss_first_kb=402064 rss_last_kb=402780
  rss_p01_kb=401756 rss_p99_kb=402776 rss_p01_to_p99_kb=1020
  max_delta_kb=4096.00 rss_slope_kb_per_s=1.02
  max_slope_kb_per_s=100.00 pass=true`.
  The full RSS series is preserved. The p01-to-p99 statistic intentionally
  excludes the one-percent tails where the operating system can reclaim or
  reacquire working-set pages; it supplies the requested absolute post-warm-up
  bound while the full series remains available for audit.
- Clean committed-tree acceptance rerun: `clean-committed-positive-run/` was
  run with the same suite config and reused the clean bootstrap fixture. Its
  release extension provenance is `d845d8e4347d59dafd2b1ed28cd252d7d7c6e134`
  with no dirty suffix. The shipped statistic passed at
  `samples=16586 stable_samples=16506 rss_p01_kb=401880
  rss_p99_kb=402832 rss_p01_to_p99_kb=952 max_delta_kb=4096.00
  rss_slope_kb_per_s=1.10 max_slope_kb_per_s=100.00 pass=true`.
- Negative-control follow-up: the preserved pre-fix control predates the
  warm-up/percentile implementation and therefore reports only the earlier
  slope fields. It failed at `rss_slope_kb_per_s=98380.15` and raw
  `rss_delta_kb=245576`, versus shipped limits of 100 KB/s and 4,096 KB.
  Warm-up removal and a one-percent tail trim cannot plausibly erase either
  margin (approximately 984x on slope and 60x on raw delta), so the existing
  red control remains decision-grade evidence; the clean committed positive
  run directly exercises the shipped statistic. A second pre-fix fixture build
  was not performed solely to restage identical 100k data.
- Required pre-fix negative control: the same suite gate ran against the
  preserved 100k fixture with the extension built from `fa84ff3b0^` (`897c690`
  plus the fixture's existing dirty provenance marker). It executed 20 calls
  and failed as intended: `rss_first_kb=21216 rss_last_kb=266792
  rss_delta_kb=245576 rss_slope_kb_per_s=98380.15
  max_slope_kb_per_s=100.00 pass=false`. Evidence is in
  `negative-control-run/distann-local-multinode.log` and
  `negative-control-run/coverage-memory-regression.series.log`.
- Unattended PG18 mechanism test: `cargo pgrx test pg18
  test_distann_physical_seed_detoast_memory_is_bounded --no-default-features`
  passed 300 owner-seed conversions in one test transaction and asserted
  backend memory growth stayed below 4 MiB. The standard command resolves
  `PGRX_FEATURES="pg18 pg_test"`, and `pg_test` includes
  `distann-head-attribution-benchmark`, so the test is not hidden behind an
  extra feature flag. Its fixture has 512 graph rows with 256-neighbor toasted
  records. The fixed output is `pg18-seed-memory-regression-standard.log`;
  the same command on `fa84ff3b0^` fails with 1,258,283,008 bytes retained,
  recorded in `pg18-seed-memory-regression-pref-fix.log`.
- Latency comparability note: the A1 mean of 36.2 ms versus the Phase 1 mean
  of 27.50 ms is neither a regression nor a win. Those runs used different
  held-transaction/single-snapshot versus autocommit protocols, a target/debug
  CLI driver, and benchmark-only cfg code that cannot execute in the
  production arm.
- Measurement surface: one shared three-owner physical `dm_idx` generation;
  one coordinator backend held one explicit transaction for all 300 coverage
  calls. No one-index-per-table control was used in this diagnostic.
- The final gate explicitly reports `fixture_decision action=reuse`, with
  source rows=100,000 and the packet bootstrap provenance. No corpus or
  PGDATA is committed; the stopped 6.8G rerun fixture will be removed after
  these packet artifacts are committed.
- Sibling conversion audit: `sibling-conversion-audit.md`.
- Task 188 follow-up rerun: `task188-fixed-no-reconnect-run-r2/` was driven by
  `task188-fixed-no-reconnect-suite.json` on the clean committed fixed build
  `d845d8e4347d59dafd2b1ed28cd252d7d7c6e134`. It used the staged
  `ec_real_100k` corpus, one necessary 100k fixture bootstrap, stage counters
  on, `benchmark_backend_batch_size=0`, no reconnect, no `skip_recall`, both
  Task 188 seed variants, and backend memory sampling. The suite succeeded
  in 3,090,025 ms; its manifest records the exact command and provenance.
- Task 188 coverage completed for 200 queries with `zero_fraction=0` and
  `physical_topology_gate pass=true`. The `bw4-control` memory series has 6
  samples over 1263 ms, RSS 246792→254764 KB, delta 7972 KB, and constant
  HWM 378020 KB. The `bw8-candidate` series has 5 samples over 1010 ms, RSS
  249388→256636 KB, delta 7248 KB, and constant HWM 379348 KB. These are
  startup working-set rises, not the prior multi-GB retention. See
  `task188-fixed-no-reconnect-outcome.md` for the concise outcome.
- This evidence supports removing the reconnect workaround, but Task 200
  does not edit Task 188's config or lane. Removal requires confirmation and
  coordination with the Task 188 owner.
