# Artifact Manifest

- head SHA: `13a6cbe2c0baece9089a21bf0cd70b7ee938d85c`
- code commit under review: `2ce73bdc3e82e840a7d6a15e7b36d066e8fddce7`
- task bucket: `reviews/task-51`
- packet path: `reviews/task-51/013-ivf-adaptive-bench-harness`
- lane: local PG18 benchmark-runner harness validation
- fixture: not applicable; unit tests only
- access method: `ec_ivf` benchmark plumbing, with existing `ec_spire` behavior preserved
- storage format / rerank mode: not applicable
- isolated one-index-per-table surface: not applicable
- timestamp: `2026-05-23T13:44:07Z`
- AWS: not used
- vchord / pgvectorscale: not used

## Artifacts

- `cargo-test-ecaz-cli-adaptive-nprobe.log`
  - command: `cargo test -p ecaz-cli adaptive_nprobe`
  - result: passed
  - key line: `test result: ok. 2 passed; 0 failed`
- `cargo-test-ecaz-cli-expands-recall.log`
  - command: `cargo test -p ecaz-cli expands_recall_with_defaults`
  - result: passed
  - key line: `test result: ok. 1 passed; 0 failed`
- `git-diff-check.log`
  - command: `git diff --check`
  - result: passed, no output

## Key Result Lines

```text
running 2 tests
test commands::bench::tests::adaptive_nprobe_threshold_requires_enabled_switch ... ok
test commands::bench::tests::adaptive_nprobe_bench_options_support_ivf_and_spire ... ok
test result: ok. 2 passed; 0 failed

running 1 test
test commands::bench::suite::tests::expands_recall_with_defaults ... ok
test result: ok. 1 passed; 0 failed
```
