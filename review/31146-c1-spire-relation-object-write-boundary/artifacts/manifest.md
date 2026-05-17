# Artifact Manifest: SPiRE Relation Object Write Boundary

Packet: `review/31146-c1-spire-relation-object-write-boundary`
Head SHA: `110c0dc217da4330beded2e9c87482a6a5ec6853`
Timestamp: `2026-05-16T22:34:36Z`

This packet is a code hardening review packet, not a benchmark measurement.
Lane, fixture, storage format, rerank mode, and isolated/shared table surface
are not applicable.

## unsafe-baseline-before.log

- Command:
  `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-906.txt`
- Source baseline:
  `git show HEAD^:scripts/unsafe_comment_baseline.txt`
- Key result:
  `entries: 4782`
- Key result:
  `files: 108`

## unsafe-baseline-after.log

- Command:
  `bash scripts/unsafe_baseline_report.sh`
- Source baseline:
  `scripts/unsafe_comment_baseline.txt`
- Key result:
  `entries: 4748`
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
