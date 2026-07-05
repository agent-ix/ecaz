# Task 79 Packet 004 Artifact Manifest

- task bucket: `reviews/task-79/`
- packet path: `reviews/task-79/004-route-time-row-budget/`
- head SHA: `2956596e6d2f0d167155432616469ea29c34f659`
- branch: `task-79-spire-candidate-surface-reduction`
- timestamp: `2026-06-01T13:27:44-07:00`
- runner: `ecaz bench suite`
- suite config: `reviews/task-79/004-route-time-row-budget/suite-rabitq-route-time-row-budget.json`
- suite config sha256: `c41197dbb98d7200d514fd9e6056661f149eb402258605686895caa96483c712`
- fixture: `ec_real_100k`, 200 queries, `task79_spire_candidate_surface`
- storage format: `rabitq`
- rerank mode: `rerank_width=25`, production local heap candidates
- surface mode: shared table/index surface, rebuilt per index geometry in suite
- PG target: PG18 over `/home/peter/.pgrx`, port `28818`
- isolated one-index-per-table: no

## Commands

Validation:

```text
cargo test collect_recursive_routing_level_diagnostics_reports_row_budget_truncation --no-default-features --features pg18
cargo test max_routed_candidate_rows --no-default-features --features pg18
cargo test caps_routed_candidate_rows --no-default-features --features pg18
cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings
target/debug/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-79/004-route-time-row-budget/artifacts/install-current-ecaz-pg18.log
/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/004-route-time-row-budget/artifacts/pg18-restart.log restart -m fast
```

Suite:

```text
target/debug/ecaz bench suite audit --config reviews/task-79/004-route-time-row-budget/suite-rabitq-route-time-row-budget.json --log-file reviews/task-79/004-route-time-row-budget/artifacts/suite-audit.log
target/debug/ecaz bench suite run --dry-run --config reviews/task-79/004-route-time-row-budget/suite-rabitq-route-time-row-budget.json --log-file reviews/task-79/004-route-time-row-budget/artifacts/suite-dry-run.log
target/debug/ecaz bench suite run --config reviews/task-79/004-route-time-row-budget/suite-rabitq-route-time-row-budget.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/004-route-time-row-budget/artifacts/suite-run-with-pg-target.log
target/debug/ecaz bench suite status --manifest reviews/task-79/004-route-time-row-budget/artifacts/suite-manifest.json --log-file reviews/task-79/004-route-time-row-budget/artifacts/suite-status.log
target/debug/ecaz bench suite report --manifest reviews/task-79/004-route-time-row-budget/artifacts/suite-manifest.json --log-file reviews/task-79/004-route-time-row-budget/artifacts/suite-report.log
```

The earlier `suite-run.log` and `suite-run-resume-with-pg-target.log` artifacts
are retained as provenance for failed setup invocations before the successful
explicit PG target run. They are not cited as benchmark evidence.

## Artifacts

| artifact | purpose |
| --- | --- |
| `suite-rabitq-route-time-row-budget.json` | checked-in SuiteConfig |
| `artifacts/suite-manifest.json` | suite run manifest |
| `artifacts/results.jsonl` | structured parsed suite rows |
| `artifacts/suite-report.log` | rendered suite report and parsed rows |
| `artifacts/suite-status.log` | completed=6, failed=0, skipped=0 |
| `artifacts/suite-audit.log` | suite audit evidence |
| `artifacts/suite-dry-run.log` | dry-run expansion evidence |
| `artifacts/suite-run-with-pg-target.log` | successful suite run log |
| `artifacts/precheck-existing-task79-surface.log` | fixture row-count and extension precheck |
| `artifacts/rebuild-100k-rabitq-n256-f16-b0-tg256.log` | n256/f16/tg256 RaBitQ rebuild |
| `artifacts/rebuild-100k-rabitq-n512-f16-b0-tg256.log` | n512/f16/tg256 RaBitQ rebuild |
| `artifacts/pipeline-100k-rabitq-n256-f16-b0-tg256-row26k.log` | n256 row26k pipeline output |
| `artifacts/pipeline-100k-rabitq-n256-f16-b0-tg256-row52k.log` | n256 row52k pipeline output |
| `artifacts/pipeline-100k-rabitq-n512-f16-b0-tg256-row26k.log` | n512 row26k pipeline output |
| `artifacts/funnel-100k-rabitq-n256-f16-b0-tg256-row26k.jsonl` | per-query funnel rows for n256 row26k |
| `artifacts/funnel-100k-rabitq-n256-f16-b0-tg256-row52k.jsonl` | per-query funnel rows for n256 row52k |
| `artifacts/funnel-100k-rabitq-n512-f16-b0-tg256-row26k.jsonl` | per-query funnel rows for n512 row26k |
| `artifacts/test-routing-row-budget-diagnostics.log` | new diagnostics unit test |
| `artifacts/test-max-routed-candidate-rows.log` | existing GUC/options test |
| `artifacts/test-caps-routed-candidate-rows.log` | existing placement diagnostics test |
| `artifacts/clippy-pg18.log` | PG18 clippy pass |
| `artifacts/install-current-ecaz-pg18.log` | extension install log |
| `artifacts/pg18-restart.log` | PG18 restart log |

## Key Results

Suite status:

```text
[suite:task79-rabitq-route-time-row-budget] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Candidate and recall rows:

| config | nprobe | route_sum | candidate_sum | retained | returned | p50 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| n256/f16/tg256 row26k | 128 | 13,270 | 5,252,750 | 5,000 | 2,000 | 48.677 ms | 0.9910 |
| n256/f16/tg256 row26k | 192 | 13,270 | 5,252,750 | 5,000 | 2,000 | 67.473 ms | 0.9975 |
| n256/f16/tg256 row26k | 256 | 13,270 | 5,252,750 | 5,000 | 2,000 | 83.561 ms | 1.0000 |
| n256/f16/tg256 row52k | 128 | 25,445 | 10,015,302 | 5,000 | 2,000 | 47.846 ms | 0.9910 |
| n256/f16/tg256 row52k | 192 | 26,538 | 10,455,918 | 5,000 | 2,000 | 65.836 ms | 0.9975 |
| n256/f16/tg256 row52k | 256 | 26,538 | 10,455,918 | 5,000 | 2,000 | 84.475 ms | 1.0000 |
| n512/f16/tg256 row26k | 128 | 24,698 | 5,147,209 | 5,000 | 2,000 | 39.566 ms | 0.9645 |
| n512/f16/tg256 row26k | 192 | 25,116 | 5,231,408 | 5,000 | 2,000 | 48.194 ms | 0.9840 |
| n512/f16/tg256 row26k | 256 | 25,116 | 5,231,408 | 5,000 | 2,000 | 58.153 ms | 0.9940 |

Route-time row budget diagnostics:

| config | nprobe | selected_child_sum | deduped_route_sum | truncation |
| --- | ---: | ---: | ---: | --- |
| n256/f16/tg256 row26k | 128 | 51,200 | 13,270 | row_budget |
| n256/f16/tg256 row26k | 192 | 51,200 | 13,270 | row_budget |
| n256/f16/tg256 row26k | 256 | 51,200 | 13,270 | row_budget |
| n256/f16/tg256 row52k | 128 | 51,200 | 25,445 | mixed |
| n256/f16/tg256 row52k | 192 | 51,200 | 26,538 | row_budget |
| n256/f16/tg256 row52k | 256 | 51,200 | 26,538 | row_budget |
| n512/f16/tg256 row26k | 128 | 102,400 | 24,698 | mixed |
| n512/f16/tg256 row26k | 192 | 102,400 | 25,116 | row_budget |
| n512/f16/tg256 row26k | 256 | 102,400 | 25,116 | row_budget |

Validation:

```text
test am::ec_spire::scan::tests::collect_recursive_routing_level_diagnostics_reports_row_budget_truncation ... ok
test am::ec_spire::options::tests::scan_max_routed_candidate_rows_resolution_disables_zero_and_uses_positive_session_cap ... ok
test am::ec_spire::scan::tests::collect_single_level_scan_plan_placement_diagnostics_caps_routed_candidate_rows ... ok
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
```

## Interpretation

The packet proves the row budget now stops leaf route selection directly:
`routing`, `placement`, and `prefetch` route counts are aligned for the n256
row26k high-recall rows. The remaining candidate count is therefore the cost of
whole-leaf scoring, not an upstream over-routing artifact.

Task 79 remains open. The best high-recall row is still just above the
candidate gate and misses the latency gate. A closing slice needs finer-grain
candidate pruning than whole selected leaves.
