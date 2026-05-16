# Artifact Manifest: IVF planner cost relation guard

Packet: `909-c1-ivf-planner-cost-relation-guard`
Head SHA: `1e878c0271a05d4f9b2d84994a6ee738a66afb90`
Timestamp: `2026-05-16T22:48:20Z`

This packet does not make performance or recall claims. It cites
unsafe-comment baseline counts and command pass/fail validation.

## `unsafe-baseline-before.log`

- Head SHA: `28312db226f54388e390cadb1cc25f7283a0b1c1`
- Packet/topic: `909-c1-ivf-planner-cost-relation-guard`
- Lane: unsafe baseline reporting before this slice
- Fixture: `HEAD:scripts/unsafe_comment_baseline.txt` from the pre-code commit
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `git show HEAD:scripts/unsafe_comment_baseline.txt > /private/tmp/tqvector-unsafe-baseline-before-909.txt` before the code commit, then `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-909.txt > review/909-c1-ivf-planner-cost-relation-guard/artifacts/unsafe-baseline-before.log`
- Key result lines: `entries: 4733`, `files: 106`

## `unsafe-baseline-after.log`

- Head SHA: `1e878c0271a05d4f9b2d84994a6ee738a66afb90`
- Packet/topic: `909-c1-ivf-planner-cost-relation-guard`
- Lane: unsafe baseline reporting after this slice
- Fixture: `scripts/unsafe_comment_baseline.txt`
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `bash scripts/unsafe_baseline_report.sh > review/909-c1-ivf-planner-cost-relation-guard/artifacts/unsafe-baseline-after.log`
- Key result lines: `entries: 4725`, `files: 106`

## `audit-unsafe.log`

- Head SHA: `1e878c0271a05d4f9b2d84994a6ee738a66afb90`
- Packet/topic: `909-c1-ivf-planner-cost-relation-guard`
- Lane: unsafe comment audit
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `bash scripts/check_unsafe_comments.sh > review/909-c1-ivf-planner-cost-relation-guard/artifacts/audit-unsafe.log 2>&1`
- Key result lines: command exited 0 with no output.

## `fmt-check.log`

- Head SHA: `1e878c0271a05d4f9b2d84994a6ee738a66afb90`
- Packet/topic: `909-c1-ivf-planner-cost-relation-guard`
- Lane: formatting
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `make fmt-check > review/909-c1-ivf-planner-cost-relation-guard/artifacts/fmt-check.log 2>&1`
- Key result lines: command exited 0.

## `git-diff-check.log`

- Head SHA: `1e878c0271a05d4f9b2d84994a6ee738a66afb90`
- Packet/topic: `909-c1-ivf-planner-cost-relation-guard`
- Lane: whitespace
- Fixture: current workspace diff at validation time
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `git diff --check > review/909-c1-ivf-planner-cost-relation-guard/artifacts/git-diff-check.log`
- Key result lines: command exited 0 with no output.

## `cargo-check-pg18.log`

- Head SHA: `1e878c0271a05d4f9b2d84994a6ee738a66afb90`
- Packet/topic: `909-c1-ivf-planner-cost-relation-guard`
- Lane: PG18 compile validation
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `cargo check --all-targets --no-default-features --features pg18,bench > review/909-c1-ivf-planner-cost-relation-guard/artifacts/cargo-check-pg18.log 2>&1`
- Key result lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.11s`
