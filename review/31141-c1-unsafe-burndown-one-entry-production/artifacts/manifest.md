# Manifest: Unsafe Burndown One-Entry Production Slice

Head SHA: `f6a780887a264e31c17151e63810b27e0aa6c47d`
Packet: `31141-c1-unsafe-burndown-one-entry-production`
Timestamp: `2026-05-16T21:09:37Z`

This packet does not cite performance or recall measurements. It cites
unsafe-comment baseline counts and command pass/fail validation.

## Artifacts

### `unsafe-baseline-before.log`

- Head SHA: `f6a780887a264e31c17151e63810b27e0aa6c47d`
- Packet/topic: `31141-c1-unsafe-burndown-one-entry-production`
- Lane: unsafe baseline reporting before this slice
- Fixture: `HEAD^:scripts/unsafe_comment_baseline.txt`
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `git show HEAD^:scripts/unsafe_comment_baseline.txt > /private/tmp/tqvector-unsafe-baseline-before.txt` then `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before.txt > review/31141-c1-unsafe-burndown-one-entry-production/artifacts/unsafe-baseline-before.log`
- Timestamp: `2026-05-16T21:09:37Z`
- Surface: local, no table/index
- Key result lines:
  - `entries: 4816`
  - `files: 124`

### `unsafe-baseline-after.log`

- Head SHA: `f6a780887a264e31c17151e63810b27e0aa6c47d`
- Packet/topic: `31141-c1-unsafe-burndown-one-entry-production`
- Lane: unsafe baseline reporting after this slice
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `bash scripts/unsafe_baseline_report.sh > review/31141-c1-unsafe-burndown-one-entry-production/artifacts/unsafe-baseline-after.log`
- Timestamp: `2026-05-16T21:09:37Z`
- Surface: local, no table/index
- Key result lines:
  - `entries: 4809`
  - `files: 117`

### `audit-unsafe.log`

- Head SHA: `f6a780887a264e31c17151e63810b27e0aa6c47d`
- Packet/topic: `31141-c1-unsafe-burndown-one-entry-production`
- Lane: unsafe comment audit
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `bash scripts/check_unsafe_comments.sh > review/31141-c1-unsafe-burndown-one-entry-production/artifacts/audit-unsafe.log 2>&1`
- Timestamp: `2026-05-16T21:09:37Z`
- Surface: local, no table/index
- Key result lines:
  - Command exited successfully with no output.

### `fmt-check.log`

- Head SHA: `f6a780887a264e31c17151e63810b27e0aa6c47d`
- Packet/topic: `31141-c1-unsafe-burndown-one-entry-production`
- Lane: formatting check
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `make fmt-check > review/31141-c1-unsafe-burndown-one-entry-production/artifacts/fmt-check.log 2>&1`
- Timestamp: `2026-05-16T21:09:37Z`
- Surface: local, no table/index
- Key result lines:
  - Command exited successfully.
  - Stable rustfmt emitted existing warnings about nightly-only import grouping options.

### `git-diff-check.log`

- Head SHA: `f6a780887a264e31c17151e63810b27e0aa6c47d`
- Packet/topic: `31141-c1-unsafe-burndown-one-entry-production`
- Lane: whitespace diff check
- Fixture: `HEAD^..HEAD`
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `git diff --check HEAD^ HEAD > review/31141-c1-unsafe-burndown-one-entry-production/artifacts/git-diff-check.log`
- Timestamp: `2026-05-16T21:09:37Z`
- Surface: local, no table/index
- Key result lines:
  - Command exited successfully with no output.

### `cargo-check-pg18.log`

- Head SHA: `f6a780887a264e31c17151e63810b27e0aa6c47d`
- Packet/topic: `31141-c1-unsafe-burndown-one-entry-production`
- Lane: PG18 cargo check
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `cargo check --all-targets --no-default-features --features pg18,bench > review/31141-c1-unsafe-burndown-one-entry-production/artifacts/cargo-check-pg18.log 2>&1`
- Timestamp: `2026-05-16T21:09:37Z`
- Surface: local, no table/index
- Key result lines:
  - `Finished dev profile`
  - Existing warnings from PostgreSQL headers and currently unused SPIRE
    re-exports remained.
