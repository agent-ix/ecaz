# Review Request: Unsafe Quality Burndown Scaffold

Head: `d036f474ce7413ebb3e3ab8b2350a5797b9fcbec`

Scope:
- `Makefile`
- `scripts/unsafe_baseline_report.sh`
- `plan/tasks/35-unsafe-quality-burndown.md`
- `plan/tasks/09-ci-and-safety.md`
- `plan/tasks/README.md`
- `docs/hardening.md`
- `docs/contributing.md`

What changed:
- Added Task 35 as the durable owner for burning
  `scripts/unsafe_comment_baseline.txt` down to zero.
- Added `make unsafe-baseline-report`, backed by
  `scripts/unsafe_baseline_report.sh`, to produce consistent before/after
  counts for unsafe-burndown packets.
- Updated Task 09 and hardening docs so TC-036 is no longer an ambiguous
  remaining audit item; it now points to the Task 35 packet workflow.
- Documented that the current unsafe baseline is temporary quality debt, not a
  permanent exception to `NFR-004`.

Baseline result:
- Current grandfathered baseline: 4,816 entries across 124 files.
- Largest buckets: `src/am` with 3,692 entries, `src/tests` with 539 entries,
  root `src` files with 517 entries, and `src/quant` with 68 entries.
- Largest single file: `src/lib.rs` with 512 entries.

Review focus:
- Whether Task 35 has the right packet sequence and acceptance criteria for
  reviewable unsafe cleanup.
- Whether `make unsafe-baseline-report` is sufficient for packet-local baseline
  accounting.
- Whether the Task 09 and hardening docs make the new policy clear: new
  undocumented unsafe remains blocked, while legacy unsafe must shrink through
  reviewed packets.

Validation:
- `make unsafe-baseline-report`
  - artifact: `artifacts/unsafe-baseline-report.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
- `git diff --check HEAD^ HEAD`
  - artifact: `artifacts/git-diff-check.log`

Tests skipped:
- No Rust behavior changed; this packet only adds a reporting script and docs.
