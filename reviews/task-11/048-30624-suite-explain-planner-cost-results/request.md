# Suite Explain Planner Cost Results

## Scope

This packet teaches `ecaz bench suite report` / results extraction to parse
planner-cost rows from explain artifacts.

Code checkpoint: `9d3e61b8` (`Parse suite explain planner cost results`)

## Changes

- Adds `explain` handling to suite result-row extraction.
- Parses table rows that include `modeled_total_cost` and emits them with
  metric `planner_cost`.
- Adds unit coverage proving planner-cost rows are parsed from psql-style table
  output.
- Marks the Phase 8 benchmark-harness task complete now that SPIRE suite
  explain generation, reusable real10k suite config, packet-local artifact
  expansion, and explain result parsing are in place.

## Files

- `crates/ecaz-cli/src/commands/bench/suite.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo test -p ecaz-cli suite`
- `git diff --check`

## Notes

This parses planner-cost proof rows from future explain artifacts; it does not
run the product-scale measurement packet.
