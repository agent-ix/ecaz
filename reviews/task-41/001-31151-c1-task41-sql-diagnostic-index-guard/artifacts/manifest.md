# Artifact Manifest: Task 41 SQL diagnostic index guard

Packet: `31151-c1-task41-sql-diagnostic-index-guard`
Head SHA: `626edae157a957b964d67c5a5e78246cdb4a5e21`
Timestamp: `2026-05-16T23:10:24Z`

This packet does not make performance or recall claims. It cites
unsafe-comment baseline counts and command pass/fail validation.

## `unsafe-baseline-before.log`

- Head SHA: `7665a0ea947e97cd2dbeecab06682a7c64a9c0d2`
- Packet/topic: `31151-c1-task41-sql-diagnostic-index-guard`
- Lane: unsafe baseline reporting before this slice
- Fixture: `HEAD^:scripts/unsafe_comment_baseline.txt`
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `git show HEAD^:scripts/unsafe_comment_baseline.txt > /private/tmp/tqvector-unsafe-baseline-before-911.txt` then `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-911.txt > review/31151-c1-task41-sql-diagnostic-index-guard/artifacts/unsafe-baseline-before.log`
- Key result lines: `entries: 4725`, `files: 106`

## `unsafe-baseline-after.log`

- Head SHA: `626edae157a957b964d67c5a5e78246cdb4a5e21`
- Packet/topic: `31151-c1-task41-sql-diagnostic-index-guard`
- Lane: unsafe baseline reporting after this slice
- Fixture: `scripts/unsafe_comment_baseline.txt`
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `bash scripts/unsafe_baseline_report.sh > review/31151-c1-task41-sql-diagnostic-index-guard/artifacts/unsafe-baseline-after.log`
- Key result lines: `entries: 4700`, `files: 106`

## `audit-unsafe.log`

- Head SHA: `626edae157a957b964d67c5a5e78246cdb4a5e21`
- Packet/topic: `31151-c1-task41-sql-diagnostic-index-guard`
- Lane: unsafe comment audit
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `bash scripts/check_unsafe_comments.sh > review/31151-c1-task41-sql-diagnostic-index-guard/artifacts/audit-unsafe.log 2>&1`
- Key result lines: command exited 0 with no output.

## `fmt-check.log`

- Head SHA: `626edae157a957b964d67c5a5e78246cdb4a5e21`
- Packet/topic: `31151-c1-task41-sql-diagnostic-index-guard`
- Lane: formatting
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `make fmt-check > review/31151-c1-task41-sql-diagnostic-index-guard/artifacts/fmt-check.log 2>&1`
- Key result lines: command exited 0.

## `git-diff-check.log`

- Head SHA: `626edae157a957b964d67c5a5e78246cdb4a5e21`
- Packet/topic: `31151-c1-task41-sql-diagnostic-index-guard`
- Lane: whitespace
- Fixture: code commit diff
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `git diff --check HEAD^ HEAD > review/31151-c1-task41-sql-diagnostic-index-guard/artifacts/git-diff-check.log`
- Key result lines: command exited 0 with no output.

## `cargo-check-pg18.log`

- Head SHA: `626edae157a957b964d67c5a5e78246cdb4a5e21`
- Packet/topic: `31151-c1-task41-sql-diagnostic-index-guard`
- Lane: PG18 compile validation
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `cargo check --all-targets --no-default-features --features pg18,bench > review/31151-c1-task41-sql-diagnostic-index-guard/artifacts/cargo-check-pg18.log 2>&1`
- Key result lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.11s`
