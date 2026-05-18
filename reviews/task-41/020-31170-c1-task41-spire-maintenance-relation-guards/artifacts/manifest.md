# Artifact Manifest

Packet: `31170-c1-task41-spire-maintenance-relation-guards`

Head SHA: `417737e91c07ae9f46fe70bef00cb43feebf82f9`

Timestamp: `2026-05-17T02:18:44Z`

## Artifacts

### `unsafe-baseline-before.txt`

- Head SHA: parent of `417737e91c07ae9f46fe70bef00cb43feebf82f9`
- Lane / fixture / storage format / rerank mode: unsafe baseline, not storage-specific
- Command: `git show 417737e9^:scripts/unsafe_comment_baseline.txt`
- Isolation: not applicable
- Key result: baseline before this checkpoint had `4427` entries and `214` `src/lib.rs` entries.

### `unsafe-baseline-after.txt`

- Head SHA: `417737e91c07ae9f46fe70bef00cb43feebf82f9`
- Lane / fixture / storage format / rerank mode: unsafe baseline, not storage-specific
- Command: `git show 417737e9:scripts/unsafe_comment_baseline.txt`
- Isolation: not applicable
- Key result: baseline after this checkpoint has `4393` entries.

### `baseline-before.log`

- Head SHA: parent of `417737e91c07ae9f46fe70bef00cb43feebf82f9`
- Lane / fixture / storage format / rerank mode: unsafe baseline summary, not storage-specific
- Command: `awk ... artifacts/unsafe-baseline-before.txt`
- Isolation: not applicable
- Key result lines: `entries: 4427`; `src/lib.rs: 214`

### `baseline-after.log`

- Head SHA: `417737e91c07ae9f46fe70bef00cb43feebf82f9`
- Lane / fixture / storage format / rerank mode: unsafe baseline summary, not storage-specific
- Command: `bash scripts/unsafe_baseline_report.sh`
- Isolation: not applicable
- Key result lines: `entries: 4393`; `src/lib.rs` appears with `188` entries.

### `unsafe-comment-audit.log`

- Head SHA: `417737e91c07ae9f46fe70bef00cb43feebf82f9`
- Lane / fixture / storage format / rerank mode: unsafe comment audit, not storage-specific
- Command: `bash scripts/check_unsafe_comments.sh`
- Isolation: not applicable
- Key result: command exited successfully with no output.

### `fmt-check.log`

- Head SHA: `417737e91c07ae9f46fe70bef00cb43feebf82f9`
- Lane / fixture / storage format / rerank mode: formatting check, not storage-specific
- Command: `make fmt-check`
- Isolation: not applicable
- Key result: command exited successfully; log contains existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `git-diff-check.log`

- Head SHA: `417737e91c07ae9f46fe70bef00cb43feebf82f9`
- Lane / fixture / storage format / rerank mode: whitespace check, not storage-specific
- Command: `git diff --check`
- Isolation: not applicable
- Key result: command exited successfully with no output.

### `cargo-check-pg18.log`

- Head SHA: `417737e91c07ae9f46fe70bef00cb43feebf82f9`
- Lane / fixture / storage format / rerank mode: PG18 compile check with `bench` feature
- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Isolation: not applicable
- Key result: command exited successfully; log contains existing PostgreSQL header warnings and existing unused re-export warnings.

### `code-diff-stat.log`

- Head SHA: `417737e91c07ae9f46fe70bef00cb43feebf82f9`
- Lane / fixture / storage format / rerank mode: code diff summary, not storage-specific
- Command: `git diff --stat 417737e9^ 417737e9`
- Isolation: not applicable
- Key result: five files changed, including `src/lib.rs`, three test files, and `scripts/unsafe_comment_baseline.txt`.
