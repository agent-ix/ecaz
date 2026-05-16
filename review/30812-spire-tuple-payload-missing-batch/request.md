# Review Request: SPIRE Tuple Payload Missing Signal and Batch Fetch

Follow-up for reviewer feedback on packet `30807`. This keeps the ADR-068
tuple-payload endpoint as the temporary bridge for CustomScan work, but removes
the N+1 heap SPI fetch pattern and makes missing heap rows explicit instead of
silently returning `{}` with a ready status.

## Scope

- Replaces per-candidate tuple-payload heap lookup with one batched CTID query
  over `unnest($1::text[])`.
- Adds `tuple_payload_missing bool` to
  `ec_spire_remote_search_tuple_payload(...)`.
- Emits status `remote_tuple_payload_missing` for payload rows whose heap tuple
  is not visible at the requested CTID.
- Keeps payload JSON key order tied to `requested_columns` ordinality.
- Adds PG18 coverage for the normal side-channel path and for the explicit
  missing-CTID signal.
- Updates the Phase 11 tracker with packet `30812`.

## Validation

- `cargo test tuple_payload --lib`
- `cargo fmt --check`
- `git diff --check HEAD -- src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Check the batched SPI query shape, especially CTID text handling and the
  left-lateral heap join.
- Check the endpoint contract change: missing payloads now have both a boolean
  flag and a distinct status.
- Check the remaining boundary: this JSON tuple-payload endpoint is still an
  endpoint/diagnostic bridge. The production CustomScan path still needs typed
  tuple-slot materialization.

## Artifacts

- `review/30812-spire-tuple-payload-missing-batch/artifacts/manifest.md`
- `review/30812-spire-tuple-payload-missing-batch/artifacts/cargo-test-tuple-payload.log`
- `review/30812-spire-tuple-payload-missing-batch/artifacts/cargo-fmt-check.log`
- `review/30812-spire-tuple-payload-missing-batch/artifacts/git-diff-check.log`
