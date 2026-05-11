# SPIRE INSERT V1 Feedback Followups

## Scope

This packet addresses focused reviewer feedback from the coordinator INSERT
front-door and descriptor-refresh packets without changing the distributed
INSERT execution path.

Changes:

- Adds PG18 coverage proving the INSERT trigger's `to_jsonb(NEW)` payload shape
  preserves exact `bytea` source-identity bytes when decoded through
  `jsonb_populate_record(...)`, matching the remote tuple-payload endpoint's
  projection model.
- Documents in ADR-069 that the v1 transparent INSERT trigger supports only the
  bigint primary-key / `ecvector` embedding / exact-16-byte `bytea`
  source-identity shape, with UUID primary-key support deferred.
- Documents in ADR-069 that concurrent INSERTs racing to refresh the same
  remote descriptor can fail closed if a newer descriptor generation wins first;
  v1 callers should retry, and Stage F can replace this with a compatibility
  check or per-node serialization.
- Updates the Phase 11 tracker with packet `30844`.

## Validation

- `cargo test insert_trigger_source_identity_json_roundtrip --lib`
  - result: pass.
  - key line:
    `test tests::pg_test_ec_spire_insert_trigger_source_identity_json_roundtrip_sql ... ok`
  - summary: `1 passed; 0 failed; 1646 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the JSON roundtrip test covers the bytea source-identity concern from
  the trigger feedback.
- Confirm ADR-069's v1 type-scope note is precise enough for operators.
- Confirm the descriptor-generation race note is acceptable v1 documentation
  while higher-concurrency INSERT behavior remains a Stage F concern.

## Artifacts

- `review/30844-spire-insert-v1-feedback-followups/artifacts/manifest.md`
- `review/30844-spire-insert-v1-feedback-followups/artifacts/cargo-test-insert-trigger-source-identity-json-roundtrip-lib.log`
- `review/30844-spire-insert-v1-feedback-followups/artifacts/cargo-fmt-check.log`
- `review/30844-spire-insert-v1-feedback-followups/artifacts/git-diff-check.log`
