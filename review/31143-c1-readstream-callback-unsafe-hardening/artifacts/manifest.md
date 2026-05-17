# Manifest: ReadStream Callback Unsafe Hardening

Head SHA: `ef554f4e7445da3842217ad1ee2ac4cdd10cabae`
Packet: `31143-c1-readstream-callback-unsafe-hardening`
Timestamp: `2026-05-16T22:14:42Z`

This packet does not cite performance or recall measurements. It cites
unsafe-comment baseline counts and command pass/fail validation.

## Artifacts

### `unsafe-baseline-before.log`

- Head SHA: `ef554f4e7445da3842217ad1ee2ac4cdd10cabae`
- Packet/topic: `31143-c1-readstream-callback-unsafe-hardening`
- Lane: unsafe baseline reporting before this slice
- Fixture: `HEAD^:scripts/unsafe_comment_baseline.txt`
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `git show HEAD^:scripts/unsafe_comment_baseline.txt > /private/tmp/tqvector-unsafe-baseline-before-903.txt` then `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-903.txt > review/31143-c1-readstream-callback-unsafe-hardening/artifacts/unsafe-baseline-before.log`
- Timestamp: `2026-05-16T22:14:42Z`
- Surface: local, no table/index
- Key result lines:
  - `entries: 4799`
  - `files: 113`

### `unsafe-baseline-after.log`

- Head SHA: `ef554f4e7445da3842217ad1ee2ac4cdd10cabae`
- Packet/topic: `31143-c1-readstream-callback-unsafe-hardening`
- Lane: unsafe baseline reporting after this slice
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `bash scripts/unsafe_baseline_report.sh > review/31143-c1-readstream-callback-unsafe-hardening/artifacts/unsafe-baseline-after.log`
- Timestamp: `2026-05-16T22:14:42Z`
- Surface: local, no table/index
- Key result lines:
  - `entries: 4795`
  - `files: 112`

### `audit-unsafe.log`

- Head SHA: `ef554f4e7445da3842217ad1ee2ac4cdd10cabae`
- Packet/topic: `31143-c1-readstream-callback-unsafe-hardening`
- Lane: unsafe comment audit
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `bash scripts/check_unsafe_comments.sh > review/31143-c1-readstream-callback-unsafe-hardening/artifacts/audit-unsafe.log 2>&1`
- Timestamp: `2026-05-16T22:14:42Z`
- Surface: local, no table/index
- Key result lines:
  - Command exited successfully with no output.

### `fmt-check.log`

- Head SHA: `ef554f4e7445da3842217ad1ee2ac4cdd10cabae`
- Packet/topic: `31143-c1-readstream-callback-unsafe-hardening`
- Lane: formatting check
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `make fmt-check > review/31143-c1-readstream-callback-unsafe-hardening/artifacts/fmt-check.log 2>&1`
- Timestamp: `2026-05-16T22:14:42Z`
- Surface: local, no table/index
- Key result lines:
  - Command exited successfully.
  - Stable rustfmt emitted existing warnings about nightly-only import grouping options.

### `git-diff-check.log`

- Head SHA: `ef554f4e7445da3842217ad1ee2ac4cdd10cabae`
- Packet/topic: `31143-c1-readstream-callback-unsafe-hardening`
- Lane: whitespace diff check
- Fixture: `HEAD^..HEAD`
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `git diff --check HEAD^ HEAD > review/31143-c1-readstream-callback-unsafe-hardening/artifacts/git-diff-check.log`
- Timestamp: `2026-05-16T22:14:42Z`
- Surface: local, no table/index
- Key result lines:
  - Command exited successfully with no output.

### `cargo-check-pg18.log`

- Head SHA: `ef554f4e7445da3842217ad1ee2ac4cdd10cabae`
- Packet/topic: `31143-c1-readstream-callback-unsafe-hardening`
- Lane: PG18 cargo check
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `cargo check --all-targets --no-default-features --features pg18,bench > review/31143-c1-readstream-callback-unsafe-hardening/artifacts/cargo-check-pg18.log 2>&1`
- Timestamp: `2026-05-16T22:14:42Z`
- Surface: local, no table/index
- Key result lines:
  - `Finished dev profile`
  - Existing warnings from PostgreSQL headers and currently unused SPIRE
    re-exports remained.
