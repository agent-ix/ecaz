# Artifact Manifest: Task 41 more SQL diagnostic index guards

Packet: `912-c1-task41-more-sql-diagnostic-index-guards`
Head SHA: `a8c2eab3ccd337f1a6625ccc48034131d8961c74`
Timestamp: `2026-05-16T23:14:04Z`

This packet does not make performance or recall claims. It cites
unsafe-comment baseline counts and command pass/fail validation.

## `unsafe-baseline-before.log`

- Head SHA: `486343213588855285c0575fbcef8828b0b92f66`
- Packet/topic: `912-c1-task41-more-sql-diagnostic-index-guards`
- Lane: unsafe baseline reporting before this slice
- Fixture: `HEAD^:scripts/unsafe_comment_baseline.txt`
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `git show HEAD^:scripts/unsafe_comment_baseline.txt > /private/tmp/tqvector-unsafe-baseline-before-912.txt` then `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-912.txt > review/912-c1-task41-more-sql-diagnostic-index-guards/artifacts/unsafe-baseline-before.log`
- Key result lines: `entries: 4700`, `files: 106`

## `unsafe-baseline-after.log`

- Head SHA: `a8c2eab3ccd337f1a6625ccc48034131d8961c74`
- Packet/topic: `912-c1-task41-more-sql-diagnostic-index-guards`
- Lane: unsafe baseline reporting after this slice
- Fixture: `scripts/unsafe_comment_baseline.txt`
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `bash scripts/unsafe_baseline_report.sh > review/912-c1-task41-more-sql-diagnostic-index-guards/artifacts/unsafe-baseline-after.log`
- Key result lines: `entries: 4684`, `files: 106`

## `audit-unsafe.log`

- Head SHA: `a8c2eab3ccd337f1a6625ccc48034131d8961c74`
- Packet/topic: `912-c1-task41-more-sql-diagnostic-index-guards`
- Lane: unsafe comment audit
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `bash scripts/check_unsafe_comments.sh > review/912-c1-task41-more-sql-diagnostic-index-guards/artifacts/audit-unsafe.log 2>&1`
- Key result lines: command exited 0 with no output.

## `fmt-check.log`

- Head SHA: `a8c2eab3ccd337f1a6625ccc48034131d8961c74`
- Packet/topic: `912-c1-task41-more-sql-diagnostic-index-guards`
- Lane: formatting
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `make fmt-check > review/912-c1-task41-more-sql-diagnostic-index-guards/artifacts/fmt-check.log 2>&1`
- Key result lines: command exited 0.

## `git-diff-check.log`

- Head SHA: `a8c2eab3ccd337f1a6625ccc48034131d8961c74`
- Packet/topic: `912-c1-task41-more-sql-diagnostic-index-guards`
- Lane: whitespace
- Fixture: code commit diff
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `git diff --check HEAD^ HEAD > review/912-c1-task41-more-sql-diagnostic-index-guards/artifacts/git-diff-check.log`
- Key result lines: command exited 0 with no output.

## `cargo-check-pg18.log`

- Head SHA: `a8c2eab3ccd337f1a6625ccc48034131d8961c74`
- Packet/topic: `912-c1-task41-more-sql-diagnostic-index-guards`
- Lane: PG18 compile validation
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `cargo check --all-targets --no-default-features --features pg18,bench > review/912-c1-task41-more-sql-diagnostic-index-guards/artifacts/cargo-check-pg18.log 2>&1`
- Key result lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.11s`
