# Review Request: SPIRE Pipeline Remote Readiness Counters

Code checkpoint: `fbd8582241f8ee0edddb8ae0e453b303705a58ee` (`Extend SPIRE pipeline remote readiness counters`)

## Scope

- Extends `ecaz bench spire-pipeline` to read
  `ec_spire_remote_search_endpoint_identity(...)` once per benchmark run and
  render tuple transport capability/default/status plus a
  `pg_binary_attr_v1_ready` summary.
- When remote diagnostics are enabled, also calls
  `ec_spire_remote_search_degraded_skip_report(...)` for each sampled query and
  aggregates degraded skip reports by `(nprobe, node_id)`.
- Reports degraded skip query count, requested epoch stability, skipped PID sum,
  first skip category, and status.
- Updates the Phase 12.9 tracker note so the remaining open artifact-capture row
  reflects the new endpoint tuple-transport and degraded-skip counters.

## Validation

- `git diff --check fbd85822^ fbd85822`
- `cargo test -p ecaz-cli spire_pipeline`
- `cargo check --no-default-features --features pg18`

Packet-local logs are under `artifacts/`; see `artifacts/manifest.md` for
commands and result lines.

## Review Focus

- Confirm the CLI report shape is sufficient for the Phase 12.9 remote
  readiness portion of the production-readiness bundle.
- Confirm degraded-skip aggregation by `(nprobe, node_id)` is the right level
  for packet-local benchmark evidence.
- Confirm it is acceptable that this slice is non-live CLI/report wiring; live
  fixture artifact capture remains for the final production-readiness bundle.
