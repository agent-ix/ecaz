# Artifact Manifest

- `cargo-test-classify-centroid-lib.log`
  - head SHA: f92b43982804f1decc2d5c302e1944f1fc76b456
  - packet/topic: 30818 / spire-classify-centroid-helper
  - lane / fixture / storage format / rerank mode: PG18 pg_test filter,
    `classify_centroid`; classifier fixture with remote leaf placement; N/A; N/A
  - command used: `cargo test classify_centroid --lib`
  - timestamp: 2026-05-11 America/Los_Angeles
  - isolated one-index-per-table or shared-table surfaces: isolated test relation
  - key result lines: `test tests::pg_test_ec_spire_classify_centroid_sql ... ok`;
    `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1617 filtered out`

- `cargo-fmt-check.log`
  - head SHA: f92b43982804f1decc2d5c302e1944f1fc76b456
  - packet/topic: 30818 / spire-classify-centroid-helper
  - lane / fixture / storage format / rerank mode: formatting check; N/A; N/A; N/A
  - command used: `cargo fmt --check`
  - timestamp: 2026-05-11 America/Los_Angeles
  - isolated one-index-per-table or shared-table surfaces: N/A
  - key result lines: command exited 0

- `git-diff-check.log`
  - head SHA: f92b43982804f1decc2d5c302e1944f1fc76b456
  - packet/topic: 30818 / spire-classify-centroid-helper
  - lane / fixture / storage format / rerank mode: whitespace check; N/A; N/A; N/A
  - command used: `git diff --check`
  - timestamp: 2026-05-11 America/Los_Angeles
  - isolated one-index-per-table or shared-table surfaces: N/A
  - key result lines: command exited 0 with no output
