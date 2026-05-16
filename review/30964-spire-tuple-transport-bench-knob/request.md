# Review Request: SPIRE Tuple Transport Benchmark Knob

## Summary

Groundwork for the Phase 12.2 throughput row:

> Measure tuple-heavy read throughput before and after typed transport.

This slice adds a session-level transport selector:

- `ec_spire.remote_tuple_transport = auto`
- `ec_spire.remote_tuple_transport = json_tuple_payload_v1`
- `ec_spire.remote_tuple_transport = pg_binary_attr_v1`

`auto` preserves current production behavior: the coordinator uses typed
payloads only when the remote endpoint advertises ready
`pg_binary_attr_v1` as its default. The JSON override forces the compatibility
path for before/after measurements. The typed override still requires the
remote endpoint to advertise ready `pg_binary_attr_v1`, so it cannot bypass the
capability gate.

The slice also adds `ecaz bench spire-pipeline --remote-tuple-transport` and
prints the selected mode in the benchmark header, so future tuple-heavy
measurement packets can run JSON-vs-typed comparisons through the CLI instead
of editing fixture scripts.

This packet does not claim a throughput result and does not close the parent
measurement row.

## Files

- `src/am/ec_spire/options.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `crates/ecaz-cli/src/commands/bench/spire_pipeline.rs`
- `crates/ecaz-cli/src/commands/bench/mod.rs`
- `crates/ecaz-cli/src/cli.rs`
- `crates/ecaz-cli/README.md`
- `plan/tasks/task30-phase12-spire-production-hardening.md`
- `review/30964-spire-tuple-transport-bench-knob/artifacts/manifest.md`

## Validation

Packet-local logs are in `artifacts/` and indexed by
`artifacts/manifest.md`.

- `cargo test -p ecaz --no-default-features --features pg18 remote_tuple_transport --lib`
- `cargo test -p ecaz-cli spire_pipeline`
- `cargo check --no-default-features --features pg18`
- `git diff --check 0aa62152^ 0aa62152`

No live PostgreSQL fixture was run for this slice; the new behavior is covered
by pure transport-selection tests and CLI parser/report tests.

## Reviewer Focus

- Confirm `json_tuple_payload_v1` forces the compatibility path and
  `pg_binary_attr_v1` still requires endpoint readiness plus capability
  advertisement.
- Confirm `auto` preserves the existing endpoint-default behavior.
- Confirm the CLI option records the selected transport mode in packet-local
  benchmark output.
- Confirm the tracker update is scoped as measurement groundwork, not a
  completed throughput measurement.
