# Artifact Manifest: SPiRE Scan Heap Relation Guard

Packet: `review/907-c1-spire-scan-heap-relation-guard`
Head SHA: `757f2faa1551ff7cf35163dc0e6c9f0a2bf81359`
Timestamp: `2026-05-16T22:39:17Z`

This packet is a code hardening review packet, not a benchmark measurement.
Lane, fixture, storage format, rerank mode, and isolated/shared table surface
are not applicable.

## unsafe-baseline-before.log

- Command:
  `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-907.txt`
- Source baseline:
  `git show HEAD^:scripts/unsafe_comment_baseline.txt`
- Key result:
  `entries: 4748`
- Key result:
  `files: 106`

## unsafe-baseline-after.log

- Command:
  `bash scripts/unsafe_baseline_report.sh`
- Source baseline:
  `scripts/unsafe_comment_baseline.txt`
- Key result:
  `entries: 4738`
- Key result:
  `files: 106`

## audit-unsafe.log

- Command:
  `bash scripts/check_unsafe_comments.sh`
- Key result:
  command exited 0 with no output.

## fmt-check.log

- Command:
  `make fmt-check`
- Key result:
  command exited 0.

## git-diff-check.log

- Command:
  `git diff --check HEAD^ HEAD`
- Key result:
  command exited 0 with no output.

## cargo-check-pg18.log

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Key result:
  `Finished dev profile`
- Notes:
  existing PostgreSQL header warnings and existing unused SPIRE re-export
  warning remain.
