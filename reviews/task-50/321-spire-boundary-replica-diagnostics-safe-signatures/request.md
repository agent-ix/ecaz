# Task 50 Review Request: SPIRE Boundary Replica Diagnostics Safe Signatures

## Summary

This slice converts SPIRE boundary replica diagnostic helpers from unsafe raw-relation entry points to safe `SpireLiveIndexRelation` helpers.

Code commit: `b19efa9de17bc52a4bf55f9ec3381468ede941ff`

## What Changed

- Converted `index_boundary_replica_identity_snapshot` to take `SpireLiveIndexRelation` and use the existing coordinator fanout anchor/object-store wrapper.
- Converted `index_boundary_replica_placement_diagnostics` to take `SpireLiveIndexRelation`.
- Converted the placement diagnostic manifest loader to take `SpireLiveIndexRelation` while retaining its special non-available placement read behavior.
- Updated SQL wrappers in `src/lib.rs` to use `with_spire_live_index_relation!`.

## Completion Audit Note

This packet does not close Task 50. The current audit still finds `1958` unsafe line hits under `src/`, so packet 030 Wave 5 closeout is not satisfied.

## Review Focus

- Please verify the safe helpers no longer accept raw `pg_sys::Relation`.
- Please check the placement diagnostics still intentionally load the raw placement directory states instead of the coordinator fanout filtered semantics.
- Please check the custom manifest loader’s remaining page reads are properly scoped behind the live relation wrapper.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- No-match audit for removed boundary replica unsafe signatures, generic raw SQL wrapper use, and `checked_live_index_relation`
- `make UNSAFE_LEDGER=reviews/task-50/321-spire-boundary-replica-diagnostics-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/321-spire-boundary-replica-diagnostics-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/321-spire-boundary-replica-diagnostics-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

## Counts

- Unsafe line count: `1958` (down from packet 320 `1960`)
- Unsafe ledger rows: `1368`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/no-unsafe-boundary-replica-signatures.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
