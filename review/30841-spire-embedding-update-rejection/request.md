# SPIRE Embedding UPDATE Rejection

## Scope

This packet wires the ADR-069 embedding-changing UPDATE rejection into the
shared coordinator UPDATE primitive. Transparent `UPDATE ... WHERE pk = ...`
front-door integration remains open, but the operation it will call now
rejects indexed embedding-column updates before placement lookup or remote
dispatch.

Changes:

- Adds index-key column detection for `ec_spire` coordinator UPDATE forwarding.
- Rejects any update to the indexed key column in
  `ec_spire_forward_coordinator_update_tuple_payload(...)`.
- Raises the ADR-069 error message with a PostgreSQL hint:
  - error: `ec_spire_distributed: UPDATE of indexed embedding column is not supported on a distributed ec_spire table. Use DELETE + INSERT.`
  - hint: `Cross-shard atomic moves will be available in a future release.`
- Adds PG18 coverage that catches the pgrx `ErrorReport` and asserts both the
  message and hint.
- Updates ADR-069 and the Phase 11 tracker.

## Validation

- `cargo test update_rejects_embedding --lib`
  - result: pass.
  - key line:
    `test tests::pg_test_ec_spire_update_rejects_embedding_column_sql ... ok`
  - summary: `1 passed; 0 failed; 1643 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the rejection belongs at the shared coordinator UPDATE primitive
  boundary for this slice, with transparent UPDATE front-door wiring still
  separate.
- Confirm using the `ec_spire` index key column as the embedding-column detector
  matches the v1 table shape.
- Confirm the error message and hint match ADR-069 exactly.

## Artifacts

- `review/30841-spire-embedding-update-rejection/artifacts/manifest.md`
- `review/30841-spire-embedding-update-rejection/artifacts/cargo-test-update-rejects-embedding-lib.log`
- `review/30841-spire-embedding-update-rejection/artifacts/cargo-fmt-check.log`
- `review/30841-spire-embedding-update-rejection/artifacts/git-diff-check.log`
