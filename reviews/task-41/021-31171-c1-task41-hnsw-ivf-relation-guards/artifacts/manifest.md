# Artifact Manifest

Packet: `31171-c1-task41-hnsw-ivf-relation-guards`

Head SHA: `c5afbecc96ff9cc2d34644821c80e5c45061b490`

Timestamp: `2026-05-17T02:29:44Z`

## Artifacts

### `unsafe-baseline-before.txt`

- Head SHA: parent of `c5afbecc96ff9cc2d34644821c80e5c45061b490`
- Lane / fixture / storage format / rerank mode: unsafe baseline, not storage-specific
- Command: `git show c5afbecc^:scripts/unsafe_comment_baseline.txt`
- Isolation: not applicable
- Key result: baseline before this checkpoint had `4390` entries and `185` `src/lib.rs` entries.

### `unsafe-baseline-after.txt`

- Head SHA: `c5afbecc96ff9cc2d34644821c80e5c45061b490`
- Lane / fixture / storage format / rerank mode: unsafe baseline, not storage-specific
- Command: `git show c5afbecc:scripts/unsafe_comment_baseline.txt`
- Isolation: not applicable
- Key result: baseline after this checkpoint has `4351` entries.

### `baseline-before.log`

- Head SHA: parent of `c5afbecc96ff9cc2d34644821c80e5c45061b490`
- Lane / fixture / storage format / rerank mode: unsafe baseline summary, not storage-specific
- Command: `awk ... artifacts/unsafe-baseline-before.txt`
- Isolation: not applicable
- Key result lines: `entries: 4390`; `src/lib.rs: 185`; selected HNSW test files: `177`.

### `baseline-after.log`

- Head SHA: `c5afbecc96ff9cc2d34644821c80e5c45061b490`
- Lane / fixture / storage format / rerank mode: unsafe baseline summary, not storage-specific
- Command: `bash scripts/unsafe_baseline_report.sh`
- Isolation: not applicable
- Key result lines: `entries: 4351`; `src/lib.rs` appears with `177` entries.

### `unsafe-comment-audit.log`

- Head SHA: `c5afbecc96ff9cc2d34644821c80e5c45061b490`
- Lane / fixture / storage format / rerank mode: unsafe comment audit, not storage-specific
- Command: `bash scripts/check_unsafe_comments.sh`
- Isolation: not applicable
- Key result: command exited successfully with no output.

### `fmt-check.log`

- Head SHA: `c5afbecc96ff9cc2d34644821c80e5c45061b490`
- Lane / fixture / storage format / rerank mode: formatting check, not storage-specific
- Command: `make fmt-check`
- Isolation: not applicable
- Key result: command exited successfully; log contains existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `git-diff-check.log`

- Head SHA: `c5afbecc96ff9cc2d34644821c80e5c45061b490`
- Lane / fixture / storage format / rerank mode: whitespace check, not storage-specific
- Command: `git diff --check`
- Isolation: not applicable
- Key result: command exited successfully with no output.

### `cargo-check-pg18.log`

- Head SHA: `c5afbecc96ff9cc2d34644821c80e5c45061b490`
- Lane / fixture / storage format / rerank mode: PG18 compile check with `bench` feature
- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Isolation: not applicable
- Key result: command exited successfully; log contains existing PostgreSQL header warnings and the existing unused re-export warning.

### `code-diff-stat.log`

- Head SHA: `c5afbecc96ff9cc2d34644821c80e5c45061b490`
- Lane / fixture / storage format / rerank mode: code diff summary, not storage-specific
- Command: `git diff --stat c5afbecc^ c5afbecc`
- Isolation: not applicable
- Key result: seven files changed, including `src/lib.rs`, HNSW test files, and `scripts/unsafe_comment_baseline.txt`.
