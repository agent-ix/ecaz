# Artifact Manifest: 30822 SPIRE CustomScan Expression Payload Fallback

## `cargo-test-customscan-loopback-expression-lib.log`

- head SHA: `9fff5b2373b22a93827ac1708f1386659c798cf3`
- packet/topic: `30822 / spire-customscan-expression-payload-fallback`
- lane / fixture / storage format / rerank mode: PG18 focused CustomScan
  loopback-remote tuple-payload fixture; `ecvector_spire_ip_ops`; `rabitq`;
  expression projection fallback
- command used:
  `script -q -e -c "cargo test customscan_returns_loopback_remote_tuple_payload --lib" review/30822-spire-customscan-expression-payload-fallback/artifacts/cargo-test-customscan-loopback-expression-lib.log`
- timestamp: 2026-05-11T08:41:12-07:00
- isolated/shared surface: isolated pg_test coordinator table plus loopback
  remote descriptor through the production conninfo path
- key result lines:
  `test tests::pg_test_ec_spire_customscan_returns_loopback_remote_tuple_payload ... ok`;
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1619 filtered out`

## `cargo-fmt-check.log`

- head SHA: `9fff5b2373b22a93827ac1708f1386659c798cf3`
- packet/topic: `30822 / spire-customscan-expression-payload-fallback`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30822-spire-customscan-expression-payload-fallback/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T08:42:00-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `9fff5b2373b22a93827ac1708f1386659c798cf3`
- packet/topic: `30822 / spire-customscan-expression-payload-fallback`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30822-spire-customscan-expression-payload-fallback/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T08:42:00-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `9fff5b2373b22a93827ac1708f1386659c798cf3`
- packet/topic: `30822 / spire-customscan-expression-payload-fallback`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30822-spire-customscan-expression-payload-fallback/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T08:42:00-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
