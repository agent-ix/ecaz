# Task 111h Packet 020 Artifact Manifest

- head SHA: `7acfc4985786eb2ac93dd1b98159d2f8d44fe910`
- branch: `bench-ivf-111g-115-attribution`
- task bucket: `reviews/task-111h/020-suite-load-index-name`
- captured at: `2026-06-20T09:43:36Z`
- lane: local Rust unit test
- surface isolation: not applicable; runner plumbing test only

## Commands

Focused unit test:

```sh
script -q -c "cargo test -p ecaz-cli expands_chunked_load_without_corpus_query_paths" reviews/task-111h/020-suite-load-index-name/artifacts/cargo-test-ecaz-cli-suite-load-index-name.log
```

## Artifact Index

- `artifacts/cargo-test-ecaz-cli-suite-load-index-name.log`: focused
  `ecaz-cli` suite argument-expansion unit test output.

## Key Result Lines

```text
test commands::bench::suite::tests::expands_chunked_load_without_corpus_query_paths ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 407 filtered out; finished in 0.00s
```
