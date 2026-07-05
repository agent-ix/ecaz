# Review Request: Task 142 Packet 002

## Scope

This packet requests review for commit `ddf73ba1ae94c3832d1baca180fd6fe86731b429`
(`Instrument SPIRE planner routing load profile`) on branch
`task-142-spire-epoch-cache-overhead`.

The slice completes the remaining Phase 0 instrumentation wiring needed before
release EXPLAIN staircase measurements:

- Adds a backend-local SPIRE planner callback profile with SQL accessors:
  `ec_spire_reset_cost_callback_profile()` and
  `ec_spire_cost_callback_profile()`.
- Times `ec_spire_amcostestimate`, its active-snapshot walk, its hierarchy
  snapshot walk, and the PG18 `amgettreeheight` callback.
- Makes `ecaz bench suite` SPIRE `explain` steps reset the profile immediately
  before `EXPLAIN` and emit the callback profile immediately after it.
- Exposes production-read `manifest_load`, `leaf_count`, `route_select`,
  `local_heap`, and `candidate_decode` metrics through the SQL profile rows.
- Adds production-read counts for coordinator routing hierarchy loads and
  top-graph loads, and renders them in the CLI profile table.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo test -p ecaz-cli explain_sql_uses_spire_profile_gucs_and_cost_snapshot -- --nocapture`
- `cargo test -p ecaz-cli spire_pipeline_renders_production_read_profile -- --nocapture`
- `cargo test --lib production_read_profile_row_preserves_metric_rollup -- --nocapture`

All three focused validations passed.

## Notes

No release benchmark matrix is claimed in this packet. This is the instrumentation
slice that makes the next Task 142 release EXPLAIN/suite measurements observable
without side scripts.
