# Artifact Manifest: Task 41 SPIRE Scan Heap Slot Guard

- head SHA: `3582fd276ea7175cf209d1f97185b4ce0cb695d6`
- packet/topic: `31177-c1-task41-spire-scan-heap-slot-guard`
- timestamp: `2026-05-17T04:02:37Z`
- lane / fixture / storage format / rerank mode: unsafe hardening static
  analysis; no benchmark fixture; not storage-format-specific; heap rerank scan
  slot handling
- surface isolation: not applicable; no PostgreSQL runtime fixture was created

## Artifacts

- `unsafe-baseline-before.txt`
  - command: `git show HEAD^:scripts/unsafe_comment_baseline.txt`
  - key result: `4315` entries; `36`
    `src/am/ec_spire/scan/relation.rs` entries
- `unsafe-baseline-after.txt`
  - command: `cp scripts/unsafe_comment_baseline.txt ...`
  - key result: `4313` entries; `34`
    `src/am/ec_spire/scan/relation.rs` entries
- `baseline-before.log`
  - command: packet-local summary of `unsafe-baseline-before.txt`
  - key result: `4315` baseline entries
- `baseline-after.log`
  - command: `bash scripts/unsafe_baseline_report.sh`
  - key result: `4313` baseline entries
- `code-diff-stat.log`
  - command: `git show --stat --oneline --summary HEAD`
  - key result: `2 files changed, 52 insertions(+), 35 deletions(-)`
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
