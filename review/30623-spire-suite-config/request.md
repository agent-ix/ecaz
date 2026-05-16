# SPIRE Real10k Benchmark Suite Config

## Scope

This packet adds a reusable `ecaz bench suite` config for the Phase 8 SPIRE
real10k harness and tightens suite validation around profile names.

Code checkpoint: `7d7bd80c` (`Add SPIRE real10k benchmark suite config`)

## Changes

- Adds `crates/ecaz-cli/suites/task30-spire-real10k.json`.
- The suite expands packet-local artifact paths under
  `review/30623-spire-suite-config/artifacts/`.
- The suite covers:
  - `ecaz corpus load --profile ec_spire`
  - `ecaz bench storage`
  - `ecaz dev sql` planner explain with `ec_spire_index_cost_snapshot(...)`
  - `ecaz bench latency --profile ec_spire`
  - `ecaz bench recall --profile ec_spire`
- Adds early suite validation for default and per-step profile names so typos
  fail during `bench suite run` / `audit` instead of silently expanding to a
  fallback profile.

## Artifacts

- `artifacts/suite-manifest.json` — dry-run expansion of the SPIRE suite.
- `artifacts/manifest.md` — packet-local artifact metadata.

## Files

- `crates/ecaz-cli/src/commands/bench/suite.rs`
- `crates/ecaz-cli/suites/task30-spire-real10k.json`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo test -p ecaz-cli suite`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --dry-run --manifest-output review/30623-spire-suite-config/artifacts/suite-manifest.json`
- `git diff --check`

## Notes

This is a dry-run/config harness packet. It does not run a product-scale
measurement or make latency/recall claims.
