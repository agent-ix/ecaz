# Artifact Manifest: PG18 pgstat Unsafe Boundary

Packet: `review/31144-c1-pg18-pgstat-shim-unsafe-boundary`
Head SHA: `1931ba8af2c849a7c472a1830d242d650186175b`
Timestamp: `2026-05-16T22:21:12Z`

This packet is a code hardening review packet, not a benchmark measurement.
Lane, fixture, storage format, rerank mode, and isolated/shared table surface
are not applicable.

## unsafe-baseline-before.log

- Command:
  `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-904.txt`
- Source baseline:
  `git show HEAD^:scripts/unsafe_comment_baseline.txt`
- Key result:
  `entries: 4795`
- Key result:
  `files: 112`

## unsafe-baseline-after.log

- Command:
  `bash scripts/unsafe_baseline_report.sh`
- Source baseline:
  `scripts/unsafe_comment_baseline.txt`
- Key result:
  `entries: 4787`
- Key result:
  `files: 110`

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
