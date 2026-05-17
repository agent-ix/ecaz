# Artifact Manifest

- `cargo-test-ec-spire-placement-lib.log`
  - head SHA: 835e6ac097bf37d8c40adc5efb8378f3246194c1
  - packet/topic: 30817 / spire-placement-directory-catalog
  - lane / fixture / storage format / rerank mode: PG18 pg_test filter,
    `ec_spire_placement`; catalog and placement snapshot fixtures; N/A; N/A
  - command used: `cargo test ec_spire_placement --lib`
  - timestamp: 2026-05-11 America/Los_Angeles
  - isolated one-index-per-table or shared-table surfaces: isolated test
    relations
  - key result lines: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1615 filtered out`

- `cargo-test-remote-catalog-lib.log`
  - head SHA: 835e6ac097bf37d8c40adc5efb8378f3246194c1
  - packet/topic: 30817 / spire-placement-directory-catalog
  - lane / fixture / storage format / rerank mode: PG18 pg_test filter,
    `remote_catalog`; catalog cleanup fixtures; N/A; N/A
  - command used: `cargo test remote_catalog --lib`
  - timestamp: 2026-05-11 America/Los_Angeles
  - isolated one-index-per-table or shared-table surfaces: isolated test
    relations
  - key result lines: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1614 filtered out`

- `cargo-fmt-check.log`
  - head SHA: 835e6ac097bf37d8c40adc5efb8378f3246194c1
  - packet/topic: 30817 / spire-placement-directory-catalog
  - lane / fixture / storage format / rerank mode: formatting check; N/A; N/A; N/A
  - command used: `cargo fmt --check`
  - timestamp: 2026-05-11 America/Los_Angeles
  - isolated one-index-per-table or shared-table surfaces: N/A
  - key result lines: command exited 0

- `git-diff-check.log`
  - head SHA: 835e6ac097bf37d8c40adc5efb8378f3246194c1
  - packet/topic: 30817 / spire-placement-directory-catalog
  - lane / fixture / storage format / rerank mode: whitespace check; N/A; N/A; N/A
  - command used: `git diff --check`
  - timestamp: 2026-05-11 America/Los_Angeles
  - isolated one-index-per-table or shared-table surfaces: N/A
  - key result lines: command exited 0 with no output
