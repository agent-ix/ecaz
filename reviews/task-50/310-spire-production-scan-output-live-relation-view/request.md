# Review Request: SPIRE Production Scan Output Live Relation View

## Summary

This checkpoint binds the SPIRE production scan handoff and heap-resolution
output helpers in `remote_candidates/scan_output.rs` to `SpireLiveIndexRelation`.

Each helper now constructs the typed live relation view once at the raw relation
boundary, then uses the view for root-control reads, object-store construction,
and relation options. This removes scattered raw-relation unsafe blocks and
reuses the typed relation view already carried into local heap candidate summary
generation.

## Code Commit

- `e1be375d22d01147f57dc201476c538325a878f2` - `Bind SPIRE production scan output to live relation view`

## Unsafe Count

- Previous packet baseline after packet 309: `2048`
- After this checkpoint: `2045`
- Net change: `-3`
- `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` by-file match count: `3`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1383 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/310-spire-production-scan-output-live-relation-view/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/310-spire-production-scan-output-live-relation-view unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/310-spire-production-scan-output-live-relation-view/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

`cargo check` passes. The log includes the known pre-existing SPIRE unused-import
warning in `src/am/mod.rs`.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
