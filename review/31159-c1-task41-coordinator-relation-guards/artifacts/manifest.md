# Artifact Manifest: Task 41 coordinator relation guards

- head SHA: `2ab0f0f4378bd5d906cab4084f45cd65426b74f0`
- packet/topic: `31159-c1-task41-coordinator-relation-guards`
- timestamp: `2026-05-16T23:43:12Z`
- lane / fixture / storage format / rerank mode: not applicable; static
  hardening audit and compile validation only
- surface isolation: not applicable; no PostgreSQL runtime fixture was run

## Artifacts

- `unsafe-baseline-before.txt`
  - command: `git show HEAD^:scripts/unsafe_comment_baseline.txt`
  - key result: pre-slice baseline input with 4602 entries
- `unsafe-baseline-before.log`
  - command: `bash scripts/unsafe_baseline_report.sh review/31159-c1-task41-coordinator-relation-guards/artifacts/unsafe-baseline-before.txt`
  - key result: `entries: 4602`
- `unsafe-baseline-after.log`
  - command: `bash scripts/unsafe_baseline_report.sh`
  - key result: `entries: 4590`
- `unsafe-baseline-report.log`
  - command: `make unsafe-baseline-report`
  - key result: `entries: 4590`
- `audit-unsafe.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output
- `fmt-check.log`
  - command: `make fmt-check`
  - key result: passed; rustfmt repeated existing warnings about unstable
    `imports_granularity` and `group_imports`
- `git-diff-check.log`
  - command: `git diff --check`
  - key result: passed with no output
- `cargo-check-pg18.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - key result: passed; existing warnings from PostgreSQL headers and
    `src/am/mod.rs` unused imports remain
