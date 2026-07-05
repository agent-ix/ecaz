# Task 50 Review Request: SPIRE Remote Search Diagnostic Probes Safe Signatures

## Summary

This slice converts two remaining SPIRE remote-search diagnostic probe helpers from raw-relation unsafe signatures to safe `SpireLiveIndexRelation` signatures.

Code commit: `d3f782ea0805cca12e77eaae405fef961d78c032`

## What Changed

- Converted `remote_search_operator_diagnostics_row` to accept `SpireLiveIndexRelation`.
- Updated the `ec_spire_remote_search_operator_diagnostics` SQL wrapper to use `with_spire_live_index_relation!`.
- Converted the pg-test-only `remote_search_libpq_identity_cache_contract_probe_counts` helper to accept `SpireLiveIndexRelation`.
- Updated the libpq identity-cache contract test to construct the typed relation under its validated index relation guard.

## Completion Audit Note

This packet does not close Task 50. The current audit still finds `1949` unsafe line hits under `src/`, so packet 030 Wave 5 closeout is not satisfied.

## Review Focus

- Please verify these diagnostics no longer accept raw `pg_sys::Relation` in their safe helper signatures.
- Please check the SQL wrapper and test caller keep their relation guards live while using `SpireLiveIndexRelation`.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- No-match audit for removed unsafe remote-search diagnostic probe signatures and old raw-relation call paths.
- `make UNSAFE_LEDGER=reviews/task-50/323-spire-remote-search-diagnostic-probes-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/323-spire-remote-search-diagnostic-probes-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/323-spire-remote-search-diagnostic-probes-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

## Counts

- Unsafe line count: `1949` (down from packet 322 `1953`)
- Unsafe ledger rows: `1362`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/no-unsafe-remote-search-diagnostic-probe-signatures.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
