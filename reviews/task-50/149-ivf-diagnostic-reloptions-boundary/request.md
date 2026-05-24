# Review Request: IVF Diagnostic Reloptions Boundary

## Summary

This checkpoint addresses a contained subset of the soundness audit's IVF
round-3 finding.

Changes:

- Marked IVF diagnostic helpers `index_drift_snapshot`,
  `index_admin_snapshot`, and `index_page_ownership` unsafe because callers
  must pass a live IVF index relation.
- Marked `options::relation_options` and `read_string_reloption` unsafe,
  preserving the raw relcache/reloptions pointer precondition at the function
  boundary.
- Moved SQL-facing IVF diagnostics to `with_live_index_relation!`, where the
  relation guard provides the safety proof.
- Added safety acknowledgments at AM callback sites that read IVF reloptions.

## Validation

- Pass: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Artifact: `artifacts/cargo-check-pg18-bench.log`
  - Note: pre-existing unused-import warning in `src/am/mod.rs`.
- Pass: `git diff --check`
  - Artifact: `artifacts/git-diff-check.log`
- Pass: `make unsafe-block-count`
  - Artifact: `artifacts/unsafe-block-count.log`
  - Summed unsafe count: 1625.

## Reviewer Focus

Please check that the raw IVF relation and reloptions-pointer validity
requirements are now visible at the correct unsafe boundaries, especially the
AM callback cases where the outer callback guard already supplies an unsafe
execution context.
