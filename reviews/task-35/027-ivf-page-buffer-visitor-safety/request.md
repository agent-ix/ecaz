# Task 35 Review Request: IVF Page Buffer Visitor Safety

## Summary

Code commit under review: `7924869166425042043f32d1527b3d3e4dc33cfc`

This slice documents the IVF page buffer visitor unsafe boundaries in
`src/am/ec_ivf/page.rs`.

The covered helpers are:

- `visit_ivf_postings_for_list_block`
- `visit_all_ivf_postings_for_block`
- `visit_all_ivf_posting_refs_for_block`
- `visit_ivf_postings_from_buffer`
- `visit_all_ivf_postings_from_buffer`
- `visit_all_ivf_posting_refs_from_buffer`

The added `SAFETY:` comments cover share-locked buffer acquisition for legacy
non-PG18 block reads, forwarding through the shared visitor helpers, list-id
filtering, and exposing per-line tuple bytes from share-locked pages only after
the helper validates item-id bounds.

## Baseline Accounting

- Global unsafe baseline: `2921 -> 2912`
- `src/am/ec_ivf/page.rs`: `100 -> 91`

## Validation

- `bash scripts/check_unsafe_comments.sh` passed with an empty log:
  `artifacts/unsafe-audit-after.log`
- `make unsafe-baseline-report` reports `2912` entries and IVF page at `91`:
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
