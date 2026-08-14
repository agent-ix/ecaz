# Task 167 packet 026 artifacts

- Code checkpoint: `2168b60d6` (owner payload materialization retry).
- Fixture checkpoint: `a57a3673a` (routed DELETE/VACUUM drill uses the benchmark
  table instead of hard-coded `dm`).
- Packet: `reviews/task-167/026-owner-retry`.
- Suite config: `artifacts/task167-owner-retry-suite.json`.
- Earlier run: `ecaz bench suite run ... --only physical-10k --continue-on-error`;
  artifact directory:
  `artifacts/bench-suite-final-remote-writers-10k`.
- Follow-up run after the payload retry: same command, artifact directory
  `artifacts/bench-suite-final-2168-10k`. Its extension was built while the
  tracked task bookkeeping was dirty, so it is diagnostic evidence only; a
  compact extract is `artifacts/diagnostic-2168-summary.log` and a clean-head
  rerun is required for provenance.
- Timestamp: 2026-08-13; three-node PG18 physical fixture, `ec_real_10k`,
  shared coordinator source table and one physical generation sharded across
  three owner nodes; graph degree 5.
- The append A/B control disables `debug_disable_append_when_room`; the
  candidate enables normal append-when-room behavior. The fixture reports
  backlink amendments and no-room counts separately.
- The pinned-probe result is diagnostic only: ANN post-filtering can produce
  either `returned_sample` or `zero_rows`; the owner-local exact probe is the
  authoritative placement check.

Key cited lines from `physical-10k/distann-local-multinode.log`:

- `release_profile_preflight status=passed ... extension_git_sha=f45a91e15... extension_build_profile=release`.
- `physical_benchmark_insert_throughput_ab ... physical_over_control=0.731232 pass=true`.
- `physical_benchmark_append_when_room_ab ... append_enabled_over_disabled=1.030609 disabled_backlink_amendments=110 enabled_backlink_amendments=62 ... pass=true`.
- `frontier_retry_probe owner=1 ... pass=true`.
- `frontier_retry_counter churn_retries=Some(1) ... steady_retries=Some(0) pass=true`.
- The follow-up run clears the reviewer’s live concurrency gate:
  `physical_concurrent_insert_query pass=true`,
  `churn_retries=Some(1)`, `steady_retries=Some(0)`,
  `reverse_edge_coverage=15/24`, and both inserted vec_ids are present in the
  target neighborhood. It then reaches the routed DELETE/VACUUM drill, which
  was fixed in `a57a3673a` after the run and must be rerun clean. No
  50k/100k closeout matrix is claimed yet.
