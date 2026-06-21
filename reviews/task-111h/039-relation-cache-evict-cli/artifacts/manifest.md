# Task 111h / 039 Relation Cache Evict CLI Manifest

- head SHA: `1751bf572205a31173859c79bd8fdec199f6f6ad`
- task bucket: `reviews/task-111h/039-relation-cache-evict-cli/`
- lane / fixture / storage format / rerank mode: local CLI support for later cold-cache IVF rerank suites; no corpus fixture or rerank format in this packet
- isolated/shared surface: not applicable; no benchmark suite was run in this packet

## Artifacts

### `artifacts/cargo-test-relation-cache.log`

- command:
  `CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo test -p ecaz-cli relation_file_match_includes_segments_and_forks`
- captured with:
  `script -q -e -c 'env CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo test -p ecaz-cli relation_file_match_includes_segments_and_forks' reviews/task-111h/039-relation-cache-evict-cli/artifacts/cargo-test-relation-cache.log`
- timestamp: `2026-06-20 15:19:44-07:00` through `2026-06-20 15:21:56-07:00`
- key result lines:
  - `test commands::dev::relation_cache::tests::relation_file_match_includes_segments_and_forks ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 408 filtered out; finished in 0.00s`

### `artifacts/dev-evict-relation-cache-help.log`

- command:
  `CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo run -p ecaz-cli -- dev evict-relation-cache --help`
- captured with:
  `script -q -e -c 'env CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo run -p ecaz-cli -- dev evict-relation-cache --help' reviews/task-111h/039-relation-cache-evict-cli/artifacts/dev-evict-relation-cache-help.log`
- timestamp: `2026-06-20 15:22:05-07:00` through `2026-06-20 15:23:00-07:00`
- key result lines:
  - `Usage: ecaz dev evict-relation-cache [OPTIONS]`
  - `--prefix <PREFIXES>`
  - `--relation <RELATIONS>`
  - `--dry-run`
  - `--log-file <LOG_FILE>`
  - `Script done ... [COMMAND_EXIT_CODE="0"]`

## Notes

- This packet intentionally does not claim cold-cache benchmark results. It adds the `ecaz dev evict-relation-cache` primitive needed for a later `ecaz bench suite` raw step to evict PostgreSQL relation files from the local OS page cache.
- The helper uses `SHOW data_directory`, resolves `<prefix>_corpus` plus indexes and toast relations through PostgreSQL catalogs, expands relation segment/fork files under the data directory, and calls `posix_fadvise(POSIX_FADV_DONTNEED)` on each file.
