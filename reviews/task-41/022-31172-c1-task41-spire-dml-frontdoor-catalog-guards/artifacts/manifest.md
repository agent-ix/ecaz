# Artifact Manifest

Packet: `31172-c1-task41-spire-dml-frontdoor-catalog-guards`

Head SHA: `48e76c6f5cc1d9dd703573a6f96449b7289a761d`

Timestamp: `2026-05-17T02:34:49Z`

## Artifacts

### `unsafe-baseline-before.txt`

- Head SHA: parent of `48e76c6f5cc1d9dd703573a6f96449b7289a761d`
- Lane / fixture / storage format / rerank mode: unsafe baseline, not storage-specific
- Command: `git show 48e76c6f^:scripts/unsafe_comment_baseline.txt`
- Isolation: not applicable
- Key result: baseline before this checkpoint had `4351` entries and `164` `src/am/ec_spire/dml_frontdoor/mod.rs` entries.

### `unsafe-baseline-after.txt`

- Head SHA: `48e76c6f5cc1d9dd703573a6f96449b7289a761d`
- Lane / fixture / storage format / rerank mode: unsafe baseline, not storage-specific
- Command: `git show 48e76c6f:scripts/unsafe_comment_baseline.txt`
- Isolation: not applicable
- Key result: baseline after this checkpoint has `4347` entries.

### `baseline-before.log`

- Head SHA: parent of `48e76c6f5cc1d9dd703573a6f96449b7289a761d`
- Lane / fixture / storage format / rerank mode: unsafe baseline summary, not storage-specific
- Command: `awk ... artifacts/unsafe-baseline-before.txt`
- Isolation: not applicable
- Key result lines: `entries: 4351`; `src/am/ec_spire/dml_frontdoor/mod.rs: 164`

### `baseline-after.log`

- Head SHA: `48e76c6f5cc1d9dd703573a6f96449b7289a761d`
- Lane / fixture / storage format / rerank mode: unsafe baseline summary, not storage-specific
- Command: `bash scripts/unsafe_baseline_report.sh`
- Isolation: not applicable
- Key result lines: `entries: 4347`; `src/am/ec_spire/dml_frontdoor/mod.rs` appears with `160` entries.

### `unsafe-comment-audit.log`

- Head SHA: `48e76c6f5cc1d9dd703573a6f96449b7289a761d`
- Lane / fixture / storage format / rerank mode: unsafe comment audit, not storage-specific
- Command: `bash scripts/check_unsafe_comments.sh`
- Isolation: not applicable
- Key result: command exited successfully with no output.

### `fmt-check.log`

- Head SHA: `48e76c6f5cc1d9dd703573a6f96449b7289a761d`
- Lane / fixture / storage format / rerank mode: formatting check, not storage-specific
- Command: `make fmt-check`
- Isolation: not applicable
- Key result: command exited successfully; log contains existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `git-diff-check.log`

- Head SHA: `48e76c6f5cc1d9dd703573a6f96449b7289a761d`
- Lane / fixture / storage format / rerank mode: whitespace check, not storage-specific
- Command: `git diff --check`
- Isolation: not applicable
- Key result: command exited successfully with no output.

### `cargo-check-pg18.log`

- Head SHA: `48e76c6f5cc1d9dd703573a6f96449b7289a761d`
- Lane / fixture / storage format / rerank mode: PG18 compile check with `bench` feature
- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Isolation: not applicable
- Key result: command exited successfully; log contains existing PostgreSQL header warnings and the existing unused re-export warning.

### `code-diff-stat.log`

- Head SHA: `48e76c6f5cc1d9dd703573a6f96449b7289a761d`
- Lane / fixture / storage format / rerank mode: code diff summary, not storage-specific
- Command: `git diff --stat 48e76c6f^ 48e76c6f`
- Isolation: not applicable
- Key result: two files changed, `src/am/ec_spire/dml_frontdoor/mod.rs` and `scripts/unsafe_comment_baseline.txt`.
