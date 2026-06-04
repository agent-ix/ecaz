# Review Request: Route-Time Row Budget

## Scope

This packet is the Task 79 follow-up to packet 003 and reviewer feedback F1/F2.

The code moves `ec_spire.max_routed_candidate_rows` from a post-route leaf
filter into recursive/top-graph leaf route selection. The scan now loads visible
leaf `assignment_count` values with the routing hierarchy and uses those cached
counts as the route-generation stop condition, avoiding the packet 003
per-route leaf header reads.

No Task 79 text revision is needed for this slice. This implements the current
Phase 3 direction and does not claim Task 79 closure.

## Evidence

- Task definition: `plan/tasks/79-spire-candidate-surface-reduction.md`
- Suite config: `reviews/task-79/004-route-time-row-budget/suite-rabitq-route-time-row-budget.json`
- Artifact manifest: `reviews/task-79/004-route-time-row-budget/artifacts/manifest.md`
- Suite status: `reviews/task-79/004-route-time-row-budget/artifacts/suite-status.log`
- Parsed report: `reviews/task-79/004-route-time-row-budget/artifacts/suite-report.log`
- Structured rows: `reviews/task-79/004-route-time-row-budget/artifacts/results.jsonl`
- Funnel rows: `reviews/task-79/004-route-time-row-budget/artifacts/funnel-*.jsonl`
- Focused validation logs: `reviews/task-79/004-route-time-row-budget/artifacts/test-*.log`,
  `reviews/task-79/004-route-time-row-budget/artifacts/clippy-pg18.log`

## Result

Route-time budgeting works mechanically and removes the upstream route/placement
mismatch from packet 003, but it still does not clear Task 79. The selected leaf
granularity remains too coarse.

| config | nprobe | routes | candidates | p50 | recall@10 | gate read |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Task 78 baseline n128/f8/tg96 | 96 | 19,200 | 15,506,227 | 60.256 ms | 0.9975 | baseline |
| n256/f16/tg256 row26k | 128 | 13,270 | 5,252,750 | 48.677 ms | 0.9910 | recall miss |
| n256/f16/tg256 row26k | 192 | 13,270 | 5,252,750 | 67.473 ms | 0.9975 | candidate/latency miss |
| n256/f16/tg256 row52k | 192 | 26,538 | 10,455,918 | 65.836 ms | 0.9975 | row52k not useful |
| n512/f16/tg256 row26k | 128 | 24,698 | 5,147,209 | 39.566 ms | 0.9645 | recall miss |
| n512/f16/tg256 row26k | 256 | 25,116 | 5,231,408 | 58.153 ms | 0.9940 | near miss, latency miss |

Best near-gate row is `n512/f16/tg256/row26k/nprobe256`: recall passes the
`>=0.9925` floor, but candidates are still `31,408` above the `<=5.2M` gate and
p50 is not a 25% improvement or `<=45 ms`.

## Interpretation

The key route-time diagnostic for `n256/f16/tg256/row26k` is now:

| nprobe | selected child sum | deduped route sum | truncation | candidate sum |
| ---: | ---: | ---: | --- | ---: |
| 128 | 51,200 | 13,270 | row_budget | 5,252,750 |
| 192 | 51,200 | 13,270 | row_budget | 5,252,750 |
| 256 | 51,200 | 13,270 | row_budget | 5,252,750 |

That proves the budget is active during route selection rather than only at
post-route placement. The fact that candidate counts match packet 003 means the
remaining blocker is not late filtering anymore. It is the size and quality of
whole-leaf units: once a leaf is selected, the scan still scores every visible
row in that leaf.

Reviewer F2 is also covered: the `row52k` n256 run was surfaced and is not a
useful closing direction. It doubles candidates to about 10.5M without improving
the high-recall latency point.

I did not run TurboQuant in this packet because there is no RaBitQ winner to
guard. Task 79 makes RaBitQ the primary/default lane, and TurboQuant comparison
is useful once a RaBitQ candidate-reduction recipe is close enough to defend.

## Next Slice

This packet narrows Task 79 to a finer-grain candidate-selection problem:

- increase effective leaf granularity further, if build/search costs stay sane;
- or add a leaf-local pruning layer so selected leaves do not imply scoring all
  rows in the leaf;
- or persist child/source row counts only if we need cheaper internal route
  estimates. Persisting counts alone will not reduce scored candidates after
  this slice because leaf route counts are already budgeted at route time.

## Review Focus

Please review:

- whether the route-time row-budget stop condition preserves disabled-by-default
  behavior and current route ordering;
- whether loading leaf assignment counts with the routing hierarchy is the right
  no-format-change answer to packet 003 F1;
- whether the packet's conclusion is correct: Task 79 now needs subleaf or
  finer-partition candidate selection, not another post-route budget pass.

## Validation

- `cargo test collect_recursive_routing_level_diagnostics_reports_row_budget_truncation --no-default-features --features pg18`
- `cargo test max_routed_candidate_rows --no-default-features --features pg18`
- `cargo test caps_routed_candidate_rows --no-default-features --features pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `target/debug/ecaz bench suite audit --config reviews/task-79/004-route-time-row-budget/suite-rabitq-route-time-row-budget.json`
- `target/debug/ecaz bench suite run --dry-run --config reviews/task-79/004-route-time-row-budget/suite-rabitq-route-time-row-budget.json`
- `target/debug/ecaz bench suite run --config reviews/task-79/004-route-time-row-budget/suite-rabitq-route-time-row-budget.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/004-route-time-row-budget/artifacts/suite-manifest.json`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/004-route-time-row-budget/artifacts/suite-manifest.json`
