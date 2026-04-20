# Task 19: PG18 Completion — Flip from Scaffolding to Primary Target

Status: in progress — PG18 callback/EXPLAIN/ReadStream wiring is live on `main`; the shared pgstat-kind path now exists via a preload-only C shim over `pgstat_internal.h`, but PG18 validation in this repo still depends on a managed PG18 install and preload-aware test coverage.

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
- [x] **`PG_MODULE_MAGIC_EXT`.** Module magic now reports tqvector name/version on PG18 while preserving the existing library identity.
- [x] **CI matrix.** PG18 is the primary row; PG17 remains the fallback row.

### IndexAmRoutine callback wiring

- [x] **`amgettreeheight`.** Bound in `IndexAmRoutine` and reflected in readiness snapshots.
- [x] **`amtranslatestrategy` / `amtranslatecmptype`.** Bound in `IndexAmRoutine` and reflected in readiness snapshots.
- [ ] **`amestimateparallelscan` / `aminitparallelscan` /
  `amparallelrescan`.** Wire task-18 callbacks if task 18 has landed;
  otherwise leave `amcanparallel=false` and flip flag in a follow-up.

### EXPLAIN hook activation

- [x] **`RegisterExtensionExplainOption`.** `_PG_init()` registers the `tqvector` EXPLAIN option on PG18.
- [x] **`explain_per_node_hook` registration.** The per-node hook is installed and chained through the previous hook.
- [x] **Counter storage in `TqScanOpaque`.** `TqExplainCounters` is live in the scan opaque and emitted through the PG18 hook.
- [ ] **Acceptance:** `EXPLAIN (ANALYZE, tqvector)` on a ec_hnsw index
  scan emits the `TQVector Stats` group with the documented properties.

### pgstat-kind activation

- [x] **Register custom pgstat-kind.** PG18 now has a preload-aware registration path through a small C shim over `pgstat_internal.h`. Registration succeeds when `tqvector` is loaded through `shared_preload_libraries`; non-preloaded sessions keep the current backend-local fallback.
- [x] **Increment sites.** Shared scan counters now increment at the live scan seams that already feed EXPLAIN.
- [x] **`tqvector_stats()` SQL function.** PG18 now exposes the shared pgstat snapshot when registration is active, and otherwise falls back to backend-local counters so non-preloaded sessions still have a descriptive SQL surface.

### ReadStream activation

- [x] **Graph prefetch stream.** `amrescan`/`amendscan` now own the graph ReadStream, and neighbor expansion consumes prefetched PG18 buffers through the shared tuple-decode seam.
- [x] **Linear prefetch stream.** Linear fallback scan and vacuum tuple counting now use sequential ReadStreams on PG18.
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

- Most of the work is flipping pre-built switches, not new design. Task 11 did the hard part by making the surface pure and testable without a running PG18.
- The remaining blocker is narrower now: shared pgstat registration needs a PG18 environment that actually preloads `tqvector`, and the repo still lacks local PG18 validation on this machine.
- Keep the PG17 fallback working until we have at least 3 months of PG18 CI history. Don't rip the `pg17` Cargo feature prematurely.
