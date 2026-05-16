# Artifact Manifest: 30823 SPIRE CustomScan Ecvector Payload Projection

## `cargo-test-customscan-ecvector-projection-lib.log`

- head SHA: `8be1523882635d0df75dd40c6131f44faaf99e3d`
- packet/topic: `30823 / spire-customscan-ecvector-payload-projection`
- lane / fixture / storage format / rerank mode: PG18 focused CustomScan
  loopback-remote tuple-payload fixture; `ecvector_spire_ip_ops`; `rabitq`;
  projected `ecvector` payload through `ecvector_to_real_array(...)`
- command used:
  `script -q -e -c "cargo test customscan_returns_loopback_remote_tuple_payload --lib" review/30823-spire-customscan-ecvector-payload-projection/artifacts/cargo-test-customscan-ecvector-projection-lib.log`
- timestamp: 2026-05-11T08:46:30-07:00
- isolated/shared surface: isolated pg_test coordinator table plus loopback
  remote descriptor through the production conninfo path
- key result lines:
  `test tests::pg_test_ec_spire_customscan_returns_loopback_remote_tuple_payload ... ok`;
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1619 filtered out`

## `cargo-fmt-check.log`

- head SHA: `8be1523882635d0df75dd40c6131f44faaf99e3d`
- packet/topic: `30823 / spire-customscan-ecvector-payload-projection`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30823-spire-customscan-ecvector-payload-projection/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T08:47:00-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `8be1523882635d0df75dd40c6131f44faaf99e3d`
- packet/topic: `30823 / spire-customscan-ecvector-payload-projection`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30823-spire-customscan-ecvector-payload-projection/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T08:47:00-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `8be1523882635d0df75dd40c6131f44faaf99e3d`
- packet/topic: `30823 / spire-customscan-ecvector-payload-projection`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30823-spire-customscan-ecvector-payload-projection/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T08:48:00-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
