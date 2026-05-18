# Artifact Manifest: Task 41 HNSW Shared Debug Index Guard

- head SHA: `20eed29accae03c35053c278d68f695adf6bda82`
- packet/topic: `31179-c1-task41-hnsw-shared-debug-index-guard`
- timestamp: `2026-05-17T04:10:58Z`
- lane / fixture / storage format / rerank mode: unsafe hardening static
  analysis; no benchmark fixture; not storage-format-specific; not rerank-mode
  specific
- surface isolation: not applicable; no PostgreSQL runtime fixture was created

## Artifacts

- `unsafe-baseline-before.txt`
  - command: `git show HEAD^:scripts/unsafe_comment_baseline.txt`
  - key result: `4311` entries; `116` `src/am/ec_hnsw/shared.rs` entries
- `unsafe-baseline-after.txt`
  - command: `cp scripts/unsafe_comment_baseline.txt ...`
  - key result: `4301` entries; `106` `src/am/ec_hnsw/shared.rs` entries
- `baseline-before.log`
  - command: packet-local summary of `unsafe-baseline-before.txt`
  - key result: `4311` baseline entries
- `baseline-after.log`
  - command: `bash scripts/unsafe_baseline_report.sh`
  - key result: `4301` baseline entries
- `code-diff-stat.log`
  - command: `git show --stat --oneline --summary HEAD`
  - key result: `2 files changed, 89 insertions(+), 69 deletions(-)`
- `unsafe-comment-audit.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed
- `fmt-check.log`
  - command: `make fmt-check`
  - key result: passed with existing stable-rustfmt warnings
- `git-diff-check.log`
  - command: `git diff --check`
  - key result: passed
- `cargo-check-pg18.log`
  - command:
    `cargo check --all-targets --no-default-features --features pg18,bench`
  - key result: passed with existing PostgreSQL header warnings and existing
    unused re-export warning
