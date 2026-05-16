# Artifact Manifest: Task 41 remote manifest catalog guards

Packet: `914-c1-task41-remote-manifest-catalog-guards`
Head SHA: `05c9f2f293317f1b343796d5c280945f3986320d`
Timestamp: `2026-05-16T23:21:32Z`

This packet does not make performance or recall claims. It cites
unsafe-comment baseline counts and command pass/fail validation.

## `unsafe-baseline-before.log`

- Head SHA: `d04d29f972167763fd148cc227ec9af55dff5d30`
- Packet/topic: `914-c1-task41-remote-manifest-catalog-guards`
- Lane: unsafe baseline reporting before this slice
- Fixture: `HEAD^:scripts/unsafe_comment_baseline.txt`
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `git show HEAD^:scripts/unsafe_comment_baseline.txt > /private/tmp/tqvector-unsafe-baseline-before-914.txt` then `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-914.txt > review/914-c1-task41-remote-manifest-catalog-guards/artifacts/unsafe-baseline-before.log`
- Key result lines: `entries: 4660`, `files: 106`

## `unsafe-baseline-after.log`

- Head SHA: `05c9f2f293317f1b343796d5c280945f3986320d`
- Packet/topic: `914-c1-task41-remote-manifest-catalog-guards`
- Lane: unsafe baseline reporting after this slice
- Fixture: `scripts/unsafe_comment_baseline.txt`
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `bash scripts/unsafe_baseline_report.sh > review/914-c1-task41-remote-manifest-catalog-guards/artifacts/unsafe-baseline-after.log`
- Key result lines: `entries: 4652`, `files: 106`

## `audit-unsafe.log`

- Head SHA: `05c9f2f293317f1b343796d5c280945f3986320d`
- Packet/topic: `914-c1-task41-remote-manifest-catalog-guards`
- Lane: unsafe comment audit
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `bash scripts/check_unsafe_comments.sh > review/914-c1-task41-remote-manifest-catalog-guards/artifacts/audit-unsafe.log 2>&1`
- Key result lines: command exited 0 with no output.

## `fmt-check.log`

- Head SHA: `05c9f2f293317f1b343796d5c280945f3986320d`
- Packet/topic: `914-c1-task41-remote-manifest-catalog-guards`
- Lane: formatting
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `make fmt-check > review/914-c1-task41-remote-manifest-catalog-guards/artifacts/fmt-check.log 2>&1`
- Key result lines: command exited 0.

## `git-diff-check.log`

- Head SHA: `05c9f2f293317f1b343796d5c280945f3986320d`
- Packet/topic: `914-c1-task41-remote-manifest-catalog-guards`
- Lane: whitespace
- Fixture: code commit diff
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `git diff --check HEAD^ HEAD > review/914-c1-task41-remote-manifest-catalog-guards/artifacts/git-diff-check.log`
- Key result lines: command exited 0 with no output.

## `cargo-check-pg18.log`

- Head SHA: `05c9f2f293317f1b343796d5c280945f3986320d`
- Packet/topic: `914-c1-task41-remote-manifest-catalog-guards`
- Lane: PG18 compile validation
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `cargo check --all-targets --no-default-features --features pg18,bench > review/914-c1-task41-remote-manifest-catalog-guards/artifacts/cargo-check-pg18.log 2>&1`
- Key result lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.11s`
