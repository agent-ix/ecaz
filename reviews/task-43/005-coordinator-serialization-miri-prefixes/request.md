# Review Request: Task 43 Completion Miri And Careful Coverage

## Summary

This checkpoint completes Task 43's remaining coverage expansion:

- Promotes eight additional pure tests into the `miri_` prefix across DiskANN
  tuple serialization, SPIRE top-graph serialization, SPIRE routing/adaptive
  nprobe, SPIRE remote coordinator state, remote payload caps, and prepared
  transaction state transitions.
- Expands `hardening/careful` from a small storage-only harness into a 67-test
  pure harness covering storage page, DiskANN tuple/vacuum/Vamana graph, and
  HNSW search modules.
- Corrects the Miri many-seeds lane to use range syntax,
  `-Zmiri-many-seeds=0..128`, and documents that override format.
- Adds packet-local completion evidence for `make careful`, `make miri-tree`,
  and `make miri-many-seeds`.

## Review Focus

- Confirm the promoted tests are pure enough to remain permanent Miri prefix
  members.
- Confirm the careful harness stays scoped to path-liftable pure modules and
  does not imply PostgreSQL callback coverage.
- Confirm the completion audit maps Task 43's exit criteria to durable
  packet-local evidence.

## Validation

Validation artifacts are in `artifacts/` and summarized by
`artifacts/manifest.md`.

- All eight targeted newly promoted `miri_` tests passed under
  `cargo +nightly miri test --lib`.
- `cargo test --manifest-path hardening/careful/Cargo.toml --lib` passed:
  67 passed, 0 failed.
- `bash scripts/hardening.sh cargo-careful` passed:
  67 passed, 0 failed.
- `bash scripts/hardening.sh miri-tree` passed:
  35 passed, 0 failed, 1756 filtered out.
- `bash scripts/hardening.sh miri-many-seeds` passed over 128 seed attempts:
  status 0, with completed batches reporting 35 passed, 0 failed.

The completion audit is in `artifacts/completion-audit.md`.
