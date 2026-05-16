# Review Request: SPIRE Remote Payload Caps

## Scope

Please review commit `54d286584e391952fa197e5f046d911ef906ecdf`.

This addresses Phase 12a.2 from
`plan/tasks/task30-phase12a-spire-readiness-followups.md`: bound
coordinator-side remote payload acceptance before Phase 13 cloud verification.

## Changes

- Adds session GUCs:
  - `ec_spire.max_remote_payload_bytes_per_row`, default `1024`.
  - `ec_spire.max_remote_payload_rows_per_batch`, default `64`.
- Adds `remote_payload_too_large` as a named production failure category with
  an operator hint in degraded skip reports.
- Applies the row cap before typed payload hex bytes are decoded into
  per-attribute byte vectors.
- Applies the batch cap to selected PID vectors and received candidate/heap
  result row batches before accepting them into coordinator merge state.
- Documents the defaults in `docs/SPIRE_LOCAL_CAPACITY_TARGETS.md` and the
  operator response in `docs/SPIRE_LIBPQ_RUNBOOK.md`.
- Marks Phase 12a.2 complete in the task tracker.

## Default Rationale

Packet `30975` measured scalar tuple payload projection at 31,510 bytes across
200 rows, or about 158 bytes per row. The row cap default is 1024 bytes, which
rounds the requested 4x safety margin up to 1 KiB. The batch cap default is 64,
matching the Phase 12 local capacity target for selected PIDs per remote node.

Source evidence remains in:

- `review/30975-spire-tuple-transport-measurement/artifacts/manifest.md`
- `review/30975-spire-tuple-transport-measurement/request.md`

This packet's evidence summary is in
`review/30985-spire-remote-payload-caps/artifacts/manifest.md`.

## Validation

```sh
cargo test remote_payload --lib
cargo test tuple_transport --lib
cargo test production_fault_matrix_covers_required_categories --lib
cargo fmt --check
git diff --check
```

`cargo fmt --check` reports the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` / `group_imports`, then exits successfully.

## Review Ask

Confirm the caps fail closed with the named category in strict mode, preserve a
degraded-mode operator hint, and are documented with an honest local-capacity
default rationale.
