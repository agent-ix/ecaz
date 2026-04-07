# Task 11: Planner Integration

Status: in progress

Progress notes:
- ADR-011 still keeps live planner costing disabled in `amcostestimate`.
- A pure FR-020 cost-model helper now exists in `src/am/cost.rs` with unit coverage for the large-
  table crossover, small-table seqscan preference, empty-index `f64::MAX`, and missing-`reltuples`
  heuristic behavior.
- A read-only `tqhnsw_index_cost_snapshot(...)` SQL surface now exposes modeled FR-020 costs and
  the still-gated live callback contract side by side for planner/admin inspection.
- The cost snapshot now also reports that its current tree-height input comes from a
  `metadata_fallback` seam rather than a live PG18 `amgettreeheight` callback, making the future
  activation boundary explicit without pretending PG18 support already exists.
- The explain snapshot now also exposes the intended PG18 strategy-translation target
  (`strategy 1` / `COMPARE_LT`) while keeping callback readiness explicitly false until the repo
  actually grows PG18 toolchain support.
- The explain snapshot now also exposes the intended custom EXPLAIN option name (`tqvector`) while
  keeping PG18 option registration and per-node hook readiness explicitly false until PG18 support
  actually exists in the repository.
- A read-only `tqhnsw_stats_snapshot()` SQL surface now exposes the intended `tqvector_stats`
  function name while keeping PG18 pgstat-kind and SQL-surface readiness explicitly false until
  PostgreSQL 18 support actually exists in the repository.
- A read-only `tqhnsw_pg18_upgrade_snapshot()` SQL surface now exposes the intended stable
  extension identity (`tqvector`, `$libdir/tqvector`) while keeping `pg18` Cargo-feature,
  default-build, and `PG_MODULE_MAGIC_EXT` readiness explicitly false until the toolchain upgrade
  actually lands.
- A read-only `tqhnsw_pg18_diagnostics_snapshot()` SQL surface now exposes the intended custom
  EXPLAIN option and statistics function names together while keeping all PG18 diagnostics
  readiness flags explicitly false until the toolchain and hook/pgstat lanes actually land.
- A read-only `tqhnsw_planner_integration_snapshot(...)` SQL surface now exposes the current
  cross-lane planner blockers in one place: modeled cost scaffolding is ready, but ordered scan
  credibility, live planner activation, and PG18 callback/diagnostics readiness remain false.

## Scope

Implement planner cost estimation, strategy translation, custom EXPLAIN, and async I/O for the tqhnsw access method. Split into two phases: scaffolding (D1, can start now) and wiring (D2, gated on Task 05 A4 recall gate).

## Subtasks

### D1: Planner Scaffolding (parallel-ready, no gate dependency)

- [x] **Cost model function.** Implement cost computation from metadata (m, ef_search, dimensions, max_level, index_pages, reltuples). Pure function, unit-testable without a running index. Place in `am/cost.rs`.
- [ ] **`amgettreeheight` callback.** Read max_level from metadata page, return as i32. PG18 feature-gated. Place in `am/cost.rs`.
- [ ] **Strategy translation stubs.** `amtranslatestrategy` returns `COMPARE_LT` for strategy 1, `amtranslatecmptype` returns strategy 1 for `COMPARE_LT`. PG18 feature-gated. Place in `am/cost.rs`.
- [ ] **EXPLAIN counter fields.** Add stats fields to `TqScanOpaque` (bootstrap_expansions, pages_read, elements_scored, elements_skipped, heap_tids_returned, quantizer_cache_hit). Place counter struct in `am/explain.rs`.
- [ ] **EXPLAIN hook skeleton.** `RegisterExtensionExplainOption` + `explain_per_node_hook` that reads counters and emits `ExplainProperty*` calls. PG18 feature-gated. Place in `am/explain.rs`.
- [ ] **ReadStream callback signatures.** Graph stream (random, `READ_STREAM_DEFAULT`) and linear stream (sequential, `READ_STREAM_SEQUENTIAL`) callback types. PG18 feature-gated. Place in `am/stream.rs`.
- [x] **Cost model unit tests.** Verify: index selected at 10K rows, seqscan preferred at 50 rows, empty index returns `f64::MAX`, zero reltuples uses heuristic estimate.

### D2: Wire Planner (gated on A4 recall gate)

- [ ] **Activate cost model.** Replace `f64::MAX` in `amcostestimate` with real cost model function from D1.
- [ ] **Wire ReadStream into scan.** Create stream instances in `amrescan`, use in scan loop, destroy in `amendscan`.
- [ ] **Activate EXPLAIN counters.** Increment counters during scan execution in `am/scan.rs`.
- [ ] **Mark ADR-011 superseded.** Update ADR status and add superseded-by reference to FR-020.
- [ ] **Acceptance validation.** FR-020-AC-1 (index scan on 10K table), FR-020-AC-2 (seqscan on 50 table), FR-020-AC-3 (costs use metadata), FR-020-AC-5 (ADR-011 superseded).

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

- Natural planner selection of tqhnsw index (no more `enable_seqscan = off`)
- PG18 observability (EXPLAIN scan stats)
- PG18 performance (async I/O prefetch)

## Deliverables

- `am/cost.rs` — cost model, `amgettreeheight`, strategy translation
- `am/explain.rs` — EXPLAIN hook and counter struct
- `am/stats.rs` — custom statistics scaffolding
- `am/stream.rs` — ReadStream callbacks
- ADR-011 marked superseded

## Primary Tests

- Cost model unit tests (D1)
- FR-020-AC-1: EXPLAIN shows index scan on 10K-row table
- FR-020-AC-2: planner prefers seqscan on 50-row table
- FR-020-AC-3: cost model reads metadata, not hardcoded defaults
- FR-020-AC-4: `amgettreeheight` returns max_level (PG18)
- FR-020-AC-5: ADR-011 superseded

## File Ownership

These files do NOT overlap with the graph search agent's `am/scan.rs` and `am/search.rs`:
- `am/cost.rs` — planner agent owns
- `am/explain.rs` — planner agent owns
- `am/stats.rs` — planner agent owns
- `am/stream.rs` — planner agent owns

Coordination point: D2 wiring touches `am/scan.rs` (counter increments, ReadStream creation). This should happen AFTER the graph search agent completes A3/A4 and is no longer actively modifying scan.

## Notes

- ADR-011 exists specifically to prevent premature planner activation. Do not remove it in D1.
- FR-019 async I/O measured 4x cold-cache improvement in pgvector benchmarks — worth the PG18 gating complexity.
- The cost model formula is fully specified in FR-020. D1 is largely translating the spec into code.
- Strategy translation (FR-023) is ~20 lines of code but enables PG18 optimizer to reason about `<#>` ordering.
