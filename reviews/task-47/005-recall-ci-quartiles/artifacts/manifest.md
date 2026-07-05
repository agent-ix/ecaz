# Task 47 Packet 005 Artifact Manifest

- head SHA: `b47398b4318bf70a0307d2fda649e7fa4fadb5a0`
- task bucket: `reviews/task-47`
- packet path: `reviews/task-47/005-recall-ci-quartiles`
- timestamp: `2026-05-18T23:02:39Z`
- lane: Task 47 recall confidence interval and per-query distribution reporting
- fixture/storage/rerank mode: pure Rust recall metric helpers plus suite table parser; no live PG fixture in this packet
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

| Artifact | Command | Key lines |
| --- | --- | --- |
| `cargo-test-recall-summary.log` | `script -q reviews/task-47/005-recall-ci-quartiles/artifacts/cargo-test-recall-summary.log cargo test -p ecaz-cli recall_summary -- --test-threads=1` | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 352 filtered out` |
| `cargo-test-recall-table-parse.log` | `script -q reviews/task-47/005-recall-ci-quartiles/artifacts/cargo-test-recall-table-parse.log cargo test -p ecaz-cli parses_recall_result_table -- --test-threads=1` | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 353 filtered out` |
| `cargo-check-ecaz-cli.log` | `script -q reviews/task-47/005-recall-ci-quartiles/artifacts/cargo-check-ecaz-cli.log cargo check -p ecaz-cli` | `Finished dev profile` |
| `git-diff-check.log` | `script -q reviews/task-47/005-recall-ci-quartiles/artifacts/git-diff-check.log git --no-pager diff HEAD~1 HEAD --check -- crates/ecaz-cli/src/commands/bench/recall.rs crates/ecaz-cli/src/commands/bench/suite.rs docs/recall-floors.md` | Clean whitespace check; no findings emitted. |

## Notes

- This packet validates the pure metric math and the suite table parser. It does not run a live recall gate.
- The `cargo` commands emit pre-existing warnings from `ecaz` library imports and PostgreSQL headers; no warning originates from the new recall reporting code.
