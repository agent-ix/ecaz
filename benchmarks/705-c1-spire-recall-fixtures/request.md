---
agent: coder1
role: coder
model: gpt-5
date: 2026-05-14
topic: c1-spire-recall-fixtures
code_commit: f98d9660
---

# Review Request: SPIRE Recall Fixtures

## Summary

Added focused 12c.6 recall coverage in a new test file:

- `test_ec_spire_recall_at_10_matches_exact_on_full_probe`
  - Builds a deterministic 64-row corpus.
  - Creates an `ec_spire` index with `nprobe = nlists`.
  - Asserts indexed top-10 ids equal the brute-force exact top-10 set.
  - Asserts returned top-k ids are unique.
- `test_ec_spire_nprobe_sweep_recall_is_monotonic`
  - Reuses the 64-row corpus shape with 16 lists.
  - Sweeps session `ec_spire.nprobe` over `1, 4, 8, 16`.
  - Asserts recall@10 does not decrease as more lists are probed.

The tests live in `src/tests/spire_recall.rs` instead of adding more weight to
`scan.rs` or `custom_scan.rs`.

## Scope

Changed:

- `src/tests/spire_recall.rs`
- `src/tests/mod.rs`

This covers the local SPIRE AM recall baseline and nprobe monotonicity parts of
12c.6. It does not cover distributed CustomScan multi-remote recall; that
remains a separate fixture.

File-size check:

- `src/tests/spire_recall.rs`: 79 lines.
- `src/tests/scan.rs`: 1329 lines.
- `src/tests/mod.rs`: 2805 lines, unchanged from the pre-slice count after the
  include-block cleanup. It is still above the 2500-line target and should be
  split in a later structural cleanup, but this slice did not make it larger.

## Validation

Passed:

- `cargo fmt --check`
  - Stable rustfmt emitted the repository's existing warnings about nightly-only
    `imports_granularity` and `group_imports`.
- `git diff --check -- src/tests/mod.rs src/tests/spire_recall.rs`
- `cargo test --no-default-features --features pg18 test_ec_spire_recall_at_10_matches_exact_on_full_probe --no-run`
- `cargo test --no-default-features --features pg18 test_ec_spire_nprobe_sweep_recall_is_monotonic --no-run`
  - Both compile-only runs emitted the existing unused-import warning in
    `src/am/mod.rs`.

I initially invoked `cargo test` with two positional filters, which Cargo
rejected before running anything; the two focused `--no-run` checks above are
the successful validations.

## Review Focus

Please check whether the deterministic corpus is strong enough as a CI recall
pin, and whether the nprobe monotonic assertion is an acceptable first-stage
coverage slice before the heavier distributed CustomScan recall fixture lands.
