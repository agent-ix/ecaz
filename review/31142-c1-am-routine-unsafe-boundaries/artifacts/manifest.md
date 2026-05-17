# Manifest: AM Routine Unsafe Boundary Reduction

Head SHA: `57e9f44929d758a9f8d308c8b62c366e7f30f67a`
Packet: `31142-c1-am-routine-unsafe-boundaries`
Timestamp: `2026-05-16T22:08:15Z`

This packet does not cite performance or recall measurements. It cites
unsafe-comment baseline counts and command pass/fail validation.

## Artifacts

### `unsafe-baseline-before.log`

- Head SHA: `57e9f44929d758a9f8d308c8b62c366e7f30f67a`
- Packet/topic: `31142-c1-am-routine-unsafe-boundaries`
- Lane: unsafe baseline reporting before this slice
- Fixture: `HEAD^:scripts/unsafe_comment_baseline.txt`
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `git show HEAD^:scripts/unsafe_comment_baseline.txt > /private/tmp/tqvector-unsafe-baseline-before-902.txt` then `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-902.txt > review/31142-c1-am-routine-unsafe-boundaries/artifacts/unsafe-baseline-before.log`
- Timestamp: `2026-05-16T22:08:15Z`
- Surface: local, no table/index
- Key result lines:
  - `entries: 4809`
  - `files: 117`

### `unsafe-baseline-after.log`

- Head SHA: `57e9f44929d758a9f8d308c8b62c366e7f30f67a`
- Packet/topic: `31142-c1-am-routine-unsafe-boundaries`
- Lane: unsafe baseline reporting after this slice
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `bash scripts/unsafe_baseline_report.sh > review/31142-c1-am-routine-unsafe-boundaries/artifacts/unsafe-baseline-after.log`
- Timestamp: `2026-05-16T22:08:15Z`
- Surface: local, no table/index
- Key result lines:
  - `entries: 4799`
  - `files: 113`

### `audit-unsafe.log`

- Head SHA: `57e9f44929d758a9f8d308c8b62c366e7f30f67a`
- Packet/topic: `31142-c1-am-routine-unsafe-boundaries`
- Lane: unsafe comment audit
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `bash scripts/check_unsafe_comments.sh > review/31142-c1-am-routine-unsafe-boundaries/artifacts/audit-unsafe.log 2>&1`
- Timestamp: `2026-05-16T22:08:15Z`
- Surface: local, no table/index
- Key result lines:
  - Command exited successfully with no output.

### `fmt-check.log`

- Head SHA: `57e9f44929d758a9f8d308c8b62c366e7f30f67a`
- Packet/topic: `31142-c1-am-routine-unsafe-boundaries`
- Lane: formatting check
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `make fmt-check > review/31142-c1-am-routine-unsafe-boundaries/artifacts/fmt-check.log 2>&1`
- Timestamp: `2026-05-16T22:08:15Z`
- Surface: local, no table/index
- Key result lines:
  - Command exited successfully.
  - Stable rustfmt emitted existing warnings about nightly-only import grouping options.

### `git-diff-check.log`

- Head SHA: `57e9f44929d758a9f8d308c8b62c366e7f30f67a`
- Packet/topic: `31142-c1-am-routine-unsafe-boundaries`
- Lane: whitespace diff check
- Fixture: `HEAD^..HEAD`
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `git diff --check HEAD^ HEAD > review/31142-c1-am-routine-unsafe-boundaries/artifacts/git-diff-check.log`
- Timestamp: `2026-05-16T22:08:15Z`
- Surface: local, no table/index
- Key result lines:
  - Command exited successfully with no output.

### `cargo-check-pg18.log`

- Head SHA: `57e9f44929d758a9f8d308c8b62c366e7f30f67a`
- Packet/topic: `31142-c1-am-routine-unsafe-boundaries`
- Lane: PG18 cargo check
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `cargo check --all-targets --no-default-features --features pg18,bench > review/31142-c1-am-routine-unsafe-boundaries/artifacts/cargo-check-pg18.log 2>&1`
- Timestamp: `2026-05-16T22:08:15Z`
- Surface: local, no table/index
- Key result lines:
  - `Finished dev profile`
  - Existing warnings from PostgreSQL headers and currently unused SPIRE
    re-exports remained.
