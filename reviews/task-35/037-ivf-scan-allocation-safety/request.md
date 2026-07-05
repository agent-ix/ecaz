# Task 35 Review Request: IVF Scan Allocation Safety

## Summary

Code commit under review: `ada94f379ca9a40da2f91d72b9387292b151851f`

This slice documents scan-local allocation, ownership, and cleanup unsafe
boundaries in `src/am/ec_ivf/scan.rs`.

The covered areas are:

- Query, centroid-score, selected-list, and posting-candidate `palloc` buffers
- Prepared query, PQ fast-scan model, and candidate-dedup `Box::into_raw`
  ownership
- Scan query-prep cleanup orchestration

The added `SAFETY:` comments cover byte-size allocation checks, non-overlapping
copies into scan-owned buffers, `palloc`/`pfree` pairing, `Box::into_raw` /
`Box::from_raw` ownership, cached PQ model references, and cleanup helpers that
clear pointers after release.

## Baseline Accounting

- Global unsafe baseline: `2810 -> 2789`
- `src/am/ec_ivf/scan.rs`: `90 -> 69`

## Validation

- `bash scripts/check_unsafe_comments.sh` passed with an empty log:
  `artifacts/unsafe-audit-after.log`
- `make unsafe-baseline-report` reports `2789` entries and IVF scan at `69`:
  `artifacts/unsafe-baseline-report-after.log`
- `cargo fmt --all` ran; known unrelated format churn was restored before
  final validation: `artifacts/cargo-fmt.log`
- `git diff --check` passed with an empty log:
  `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed with the existing unrelated warnings in `src/am/common/parallel.rs`
  and `src/am/mod.rs`: `artifacts/cargo-check-pg18-bench.log`

## Artifacts

See `artifacts/manifest.md` for command lines, timestamps, and packet-local
evidence files.
