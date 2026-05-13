# Review Request: SPIRE CustomScan Typed Receive

## Summary

Closes two Phase 12.2 rows:

> Add a production endpoint negotiation bit so coordinators can prefer typed
> tuple payloads only when the remote advertises support.

> Wire typed tuple payload receive into `EcSpireDistributedScan` slot
> materialization instead of going through JSON/text coercion.

This slice makes the CustomScan production path prefer endpoint-advertised
`pg_binary_attr_v1` tuple payloads. The coordinator still keeps the existing
JSON payload path as a migration fallback, but remote-origin CustomScan rows
with typed payload metadata now materialize virtual tuple slots through
PostgreSQL binary receive functions.

The PG18 fixture now includes `text[]`, a domain, and a named composite in the
remote row projection. The successful proof shows:

- the plan uses `Custom Scan (EcSpireDistributedScan)`;
- the remote row round-trips array, domain text, and composite text output; and
- the endpoint reports `typed_payload_probe=ready,pg_binary_attr_v1,t,t`.

This packet does not claim tuple-heavy throughput measurement or removal of the
production JSON endpoint. Those Phase 12.2 rows remain open.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/custom_scan.rs`
- `src/am/ec_spire/root/tests.rs`
- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/ec_spire/scan/tests/runtime_state.rs`
- `scripts/run_spire_multicluster_customscan_read_pg18.sh`
- `plan/tasks/task30-phase12-spire-production-hardening.md`
- `review/30963-spire-customscan-typed-receive/artifacts/manifest.md`

## Validation

Packet-local logs are in `artifacts/` and indexed by
`artifacts/manifest.md`.

- `cargo fmt --package ecaz`
- `cargo check --no-default-features --features pg18`
- `bash -n scripts/run_spire_multicluster_customscan_read_pg18.sh`
- `git diff --check -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/root/types.rs src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/tests.rs src/am/ec_spire/root/hierarchy_snapshots.rs src/am/ec_spire/scan/tests/runtime_state.rs scripts/run_spire_multicluster_customscan_read_pg18.sh plan/tasks/task30-phase12-spire-production-hardening.md`
- `bash scripts/run_spire_multicluster_customscan_read_pg18.sh --skip-install --artifact-dir /home/peter/dev/ecaz/review/30963-spire-customscan-typed-receive/artifacts/customscan-read-typed-success --run-dir /home/peter/dev/ecaz/target/se30963-customscan-read-typed-final`

The earlier `customscan-read-typed*` artifact directories document failed
debug attempts and are intentionally not cited as success evidence. They show
the intermediate remote heap resolution and typed collation decode failures
that led to the final typed receive fix.

## Reviewer Focus

- Confirm the endpoint negotiation prefers `pg_binary_attr_v1` only when the
  remote identity reports it ready, and JSON remains only as a compatibility
  fallback.
- Confirm CustomScan slot materialization uses per-attribute binary receive for
  typed payloads, including the array/domain/composite fixture columns.
- Confirm the tracker closure scope excludes throughput measurement and
  production JSON endpoint retirement.
- Confirm the failed debug artifacts are documented but not used as passing
  evidence.
