---
task: 50
packet: 237
topic: hnsw-reloptions-view-boundary
role: coder
status: ready-for-review
created: 2026-05-21T05:36:58-07:00
head_sha: 4358ade51b665d6adab76f8961ed18ae7d8d68ac
---

# Review Request: HNSW Reloptions View Boundary

## Summary

This packet applies the typed reloptions-view pattern to HNSW option parsing.

Changes:

- Added `TqHnswReloptionsView`, binding PostgreSQL's relation-owned `rd_options` pointer to the `TqHnswReloptions` layout registered by `ec_hnsw_amoptions`.
- Removed the free `unsafe fn read_string_reloption` helper.
- Moved HNSW string reloption reads onto the typed view for `build_source_column`, `rerank_source_column`, and `storage_format`.

## Safety Notes

- Raw `rd_options` reads remain isolated inside the new view boundary.
- HNSW `relation_options` still returns defaults for null reloptions and materializes owned Rust values before the relation-owned storage borrow ends.
- This keeps the string-offset contract attached to the HNSW reloptions layout rather than a standalone raw-pointer helper.

## Unsafe Count

- `src/am/ec_hnsw/options.rs`: `9 -> 7`
- Previous repo count: `2480`
- Current repo count: `2478`
- Delta: `-2`

The packet-local count log is:

- `artifacts/unsafe-counts.log`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_hnsw/options.rs` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-lib-ec-hnsw-pg18-no-run.log`: `cargo test --lib ec_hnsw --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.
