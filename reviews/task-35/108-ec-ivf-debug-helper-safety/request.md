# Task 35 Packet 108: EC IVF Test Debug Helper Safety

## Code Under Review

- Commit: `ad6aa9b6fdf79a076c158dc03e309916ce2a5ee6`
- Scope: `src/tests/ec_ivf.rs` plus `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet clears the `src/tests/ec_ivf.rs` unsafe-comment baseline by routing repeated `am::debug_ec_ivf_*` calls through one documented test-only macro.

The macro captures the shared invariant for these tests: each `pg_test` creates the referenced IVF index before calling the extension debug helper, and the debug helper owns PostgreSQL relation access for the supplied OID. This avoids duplicating the same safety comment at every debug call site.

## Result

- Global unsafe-comment baseline moved from `499` entries across `35` files to `416` entries across `34` files.
- `src/tests/ec_ivf.rs` moved from `83` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `499` entries, with `83` in `src/tests/ec_ivf.rs`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `416` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `416` entries across `34` files.
- `artifacts/ec-ivf-baseline-after.log`: `src/tests/ec_ivf.rs` residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
