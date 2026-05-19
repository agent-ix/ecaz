# Review Request: Baseline Normalization Audit Gate

Head: `2cf7fb73a28443855d3d1e4436d1cb1534e04f25`

Scope:
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/005-baseline-normalization-audit-gate/request.md`
- `reviews/task-35/005-baseline-normalization-audit-gate/artifacts/*`

What changed:
- Rebuilt `scripts/unsafe_comment_baseline.txt` with
  `bash scripts/check_unsafe_comments.sh --update-baseline`.
- Restored `bash scripts/check_unsafe_comments.sh` to a passing state on the
  current checkout.
- Captured before/after baseline files and drift lists so this packet is
  auditable as line-number normalization, not semantic acceptance of new unsafe
  sites.

Baseline result:
- Start: 3,686 entries across 106 files.
- End: 3,657 entries across 107 files.
- Net reduction: 29 exact `file:line` entries.
- Drift handling: 1,596 current missing-SAFETY entries were added at their
  current line numbers, while 1,625 stale line entries were removed.

Review focus:
- Confirm this is acceptable as the Task 35 gate-repair packet identified by
  `004-master-unsafe-burndown-plan`.
- Confirm the update is generated bookkeeping only: no source unsafe site was
  accepted, documented, deleted, or hidden in this packet.
- Confirm future burndown packets can again rely on
  `bash scripts/check_unsafe_comments.sh` as a guard against newly introduced
  undocumented unsafe blocks.

Validation:
- `make unsafe-baseline-report` before update
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/check_unsafe_comments.sh` before update
  - artifact: `artifacts/audit-before.log`
  - result: failed with 1,596 missing baseline entries.
- `bash scripts/check_unsafe_comments.sh --update-baseline`
  - artifact: `artifacts/update-baseline.log`
- `make unsafe-baseline-report` after update
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh` after update
  - artifact: `artifacts/audit-after.log`
  - result: passed with no output.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passed with no output.

Tests skipped:
- No Rust behavior changed; this packet only reconciles the line-based unsafe
  baseline with the current source tree.
