# Artifact Manifest: 30827 SPIRE CustomScan Cost Model

## `cargo-test-custom-scan-cost-lib.log`

- head SHA: `d2b15f24faeb8a19fdfad3d33bd619da7e2164bb`
- packet/topic: `30827 / spire-customscan-cost-model`
- lane / fixture / storage format / rerank mode: Rust CustomScan cost model
  unit tests; fanout/output-row monotonicity; no storage fixture
- command used:
  `script -q -e -c "cargo test custom_scan_cost --lib" review/30827-spire-customscan-cost-model/artifacts/cargo-test-custom-scan-cost-lib.log`
- timestamp: 2026-05-11T09:21:20-07:00
- isolated/shared surface: Rust unit-test model surface
- key result lines:
  `test am::ec_spire::custom_scan::tests::custom_scan_cost_increases_with_output_rows ... ok`;
  `test am::ec_spire::custom_scan::tests::custom_scan_cost_increases_with_remote_fanout ... ok`;
  `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1624 filtered out`

## `cargo-test-customscan-explain-lib.log`

- head SHA: `d2b15f24faeb8a19fdfad3d33bd619da7e2164bb`
- packet/topic: `30827 / spire-customscan-cost-model`
- lane / fixture / storage format / rerank mode: PG18 CustomScan EXPLAIN
  fixture; remote-placement ORDER BY LIMIT plan shape; no rerank
- command used:
  `script -q -e -c "cargo test customscan_explain_remote_order_limit --lib" review/30827-spire-customscan-cost-model/artifacts/cargo-test-customscan-explain-lib.log`
- timestamp: 2026-05-11T09:21:20-07:00
- isolated/shared surface: isolated pg_test coordinator relation with remote
  placement metadata
- key result lines:
  `test tests::pg_test_ec_spire_customscan_explain_remote_order_limit ... ok`;
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1625 filtered out`

## `cargo-fmt-check.log`

- head SHA: `d2b15f24faeb8a19fdfad3d33bd619da7e2164bb`
- packet/topic: `30827 / spire-customscan-cost-model`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30827-spire-customscan-cost-model/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T09:21:20-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `d2b15f24faeb8a19fdfad3d33bd619da7e2164bb`
- packet/topic: `30827 / spire-customscan-cost-model`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30827-spire-customscan-cost-model/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T09:21:20-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `d2b15f24faeb8a19fdfad3d33bd619da7e2164bb`
- packet/topic: `30827 / spire-customscan-cost-model`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30827-spire-customscan-cost-model/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T09:21:20-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
