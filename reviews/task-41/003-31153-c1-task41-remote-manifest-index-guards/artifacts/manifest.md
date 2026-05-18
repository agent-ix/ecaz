# Artifact Manifest: Task 41 remote manifest index guards

Packet: `31153-c1-task41-remote-manifest-index-guards`
Head SHA: `5d813474c3e09b3917753589eaf55ec497611d13`
Timestamp: `2026-05-16T23:17:59Z`

This packet does not make performance or recall claims. It cites
unsafe-comment baseline counts and command pass/fail validation.

## `unsafe-baseline-before.log`

- Head SHA: `6f0eb05dce22c8eabd913cfa3574792d70673757`
- Packet/topic: `31153-c1-task41-remote-manifest-index-guards`
- Lane: unsafe baseline reporting before this slice
- Fixture: `HEAD^:scripts/unsafe_comment_baseline.txt`
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `git show HEAD^:scripts/unsafe_comment_baseline.txt > /private/tmp/tqvector-unsafe-baseline-before-913.txt` then `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-913.txt > review/31153-c1-task41-remote-manifest-index-guards/artifacts/unsafe-baseline-before.log`
- Key result lines: `entries: 4684`, `files: 106`

## `unsafe-baseline-after.log`

- Head SHA: `5d813474c3e09b3917753589eaf55ec497611d13`
- Packet/topic: `31153-c1-task41-remote-manifest-index-guards`
- Lane: unsafe baseline reporting after this slice
- Fixture: `scripts/unsafe_comment_baseline.txt`
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `bash scripts/unsafe_baseline_report.sh > review/31153-c1-task41-remote-manifest-index-guards/artifacts/unsafe-baseline-after.log`
- Key result lines: `entries: 4660`, `files: 106`

## `audit-unsafe.log`

- Head SHA: `5d813474c3e09b3917753589eaf55ec497611d13`
- Packet/topic: `31153-c1-task41-remote-manifest-index-guards`
- Lane: unsafe comment audit
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `bash scripts/check_unsafe_comments.sh > review/31153-c1-task41-remote-manifest-index-guards/artifacts/audit-unsafe.log 2>&1`
- Key result lines: command exited 0 with no output.

## `fmt-check.log`

- Head SHA: `5d813474c3e09b3917753589eaf55ec497611d13`
- Packet/topic: `31153-c1-task41-remote-manifest-index-guards`
- Lane: formatting
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `make fmt-check > review/31153-c1-task41-remote-manifest-index-guards/artifacts/fmt-check.log 2>&1`
- Key result lines: command exited 0.

## `git-diff-check.log`

- Head SHA: `5d813474c3e09b3917753589eaf55ec497611d13`
- Packet/topic: `31153-c1-task41-remote-manifest-index-guards`
- Lane: whitespace
- Fixture: code commit diff
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `git diff --check HEAD^ HEAD > review/31153-c1-task41-remote-manifest-index-guards/artifacts/git-diff-check.log`
- Key result lines: command exited 0 with no output.

## `cargo-check-pg18.log`

- Head SHA: `5d813474c3e09b3917753589eaf55ec497611d13`
- Packet/topic: `31153-c1-task41-remote-manifest-index-guards`
- Lane: PG18 compile validation
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `cargo check --all-targets --no-default-features --features pg18,bench > review/31153-c1-task41-remote-manifest-index-guards/artifacts/cargo-check-pg18.log 2>&1`
- Key result lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.10s`
