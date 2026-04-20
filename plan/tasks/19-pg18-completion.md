# Task 19: PG18 Completion — Flip from Scaffolding to Primary Target

Status: proposed — gated on PG18 GA and pgrx PG18 support landing.

Executes ADR-016 (PG18 primary target) and ADR-017 (module identity).

## Scope

Flip tqvector from "PG18-ready, PG17-running" to "PG18-primary, PG17-fallback"
once PG18 GA is tagged and pgrx supports it. Activate the PG18 callback and
diagnostic scaffolding that task 11 already built but left gated under
`readiness=false` snapshots.

When this task lands: tqvector's default CI matrix runs PG18; PG14–PG16 are
dropped; single `tqvector` extension identity is preserved across the
upgrade.

## Context

Task 11 deliberately built the PG18 surface pure, unbound, and gated:

- `amgettreeheight_callback_value(...)` exists as pure code.
- `amtranslatestrategy_callback(...)` / `amtranslatecmptype_callback(...)`
  exist with unit coverage.
- `RegisterExtensionExplainOption` hook plumbing lives in `src/am/explain.rs`
  as pure emission helpers plus a gated output contract.
- `TqExplainCounters` and `TqStatsCounters` structs exist.
- `GraphPrefetchState` / `LinearPrefetchState` exist in `src/am/stream.rs`
  with pure callback functions.
- Read-only snapshots (`ec_hnsw_pg18_upgrade_snapshot()`,
  `ec_hnsw_pg18_diagnostics_snapshot()`,
  `ec_hnsw_read_stream_snapshot()`) expose readiness flags — all currently
  `false`.

This task flips those flags to `true` by wiring the pure helpers into the
actual `IndexAmRoutine` / hook / pgstat surface.

## Subtasks

### Cargo / module identity

- [ ] **`pg18` Cargo feature.** Add to `Cargo.toml` as the default feature
  once pgrx PG18 support lands.
- [ ] **`PG_MODULE_MAGIC_EXT`.** Replace existing module magic macro per
  ADR-017. Keep `tqvector` / `$libdir/tqvector` identity unchanged.
- [ ] **CI matrix.** Add PG18 as primary; drop PG14–PG16 rows per ADR-016.
  Keep PG17 as the fallback row.

### IndexAmRoutine callback wiring

- [ ] **`amgettreeheight`.** Bind the pure helper in `src/am/cost.rs` to
  the `IndexAmRoutine` slot. Flip `ec_hnsw_pg18_upgrade_snapshot` readiness
  for this callback to true.
- [ ] **`amtranslatestrategy` / `amtranslatecmptype`.** Same — bind pure
  helpers to routine slots. Flip readiness.
- [ ] **`amestimateparallelscan` / `aminitparallelscan` /
  `amparallelrescan`.** Wire task-18 callbacks if task 18 has landed;
  otherwise leave `amcanparallel=false` and flip flag in a follow-up.

### EXPLAIN hook activation

- [ ] **`RegisterExtensionExplainOption`.** Register the `tqvector` custom
  EXPLAIN option at module init.
- [ ] **`explain_per_node_hook` registration.** Install the per-node hook
  using the emission gate from `src/am/explain.rs` (option present +
  `IndexScan` node + `ec_hnsw` access method).
- [ ] **Counter storage in `TqScanOpaque`.** Embed `TqExplainCounters` in
  the scan opaque struct; increment at the existing scan seam points.
  This is the first `src/am/scan.rs` edit this task requires.
- [ ] **Acceptance:** `EXPLAIN (ANALYZE, tqvector)` on a ec_hnsw index
  scan emits the `TQVector Stats` group with the documented properties.

### pgstat-kind activation

- [ ] **Register custom pgstat-kind.** Use the name surface already in
  `ec_hnsw_stats_snapshot()` / `src/am/stats.rs`.
- [ ] **Increment sites.** Wire `TqStatsCounters` into scan and build
  paths at the same seams as the EXPLAIN counters.
- [ ] **`tqvector_stats()` SQL function.** Bind to the pure summary
  helpers in `src/am/stats.rs`. Flip the snapshot readiness flag.

### ReadStream activation

- [ ] **Graph prefetch stream.** Create a `ReadStream` instance in
  `amrescan` using the graph callback from `src/am/stream.rs`; destroy
  in `amendscan`. Reset on `amrescan`.
- [ ] **Linear prefetch stream.** Same for the linear scan path (build,
  vacuum sequential reads).
- [ ] **Measurement.** Confirm the 4x cold-cache improvement cited in
  FR-019 on the 50k warm real seam at cold start.

### ADR updates

- [ ] **ADR-016 → DECIDED.** Update status once PG18 CI is green.
- [ ] **ADR-011 → SUPERSEDED.** Already planned in task 11 D2, but
  explicitly gated on live costing which in turn depends on PG18
  readiness. Close out here if task 11 D2 hasn't already.

## Owns

- ADR-016 execution (primary-target flip)
- ADR-017 execution (single-identity upgrade)
- FR-019 (async I/O) activation
- FR-024 (custom EXPLAIN) activation
- FR-025 (pgstat-kind) activation

## Dependencies

- **PG18 GA.** External timing. PG18 beta already landed; GA is expected
  mid-2026.
- **pgrx PG18 support.** External dependency on the pgrx project.
- Task 11 D2 planner wiring. Much of this task is flipping gates that
  task 11 D1 already prepared.
- Task 18 (parallel scan) is optional but composes cleanly — wire its
  callbacks here if it has merged.

## Unblocks

- Natural planner selection of ec_hnsw on PG18.
- `EXPLAIN (ANALYZE, tqvector)` visibility for operators.
- Async I/O cold-cache performance.
- Dropping the PG14–PG16 compatibility burden from the build matrix.

## Out of scope

- Any new feature work that doesn't come from the PG18 surface. This task
  is infrastructure, not a feature vehicle.
- pg_upgrade testing for other extensions. ADR-017 only covers tqvector
  identity.

## Notes

- Most of the work is flipping pre-built switches, not new design. Task
  11 did the hard part by making the surface pure and testable without a
  running PG18.
- Watch for pgrx API churn around `IndexAmRoutine` — PG18 adds/moves
  several fields; most will be additive but verify before landing.
- Keep the PG17 fallback working until we have at least 3 months of
  PG18 CI history. Don't rip the `pg17` Cargo feature prematurely.
