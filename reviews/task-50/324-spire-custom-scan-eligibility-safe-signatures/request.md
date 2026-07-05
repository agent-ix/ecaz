# Task 50 Review Request: SPIRE Custom Scan Eligibility Safe Signatures

## Summary

This slice converts custom-scan index eligibility from a raw-relation unsafe helper to a safe `SpireLiveIndexRelation` helper.

Code commit: `c63be45fa480b4e01438514d9d1b64677687ebbe`

## What Changed

- Converted `custom_scan_index_eligibility_result` to accept `SpireLiveIndexRelation`.
- Converted the internal placement-directory loader to accept `SpireLiveIndexRelation`.
- Updated the SQL eligibility wrapper to use `with_spire_live_index_relation!`.
- Updated custom-scan planner and EXPLAIN paths to construct the typed relation under their existing `IndexRelationGuard` scopes.

## Completion Audit Note

This packet does not close Task 50. The current audit still finds `1946` unsafe line hits under `src/`, so packet 030 Wave 5 closeout is not satisfied.

## Review Focus

- Please verify custom-scan eligibility no longer exposes a raw `pg_sys::Relation` helper signature.
- Please check that planner and EXPLAIN call sites keep the index relation guard live while using `SpireLiveIndexRelation`.
- Please check that the placement-directory read remains fail-closed for empty active epochs.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- No-match audit for removed unsafe custom-scan eligibility signatures and old raw-relation call paths.
- `make UNSAFE_LEDGER=reviews/task-50/324-spire-custom-scan-eligibility-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/324-spire-custom-scan-eligibility-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/324-spire-custom-scan-eligibility-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

## Counts

- Unsafe line count: `1946` (down from packet 323 `1949`)
- Unsafe ledger rows: `1361`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/no-unsafe-custom-scan-eligibility-signatures.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
