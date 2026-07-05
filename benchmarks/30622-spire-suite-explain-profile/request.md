# SPIRE Suite Explain Profile

## Scope

This packet makes `ecaz bench suite` explain steps profile-aware so SPIRE
benchmark suites can emit planner proof artifacts without raw SQL workarounds.

Code checkpoint: `a45849f4` (`Make suite explain profile-aware for SPIRE`)

## Changes

- Adds optional `profile` to `explain` suite steps, falling back to suite
  defaults and then `ec_ivf` for backward compatibility.
- Emits the correct scan tuning GUC for the selected profile:
  - `ec_ivf.nprobe`
  - `ec_spire.nprobe`
  - existing profile sweep GUCs where applicable.
- Emits rerank-width GUCs only for profiles that support them:
  - `ec_ivf.rerank_width`
  - `ec_spire.rerank_width`
- Includes the matching planner cost snapshot in generated explain SQL:
  - `ec_hnsw_index_cost_snapshot`
  - `ec_ivf_index_cost_snapshot`
  - `ec_spire_index_cost_snapshot`
- Adds tests for default IVF explain SQL and SPIRE explain SQL.

## Files

- `crates/ecaz-cli/src/commands/bench/suite.rs`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo test -p ecaz-cli explain_sql`
- `git diff --check`

## Notes

This is a harness expansion only. It does not execute a product-scale suite;
that remains the later Phase 8 measurement packet.
