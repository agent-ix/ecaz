# Review Request: SPIRE Remote Tuple Payload Side Channel

Code slice for Step 1 of the ADR-067 / ADR-068 CustomScan pivot. This keeps
the existing 18-column `ec_spire_remote_search(...)` envelope unchanged and
adds a sibling tuple-payload endpoint for the CustomScan read path.

## Scope

- Adds `ec_spire_remote_search_tuple_payload(...)`.
- Reuses `ec_spire_remote_search_local_heap_candidate_rows(...)` so payload
  rows are emitted only after the existing origin-node heap visibility path
  succeeds.
- Accepts `requested_columns text[]` at request build time, validates the names
  against the indexed heap relation, rejects empty or duplicate names, and
  returns a JSON payload containing only those columns.
- Keys payload rows with `payload_key = 'node_id_vec_id'` and includes
  `(node_id, vec_id)` plus the existing candidate identity fields so the
  coordinator can attach payloads to the unchanged Stage B envelope.
- Updates the Phase 11 checklist for packet `30807`.

## Validation

- `cargo test tuple_payload --lib`
  - `test tests::pg_test_ec_spire_remote_search_tuple_payload_side_channel ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1605 filtered out`
- `cargo fmt --check`
  - Passed; rustfmt still prints the repository's stable-toolchain warnings for
    nightly-only config keys.
- `git diff --check`
  - Passed.

## Review Focus

- Confirm this is the right first ADR-068 shape: sibling endpoint, stable
  envelope, JSON payload side channel keyed by `(node_id, vec_id)`.
- Check whether returning JSON is acceptable until the CustomScan executor maps
  payloads into PostgreSQL tuple slots, or whether the next slice should move
  directly to a typed internal payload representation.
- Check the requested-column validation and dynamic heap fetch query for any
  missing schema-drift or system-column concerns.

## Artifacts

- `review/30807-spire-remote-tuple-payload/artifacts/manifest.md`
- `review/30807-spire-remote-tuple-payload/artifacts/cargo-test-tuple-payload-lib.log`
- `review/30807-spire-remote-tuple-payload/artifacts/git-diff-check.log`
