# Task 43 Review Request: Remote Typed Payload Parser Extraction

## Summary

This packet closes the remote parser gap in the Task 43 campaign tracker by
extracting SPIRE remote typed tuple payload validation out of the
`postgres::Row` decoder and into a Row-independent field-level decoder.

The production libpq row path now gathers raw fields and delegates to the pure
decoder. The new Miri tests exercise:

- valid typed payload decoding with explicit and omitted collations,
- odd-length and invalid hex payload rejection,
- decoded byte cap rejection,
- width mismatches across all typed payload vectors,
- invalid attnums,
- invalid type OID and collation OID text,
- unsupported tuple transport and non-ready transport status,
- unsupported per-column payload format.

This is still not the final Task 43 completion packet. SPIRE delete-delta /
vacuum visibility, mutation probes, aggregate final lanes, and final audit
remain open in the tracker.

## Code Under Review

Code commit: `19834f9ee6b6c7986dcf5ecd5eefc451bff7ab64`

Changed files:

- `src/am/ec_spire/coordinator/remote_candidates/payload_limits.rs`
- `src/am/ec_spire/coordinator/remote_candidates/payload.rs`
- `src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`

## Validation

Artifacts are packet-local under `artifacts/`; see
`artifacts/manifest.md` for commands and key result lines.

- `miri-remote-typed-payload-fields.log`: 2 passed; 0 failed.
- `miri-remote-payload-caps.log`: 1 passed; 0 failed.
- `cargo-fmt-check.log`: exit 0.
- `git-diff-check.log`: exit 0.

Normal `cargo test --lib miri_remote_typed_payload_fields` was attempted as a
compile sanity check, but the pgrx-linked test binary cannot execute outside
PostgreSQL in this workspace because it cannot resolve `LockBuffer`.

## Tracker Update

`reviews/task-43/001-coverage-survey-strategy/artifacts/campaign-tracker.md`
has been updated to mark G4 and the remote parser rows closed, while keeping
the cargo-careful mirror row blocked on the remaining pgrx/Oid dependency.
