# Task 35 Review Request: HNSW Source Safety

## Summary

This packet reviews commit `6cda9c8a57d0d1a956989904322833652ddeaa59`, which documents the remaining unsafe contracts in `src/am/ec_hnsw/source.rs` and removes that file from `scripts/unsafe_comment_baseline.txt`.

The slice covers:

- AVX2/FMA and NEON inner-product dispatch and lane loads/stores
- heap/source attribute resolution from PostgreSQL relation metadata
- `IndexInfo` validation, indexed vector attribute resolution, and `BuildIndexInfo` cleanup
- PostgreSQL type-name formatting and base-type lookup
- heap row version fetches and tuple slot Datum access
- detoasted `real[]`, `bytea`, and `ecvector` source views
- flat ArrayType dimension/data offset handling
- scoped Datum-backed source access helpers

## Result

Before this slice, the unsafe baseline had `707` total entries across `38` files, with `78` entries in `src/am/ec_hnsw/source.rs`.

After this slice, the unsafe baseline has `629` total entries across `37` files, and `src/am/ec_hnsw/source.rs` has `0` baseline entries.

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline` exited 0 and wrote `629` entries.
- `bash scripts/check_unsafe_comments.sh` exited 0.
- `bash scripts/unsafe_baseline_report.sh` reported `entries: 629`, `files: 37`.
- `git diff --check` exited 0.
- `cargo check --all-targets --no-default-features --features pg18,bench` exited 0.

`cargo check` still emits the known unrelated unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the full artifact list and key result lines. The main after-state evidence is:

- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/hnsw-source-count-after-format.log`
- `artifacts/unsafe-audit-after.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
