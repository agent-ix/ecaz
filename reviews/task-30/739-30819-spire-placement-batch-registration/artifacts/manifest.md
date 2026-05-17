# Artifact Manifest

- `cargo-test-placement-batch-lib.log`
  - head SHA: e339379eb881dbb6bdb0aa0ddf1c124557174f97
  - packet/topic: 30819 / spire-placement-batch-registration
  - lane / fixture / storage format / rerank mode: PG18 pg_test filter,
    `placement_batch`; placement batch SQL fixture; N/A; N/A
  - command used: `cargo test placement_batch --lib`
  - timestamp: 2026-05-11 America/Los_Angeles
  - isolated one-index-per-table or shared-table surfaces: isolated direct
    placement-directory rows
  - key result lines: `test tests::pg_test_ec_spire_register_placement_batch_sql ... ok`;
    `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1618 filtered out`

- `cargo-fmt-check.log`
  - head SHA: e339379eb881dbb6bdb0aa0ddf1c124557174f97
  - packet/topic: 30819 / spire-placement-batch-registration
  - lane / fixture / storage format / rerank mode: formatting check; N/A; N/A; N/A
  - command used: `cargo fmt --check`
  - timestamp: 2026-05-11 America/Los_Angeles
  - isolated one-index-per-table or shared-table surfaces: N/A
  - key result lines: command exited 0

- `git-diff-check.log`
  - head SHA: e339379eb881dbb6bdb0aa0ddf1c124557174f97
  - packet/topic: 30819 / spire-placement-batch-registration
  - lane / fixture / storage format / rerank mode: whitespace check; N/A; N/A; N/A
  - command used: `git diff --check`
  - timestamp: 2026-05-11 America/Los_Angeles
  - isolated one-index-per-table or shared-table surfaces: N/A
  - key result lines: command exited 0 with no output
