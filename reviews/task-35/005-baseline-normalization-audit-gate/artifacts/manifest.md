# Artifact Manifest: Baseline Normalization Audit Gate

Head SHA: `2cf7fb73a28443855d3d1e4436d1cb1534e04f25`

Task bucket: `reviews/task-35`

Packet path: `reviews/task-35/005-baseline-normalization-audit-gate`

Timestamp: `2026-05-19T03:11:35Z`

Lane / fixture / storage format / rerank mode: not applicable; static audit
baseline normalization.

Artifacts:

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,686 entries across 106 files.
- `audit-before.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: failed with current missing-SAFETY entries absent from the
    stale line-number baseline.
- `unsafe-comment-baseline-before.txt`
  - command: `cp scripts/unsafe_comment_baseline.txt artifacts/unsafe-comment-baseline-before.txt`
  - key result: 3,686 exact `file:line` baseline entries before
    normalization.
- `update-baseline.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - key result: wrote `scripts/unsafe_comment_baseline.txt` with 3,657
    entries.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,657 entries across 107 files.
- `audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
- `unsafe-comment-baseline-after.txt`
  - command: `cp scripts/unsafe_comment_baseline.txt artifacts/unsafe-comment-baseline-after.txt`
  - key result: 3,657 exact `file:line` baseline entries after normalization.
- `entries-added-by-normalization.txt`
  - command: `comm -23 unsafe-comment-baseline-after.txt unsafe-comment-baseline-before.txt`
  - key result: 1,596 current entries added because they were missing from the
    stale baseline.
- `stale-entries-removed-by-normalization.txt`
  - command: `comm -13 unsafe-comment-baseline-after.txt unsafe-comment-baseline-before.txt`
  - key result: 1,625 stale line entries removed because they no longer
    corresponded to current missing-SAFETY blocks.
- `git-diff-check.log`
  - command: `git diff --check`
  - key result: passed with no output.

Isolated one-index-per-table or shared-table surfaces: not applicable.
