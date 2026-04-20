# Task 11: Planner Integration

Status: substantially complete — the shared PG18 planner/diagnostics/read-stream slice is live on
`pg18-shared-infra-merge`; remaining follow-ons are preload-aware shared-pgstat activation
coverage, measurement, and optional parallel-scan callbacks.

Progress notes:
- Task 19 has now completed the planned PG18 shared-infrastructure landing on
  `pg18-shared-infra-merge`: live `amcostestimate`, PG18 callback registration, EXPLAIN hook
  registration, ReadStream scan/vacuum wiring, preload-aware shared pgstat registration, and
  PG18 module identity are all in place with PG17 fallback preserved.
- Local validation now passes on both supported versions:
  `cargo test`, `cargo pgrx test pg18`, `cargo pgrx test pg17`,
  `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`,
  `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`, and
  `bash scripts/run_pgrx_pg17_test.sh`.
- The original planner scaffolding from `planner-integration-lane` / `planner-part2` remains
  merged on `main`, including the pure FR-020 / FR-023 / FR-024 / FR-025 / FR-019 helpers and
  snapshot surfaces that made the later PG18 binding work narrow.
- The `ef_search` control surface remains fully wired through `resolve_scan_tuning(...)`, including
  reloption-versus-session precedence and explicit `SET ec_hnsw.ef_search = 40` overrides.
- The planner/admin snapshot family now reports live state instead of pure readiness staging:
  planner cost snapshots show the live callback path, diagnostics snapshots distinguish preload-time
  shared-pgstat gating from the otherwise-live PG18 surfaces, and ReadStream snapshots report the
  active PG18 scan/vacuum wiring.
- The remaining gap in this task is not callback wiring. It is follow-on validation of the
  preload-only shared pgstat lane plus later measurement/parallel-scan work that was always outside
  the narrow shared-infrastructure landing.

## Scope

Implement planner cost estimation, strategy translation, custom EXPLAIN, and async I/O for the ec_hnsw access method. Split into two phases: scaffolding (D1, can start now) and wiring (D2, gated on Task 05 A4 recall gate).

## Subtasks

### D1: Planner Scaffolding (parallel-ready, no gate dependency)

- [x] **Cost model function.** Implement cost computation from metadata (m, ef_search, dimensions, max_level, index_pages, reltuples). Pure function, unit-testable without a running index. Place in `am/cost.rs`.
- [x] **Pure callback scaffolding.** `src/am/cost.rs`, `src/am/explain.rs`, and `src/am/stream.rs` now provide the pure callback/value helpers, signatures, counter structs, and gating contracts for the future PG18 bindings without wiring them into runtime execution yet.
- [x] **`amgettreeheight` callback.** Read max_level from metadata page, return as i32, and bind it
  into the PG18 `IndexAmRoutine`.
- [x] **Strategy translation stubs.** `amtranslatestrategy` returns `COMPARE_LT` for strategy 1,
  `amtranslatecmptype` returns strategy 1 for `COMPARE_LT`, and both callbacks are now bound on
  PG18.
- [x] **EXPLAIN counter fields.** `TqScanOpaque` now stores the reusable `TqExplainCounters`
  contract and the runtime scan seams increment the live counters.
- [x] **EXPLAIN hook skeleton.** `RegisterExtensionExplainOption` plus chained
  `explain_per_node_hook` registration are live on PG18.
- [x] **ReadStream callback signatures.** The pure callback/state helpers still exist in
  `am/stream.rs`, and the actual PG18 callback bindings are now live in scan/vacuum execution.
- [x] **Cost model unit tests.** Verify: index selected at 10K rows, seqscan preferred at 50 rows, empty index returns `f64::MAX`, zero reltuples uses heuristic estimate.

### D2: Wire Planner (gated on A4 recall gate)

- [x] **Activate cost model.** `amcostestimate` now uses the real cost model instead of the
  `f64::MAX` override.
- [x] **Wire ReadStream into scan.** Stream instances are created in `amrescan`, used in the scan
  loop, and destroyed in `amendscan`; vacuum tuple counting also uses the sequential stream.
- [x] **Activate EXPLAIN counters.** Scan execution increments the live counters and exposes them
  through the PG18 EXPLAIN hook.
- [x] **Mark ADR-011 superseded.** The planner gate ADR is retired in favor of live costing.
- [x] **Acceptance validation.** FR-020-AC-1, FR-020-AC-2, FR-020-AC-3, FR-020-AC-4, and
  FR-020-AC-5 are covered by the current PG17/PG18 test matrix on this branch.

## Owns

- FR-020 (planner cost estimation)
- FR-023 (strategy translation)
- FR-024 (custom EXPLAIN)
- FR-025 (custom cumulative statistics scaffolding)
- FR-019 (async I/O)

## Dependencies

- D1: None (can start immediately)
- D2: Task 05 / A4 (recall gate must pass)

## Unblocks

- Natural planner selection of ec_hnsw index (no more `enable_seqscan = off`)
- PG18 observability (EXPLAIN scan stats)
- PG18 performance (async I/O prefetch)

## Deliverables

- `am/cost.rs` — cost model, `amgettreeheight`, strategy translation
- `am/explain.rs` — EXPLAIN hook and counter struct
- `am/stats.rs` — custom statistics scaffolding
- `am/stream.rs` — ReadStream callbacks
- ADR-011 marked superseded (D2)

## Primary Tests

- Cost model unit tests (D1)
- FR-020-AC-1: EXPLAIN shows index scan on 10K-row table
- FR-020-AC-2: planner prefers seqscan on 50-row table
- FR-020-AC-3: cost model reads metadata, not hardcoded defaults
- FR-020-AC-4: `amgettreeheight` returns max_level (PG18)
- FR-020-AC-5: ADR-011 superseded

## File Ownership

These files do NOT overlap with the graph search agent's `am/scan.rs` and `am/search.rs`:
- `am/cost.rs` — planner agent owned, merged to `main`
- `am/explain.rs` — planner agent owned, merged to `main`
- `am/stats.rs` — planner agent owned, merged to `main`
- `am/stream.rs` — planner agent owned, merged to `main`
- `lib.rs` planner snapshot entry points — merged to `main` and later consolidated post-merge

Coordination point: D2 wiring touches `am/scan.rs` (counter increments, ReadStream creation). This should happen AFTER the graph search agent completes A3/A4 and is no longer actively modifying scan.

## Notes

- ADR-011 exists specifically to prevent premature planner activation. Do not remove it in D1.
- FR-019 async I/O measured 4x cold-cache improvement in pgvector benchmarks — worth the PG18 gating complexity.
- The cost model formula is fully specified in FR-020. D1 is largely translating the spec into code.
- Strategy translation (FR-023) is ~20 lines of code but enables PG18 optimizer to reason about `<#>` ordering.
- Task-15 merge follow-up owner: Agent 2 (Planner Integration track).
- Task-15 merge follow-up target: April 24, 2026 to write the shared-table planner investigation note for sibling `m=8` / `m=16` index cross-choosing and decide whether it is a costing bug, a statistics gap, or expected planner behavior.
