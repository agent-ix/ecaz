# Review Request: Row-Budgeted Routing

## Scope

This packet implements Task 79 Phase 3 row-budgeted routing for the RaBitQ
primary lane.

The code adds `ec_spire.max_routed_candidate_rows` and wires it through SPIRE
scan planning, diagnostics, `ecaz bench spire-pipeline`, and `ecaz bench suite`.
The current implementation applies the row budget after ordered leaf routing and
before leaf payload reads/prefetch.

## Evidence

- Task definition: `plan/tasks/79-spire-candidate-surface-reduction.md`
- Suite config: `reviews/task-79/003-row-budgeted-routing/suite-rabitq-row-budget.json`
- Artifact manifest: `reviews/task-79/003-row-budgeted-routing/artifacts/manifest.md`
- Suite status: `reviews/task-79/003-row-budgeted-routing/artifacts/suite-status.log`
- Parsed report: `reviews/task-79/003-row-budgeted-routing/artifacts/suite-report.md`
- Normalized rows: `reviews/task-79/003-row-budgeted-routing/artifacts/report-results.jsonl`
- Focused tests: `reviews/task-79/003-row-budgeted-routing/artifacts/test-*.log`

## Result

The row budget directly reduces the selected/scored candidate surface, but this
post-route placement point does not yet satisfy the full Task 79 gate.

| config | nprobe | candidates | p50 | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| baseline n128/f8/tg96 | 96 | 15,506,227 | 61.234 ms | 0.9975 |
| n256/f16/row26k | 128 | 5,252,750 | 47.380 ms | 0.9910 |
| n256/f16/row26k | 192 | 5,252,750 | 65.288 ms | 0.9975 |
| n256/f16/row36k | 192 | 7,250,965 | 67.415 ms | 0.9975 |
| n512/f16/row26k | 128 | 5,147,209 | 39.216 ms | 0.9645 |
| n512/f16/row26k | 256 | 5,231,408 | 59.326 ms | 0.9940 |

The best candidate-gate row is `n512/f16/row26k/nprobe128`, but recall is far
short. The closest high-recall/candidate row is `n512/f16/row26k/nprobe256`,
but it is slightly above the `<=5.2M` candidate gate and does not improve p50
enough.

## Interpretation

The critical diagnostic is that high-nprobe rows still generate the full route
frontier before the row cap is applied:

| config | nprobe | routing route_sum | placed route_sum | candidates |
| --- | ---: | ---: | ---: | ---: |
| n256/f16/row26k | 128 | 25,600 | 13,270 | 5,252,750 |
| n256/f16/row26k | 192 | 38,400 | 13,270 | 5,252,750 |
| n256/f16/row26k | 256 | 51,200 | 13,270 | 5,252,750 |
| n512/f16/row26k | 128 | 25,600 | 24,698 | 5,147,209 |
| n512/f16/row26k | 256 | 51,200 | 25,116 | 5,231,408 |

So this slice proves the candidate-surface lever is real, but also proves that
the row budget must move earlier into route generation/top-graph expansion to
help latency at matched high recall.

## Review Focus

Please review:

- whether the `ec_spire.max_routed_candidate_rows` contract is safe as a
  disabled-by-default session GUC;
- whether applying the cap before prefetch/leaf payload reads is correct for
  current SPIRE ownership and routing semantics;
- whether the diagnostics and suite plumbing are sufficient for the next slice.

The proposed next implementation slice is to make row budget a first-class
route-generation stop condition, rather than a post-route filter.

## Validation

- `cargo test max_routed_candidate_rows --no-default-features --features pg18`
- `cargo test caps_routed_candidate_rows --no-default-features --features pg18`
- `cargo test -p ecaz-cli expands_spire_pipeline_with_production_profile`
- `target/debug/ecaz ... bench suite audit --config reviews/task-79/003-row-budgeted-routing/suite-rabitq-row-budget.json`
- `target/debug/ecaz ... bench suite run --dry-run --config reviews/task-79/003-row-budgeted-routing/suite-rabitq-row-budget.json`
- `target/debug/ecaz ... bench suite run --config reviews/task-79/003-row-budgeted-routing/suite-rabitq-row-budget.json`
- `target/debug/ecaz ... bench suite status --manifest reviews/task-79/003-row-budgeted-routing/artifacts/suite-manifest.json`
- `target/debug/ecaz ... bench suite report --manifest reviews/task-79/003-row-budgeted-routing/artifacts/suite-manifest.json`
