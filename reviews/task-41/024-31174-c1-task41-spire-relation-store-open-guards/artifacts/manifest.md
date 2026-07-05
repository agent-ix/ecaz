# Artifact Manifest

Packet: `31174-c1-task41-spire-relation-store-open-guards`

Head SHA: `1dfbe9890252c5aa135df5615e980ea36bbcf68e`

Timestamp: `2026-05-17T03:30:31Z`

## Artifacts

### `unsafe-baseline-before.txt`

- Head SHA: parent of `1dfbe9890252c5aa135df5615e980ea36bbcf68e`
- Lane / fixture / storage format / rerank mode: unsafe baseline, not storage-specific
- Command: `git show 1dfbe989^:scripts/unsafe_comment_baseline.txt`
- Isolation: not applicable
- Key result: baseline before this checkpoint had `4325` entries and `55` `src/am/ec_spire/storage/relation_store.rs` entries.

### `unsafe-baseline-after.txt`

- Head SHA: `1dfbe9890252c5aa135df5615e980ea36bbcf68e`
- Lane / fixture / storage format / rerank mode: unsafe baseline, not storage-specific
- Command: `git show 1dfbe989:scripts/unsafe_comment_baseline.txt`
- Isolation: not applicable
- Key result: baseline after this checkpoint has `4321` entries.

### `baseline-before.log`

- Head SHA: parent of `1dfbe9890252c5aa135df5615e980ea36bbcf68e`
- Lane / fixture / storage format / rerank mode: unsafe baseline summary, not storage-specific
- Command: `awk ... artifacts/unsafe-baseline-before.txt`
- Isolation: not applicable
- Key result lines: `entries: 4325`; `src/am/ec_spire/storage/relation_store.rs: 55`

### `baseline-after.log`

- Head SHA: `1dfbe9890252c5aa135df5615e980ea36bbcf68e`
- Lane / fixture / storage format / rerank mode: unsafe baseline summary, not storage-specific
- Command: `bash scripts/unsafe_baseline_report.sh`
- Isolation: not applicable
- Key result lines: `entries: 4321`; `src/am/ec_spire/storage/relation_store.rs` appears with `51` entries.

### `unsafe-comment-audit.log`

- Head SHA: `1dfbe9890252c5aa135df5615e980ea36bbcf68e`
- Lane / fixture / storage format / rerank mode: unsafe comment audit, not storage-specific
- Command: `bash scripts/check_unsafe_comments.sh`
- Isolation: not applicable
- Key result: command exited successfully with no output.

### `fmt-check.log`

- Head SHA: `1dfbe9890252c5aa135df5615e980ea36bbcf68e`
- Lane / fixture / storage format / rerank mode: formatting check, not storage-specific
- Command: `make fmt-check`
- Isolation: not applicable
- Key result: command exited successfully; log contains existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `git-diff-check.log`

- Head SHA: `1dfbe9890252c5aa135df5615e980ea36bbcf68e`
- Lane / fixture / storage format / rerank mode: whitespace check, not storage-specific
- Command: `git diff --check`
- Isolation: not applicable
- Key result: command exited successfully with no output.

### `cargo-check-pg18.log`

- Head SHA: `1dfbe9890252c5aa135df5615e980ea36bbcf68e`
- Lane / fixture / storage format / rerank mode: PG18 compile check with `bench` feature
- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Isolation: not applicable
- Key result: command exited successfully; log contains existing PostgreSQL header warnings and the existing unused re-export warning.

### `code-diff-stat.log`

- Head SHA: `1dfbe9890252c5aa135df5615e980ea36bbcf68e`
- Lane / fixture / storage format / rerank mode: code diff summary, not storage-specific
- Command: `git diff --stat 1dfbe989^ 1dfbe989`
- Isolation: not applicable
- Key result: two files changed, `src/am/ec_spire/storage/relation_store.rs` and `scripts/unsafe_comment_baseline.txt`.
