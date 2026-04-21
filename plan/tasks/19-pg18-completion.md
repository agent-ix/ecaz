# Task 19: PG18 Completion — Flip from Scaffolding to Primary Target

Status: in progress — the shared PG18 infrastructure slice is now wired and validated on `main`; PG17 fallback is preserved, preload-aware shared-pgstat activation coverage now has a dedicated repo lane, and the remaining follow-ons are optional parallel-scan callbacks and post-merge measurement.

Executes ADR-016 (PG18 primary target) and ADR-017 (module identity).

## Scope

Flip Ecaz from "PG18-ready, PG17-running" to "PG18-primary, PG17-fallback"
once PG18 GA is tagged and pgrx supports it. Activate the PG18 callback and
diagnostic scaffolding that task 11 already built but left gated under
`readiness=false` snapshots.

This shared-infrastructure slice now has:
- PG18 as the default CI/build target
- PG17 preserved as the compatibility fallback
- one stable `ecaz` extension identity across the upgrade

## Context

Task 11 deliberately built the PG18 surface pure, unbound, and gated. The current staged completion work has now flipped the non-blocked infrastructure live:

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

- [x] **`pg18` Cargo feature.** `pg18` is the default Cargo feature and PG14–PG16 are dropped.
- [x] **`PG_MODULE_MAGIC_EXT`.** Module magic now reports ecaz name/version on PG18 while preserving the existing library identity.
- [x] **CI matrix.** PG18 is the primary row; PG17 remains the fallback row.

### IndexAmRoutine callback wiring

- [x] **`amgettreeheight`.** Bound in `IndexAmRoutine` and reflected in readiness snapshots.
- [x] **`amtranslatestrategy` / `amtranslatecmptype`.** Bound in `IndexAmRoutine` and reflected in readiness snapshots.
- [ ] **`amestimateparallelscan` / `aminitparallelscan` /
  `amparallelrescan`.** Wire task-18 callbacks if task 18 has landed;
  otherwise leave `amcanparallel=false` and flip flag in a follow-up.

### EXPLAIN hook activation

- [x] **`RegisterExtensionExplainOption`.** `_PG_init()` registers the `ecaz` EXPLAIN option on PG18.
- [x] **`explain_per_node_hook` registration.** The per-node hook is installed and chained through the previous hook.
- [x] **Counter storage in `TqScanOpaque`.** `TqExplainCounters` is live in the scan opaque and emitted through the PG18 hook.
- [x] **Acceptance.** `EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)` on an `ec_hnsw`
  index scan emits the `Ecaz Stats` group with the documented properties. Text output still
  exposes the same properties even though core PostgreSQL does not render the group wrapper there.

### pgstat-kind activation

- [x] **Register custom pgstat-kind.** PG18 now has a preload-aware registration path through a small C shim over `pgstat_internal.h`. Registration succeeds when `ecaz` is loaded through `shared_preload_libraries`; non-preloaded sessions keep the current backend-local fallback.
- [x] **Increment sites.** Shared scan counters now increment at the live scan seams that already feed EXPLAIN.
- [x] **`ecaz_stats()` SQL function.** PG18 now exposes the shared pgstat snapshot when registration is active, and otherwise falls back to backend-local counters so non-preloaded sessions still have a descriptive SQL surface.
- [x] **Preload-aware validation coverage.** `ecaz dev test pg18-preload-pgstat` now starts a repo-local PG18 cluster with `shared_preload_libraries = 'ecaz'`, verifies the planner snapshot clears the PG18 blocker, and checks that `ecaz_stats()` exposes scan deltas across backend boundaries.

### ReadStream activation

- [x] **Graph prefetch stream.** `amrescan`/`amendscan` now own the graph ReadStream, and neighbor expansion consumes prefetched PG18 buffers through the shared tuple-decode seam.
- [x] **Linear prefetch stream.** Linear fallback scan and vacuum tuple counting now use sequential ReadStreams on PG18.
- [ ] **Measurement.** Confirm the 4x cold-cache improvement cited in
  FR-019 on the 50k warm real seam at cold start.

### ADR updates

- [x] **ADR-016 → DECIDED.** The repo now treats PG18 as the primary target while preserving PG17
  fallback.
- [x] **ADR-011 → SUPERSEDED.** Live costing is active; the old `f64::MAX` planner override is no
  longer the staged blocker.

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
- `EXPLAIN (ANALYZE, ecaz)` visibility for operators.
- Async I/O cold-cache performance.
- Dropping the PG14–PG16 compatibility burden from the build matrix.

## Out of scope

- Any new feature work that doesn't come from the PG18 surface. This task
  is infrastructure, not a feature vehicle.
- pg_upgrade testing for other extensions. ADR-047 only covers the Ecaz
  extension identity, while the `tqvector` datum remains the TurboQuant family surface.

## Notes

- Most of the work was flipping pre-built switches, not inventing new design. Task 11 did the
  hard part by making the surface pure and testable before the live PG18 binding work landed.
- Local PG18 validation now covers both the ordinary fallback lane and the preload-aware shared
  pgstat lane. The remaining PG18 follow-ons are measurement plus the still-optional parallel-scan
  callback work.
- Keep the PG17 fallback working until we have at least 3 months of PG18 CI history. Don't rip the `pg17` Cargo feature prematurely.
