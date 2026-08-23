# Task 167 packet 061 artifact manifest

- Head under review: `f58a69b41efbf5753b098b7476e7d7e7ba438c43`.
- Owning packet: `reviews/task-167/061-heldout-baseline-relative/`.
- Timestamp: `2026-08-23` (America/Los_Angeles).
- Scope: Task 167 CLI fixture and suite-runner gate semantics only; no product
  index-format, scan, storage, or unrelated benchmark behavior changed.
- Benchmark lane / fixture / storage format / rerank mode: not applicable;
  this is a code-review packet with focused unit-test evidence.
- Isolation: no PostgreSQL fixture or shared-table measurement was run.

## Focused tests

- Artifact: `focused-tests.log`.
- Command:
  `cargo test -p ecaz-cli --no-default-features task167_`.
- Result: passed; 14 passed, 0 failed, 498 filtered.
- SHA-256:
  `946ed9cfb49c27aea22437d11fe80cfda253a2e5bd9ae03344d394c9236184ca`.
- Key result line:
  `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out`.
