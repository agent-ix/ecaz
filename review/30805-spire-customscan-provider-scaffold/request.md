# Review Request: SPIRE CustomScan Provider Scaffold

First code slice for the ADR-067 CustomScan pivot. This registers the
`EcSpireDistributedScan` provider and installs the planner hook without yet
generating CustomScan paths.

## Scope

- Adds `src/am/ec_spire/custom_scan.rs`.
- Registers `EcSpireDistributedScan` from `_PG_init` via
  `RegisterCustomScanMethods`.
- Installs and chains a `set_rel_pathlist_hook`.
- Adds fail-closed CustomScan executor callbacks. If a CustomScan plan somehow
  reaches execution before the planner path and tuple payload work land, it
  errors instead of silently returning zero rows.
- Adds `ec_spire_custom_scan_status()` so packet fixtures and reviewers can
  verify the provider and hook are installed.
- Updates the Phase 11 task file to mark only the provider-registration subitem
  complete.

## Explicit Non-Scope

- No CustomPath is generated yet.
- No query shape detection for
  `ORDER BY <vector-distance-op> LIMIT k` is implemented yet.
- No path keys, costing, EXPLAIN output, tuple payload decode, or
  `SpireRemoteFanoutExecutor` execution wiring is implemented yet.
- No behavior change to the local-only `ec_spire` index AM path.

## Files

- `src/am/ec_spire/custom_scan.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `review/30805-spire-customscan-provider-scaffold/artifacts/manifest.md`

## Validation

- `cargo test custom_scan --lib`
- `git diff --check`

The focused test command covered:

- Rust unit status shape for the provider scaffold.
- PG18 `pg_test` proof that `ec_spire_custom_scan_status()` reports
  `registered = true`, `rel_pathlist_hook_installed = true`,
  `path_generation_enabled = false`, and `exec_wiring_enabled = false`.

## Reviewer Focus

- Confirm registering the provider and hook in `_PG_init` is acceptable for the
  first slice.
- Confirm the fail-closed executor callback behavior is the right temporary
  safety boundary.
- Confirm the status surface makes the partial state visible enough before the
  planner-path packet lands.
